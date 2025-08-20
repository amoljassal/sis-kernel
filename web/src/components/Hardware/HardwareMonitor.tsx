import React, { useState, useEffect } from 'react';
import { HardwareMonitor as HardwareMonitorType, FPGADevice, SafetyCheck } from '../../types/hardware';
import { FPGAService } from '../../services/fpga';

interface HardwareMonitorProps {
  className?: string;
}

const HardwareMonitor: React.FC<HardwareMonitorProps> = ({ className = '' }) => {
  const [connectedDevices, setConnectedDevices] = useState<FPGADevice[]>([]);
  const [selectedDevice, setSelectedDevice] = useState<string | null>(null);
  const [monitorData, setMonitorData] = useState<HardwareMonitorType | null>(null);
  const [safetyChecks, setSafetyChecks] = useState<SafetyCheck[]>([]);
  const [isRunningChecks, setIsRunningChecks] = useState(false);

  const fpgaService = FPGAService.getInstance();

  useEffect(() => {
    const updateDevices = () => {
      const devices = fpgaService.getConnectedDevices();
      setConnectedDevices(devices);
      
      if (devices.length > 0 && !selectedDevice) {
        setSelectedDevice(devices[0].id);
      }
    };

    updateDevices();
    const interval = setInterval(updateDevices, 2000);
    return () => clearInterval(interval);
  }, [selectedDevice]);

  useEffect(() => {
    if (!selectedDevice) return;

    const updateMonitorData = async () => {
      try {
        const data = await fpgaService.getHardwareMonitor(selectedDevice);
        if (data) {
          setMonitorData(data);
        }
      } catch (error) {
        console.error('Failed to get hardware monitor data:', error);
      }
    };

    updateMonitorData();
    const interval = setInterval(updateMonitorData, 3000);
    return () => clearInterval(interval);
  }, [selectedDevice]);

  const handleRunSafetyChecks = async () => {
    if (!selectedDevice) return;

    setIsRunningChecks(true);
    try {
      const checks = await fpgaService.runSafetyChecks(selectedDevice);
      setSafetyChecks(checks);
    } catch (error) {
      console.error('Failed to run safety checks:', error);
    } finally {
      setIsRunningChecks(false);
    }
  };

  const getCriticalityColor = (criticality: SafetyCheck['criticality']): string => {
    switch (criticality) {
      case 'critical': return 'text-red-400 bg-red-400/10';
      case 'error': return 'text-red-300 bg-red-300/10';
      case 'warning': return 'text-yellow-400 bg-yellow-400/10';
      default: return 'text-sis-blue-400 bg-sis-blue-400/10';
    }
  };

  const getStatusColor = (status: SafetyCheck['status']): string => {
    switch (status) {
      case 'passed': return 'text-green-400';
      case 'failed': return 'text-red-400';
      case 'running': return 'text-yellow-400';
      default: return 'text-sis-gray-400';
    }
  };

  const getAlarmColor = (active: boolean): string => {
    return active ? 'text-red-400 bg-red-400/20' : 'text-green-400 bg-green-400/10';
  };

  const MetricCard: React.FC<{
    title: string;
    value: number;
    unit: string;
    warning?: boolean;
    critical?: boolean;
    icon: string;
  }> = ({ title, value, unit, warning = false, critical = false, icon }) => (
    <div className={`card p-4 ${critical ? 'border-red-600/50' : warning ? 'border-yellow-600/50' : ''}`}>
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center space-x-2">
          <span className="text-lg">{icon}</span>
          <h3 className="text-sm font-medium text-sis-gray-300">{title}</h3>
        </div>
        {(warning || critical) && (
          <div className={`w-2 h-2 rounded-full ${critical ? 'bg-red-400' : 'bg-yellow-400'}`} />
        )}
      </div>
      <div className={`text-2xl font-bold ${
        critical ? 'text-red-400' : warning ? 'text-yellow-400' : 'text-white'
      }`}>
        {value.toFixed(1)} <span className="text-sm font-normal text-sis-gray-400">{unit}</span>
      </div>
    </div>
  );

  if (connectedDevices.length === 0) {
    return (
      <div className={`card p-8 text-center ${className}`}>
        <div className="text-4xl mb-4">🔌</div>
        <h3 className="text-lg font-medium text-white mb-2">No Connected Devices</h3>
        <p className="text-sis-gray-400">Connect FPGA devices to monitor hardware status</p>
      </div>
    );
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold text-white">Hardware Monitor</h2>
        <div className="flex items-center space-x-4">
          {connectedDevices.length > 1 && (
            <select
              value={selectedDevice || ''}
              onChange={(e) => setSelectedDevice(e.target.value)}
              className="bg-sis-gray-800 border border-sis-gray-600 rounded-md px-3 py-1 text-sm text-white"
            >
              {connectedDevices.map(device => (
                <option key={device.id} value={device.id}>
                  {device.name}
                </option>
              ))}
            </select>
          )}
          <button
            onClick={handleRunSafetyChecks}
            disabled={isRunningChecks}
            className="btn-primary text-sm px-4 py-2"
          >
            {isRunningChecks ? 'Running Checks...' : '🛡️ Safety Check'}
          </button>
        </div>
      </div>

      {monitorData && (
        <>
          {/* Metrics Grid */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            <MetricCard
              title="Temperature"
              value={monitorData.metrics.temperature_c}
              unit="°C"
              warning={monitorData.alarms.thermal_warning}
              critical={monitorData.metrics.temperature_c > 80}
              icon="🌡️"
            />
            <MetricCard
              title="Power Consumption"
              value={monitorData.metrics.power_w}
              unit="W"
              warning={monitorData.alarms.power_budget_exceeded}
              critical={monitorData.metrics.power_w > 45}
              icon="⚡"
            />
            <MetricCard
              title="Voltage"
              value={monitorData.metrics.voltage_v}
              unit="V"
              icon="🔋"
            />
            <MetricCard
              title="Utilization"
              value={monitorData.metrics.utilization_percent}
              unit="%"
              warning={monitorData.metrics.utilization_percent > 85}
              critical={monitorData.metrics.utilization_percent > 95}
              icon="💾"
            />
          </div>

          {/* Real-time Chart Area */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Status Panel */}
            <div className="card p-4">
              <h3 className="text-lg font-medium text-white mb-4">System Status</h3>
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-sis-gray-300">Uptime</span>
                  <span className="text-sm font-mono text-white">
                    {Math.floor(monitorData.metrics.uptime_seconds / 3600)}h {Math.floor((monitorData.metrics.uptime_seconds % 3600) / 60)}m
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-sis-gray-300">Error Count</span>
                  <span className={`text-sm font-mono ${
                    monitorData.metrics.error_count > 0 ? 'text-red-400' : 'text-green-400'
                  }`}>
                    {monitorData.metrics.error_count}
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-sis-gray-300">Current Draw</span>
                  <span className="text-sm font-mono text-white">
                    {monitorData.metrics.current_a.toFixed(2)} A
                  </span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-sis-gray-300">Last Updated</span>
                  <span className="text-sm font-mono text-sis-gray-400">
                    {monitorData.timestamp.toLocaleTimeString()}
                  </span>
                </div>
              </div>
            </div>

            {/* Alarms Panel */}
            <div className="card p-4">
              <h3 className="text-lg font-medium text-white mb-4">Active Alarms</h3>
              <div className="space-y-3">
                {Object.entries(monitorData.alarms).map(([key, active]) => (
                  <div key={key} className={`flex items-center justify-between p-2 rounded text-xs ${getAlarmColor(active)}`}>
                    <span className="capitalize">{key.replace('_', ' ')}</span>
                    <span className="font-medium">{active ? 'ACTIVE' : 'OK'}</span>
                  </div>
                ))}
              </div>
              
              {Object.values(monitorData.alarms).some(Boolean) && (
                <div className="mt-4 pt-4 border-t border-sis-gray-700">
                  <button className="btn-danger text-sm w-full">
                    🚨 Acknowledge Alarms
                  </button>
                </div>
              )}
            </div>
          </div>
        </>
      )}

      {/* Safety Checks Results */}
      {safetyChecks.length > 0 && (
        <div className="card p-4">
          <h3 className="text-lg font-medium text-white mb-4">Safety Check Results</h3>
          <div className="space-y-3">
            {safetyChecks.map(check => (
              <div key={check.id} className={`p-3 rounded-lg ${getCriticalityColor(check.criticality)}`}>
                <div className="flex items-start justify-between mb-2">
                  <div className="flex-1">
                    <div className="flex items-center space-x-2">
                      <h4 className="font-medium">{check.name}</h4>
                      <span className={`text-xs px-2 py-1 rounded-full ${getStatusColor(check.status)}`}>
                        {check.status}
                      </span>
                    </div>
                    <p className="text-sm mt-1 opacity-90">{check.message}</p>
                    {check.details && (
                      <p className="text-xs mt-2 opacity-75">{check.details}</p>
                    )}
                  </div>
                  <div className="flex flex-col items-end space-y-1">
                    <span className="text-xs px-2 py-1 rounded capitalize bg-current/20">
                      {check.criticality}
                    </span>
                    {check.auto_fix_available && check.status === 'failed' && (
                      <button className="text-xs btn-primary px-2 py-1">
                        Auto Fix
                      </button>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>

          <div className="mt-4 pt-4 border-t border-sis-gray-700 flex items-center justify-between">
            <div className="text-sm text-sis-gray-400">
              {safetyChecks.filter(c => c.status === 'passed').length} passed, 
              {safetyChecks.filter(c => c.status === 'failed').length} failed
            </div>
            <button
              onClick={handleRunSafetyChecks}
              disabled={isRunningChecks}
              className="btn-secondary text-sm px-4 py-2"
            >
              Re-run Checks
            </button>
          </div>
        </div>
      )}

      {/* Historical Data Preview */}
      <div className="card p-4">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-medium text-white">Performance History</h3>
          <div className="flex space-x-2">
            <button className="text-xs btn-secondary px-3 py-1">1H</button>
            <button className="text-xs btn-secondary px-3 py-1">6H</button>
            <button className="text-xs btn-primary px-3 py-1">24H</button>
            <button className="text-xs btn-secondary px-3 py-1">7D</button>
          </div>
        </div>
        
        <div className="h-32 bg-sis-gray-800 rounded-lg flex items-center justify-center">
          <div className="text-center text-sis-gray-400">
            <div className="text-2xl mb-2">📊</div>
            <div className="text-sm">Historical charts would appear here</div>
            <div className="text-xs opacity-75">Temperature, Power, Utilization trends</div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default HardwareMonitor;