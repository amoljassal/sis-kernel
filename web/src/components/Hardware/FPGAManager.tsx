import React, { useState, useEffect } from 'react';
import { FPGADevice, CloudFPGAInstance, DeploymentJob, ConnectionStatus } from '../../types/hardware';
import { FPGAService } from '../../services/fpga';

interface FPGAManagerProps {
  className?: string;
}

const FPGAManager: React.FC<FPGAManagerProps> = ({ className = '' }) => {
  const [localDevices, setLocalDevices] = useState<FPGADevice[]>([]);
  const [cloudInstances, setCloudInstances] = useState<CloudFPGAInstance[]>([]);
  const [connectedDevices, setConnectedDevices] = useState<FPGADevice[]>([]);
  const [activeJobs, setActiveJobs] = useState<DeploymentJob[]>([]);
  const [isDiscovering, setIsDiscovering] = useState(false);
  const [selectedTab, setSelectedTab] = useState<'local' | 'cloud' | 'jobs'>('local');

  const fpgaService = FPGAService.getInstance();

  useEffect(() => {
    loadDevices();
    const interval = setInterval(() => {
      updateConnectedDevices();
      updateActiveJobs();
    }, 2000);
    return () => clearInterval(interval);
  }, []);

  const loadDevices = async () => {
    try {
      const [local, cloud] = await Promise.all([
        fpgaService.discoverLocalDevices(),
        fpgaService.getAvailableCloudInstances()
      ]);
      setLocalDevices(local);
      setCloudInstances(cloud);
    } catch (error) {
      console.error('Failed to load devices:', error);
    }
  };

  const updateConnectedDevices = () => {
    setConnectedDevices(fpgaService.getConnectedDevices());
  };

  const updateActiveJobs = () => {
    setActiveJobs(fpgaService.getActiveJobs());
  };

  const handleDiscoverDevices = async () => {
    setIsDiscovering(true);
    try {
      const devices = await fpgaService.discoverLocalDevices();
      setLocalDevices(devices);
    } finally {
      setIsDiscovering(false);
    }
  };

  const handleConnectDevice = async (deviceId: string) => {
    try {
      await fpgaService.connectToDevice(deviceId);
      updateConnectedDevices();
    } catch (error) {
      console.error('Failed to connect to device:', error);
    }
  };

  const handleDisconnectDevice = async (deviceId: string) => {
    try {
      await fpgaService.disconnectDevice(deviceId);
      updateConnectedDevices();
    } catch (error) {
      console.error('Failed to disconnect device:', error);
    }
  };

  const handleProvisionCloudInstance = async (instanceId: string) => {
    try {
      await fpgaService.provisionCloudInstance(instanceId);
      updateConnectedDevices();
    } catch (error) {
      console.error('Failed to provision cloud instance:', error);
    }
  };

  const getStatusColor = (status: ConnectionStatus): string => {
    switch (status) {
      case 'connected': return 'text-green-400';
      case 'connecting': return 'text-yellow-400';
      case 'error': return 'text-red-400';
      case 'busy': return 'text-purple-400';
      default: return 'text-sis-gray-400';
    }
  };

  const formatCapabilities = (device: FPGADevice): string => {
    const caps = device.capabilities;
    return `${caps.logic_cells.toLocaleString()} LCs, ${caps.block_ram_kb}KB RAM, ${caps.dsp_blocks} DSP`;
  };

  const DeviceCard: React.FC<{ device: FPGADevice; isCloud?: boolean }> = ({ device, isCloud = false }) => (
    <div className="card p-4 space-y-3">
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <h3 className="font-medium text-white">{device.name}</h3>
          <p className="text-sm text-sis-gray-400">{device.vendor} {device.family} • {device.part_number}</p>
          <p className="text-xs text-sis-gray-500 mt-1">{formatCapabilities(device)}</p>
        </div>
        <div className="flex flex-col items-end space-y-1">
          <div className={`text-xs font-medium ${getStatusColor(device.status)}`}>
            {device.status.toUpperCase()}
          </div>
          {device.temperature_c && (
            <div className="text-xs text-sis-gray-400">{device.temperature_c.toFixed(1)}°C</div>
          )}
        </div>
      </div>

      {device.utilization && (
        <div className="space-y-2">
          <div className="text-xs text-sis-gray-400">Resource Utilization</div>
          {Object.entries(device.utilization).map(([key, value]) => (
            <div key={key} className="flex items-center space-x-2">
              <div className="text-xs text-sis-gray-500 w-16 capitalize">
                {key.replace('_percent', '')}
              </div>
              <div className="flex-1 bg-sis-gray-700 rounded-full h-1.5">
                <div 
                  className="bg-sis-blue-500 h-1.5 rounded-full"
                  style={{ width: `${value}%` }}
                />
              </div>
              <div className="text-xs text-sis-gray-400 w-8 text-right">{value.toFixed(0)}%</div>
            </div>
          ))}
        </div>
      )}

      <div className="flex space-x-2">
        {(device.status === 'disconnected' || device.status === 'connecting') ? (
          <button
            onClick={() => isCloud ? handleProvisionCloudInstance(device.id) : handleConnectDevice(device.id)}
            className="flex-1 btn-primary text-xs py-1"
            disabled={device.status === 'connecting'}
          >
            {device.status === 'connecting' ? 'Connecting...' : (isCloud ? 'Provision' : 'Connect')}
          </button>
        ) : (
          <button
            onClick={() => handleDisconnectDevice(device.id)}
            className="flex-1 btn-secondary text-xs py-1"
            disabled={device.status === 'busy'}
          >
            {device.status === 'busy' ? 'Busy' : 'Disconnect'}
          </button>
        )}
        
        {device.status === 'connected' && (
          <button className="btn-primary text-xs py-1 px-3">
            Deploy
          </button>
        )}
      </div>
    </div>
  );

  const JobCard: React.FC<{ job: DeploymentJob }> = ({ job }) => (
    <div className="card p-4 space-y-3">
      <div className="flex items-start justify-between">
        <div>
          <h3 className="font-medium text-white">Job {job.id.split('_')[1]}</h3>
          <p className="text-sm text-sis-gray-400">Design: {job.design_id}</p>
          <p className="text-xs text-sis-gray-500">
            Target: {(job.target_device as FPGADevice).name}
          </p>
        </div>
        <div className="text-right">
          <div className={`text-xs font-medium ${
            job.state === 'deployed' ? 'text-green-400' :
            job.state === 'failed' ? 'text-red-400' :
            'text-yellow-400'
          }`}>
            {job.state.toUpperCase()}
          </div>
          <div className="text-xs text-sis-gray-400 mt-1">
            {job.progress_percent}%
          </div>
        </div>
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between text-xs">
          <span className="text-sis-gray-400">Progress</span>
          <span className="text-sis-gray-300">{job.progress_percent}%</span>
        </div>
        <div className="w-full bg-sis-gray-700 rounded-full h-2">
          <div 
            className={`h-2 rounded-full transition-all duration-300 ${
              job.state === 'failed' ? 'bg-red-500' : 
              job.state === 'deployed' ? 'bg-green-500' : 'bg-sis-blue-500'
            }`}
            style={{ width: `${job.progress_percent}%` }}
          />
        </div>
      </div>

      {job.error_message && (
        <div className="text-xs text-red-400 bg-red-400/10 p-2 rounded">
          {job.error_message}
        </div>
      )}

      {job.synthesis_report && (
        <div className="text-xs space-y-1">
          <div className="text-sis-gray-400">Synthesis Report:</div>
          <div className={`${job.synthesis_report.timing_met ? 'text-green-400' : 'text-red-400'}`}>
            Timing: {job.synthesis_report.timing_met ? 'MET' : 'FAILED'}
          </div>
          <div className="text-sis-gray-300">
            Max Freq: {job.synthesis_report.max_frequency_achieved_mhz.toFixed(1)} MHz
          </div>
        </div>
      )}

      <div className="text-xs text-sis-gray-500">
        Started: {job.started_at.toLocaleTimeString()}
        {job.completed_at && ` • Completed: ${job.completed_at.toLocaleTimeString()}`}
      </div>
    </div>
  );

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold text-white">FPGA Manager</h2>
        <div className="flex items-center space-x-2">
          <div className="text-sm text-sis-gray-400">
            {connectedDevices.length} connected
          </div>
          <button
            onClick={handleDiscoverDevices}
            disabled={isDiscovering}
            className="btn-secondary text-sm px-3 py-1"
          >
            {isDiscovering ? 'Discovering...' : 'Refresh'}
          </button>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex space-x-1 bg-sis-gray-800 p-1 rounded-lg">
        {[
          { key: 'local', label: 'Local Devices', count: localDevices.length },
          { key: 'cloud', label: 'Cloud FPGAs', count: cloudInstances.length },
          { key: 'jobs', label: 'Deployment Jobs', count: activeJobs.length }
        ].map(tab => (
          <button
            key={tab.key}
            onClick={() => setSelectedTab(tab.key as any)}
            className={`flex-1 px-3 py-2 text-sm font-medium rounded-md transition-colors ${
              selectedTab === tab.key
                ? 'bg-sis-blue-600 text-white'
                : 'text-sis-gray-300 hover:text-white hover:bg-sis-gray-700'
            }`}
          >
            {tab.label} ({tab.count})
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="space-y-3">
        {selectedTab === 'local' && (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            {localDevices.map(device => (
              <DeviceCard key={device.id} device={device} />
            ))}
            {localDevices.length === 0 && (
              <div className="col-span-2 text-center py-8 text-sis-gray-400">
                No local devices found. Connect FPGA boards via USB/JTAG.
              </div>
            )}
          </div>
        )}

        {selectedTab === 'cloud' && (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            {cloudInstances.map(instance => (
              <div key={instance.id} className="space-y-2">
                <div className="card p-3">
                  <div className="flex items-center justify-between text-xs mb-2">
                    <span className="text-sis-gray-400">{instance.provider.toUpperCase()}</span>
                    <span className="text-green-400">${instance.cost_per_hour}/hr</span>
                  </div>
                  <div className="text-sm font-medium text-white">{instance.instance_type}</div>
                  <div className="text-xs text-sis-gray-400">{instance.region}</div>
                  {instance.spot_instance && (
                    <div className="text-xs text-yellow-400">Spot Instance</div>
                  )}
                </div>
                <DeviceCard device={instance.device} isCloud />
              </div>
            ))}
          </div>
        )}

        {selectedTab === 'jobs' && (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            {activeJobs.map(job => (
              <JobCard key={job.id} job={job} />
            ))}
            {activeJobs.length === 0 && (
              <div className="col-span-2 text-center py-8 text-sis-gray-400">
                No active deployment jobs.
              </div>
            )}
          </div>
        )}
      </div>

      {/* Emergency Controls */}
      {connectedDevices.length > 0 && (
        <div className="card p-4 border-red-600/30">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-sm font-medium text-red-400">Emergency Controls</h3>
              <p className="text-xs text-sis-gray-400">Stop all operations immediately</p>
            </div>
            <button 
              className="btn-danger text-sm px-4 py-2"
              onClick={() => fpgaService.triggerEmergencyStop(
                connectedDevices.map(d => d.id), 
                'Manual emergency stop from FPGA Manager'
              )}
            >
              🚨 EMERGENCY STOP
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default FPGAManager;