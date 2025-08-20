import React, { useState, useEffect } from 'react';
import { DeploymentJob, FPGADevice, SafetyCheck } from '../../types/hardware';
import { FPGAService } from '../../services/fpga';
// Mock useAppSelector for Redux compatibility
const useAppSelector = (selector: (state: any) => any) => {
  return selector({ designer: { designName: 'Mock Design' } });
};

interface DeploymentPipelineProps {
  className?: string;
  designId?: string;
}

const DeploymentPipeline: React.FC<DeploymentPipelineProps> = ({ className = '', designId }) => {
  const [connectedDevices, setConnectedDevices] = useState<FPGADevice[]>([]);
  const [selectedDevice, setSelectedDevice] = useState<string>('');
  const [deploymentConfig, setDeploymentConfig] = useState({
    clock_frequency_mhz: 100,
    optimization_target: 'speed' as 'speed' | 'area' | 'power',
    enable_debug_cores: false,
    two_person_approval_required: false
  });
  const [activeJob, setActiveJob] = useState<DeploymentJob | null>(null);
  const [safetyChecks, setSafetyChecks] = useState<SafetyCheck[]>([]);
  const [isDeploying, setIsDeploying] = useState(false);
  const [showAdvancedConfig, setShowAdvancedConfig] = useState(false);

  const { designName } = useAppSelector(state => state.designer);
  const fpgaService = FPGAService.getInstance();

  const currentDesignId = designId || designName || 'current_design';

  useEffect(() => {
    const updateDevices = () => {
      const devices = fpgaService.getConnectedDevices().filter(d => d.status === 'connected');
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
    if (!activeJob) return;

    const interval = setInterval(() => {
      const job = fpgaService.getJob(activeJob.id);
      if (job) {
        setActiveJob(job);
        if (job.state === 'deployed' || job.state === 'failed') {
          setIsDeploying(false);
        }
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [activeJob]);

  const handleRunPreflightChecks = async () => {
    if (!currentDesignId) return;

    try {
      const checks = await fpgaService.runSafetyChecks(currentDesignId);
      setSafetyChecks(checks);
    } catch (error) {
      console.error('Failed to run preflight checks:', error);
    }
  };

  const handleDeploy = async () => {
    if (!selectedDevice || !currentDesignId || isDeploying) return;

    // Check if safety checks have been run and passed critical items
    const criticalFailures = safetyChecks.filter(
      c => c.criticality === 'critical' && c.status === 'failed'
    );

    if (criticalFailures.length > 0) {
      alert(`Cannot deploy: ${criticalFailures.length} critical safety check(s) failed`);
      return;
    }

    setIsDeploying(true);
    
    try {
      const jobId = await fpgaService.deployDesign(currentDesignId, selectedDevice, deploymentConfig);
      const job = fpgaService.getJob(jobId);
      if (job) {
        setActiveJob(job);
      }
    } catch (error) {
      console.error('Deployment failed:', error);
      setIsDeploying(false);
    }
  };

  const getStageIcon = (state: DeploymentJob['state']): string => {
    switch (state) {
      case 'synthesizing': return '🔄';
      case 'placing': return '📐';
      case 'routing': return '🛤️';
      case 'bitstream': return '💾';
      case 'programming': return '📡';
      case 'deployed': return '✅';
      case 'failed': return '❌';
      default: return '⏸️';
    }
  };

  const getStageProgress = (state: DeploymentJob['state'], progress: number): number => {
    const stageProgress = {
      idle: 0,
      synthesizing: Math.min(progress, 20),
      placing: Math.min(progress, 50),
      routing: Math.min(progress, 75),
      bitstream: Math.min(progress, 90),
      programming: progress,
      deployed: 100,
      failed: progress
    };
    return stageProgress[state] || 0;
  };

  const selectedDeviceInfo = connectedDevices.find(d => d.id === selectedDevice);

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-xl font-bold text-white">Deployment Pipeline</h2>
          <p className="text-sm text-sis-gray-400">Deploy design: {currentDesignId}</p>
        </div>
        {!isDeploying && (
          <button
            onClick={handleRunPreflightChecks}
            className="btn-secondary text-sm px-4 py-2"
          >
            🛡️ Preflight Checks
          </button>
        )}
      </div>

      {connectedDevices.length === 0 ? (
        <div className="card p-8 text-center">
          <div className="text-4xl mb-4">🔌</div>
          <h3 className="text-lg font-medium text-white mb-2">No Connected Devices</h3>
          <p className="text-sis-gray-400">Connect FPGA devices to enable deployment</p>
        </div>
      ) : (
        <>
          {/* Device Selection */}
          <div className="card p-4">
            <h3 className="text-lg font-medium text-white mb-4">Target Device</h3>
            <div className="space-y-4">
              <select
                value={selectedDevice}
                onChange={(e) => setSelectedDevice(e.target.value)}
                disabled={isDeploying}
                className="w-full bg-sis-gray-800 border border-sis-gray-600 rounded-md px-3 py-2 text-white"
              >
                {connectedDevices.map(device => (
                  <option key={device.id} value={device.id}>
                    {device.name} ({device.vendor} {device.family})
                  </option>
                ))}
              </select>
              
              {selectedDeviceInfo && (
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                  <div>
                    <span className="text-sis-gray-400">Logic Cells:</span>
                    <div className="font-mono text-white">
                      {selectedDeviceInfo.capabilities.logic_cells.toLocaleString()}
                    </div>
                  </div>
                  <div>
                    <span className="text-sis-gray-400">Block RAM:</span>
                    <div className="font-mono text-white">
                      {selectedDeviceInfo.capabilities.block_ram_kb} KB
                    </div>
                  </div>
                  <div>
                    <span className="text-sis-gray-400">DSP Blocks:</span>
                    <div className="font-mono text-white">
                      {selectedDeviceInfo.capabilities.dsp_blocks}
                    </div>
                  </div>
                  <div>
                    <span className="text-sis-gray-400">Max Freq:</span>
                    <div className="font-mono text-white">
                      {selectedDeviceInfo.capabilities.max_frequency_mhz} MHz
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Deployment Configuration */}
          <div className="card p-4">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-medium text-white">Configuration</h3>
              <button
                onClick={() => setShowAdvancedConfig(!showAdvancedConfig)}
                className="text-sm text-sis-blue-400 hover:text-sis-blue-300"
              >
                {showAdvancedConfig ? 'Hide' : 'Show'} Advanced
              </button>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {/* Clock Frequency */}
              <div>
                <label className="label">Clock Frequency (MHz)</label>
                <input
                  type="number"
                  min="1"
                  max={selectedDeviceInfo?.capabilities.max_frequency_mhz || 1000}
                  value={deploymentConfig.clock_frequency_mhz}
                  onChange={(e) => setDeploymentConfig(prev => ({
                    ...prev,
                    clock_frequency_mhz: parseInt(e.target.value) || 100
                  }))}
                  disabled={isDeploying}
                  className="input"
                />
              </div>

              {/* Optimization Target */}
              <div>
                <label className="label">Optimization Target</label>
                <select
                  value={deploymentConfig.optimization_target}
                  onChange={(e) => setDeploymentConfig(prev => ({
                    ...prev,
                    optimization_target: e.target.value as any
                  }))}
                  disabled={isDeploying}
                  className="input"
                >
                  <option value="speed">Speed (Performance)</option>
                  <option value="area">Area (Size)</option>
                  <option value="power">Power (Efficiency)</option>
                </select>
              </div>
            </div>

            {showAdvancedConfig && (
              <div className="mt-4 space-y-3">
                <div className="flex items-center space-x-2">
                  <input
                    type="checkbox"
                    id="debug_cores"
                    checked={deploymentConfig.enable_debug_cores}
                    onChange={(e) => setDeploymentConfig(prev => ({
                      ...prev,
                      enable_debug_cores: e.target.checked
                    }))}
                    disabled={isDeploying}
                    className="rounded text-sis-blue-600"
                  />
                  <label htmlFor="debug_cores" className="text-sm text-sis-gray-300">
                    Enable Debug Cores (ChipScope/SignalTap)
                  </label>
                </div>
                
                <div className="flex items-center space-x-2">
                  <input
                    type="checkbox"
                    id="two_person_approval"
                    checked={deploymentConfig.two_person_approval_required}
                    onChange={(e) => setDeploymentConfig(prev => ({
                      ...prev,
                      two_person_approval_required: e.target.checked
                    }))}
                    disabled={isDeploying}
                    className="rounded text-sis-blue-600"
                  />
                  <label htmlFor="two_person_approval" className="text-sm text-sis-gray-300">
                    Require Two-Person Approval for Production
                  </label>
                </div>
              </div>
            )}
          </div>

          {/* Safety Checks Results */}
          {safetyChecks.length > 0 && (
            <div className="card p-4">
              <h3 className="text-lg font-medium text-white mb-4">Preflight Safety Checks</h3>
              <div className="space-y-2">
                {safetyChecks.map(check => (
                  <div key={check.id} className="flex items-center justify-between p-3 bg-sis-gray-800 rounded-lg">
                    <div className="flex items-center space-x-3">
                      <div className={`w-2 h-2 rounded-full ${
                        check.status === 'passed' ? 'bg-green-400' :
                        check.status === 'failed' ? 'bg-red-400' : 'bg-yellow-400'
                      }`} />
                      <div>
                        <div className="text-sm font-medium text-white">{check.name}</div>
                        <div className="text-xs text-sis-gray-400">{check.message}</div>
                      </div>
                    </div>
                    <div className="flex items-center space-x-2">
                      <span className={`text-xs px-2 py-1 rounded-full ${
                        check.criticality === 'critical' ? 'bg-red-400/20 text-red-400' :
                        check.criticality === 'error' ? 'bg-red-300/20 text-red-300' :
                        check.criticality === 'warning' ? 'bg-yellow-400/20 text-yellow-400' :
                        'bg-sis-blue-400/20 text-sis-blue-400'
                      }`}>
                        {check.criticality}
                      </span>
                      {check.auto_fix_available && check.status === 'failed' && (
                        <button className="text-xs btn-primary px-2 py-1">Fix</button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Deployment Progress */}
          {activeJob && (
            <div className="card p-4">
              <h3 className="text-lg font-medium text-white mb-4">Deployment Progress</h3>
              
              {/* Progress Bar */}
              <div className="mb-6">
                <div className="flex items-center justify-between text-sm mb-2">
                  <span className="text-sis-gray-300">
                    {getStageIcon(activeJob.state)} {activeJob.state.replace('_', ' ').toUpperCase()}
                  </span>
                  <span className="text-white font-mono">{activeJob.progress_percent}%</span>
                </div>
                <div className="w-full bg-sis-gray-700 rounded-full h-3">
                  <div 
                    className={`h-3 rounded-full transition-all duration-300 ${
                      activeJob.state === 'failed' ? 'bg-red-500' :
                      activeJob.state === 'deployed' ? 'bg-green-500' : 'bg-sis-blue-500'
                    }`}
                    style={{ width: `${activeJob.progress_percent}%` }}
                  />
                </div>
              </div>

              {/* Pipeline Stages */}
              <div className="grid grid-cols-2 md:grid-cols-5 gap-4 mb-4">
                {['synthesizing', 'placing', 'routing', 'bitstream', 'programming'].map((stage, index) => (
                  <div key={stage} className="text-center">
                    <div className={`w-8 h-8 rounded-full mx-auto mb-2 flex items-center justify-center text-sm ${
                      getStageProgress(activeJob.state, activeJob.progress_percent) > index * 20 
                        ? 'bg-sis-blue-600 text-white' 
                        : 'bg-sis-gray-700 text-sis-gray-400'
                    }`}>
                      {index + 1}
                    </div>
                    <div className="text-xs text-sis-gray-400 capitalize">{stage}</div>
                  </div>
                ))}
              </div>

              {/* Job Info */}
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-sis-gray-400">Job ID:</span>
                  <div className="font-mono text-white">{activeJob.id}</div>
                </div>
                <div>
                  <span className="text-sis-gray-400">Started:</span>
                  <div className="font-mono text-white">{activeJob.started_at.toLocaleTimeString()}</div>
                </div>
              </div>

              {/* Error Message */}
              {activeJob.error_message && (
                <div className="mt-4 p-3 bg-red-400/10 border border-red-400/30 rounded-lg">
                  <div className="text-red-400 text-sm font-medium mb-1">Deployment Failed</div>
                  <div className="text-red-300 text-sm">{activeJob.error_message}</div>
                </div>
              )}

              {/* Synthesis Report */}
              {activeJob.synthesis_report && (
                <div className="mt-4 p-3 bg-sis-gray-800 rounded-lg">
                  <div className="text-white text-sm font-medium mb-2">Synthesis Report</div>
                  <div className="grid grid-cols-2 gap-4 text-xs">
                    <div>
                      <span className="text-sis-gray-400">Timing:</span>
                      <div className={activeJob.synthesis_report.timing_met ? 'text-green-400' : 'text-red-400'}>
                        {activeJob.synthesis_report.timing_met ? 'MET' : 'FAILED'}
                      </div>
                    </div>
                    <div>
                      <span className="text-sis-gray-400">Max Frequency:</span>
                      <div className="text-white">
                        {activeJob.synthesis_report.max_frequency_achieved_mhz.toFixed(1)} MHz
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Deploy Button */}
          {!activeJob || activeJob.state === 'deployed' || activeJob.state === 'failed' ? (
            <div className="flex justify-center">
              <button
                onClick={handleDeploy}
                disabled={isDeploying || !selectedDevice || safetyChecks.some(c => c.criticality === 'critical' && c.status === 'failed')}
                className="btn-primary px-8 py-3 text-lg"
              >
                {isDeploying ? 'Deploying...' : '🚀 Deploy to Hardware'}
              </button>
            </div>
          ) : (
            <div className="flex justify-center space-x-4">
              <button
                onClick={() => fpgaService.triggerEmergencyStop([selectedDevice], 'User requested deployment cancellation')}
                className="btn-danger px-6 py-2"
              >
                🚨 Emergency Stop
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
};

export default DeploymentPipeline;