import React, { useState } from 'react'
import DesignCanvas from '../../components/Designer/Canvas/DesignCanvas'
import ComponentPalette from '../../components/Designer/Palette/ComponentPalette'
import SafetyPanel from '../../components/Designer/Safety/SafetyPanel'
import ChipVisualization from '../../components/Designer/Visualization/ChipVisualization'
import DesignBrowser from '../../components/Designer/Browser/DesignBrowser'

// AI Training Laboratory Components
import AIArchitectureCanvas from '../../components/AILab/Designer/AIArchitectureCanvas'
import AIComponentPalette from '../../components/AILab/Designer/AIComponentPalette'
import AIEthicsPanel from '../../components/AILab/Designer/AIEthicsPanel'
import NeuralNetworkVisualization from '../../components/AILab/Designer/NeuralNetworkVisualization'
import ModelBrowser from '../../components/AILab/Designer/ModelBrowser'
import InteractiveTrainingController from '../../components/AILab/Canvas/InteractiveTrainingController'

type RightPanelTab = 'browser' | 'safety' | '3d' | 'training'
type DesignerMode = 'hardware' | 'ai'

const Designer: React.FC = () => {
  const [activeTab, setActiveTab] = useState<RightPanelTab>('browser')
  const [mode, setMode] = useState<DesignerMode>('ai') // Default to AI mode for training lab

  const tabs = mode === 'ai' ? [
    { id: 'browser' as const, name: 'Models', icon: 'M' },
    { id: 'training' as const, name: 'Training', icon: 'T' },
    { id: 'safety' as const, name: 'Ethics', icon: 'E' },
    { id: '3d' as const, name: 'Neural Viz', icon: 'N' },
  ] : [
    { id: 'browser' as const, name: 'Browser', icon: 'B' },
    { id: 'safety' as const, name: 'Safety', icon: 'S' },
    { id: '3d' as const, name: '3D View', icon: '3' },
  ]

  return (
    <div className="h-full flex">
      {/* Mode Toggle */}
      <div className="absolute top-4 right-4 z-50">
        <div className="bg-sis-gray-800 rounded-lg p-1 flex">
          <button
            onClick={() => setMode('ai')}
            className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
              mode === 'ai'
                ? 'bg-sis-blue-600 text-white'
                : 'text-sis-gray-400 hover:text-white'
            }`}
          >
            AI Lab
          </button>
          <button
            onClick={() => setMode('hardware')}
            className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
              mode === 'hardware'
                ? 'bg-sis-blue-600 text-white'
                : 'text-sis-gray-400 hover:text-white'
            }`}
          >
            Hardware
          </button>
        </div>
      </div>

      {/* Component Palette - Left Side */}
      {mode === 'ai' ? (
        <AIComponentPalette className="w-80 flex-shrink-0" />
      ) : (
        <ComponentPalette className="w-80 flex-shrink-0" />
      )}
      
      {/* Design Canvas - Main Area */}
      <div className="flex-1 relative">
        {mode === 'ai' ? (
          <AIArchitectureCanvas className="w-full h-full" />
        ) : (
          <DesignCanvas className="w-full h-full" />
        )}
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
          {mode === 'ai' ? (
            <>
              {activeTab === 'browser' && (
                <div className="h-full p-4">
                  <ModelBrowser className="h-full" />
                </div>
              )}
              
              {activeTab === 'training' && (
                <div className="h-full p-4">
                  <InteractiveTrainingController />
                </div>
              )}
              
              {activeTab === 'safety' && (
                <div className="h-full p-4">
                  <AIEthicsPanel className="h-full" />
                </div>
              )}
              
              {activeTab === '3d' && (
                <div className="h-full p-4">
                  <NeuralNetworkVisualization className="h-full" height={600} />
                </div>
              )}
            </>
          ) : (
            <>
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
            </>
          )}
        </div>
      </div>
    </div>
  )
}

export default Designer