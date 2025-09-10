// SIS Kernel QEMU Runtime Integration
// Interfaces with QEMU instances running the SIS kernel for real testing

use crate::{TestSuiteConfig, TestError};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::collections::HashMap;
use tokio::process::{Command, Child};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QEMUInstance {
    pub node_id: usize,
    pub serial_port: u16,
    pub monitor_port: u16,
    pub network_port: u16,
    pub esp_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QEMUCluster {
    pub instances: Vec<QEMUInstance>,
    pub base_port: u16,
    pub total_nodes: usize,
}

#[derive(Debug)]
pub struct QEMURuntimeManager {
    config: TestSuiteConfig,
    cluster: QEMUCluster,
    processes: HashMap<usize, Child>,
}

impl QEMURuntimeManager {
    /// Detect if running on Apple Silicon for HVF acceleration
    async fn is_apple_silicon() -> bool {
        if cfg!(target_os = "macos") {
            // Check if we're on Apple Silicon by looking for the "Apple" brand in CPU info
            match Command::new("sysctl")
                .args(["-n", "machdep.cpu.brand_string"])
                .output()
                .await
            {
                Ok(output) => {
                    let cpu_info = String::from_utf8_lossy(&output.stdout);
                    cpu_info.contains("Apple")
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    pub fn new(config: &TestSuiteConfig) -> Self {
        let base_port = 7000;
        let instances = (0..config.qemu_nodes)
            .map(|node_id| QEMUInstance {
                node_id,
                serial_port: base_port + node_id as u16,
                monitor_port: base_port + 100 + node_id as u16,
                network_port: base_port + 200 + node_id as u16,
                esp_directory: format!("target/testing/esp-node{}", node_id),
            })
            .collect();

        let cluster = QEMUCluster {
            instances,
            base_port,
            total_nodes: config.qemu_nodes,
        };

        Self {
            config: config.clone(),
            cluster,
            processes: HashMap::new(),
        }
    }

    pub async fn build_kernel(&self) -> Result<(), TestError> {
        log::info!("Building SIS kernel for QEMU testing");
        
        // Build the kernel in release mode for accurate performance testing
        let output = Command::new("cargo")
            .args([
                "+nightly",
                "build",
                "--release",  // Use release mode for production-like performance
                "-p", "sis_kernel",
                "-Z", "build-std=core,alloc",
                "--target", "aarch64-unknown-none",
                "--features", "bringup,neon-optimized"  // Enable NEON optimizations
            ])
            .current_dir("../../")  // Go to workspace root
            .env("RUSTFLAGS", "-C link-arg=-Tsrc/arch/aarch64/aarch64-qemu.ld")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| TestError::QEMUError { 
                message: format!("Failed to run kernel build: {}", e) 
            })?;

        if !output.status.success() {
            return Err(TestError::QEMUError {
                message: format!("Kernel build failed: {}", String::from_utf8_lossy(&output.stderr))
            });
        }

        // Build the UEFI bootloader - run from workspace root
        let output = Command::new("cargo")
            .args([
                "build",
                "-p", "uefi-boot",
                "--release",
                "--target", "aarch64-unknown-uefi"
            ])
            .current_dir("../../")  // Go to workspace root
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| TestError::QEMUError { 
                message: format!("Failed to run UEFI build: {}", e) 
            })?;

        if !output.status.success() {
            return Err(TestError::QEMUError {
                message: format!("UEFI build failed: {}", String::from_utf8_lossy(&output.stderr))
            });
        }

        log::info!("SIS kernel and UEFI bootloader built successfully");
        Ok(())
    }

    pub async fn prepare_esp_directories(&self) -> Result<(), TestError> {
        log::info!("Preparing ESP directories for {} QEMU instances", self.cluster.total_nodes);

        for instance in &self.cluster.instances {
            // Create ESP directory structure
            let esp_dir = &instance.esp_directory;
            let efi_boot_dir = format!("{}/EFI/BOOT", esp_dir);
            let efi_sis_dir = format!("{}/EFI/SIS", esp_dir);

            std::fs::create_dir_all(&efi_boot_dir)
                .map_err(|e| TestError::QEMUError {
                    message: format!("Failed to create ESP directory {}: {}", efi_boot_dir, e)
                })?;

            std::fs::create_dir_all(&efi_sis_dir)
                .map_err(|e| TestError::QEMUError {
                    message: format!("Failed to create ESP directory {}: {}", efi_sis_dir, e)
                })?;

            // Copy UEFI and kernel binaries
            let uefi_source = "../../target/aarch64-unknown-uefi/release/uefi-boot.efi";
            let kernel_source = "../../target/aarch64-unknown-none/release/sis_kernel";  // Use release build
            let uefi_dest = format!("{}/BOOTAA64.EFI", efi_boot_dir);
            let kernel_dest = format!("{}/KERNEL.ELF", efi_sis_dir);

            std::fs::copy(uefi_source, &uefi_dest)
                .map_err(|e| TestError::QEMUError {
                    message: format!("Failed to copy UEFI binary to {}: {}", uefi_dest, e)
                })?;

            std::fs::copy(kernel_source, &kernel_dest)
                .map_err(|e| TestError::QEMUError {
                    message: format!("Failed to copy kernel binary to {}: {}", kernel_dest, e)
                })?;
        }

        log::info!("ESP directories prepared for all instances");
        Ok(())
    }

    pub async fn launch_cluster(&mut self) -> Result<(), TestError> {
        log::info!("Launching QEMU cluster with {} nodes", self.cluster.total_nodes);

        let instances = self.cluster.instances.clone();
        for instance in instances {
            self.launch_instance(&instance).await?;
            sleep(Duration::from_secs(3)).await; // Stagger launches
        }

        log::info!("All QEMU instances launched successfully");
        Ok(())
    }

    async fn launch_instance(&mut self, instance: &QEMUInstance) -> Result<(), TestError> {
        log::info!("Launching QEMU instance {} on ports {}/{}/{}", 
                  instance.node_id, instance.serial_port, instance.monitor_port, instance.network_port);
        
        // Detect if running on Apple Silicon Mac for HVF acceleration
        let use_hvf = Self::is_apple_silicon().await;
        let cpu_type = if use_hvf {
            "host"  // Use host CPU for better Apple Silicon emulation
        } else {
            "max"   // Use maximum features on other platforms
        };
        
        // Optimize QEMU configuration for Apple Silicon development
        let firmware_path = "/opt/homebrew/share/qemu/edk2-aarch64-code.fd";
        
        let mut qemu_args = vec![
            "-name".to_string(), format!("sis-node{}", instance.node_id),
            "-M".to_string(), "virt,gic-version=3,highmem=on,secure=off".to_string(),  // Enable highmem for M-series simulation
            "-cpu".to_string(), cpu_type.to_string(),
            "-smp".to_string(), "4".to_string(),  // Multi-core for realistic M-series behavior
            "-m".to_string(), "1G".to_string(),  // Increased memory for better performance
            "-nographic".to_string(),
            "-serial".to_string(), format!("tcp:localhost:{},server,nowait", instance.serial_port),
            "-monitor".to_string(), format!("tcp:localhost:{},server,nowait", instance.monitor_port),
            "-bios".to_string(), firmware_path.to_string(),
            "-drive".to_string(), format!("if=none,id=esp,format=raw,file=fat:rw:{}", instance.esp_directory),
            "-device".to_string(), "virtio-blk-pci,drive=esp".to_string(),
            "-device".to_string(), "virtio-rng-pci".to_string(),
            "-no-reboot".to_string(),
            "-append".to_string(), "console=ttyAMA0,115200 earlycon=pl011,0x09000000".to_string(),
            "-d".to_string(), "unimp,guest_errors".to_string(),
        ];
        
        // Add HVF acceleration if on Apple Silicon
        if use_hvf {
            qemu_args.extend(["-accel".to_string(), "hvf".to_string()]);
            log::info!("Using HVF acceleration on Apple Silicon for instance {}", instance.node_id);
        }
        
        // Add cycle-accurate simulation for performance measurement
        qemu_args.extend([
            "-icount".to_string(), "shift=0".to_string(),  // Cycle-accurate for benchmarking
            "-object".to_string(), "memory-backend-ram,id=ram,size=1G,prealloc=on".to_string(),  // Preallocate memory
            "-numa".to_string(), "node,memdev=ram".to_string(),  // NUMA awareness for M-series simulation
        ]);
        
        // Add network device for distributed testing
        qemu_args.extend([
            "-netdev".to_string(), format!("user,id=net0,hostfwd=tcp::{}-:22", instance.network_port),
            "-device".to_string(), "virtio-net-pci,netdev=net0".to_string(),
        ]);
        
        log::debug!("QEMU command: qemu-system-aarch64 {}", qemu_args.join(" "));
        
        let qemu_process = Command::new("qemu-system-aarch64")
            .args(qemu_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| TestError::QEMUError {
                message: format!("Failed to launch QEMU instance {}: {}", instance.node_id, e)
            })?;

        self.processes.insert(instance.node_id, qemu_process);

        log::info!("Instance {} launched (serial: telnet localhost {})", 
                  instance.node_id, instance.serial_port);
        Ok(())
    }

    pub async fn shutdown_cluster(&mut self) -> Result<(), TestError> {
        log::info!("Shutting down QEMU cluster");

        for (node_id, mut process) in self.processes.drain() {
            log::info!("Terminating QEMU instance {}", node_id);
            let _ = process.kill().await;
        }

        // Clean up any remaining QEMU processes
        let _ = Command::new("pkill")
            .args(["-f", "qemu-system-aarch64.*sis-node"])
            .output()
            .await;

        log::info!("QEMU cluster shutdown complete");
        Ok(())
    }

    pub async fn connect_to_instance(&self, node_id: usize) -> Result<tokio::net::TcpStream, TestError> {
        if let Some(instance) = self.cluster.instances.iter().find(|i| i.node_id == node_id) {
            let addr = format!("localhost:{}", instance.serial_port);
            
            // Retry connection up to 10 times with 1 second delays
            for attempt in 1..=10 {
                match tokio::net::TcpStream::connect(&addr).await {
                    Ok(stream) => {
                        log::info!("Connected to QEMU instance {} on attempt {}", node_id, attempt);
                        return Ok(stream);
                    }
                    Err(e) if attempt == 10 => {
                        return Err(TestError::QEMUError {
                            message: format!("Failed to connect to instance {} after {} attempts: {}", 
                                           node_id, attempt, e)
                        });
                    }
                    Err(_) => {
                        log::debug!("Connection attempt {} to instance {} failed, retrying...", attempt, node_id);
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            }

            unreachable!()
        } else {
            Err(TestError::QEMUError {
                message: format!("Instance {} not found in cluster", node_id)
            })
        }
    }

    pub async fn send_command(&self, node_id: usize, command: &str) -> Result<String, TestError> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let mut stream = self.connect_to_instance(node_id).await?;
        
        // Send command
        stream.write_all(format!("{}\n", command).as_bytes()).await
            .map_err(|e| TestError::QEMUError {
                message: format!("Failed to send command to instance {}: {}", node_id, e)
            })?;

        // Read response (with timeout)
        let mut buffer = vec![0; 4096];
        let bytes_read = tokio::time::timeout(Duration::from_secs(5), stream.read(&mut buffer)).await
            .map_err(|_| TestError::QEMUError {
                message: format!("Timeout reading response from instance {}", node_id)
            })?
            .map_err(|e| TestError::QEMUError {
                message: format!("Failed to read response from instance {}: {}", node_id, e)
            })?;

        let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
        Ok(response)
    }

    pub async fn read_boot_output(&self, node_id: usize) -> Result<String, TestError> {
        use tokio::io::AsyncReadExt;

        let mut stream = self.connect_to_instance(node_id).await?;
        
        // Read any available output (with longer timeout to allow kernel boot messages)
        let mut buffer = vec![0; 4096];
        let bytes_read = tokio::time::timeout(Duration::from_secs(3), stream.read(&mut buffer)).await
            .map_err(|_| TestError::QEMUError {
                message: format!("Timeout reading boot output from instance {}", node_id)
            })?
            .map_err(|e| TestError::QEMUError {
                message: format!("Failed to read boot output from instance {}: {}", node_id, e)
            })?;

        let output = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
        Ok(output)
    }

    pub fn get_cluster_info(&self) -> &QEMUCluster {
        &self.cluster
    }

    pub async fn wait_for_boot(&self, node_id: usize, timeout_secs: u64) -> Result<bool, TestError> {
        log::info!("Waiting for instance {} to boot (timeout: {}s)", node_id, timeout_secs);
        
        let start_time = std::time::Instant::now();
        let timeout_duration = Duration::from_secs(timeout_secs);

        while start_time.elapsed() < timeout_duration {
            // Try to read boot output instead of sending commands
            match self.read_boot_output(node_id).await {
                Ok(output) if output.contains("SIS Kernel") || output.contains("SHELL") || output.contains("HEAP") || output.contains("sis>") => {
                    log::info!("Instance {} booted successfully", node_id);
                    return Ok(true);
                }
                Ok(output) if !output.is_empty() => {
                    log::info!("Instance {} boot output: {}", node_id, output.chars().take(200).collect::<String>());
                }
                Ok(_) => {
                    // Empty output, continue waiting
                    log::debug!("Instance {} - no output yet", node_id);
                }
                Err(_) => {
                    log::debug!("Cannot connect to instance {} yet, waiting...", node_id);
                }
            }

            sleep(Duration::from_secs(3)).await;  // Slightly longer wait to reduce resource usage
        }

        log::warn!("Instance {} failed to boot within {} seconds", node_id, timeout_secs);
        Ok(false)
    }
}

impl Drop for QEMURuntimeManager {
    fn drop(&mut self) {
        // Ensure cleanup in case shutdown_cluster wasn't called
        for (node_id, mut process) in self.processes.drain() {
            log::debug!("Force terminating QEMU instance {} in drop", node_id);
            let _ = process.start_kill();
        }
    }
}