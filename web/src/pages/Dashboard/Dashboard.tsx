import React, { useState, useEffect } from 'react'
import { useSelector } from 'react-redux'
import type { RootState } from '../../store/store'
import { AutoScalingDashboard } from '../../components/AutoScalingDashboard'
import { MultiModalAIInterface } from '../../components/MultiModalAIInterface'
import GlobalInfrastructureDashboard from '../../components/GlobalInfrastructureDashboard'
import { infrastructureIntegration } from '../../services/infrastructure-integration'
import type { InfrastructureStatus } from '../../services/infrastructure-integration'
import {
  ClipboardDocumentListIcon,
  WrenchIcon,
  LinkIcon,
  ShieldCheckIcon,
  ComputerDesktopIcon,
  CircleStackIcon,
  BoltIcon,
  RocketLaunchIcon,
  ChartBarIcon,
  SignalIcon,
  CpuChipIcon,
  PlusIcon,
  FolderOpenIcon,
  ShoppingCartIcon,
  GlobeAltIcon
} from '@heroicons/react/24/outline'

const Dashboard: React.FC = () => {
  const { designName, nodes, connections } = useSelector((state: RootState) => state.designer)
  const { safetyMode } = useSelector((state: RootState) => state.settings)
  
  // Auto-scaling dashboard state
  const [showAutoScalingDashboard, setShowAutoScalingDashboard] = useState(false)
  const [infrastructureStatus, setInfrastructureStatus] = useState<InfrastructureStatus | null>(null)
  
  // AI interface state
  const [showAIInterface, setShowAIInterface] = useState(false)
  
  // Global infrastructure state
  const [showGlobalInfrastructure, setShowGlobalInfrastructure] = useState(false)
  
  // Load infrastructure status
  useEffect(() => {
    const updateInfrastructureStatus = () => {
      setInfrastructureStatus(infrastructureIntegration.getInfrastructureStatus())
    }
    
    updateInfrastructureStatus()
    const interval = setInterval(updateInfrastructureStatus, 10000) // Update every 10 seconds
    
    return () => clearInterval(interval)
  }, [])
  
  const stats = [
    {
      label: 'Active Designs',
      value: '1',
      icon: ClipboardDocumentListIcon,
      color: 'text-sis-blue-400'
    },
    {
      label: 'Components',
      value: nodes.length.toString(),
      icon: WrenchIcon,
      color: 'text-green-400'
    },
    {
      label: 'Connections',
      value: connections.length.toString(),
      icon: LinkIcon,
      color: 'text-yellow-400'
    },
    {
      label: 'Safety Mode',
      value: safetyMode,
      icon: ShieldCheckIcon,
      color: 'text-purple-400'
    },
    {
      label: 'Web Servers',
      value: infrastructureStatus?.webServers.instances.toString() || '0',
      icon: ComputerDesktopIcon,
      color: 'text-blue-400'
    },
    {
      label: 'Database Replicas',
      value: infrastructureStatus?.database.primary.instances.toString() || '0',
      icon: CircleStackIcon,
      color: 'text-emerald-400'
    },
    {
      label: 'Active Connections',
      value: infrastructureStatus?.loadBalancer.activeConnections.toString() || '0',
      icon: SignalIcon,
      color: 'text-orange-400'
    },
    {
      label: 'Response Time',
      value: infrastructureStatus ? `${Math.round(infrastructureStatus.webServers.responseTime)}ms` : '0ms',
      icon: BoltIcon,
      color: infrastructureStatus && infrastructureStatus.webServers.responseTime < 100 ? 'text-green-400' : 'text-yellow-400'
    },
  ]
  
  return (
    <div className="p-6 space-y-6">
      {/* Welcome header */}
      <div className="text-center py-12">
        <h1 className="text-4xl font-bold text-gradient mb-4">
          Welcome to SIS AI-Lab
        </h1>
        <p className="text-sis-gray-400 text-lg max-w-2xl mx-auto">
          Hardware-Software Co-Design Platform with Natural Language Interface.
          Design, validate, and deploy your ideas with AI-powered assistance.
        </p>
      </div>
      
      {/* Stats grid */}
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-8 gap-4">
        {stats.map((stat, index) => {
          const IconComponent = stat.icon;
          return (
            <div key={index} className="card p-6">
              <div className="flex items-center justify-between mb-4">
                <IconComponent className="w-8 h-8 text-gray-400" />
                <span className={`text-2xl font-bold ${stat.color}`}>
                  {stat.value}
                </span>
              </div>
              <h3 className="text-sis-gray-300 font-medium">{stat.label}</h3>
            </div>
          );
        })}
      </div>
      
      {/* Infrastructure Status */}
      {infrastructureStatus && (
        <div className="card p-6">
          <div className="flex justify-between items-center mb-6">
            <div className="flex items-center space-x-3">
              <RocketLaunchIcon className="w-6 h-6 text-blue-400" />
              <h2 className="text-xl font-bold text-white">Phase 5C Infrastructure Status</h2>
            </div>
            <button
              onClick={() => setShowAutoScalingDashboard(true)}
              className="btn-primary px-4 py-2 text-sm flex items-center space-x-2"
            >
              <ChartBarIcon className="w-4 h-4" />
              <span>Auto-Scaling Dashboard</span>
            </button>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {/* Database Status */}
            <div className="bg-gray-800 rounded-lg p-4">
              <h3 className="font-semibold text-white mb-3 flex items-center">
                <CircleStackIcon className="w-5 h-5 mr-2 text-emerald-400" />
                Database Cluster
              </h3>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-400">Primary:</span>
                  <span className="text-white">{infrastructureStatus.database.primary.instances} instances</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Connections:</span>
                  <span className="text-white">{infrastructureStatus.database.primary.connections}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Status:</span>
                  <span className="text-green-400">Mumbai Primary</span>
                </div>
              </div>
            </div>
            
            {/* Redis Cache Status */}
            <div className="bg-gray-800 rounded-lg p-4">
              <h3 className="font-semibold text-white mb-3 flex items-center">
                <BoltIcon className="w-5 h-5 mr-2 text-yellow-400" />
                Multi-Layer Cache
              </h3>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-400">L1 Hit Rate:</span>
                  <span className="text-green-400">{(infrastructureStatus.redis.l1.hitRate * 100).toFixed(1)}%</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">L2 Instances:</span>
                  <span className="text-white">{infrastructureStatus.redis.l2.instances}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Memory Usage:</span>
                  <span className="text-yellow-400">{(infrastructureStatus.redis.l1.memory * 100).toFixed(0)}%</span>
                </div>
              </div>
            </div>
            
            {/* WebSocket Status */}
            <div className="bg-gray-800 rounded-lg p-4">
              <h3 className="font-semibold text-white mb-3 flex items-center">
                <SignalIcon className="w-5 h-5 mr-2 text-blue-400" />
                Real-time Collaboration
              </h3>
              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-400">Mumbai Gateway:</span>
                  <span className="text-white">{infrastructureStatus.websocket.gateways.mumbai?.connections || 0} users</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Kafka Brokers:</span>
                  <span className="text-white">{infrastructureStatus.websocket.kafka.brokers}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Message Lag:</span>
                  <span className="text-green-400">{infrastructureStatus.websocket.kafka.lag}ms</span>
                </div>
              </div>
            </div>
          </div>
          
          {/* Peak Hours Indicator */}
          <div className="mt-6 p-4 bg-gradient-to-r from-purple-900/50 to-indigo-900/50 rounded-lg">
            <div className="flex items-center justify-between">
              <div>
                <h4 className="font-semibold text-white">Indian Peak Hours Auto-Scaling</h4>
                <p className="text-sm text-gray-300">Optimized for 9 AM - 11 PM IST traffic patterns</p>
              </div>
              <div className="text-right">
                <div className="text-2xl font-bold text-green-400">
                  {infrastructureStatus.webServers.instances + 
                   infrastructureStatus.database.primary.instances + 
                   Object.values(infrastructureStatus.websocket.gateways).reduce((sum, gw) => sum + gw.instances, 0)}
                </div>
                <div className="text-sm text-gray-400">Total Instances</div>
              </div>
            </div>
          </div>
        </div>
      )}
      
      {/* Quick start */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="card p-6">
          <div className="flex items-center space-x-3 mb-4">
            <RocketLaunchIcon className="w-6 h-6 text-blue-400" />
            <h2 className="text-xl font-bold text-white">Quick Start</h2>
          </div>
          <div className="space-y-3">
            <button className="w-full btn-primary text-left px-4 py-3">
              <div className="flex items-center space-x-3">
                <PlusIcon className="w-5 h-5" />
                <div>
                  <div className="font-medium">Create New Design</div>
                  <div className="text-sm opacity-75">Start with a blank canvas</div>
                </div>
              </div>
            </button>
            <button className="w-full btn-secondary text-left px-4 py-3">
              <div className="flex items-center space-x-3">
                <FolderOpenIcon className="w-5 h-5" />
                <div>
                  <div className="font-medium">Import Design</div>
                  <div className="text-sm opacity-75">Load existing project</div>
                </div>
              </div>
            </button>
            <button className="w-full btn-secondary text-left px-4 py-3">
              <div className="flex items-center space-x-3">
                <ShoppingCartIcon className="w-5 h-5" />
                <div>
                  <div className="font-medium">Browse Marketplace</div>
                  <div className="text-sm opacity-75">Find IP blocks and templates</div>
                </div>
              </div>
            </button>
            <button 
              onClick={() => setShowAIInterface(true)}
              className="w-full bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-700 hover:to-indigo-700 text-white px-4 py-3 rounded-lg transition-colors text-left"
            >
              <div className="flex items-center space-x-3">
                <CpuChipIcon className="w-5 h-5" />
                <div>
                  <div className="font-medium">AI Assistant</div>
                  <div className="text-sm opacity-90">Voice, sketch & natural language design</div>
                </div>
              </div>
            </button>
            <button 
              onClick={() => setShowGlobalInfrastructure(true)}
              className="w-full bg-gradient-to-r from-blue-600 to-cyan-600 hover:from-blue-700 hover:to-cyan-700 text-white px-4 py-3 rounded-lg transition-colors text-left"
            >
              <div className="flex items-center space-x-3">
                <GlobeAltIcon className="w-5 h-5" />
                <div>
                  <div className="font-medium">Global Infrastructure</div>
                  <div className="text-sm opacity-90">Multi-region deployment & CDN</div>
                </div>
              </div>
            </button>
          </div>
        </div>
        
        <div className="card p-6">
          <div className="flex items-center space-x-3 mb-4">
            <ChartBarIcon className="w-6 h-6 text-green-400" />
            <h2 className="text-xl font-bold text-white">Recent Activity</h2>
          </div>
          <div className="space-y-3">
            <div className="flex items-center space-x-3 text-sm">
              <div className="w-2 h-2 bg-green-400 rounded-full" />
              <span className="text-sis-gray-300">Design validation completed</span>
              <span className="text-sis-gray-500 ml-auto">2 min ago</span>
            </div>
            <div className="flex items-center space-x-3 text-sm">
              <div className="w-2 h-2 bg-sis-blue-400 rounded-full" />
              <span className="text-sis-gray-300">New design created: {designName}</span>
              <span className="text-sis-gray-500 ml-auto">5 min ago</span>
            </div>
            <div className="flex items-center space-x-3 text-sm">
              <div className="w-2 h-2 bg-yellow-400 rounded-full" />
              <span className="text-sis-gray-300">Safety framework initialized</span>
              <span className="text-sis-gray-500 ml-auto">10 min ago</span>
            </div>
          </div>
        </div>
      </div>
      
      {/* Feature highlights */}
      <div className="card p-6">
        <div className="flex items-center space-x-3 mb-6">
          <CpuChipIcon className="w-6 h-6 text-purple-400" />
          <h2 className="text-xl font-bold text-white">Key Features</h2>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <div className="text-center p-4">
            <CpuChipIcon className="w-12 h-12 mx-auto mb-3 text-blue-400" />
            <h3 className="font-semibold text-white mb-2">AI-Powered Design</h3>
            <p className="text-sm text-sis-gray-400">
              Natural language interface for hardware description
            </p>
          </div>
          <div className="text-center p-4">
            <BoltIcon className="w-12 h-12 mx-auto mb-3 text-yellow-400" />
            <h3 className="font-semibold text-white mb-2">Real-time Validation</h3>
            <p className="text-sm text-sis-gray-400">
              Less than 5 minute end-to-end validation cycles
            </p>
          </div>
          <div className="text-center p-4">
            <ShieldCheckIcon className="w-12 h-12 mx-auto mb-3 text-green-400" />
            <h3 className="font-semibold text-white mb-2">Safety-First</h3>
            <p className="text-sm text-sis-gray-400">
              Comprehensive safety framework prevents hardware damage
            </p>
          </div>
          <div className="text-center p-4">
            <WrenchIcon className="w-12 h-12 mx-auto mb-3 text-orange-400" />
            <h3 className="font-semibold text-white mb-2">FPGA Integration</h3>
            <p className="text-sm text-sis-gray-400">
              Direct deployment to real hardware
            </p>
          </div>
          <div className="text-center p-4">
            <SignalIcon className="w-12 h-12 mx-auto mb-3 text-purple-400" />
            <h3 className="font-semibold text-white mb-2">Real-time Collaboration</h3>
            <p className="text-sm text-sis-gray-400">
              Work together on designs with live cursors
            </p>
          </div>
          <div className="text-center p-4">
            <ComputerDesktopIcon className="w-12 h-12 mx-auto mb-3 text-indigo-400" />
            <h3 className="font-semibold text-white mb-2">Progressive Web App</h3>
            <p className="text-sm text-sis-gray-400">
              Works offline, installs like native app
            </p>
          </div>
        </div>
      </div>
      
      {/* Auto-Scaling Dashboard Modal */}
      <AutoScalingDashboard
        isVisible={showAutoScalingDashboard}
        onClose={() => setShowAutoScalingDashboard(false)}
      />
      
      {/* AI Interface Modal */}
      <MultiModalAIInterface
        isVisible={showAIInterface}
        onClose={() => setShowAIInterface(false)}
        onCodeGenerated={(code, explanation) => {
          console.log('AI generated code:', { code, explanation });
          // You can integrate this with your designer state
          // For now, just log the generated code
        }}
      />
      
      {/* Global Infrastructure Modal */}
      <GlobalInfrastructureDashboard
        isOpen={showGlobalInfrastructure}
        onClose={() => setShowGlobalInfrastructure(false)}
      />
    </div>
  )
}

export default Dashboard