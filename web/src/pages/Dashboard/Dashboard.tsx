import React from 'react'
import { useSelector } from 'react-redux'
import type { RootState } from '../../store/store'

const Dashboard: React.FC = () => {
  const { designName, nodes, connections } = useSelector((state: RootState) => state.designer)
  const { safetyMode } = useSelector((state: RootState) => state.settings)
  
  const stats = [
    {
      label: 'Active Designs',
      value: '1',
      icon: '📋',
      color: 'text-sis-blue-400'
    },
    {
      label: 'Components',
      value: nodes.length.toString(),
      icon: '🔧',
      color: 'text-green-400'
    },
    {
      label: 'Connections',
      value: connections.length.toString(),
      icon: '🔗',
      color: 'text-yellow-400'
    },
    {
      label: 'Safety Mode',
      value: safetyMode,
      icon: '🛡️',
      color: 'text-purple-400'
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
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {stats.map((stat, index) => (
          <div key={index} className="card p-6">
            <div className="flex items-center justify-between mb-4">
              <span className="text-2xl">{stat.icon}</span>
              <span className={`text-2xl font-bold ${stat.color}`}>
                {stat.value}
              </span>
            </div>
            <h3 className="text-sis-gray-300 font-medium">{stat.label}</h3>
          </div>
        ))}
      </div>
      
      {/* Quick start */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="card p-6">
          <h2 className="text-xl font-bold text-white mb-4">🚀 Quick Start</h2>
          <div className="space-y-3">
            <button className="w-full btn-primary text-left px-4 py-3">
              <div className="flex items-center space-x-3">
                <span className="text-lg">⚡</span>
                <div>
                  <div className="font-medium">Create New Design</div>
                  <div className="text-sm opacity-75">Start with a blank canvas</div>
                </div>
              </div>
            </button>
            <button className="w-full btn-secondary text-left px-4 py-3">
              <div className="flex items-center space-x-3">
                <span className="text-lg">📥</span>
                <div>
                  <div className="font-medium">Import Design</div>
                  <div className="text-sm opacity-75">Load existing project</div>
                </div>
              </div>
            </button>
            <button className="w-full btn-secondary text-left px-4 py-3">
              <div className="flex items-center space-x-3">
                <span className="text-lg">🛒</span>
                <div>
                  <div className="font-medium">Browse Marketplace</div>
                  <div className="text-sm opacity-75">Find IP blocks and templates</div>
                </div>
              </div>
            </button>
          </div>
        </div>
        
        <div className="card p-6">
          <h2 className="text-xl font-bold text-white mb-4">📊 Recent Activity</h2>
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
        <h2 className="text-xl font-bold text-white mb-6">✨ Key Features</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <div className="text-center p-4">
            <div className="text-3xl mb-3">🧠</div>
            <h3 className="font-semibold text-white mb-2">AI-Powered Design</h3>
            <p className="text-sm text-sis-gray-400">
              Natural language interface for hardware description
            </p>
          </div>
          <div className="text-center p-4">
            <div className="text-3xl mb-3">⚡</div>
            <h3 className="font-semibold text-white mb-2">Real-time Validation</h3>
            <p className="text-sm text-sis-gray-400">
              &lt;5 minute end-to-end validation cycles
            </p>
          </div>
          <div className="text-center p-4">
            <div className="text-3xl mb-3">🛡️</div>
            <h3 className="font-semibold text-white mb-2">Safety-First</h3>
            <p className="text-sm text-sis-gray-400">
              Comprehensive safety framework prevents hardware damage
            </p>
          </div>
          <div className="text-center p-4">
            <div className="text-3xl mb-3">🔧</div>
            <h3 className="font-semibold text-white mb-2">FPGA Integration</h3>
            <p className="text-sm text-sis-gray-400">
              Direct deployment to real hardware
            </p>
          </div>
          <div className="text-center p-4">
            <div className="text-3xl mb-3">🤝</div>
            <h3 className="font-semibold text-white mb-2">Real-time Collaboration</h3>
            <p className="text-sm text-sis-gray-400">
              Work together on designs with live cursors
            </p>
          </div>
          <div className="text-center p-4">
            <div className="text-3xl mb-3">📱</div>
            <h3 className="font-semibold text-white mb-2">Progressive Web App</h3>
            <p className="text-sm text-sis-gray-400">
              Works offline, installs like native app
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}

export default Dashboard