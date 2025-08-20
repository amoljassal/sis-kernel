import React, { useState } from 'react'
import DesignCanvas from '../../components/Designer/Canvas/DesignCanvas'
import ComponentPalette from '../../components/Designer/Palette/ComponentPalette'
import SafetyPanel from '../../components/Designer/Safety/SafetyPanel'
import ChipVisualization from '../../components/Designer/Visualization/ChipVisualization'
import DesignBrowser from '../../components/Designer/Browser/DesignBrowser'

type RightPanelTab = 'browser' | 'safety' | '3d'

const Designer: React.FC = () => {
  const [activeTab, setActiveTab] = useState<RightPanelTab>('browser')

  const tabs = [
    { id: 'browser' as const, name: 'Browser', icon: '🌳' },
    { id: 'safety' as const, name: 'Safety', icon: '🛡️' },
    { id: '3d' as const, name: '3D View', icon: '🔷' },
  ]

  return (
    <div className="h-full flex">
      {/* Component Palette - Left Side */}
      <ComponentPalette className="w-80 flex-shrink-0" />
      
      {/* Design Canvas - Main Area */}
      <div className="flex-1 relative">
        <DesignCanvas className="w-full h-full" />
      </div>
      
      {/* Right Panel - Tabbed Interface */}
      <div className="w-80 flex-shrink-0 flex flex-col bg-sis-gray-900">
        {/* Tab Navigation */}
        <div className="flex border-b border-sis-gray-700">
          {tabs.map(tab => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex-1 px-3 py-2 text-sm font-medium transition-colors ${
                activeTab === tab.id
                  ? 'bg-sis-gray-800 text-white border-b-2 border-sis-blue-500'
                  : 'text-sis-gray-400 hover:text-white hover:bg-sis-gray-800'
              }`}
            >
              <div className="flex items-center justify-center space-x-1">
                <span>{tab.icon}</span>
                <span className="hidden sm:inline">{tab.name}</span>
              </div>
            </button>
          ))}
        </div>

        {/* Tab Content */}
        <div className="flex-1 overflow-hidden">
          {activeTab === 'browser' && (
            <div className="h-full p-4">
              <DesignBrowser className="h-full" />
            </div>
          )}
          
          {activeTab === 'safety' && (
            <div className="h-full p-4">
              <SafetyPanel className="h-full" />
            </div>
          )}
          
          {activeTab === '3d' && (
            <div className="h-full p-4">
              <ChipVisualization className="h-full" height={600} />
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default Designer