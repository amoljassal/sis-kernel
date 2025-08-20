import React, { useState } from 'react';
import FPGAManager from '../../components/Hardware/FPGAManager';
import HardwareMonitor from '../../components/Hardware/HardwareMonitor';
import DeploymentPipeline from '../../components/Hardware/DeploymentPipeline';

type HardwareTab = 'fpga' | 'monitor' | 'deploy';

const Hardware: React.FC = () => {
  const [activeTab, setActiveTab] = useState<HardwareTab>('fpga');

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-6 border-b border-sis-gray-700">
        <div>
          <h1 className="text-2xl font-bold text-white">Hardware Integration</h1>
          <p className="text-sis-gray-400">FPGA connectivity and deployment pipeline</p>
        </div>
        <div className="flex items-center space-x-2">
          <div className="w-2 h-2 bg-green-400 rounded-full"></div>
          <span className="text-sm text-sis-gray-400">Phase 4C: Hardware Integration</span>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex space-x-1 p-6 pb-0">
        {[
          { key: 'fpga', label: 'FPGA Manager', icon: '🔧' },
          { key: 'monitor', label: 'Hardware Monitor', icon: '📊' },
          { key: 'deploy', label: 'Deployment Pipeline', icon: '🚀' }
        ].map(tab => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key as HardwareTab)}
            className={`flex items-center space-x-2 px-4 py-2 text-sm font-medium rounded-t-lg transition-colors ${
              activeTab === tab.key
                ? 'bg-sis-gray-800 text-white border-b-2 border-sis-blue-500'
                : 'text-sis-gray-400 hover:text-white hover:bg-sis-gray-800/50'
            }`}
          >
            <span>{tab.icon}</span>
            <span>{tab.label}</span>
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 p-6 overflow-auto">
        {activeTab === 'fpga' && <FPGAManager />}
        {activeTab === 'monitor' && <HardwareMonitor />}
        {activeTab === 'deploy' && <DeploymentPipeline />}
      </div>
    </div>
  );
}

export default Hardware