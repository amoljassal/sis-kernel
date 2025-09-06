#![no_std]
#![no_main]

#[cfg(target_arch = "aarch64")]
#[link_section = ".text._start"]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    let uart = 0x0900_0000 as *mut u32;
    unsafe {
        let msg = b"KERNEL(U)\n";
        for &b in msg.iter() {
            core::ptr::write_volatile(uart, b as u32);
        }
    }
    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }

