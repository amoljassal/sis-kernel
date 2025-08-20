import React from 'react'

interface ComponentPaletteProps {
  className?: string
}

interface ComponentType {
  id: string
  name: string
  icon: string
  description: string
  category: 'Processors' | 'Memory' | 'I/O' | 'Logic' | 'AI/ML' | 'Interfaces' | 'Custom'
  difficulty: 'Beginner' | 'Intermediate' | 'Advanced' | 'Expert'
  powerEstimate?: number // mW
  gateEstimate?: number
}

const componentLibrary: ComponentType[] = [
  // Processors
  {
    id: 'cpu',
    name: 'CPU Core',
    icon: '🧠',
    description: 'General-purpose processor core with RISC-V architecture',
    category: 'Processors',
    difficulty: 'Intermediate',
    powerEstimate: 250,
    gateEstimate: 50000,
  },
  {
    id: 'dsp',
    name: 'DSP Core',
    icon: '📊',
    description: 'Digital Signal Processor for audio/video processing',
    category: 'Processors',
    difficulty: 'Advanced',
    powerEstimate: 150,
    gateEstimate: 25000,
  },
  {
    id: 'mcu',
    name: 'Microcontroller',
    icon: '🔧',
    description: 'Low-power microcontroller with integrated peripherals',
    category: 'Processors',
    difficulty: 'Beginner',
    powerEstimate: 50,
    gateEstimate: 10000,
  },
  
  // Memory
  {
    id: 'memory',
    name: 'SRAM Block',
    icon: '💾',
    description: 'Static Random Access Memory for fast data storage',
    category: 'Memory',
    difficulty: 'Beginner',
    powerEstimate: 30,
    gateEstimate: 1000,
  },
  {
    id: 'cache',
    name: 'Cache Memory',
    icon: '⚡',
    description: 'High-speed cache memory with configurable associativity',
    category: 'Memory',
    difficulty: 'Advanced',
    powerEstimate: 80,
    gateEstimate: 15000,
  },
  {
    id: 'rom',
    name: 'ROM Block',
    icon: '📚',
    description: 'Read-Only Memory for firmware and constants',
    category: 'Memory',
    difficulty: 'Beginner',
    powerEstimate: 5,
    gateEstimate: 500,
  },
  
  // I/O
  {
    id: 'io',
    name: 'GPIO Block',
    icon: '🔌',
    description: 'General Purpose Input/Output pins with configurable modes',
    category: 'I/O',
    difficulty: 'Beginner',
    powerEstimate: 10,
    gateEstimate: 200,
  },
  {
    id: 'uart',
    name: 'UART Controller',
    icon: '📡',
    description: 'Universal Asynchronous Receiver-Transmitter for serial communication',
    category: 'I/O',
    difficulty: 'Intermediate',
    powerEstimate: 15,
    gateEstimate: 800,
  },
  {
    id: 'spi',
    name: 'SPI Controller',
    icon: '🔄',
    description: 'Serial Peripheral Interface for high-speed device communication',
    category: 'I/O',
    difficulty: 'Intermediate',
    powerEstimate: 12,
    gateEstimate: 600,
  },
  {
    id: 'i2c',
    name: 'I2C Controller',
    icon: '🔗',
    description: 'Inter-Integrated Circuit bus for sensor and peripheral communication',
    category: 'I/O',
    difficulty: 'Intermediate',
    powerEstimate: 8,
    gateEstimate: 400,
  },
  
  // Logic
  {
    id: 'logic',
    name: 'Logic Gates',
    icon: '⚡',
    description: 'Configurable logic gates (AND, OR, XOR, NOT)',
    category: 'Logic',
    difficulty: 'Beginner',
    powerEstimate: 1,
    gateEstimate: 10,
  },
  {
    id: 'mux',
    name: 'Multiplexer',
    icon: '🔀',
    description: 'Data selector with configurable input width',
    category: 'Logic',
    difficulty: 'Beginner',
    powerEstimate: 5,
    gateEstimate: 50,
  },
  {
    id: 'alu',
    name: 'ALU',
    icon: '🧮',
    description: 'Arithmetic Logic Unit for mathematical operations',
    category: 'Logic',
    difficulty: 'Intermediate',
    powerEstimate: 40,
    gateEstimate: 2000,
  },
  
  // AI/ML
  {
    id: 'ai_accelerator',
    name: 'Neural Engine',
    icon: '🤖',
    description: 'Specialized hardware for neural network acceleration',
    category: 'AI/ML',
    difficulty: 'Expert',
    powerEstimate: 500,
    gateEstimate: 100000,
  },
  {
    id: 'tensor_core',
    name: 'Tensor Core',
    icon: '🧠',
    description: 'High-performance tensor processing unit for AI workloads',
    category: 'AI/ML',
    difficulty: 'Expert',
    powerEstimate: 300,
    gateEstimate: 75000,
  },
  
  // Interfaces
  {
    id: 'pcie',
    name: 'PCIe Controller',
    icon: '🚄',
    description: 'PCI Express interface for high-speed data transfer',
    category: 'Interfaces',
    difficulty: 'Expert',
    powerEstimate: 200,
    gateEstimate: 30000,
  },
  {
    id: 'usb',
    name: 'USB Controller',
    icon: '🔌',
    description: 'Universal Serial Bus interface for device connectivity',
    category: 'Interfaces',
    difficulty: 'Advanced',
    powerEstimate: 100,
    gateEstimate: 8000,
  },
]

const ComponentPalette: React.FC<ComponentPaletteProps> = ({ className = '' }) => {
  const [selectedCategory, setSelectedCategory] = React.useState<string>('All')
  const [searchQuery, setSearchQuery] = React.useState('')
  
  const categories = ['All', ...Array.from(new Set(componentLibrary.map(c => c.category)))]
  
  const filteredComponents = componentLibrary.filter(component => {
    const matchesCategory = selectedCategory === 'All' || component.category === selectedCategory
    const matchesSearch = component.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         component.description.toLowerCase().includes(searchQuery.toLowerCase())
    return matchesCategory && matchesSearch
  })
  
  const onDragStart = (event: React.DragEvent, componentType: string) => {
    event.dataTransfer.setData('application/reactflow', componentType)
    event.dataTransfer.effectAllowed = 'move'
  }
  
  const getDifficultyColor = (difficulty: string) => {
    switch (difficulty) {
      case 'Beginner': return 'text-green-400'
      case 'Intermediate': return 'text-yellow-400'
      case 'Advanced': return 'text-orange-400'
      case 'Expert': return 'text-red-400'
      default: return 'text-sis-gray-400'
    }
  }
  
  return (
    <div className={`bg-sis-gray-800 border-r border-sis-gray-700 ${className}`}>
      {/* Header */}
      <div className="p-4 border-b border-sis-gray-700">
        <h2 className="text-lg font-semibold text-white mb-3">Component Library</h2>
        
        {/* Search */}
        <input
          type="text"
          placeholder="Search components..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="input text-sm mb-3"
        />
        
        {/* Category Filter */}
        <div className="flex flex-wrap gap-1">
          {categories.map(category => (
            <button
              key={category}
              onClick={() => setSelectedCategory(category)}
              className={`px-2 py-1 text-xs rounded transition-colors ${
                selectedCategory === category
                  ? 'bg-sis-blue-600 text-white'
                  : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
              }`}
            >
              {category}
            </button>
          ))}
        </div>
      </div>
      
      {/* Component List */}
      <div className="p-2 space-y-2 overflow-y-auto">
        {filteredComponents.map(component => (
          <div
            key={component.id}
            draggable
            onDragStart={(event) => onDragStart(event, component.id)}
            className="p-3 bg-sis-gray-700 rounded-lg border border-sis-gray-600 hover:border-sis-blue-500 cursor-grab active:cursor-grabbing transition-all duration-200 hover:bg-sis-gray-600"
          >
            {/* Component Header */}
            <div className="flex items-start space-x-3">
              <span className="text-xl flex-shrink-0">{component.icon}</span>
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between mb-1">
                  <h3 className="text-sm font-medium text-white truncate">
                    {component.name}
                  </h3>
                  <span className={`text-xs ${getDifficultyColor(component.difficulty)}`}>
                    {component.difficulty}
                  </span>
                </div>
                
                <p className="text-xs text-sis-gray-400 mb-2 line-clamp-2">
                  {component.description}
                </p>
                
                {/* Component Stats */}
                <div className="flex items-center justify-between text-xs">
                  <div className="flex items-center space-x-3">
                    {component.powerEstimate && (
                      <span className="text-sis-gray-500">
                        ⚡ {component.powerEstimate}mW
                      </span>
                    )}
                    {component.gateEstimate && (
                      <span className="text-sis-gray-500">
                        🔧 {component.gateEstimate.toLocaleString()} gates
                      </span>
                    )}
                  </div>
                </div>
              </div>
            </div>
            
            {/* Drag Hint */}
            <div className="mt-2 text-center">
              <span className="text-xs text-sis-gray-500">
                Drag to canvas →
              </span>
            </div>
          </div>
        ))}
        
        {filteredComponents.length === 0 && (
          <div className="text-center py-8">
            <div className="text-sis-gray-500 mb-2">No components found</div>
            <div className="text-xs text-sis-gray-600">
              Try adjusting your search or category filter
            </div>
          </div>
        )}
      </div>
      
      {/* Footer Info */}
      <div className="p-3 border-t border-sis-gray-700 bg-sis-gray-900">
        <div className="text-xs text-sis-gray-500">
          <div className="mb-1">
            {filteredComponents.length} component{filteredComponents.length !== 1 ? 's' : ''} available
          </div>
          <div className="text-sis-gray-600">
            Drag components to the design canvas to start building
          </div>
        </div>
      </div>
    </div>
  )
}

export default ComponentPalette