import React, { useState, useMemo } from 'react'
import { useSelector, useDispatch } from 'react-redux'
import type { RootState } from '../../../store/store'
import { selectNode, deleteNode } from '../../../store/slices/designerSlice'
import type { DesignNode } from '../../../types/design'

interface DesignBrowserProps {
  className?: string
}

interface TreeNode {
  id: string
  name: string
  type: string
  children: TreeNode[]
  level: number
  isExpanded: boolean
  node?: DesignNode
}

const DesignBrowser: React.FC<DesignBrowserProps> = ({ className = '' }) => {
  const dispatch = useDispatch()
  const { nodes, connections, selectedNodes } = useSelector((state: RootState) => state.designer)
  
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set(['root', 'processors', 'memory', 'io']))
  const [searchQuery, setSearchQuery] = useState('')
  const [viewMode, setViewMode] = useState<'hierarchy' | 'flat' | 'connections'>('hierarchy')
  const [sortBy, setSortBy] = useState<'name' | 'type' | 'date'>('type')

  // Build hierarchical tree structure
  const designTree: TreeNode = useMemo(() => {
    const root: TreeNode = {
      id: 'root',
      name: 'Design Hierarchy',
      type: 'root',
      children: [],
      level: 0,
      isExpanded: expandedNodes.has('root')
    }

    // Group nodes by category
    const categories = {
      processors: { name: 'Processors', nodes: [] as DesignNode[] },
      memory: { name: 'Memory Systems', nodes: [] as DesignNode[] },
      io: { name: 'I/O & Communication', nodes: [] as DesignNode[] },
      logic: { name: 'Logic & Control', nodes: [] as DesignNode[] },
      ai: { name: 'AI/ML Accelerators', nodes: [] as DesignNode[] },
      interfaces: { name: 'Interfaces', nodes: [] as DesignNode[] },
      custom: { name: 'Custom Components', nodes: [] as DesignNode[] }
    }

    // Categorize nodes
    nodes.forEach(node => {
      switch (node.type) {
        case 'cpu':
        case 'dsp':
        case 'mcu':
          categories.processors.nodes.push(node)
          break
        case 'memory':
        case 'cache':
        case 'rom':
          categories.memory.nodes.push(node)
          break
        case 'io':
        case 'uart':
        case 'spi':
        case 'i2c':
          categories.io.nodes.push(node)
          break
        case 'logic':
        case 'mux':
        case 'alu':
          categories.logic.nodes.push(node)
          break
        case 'ai_accelerator':
        case 'tensor_core':
          categories.ai.nodes.push(node)
          break
        case 'pcie':
        case 'usb':
          categories.interfaces.nodes.push(node)
          break
        default:
          categories.custom.nodes.push(node)
          break
      }
    })

    // Build category nodes
    Object.entries(categories).forEach(([key, category]) => {
      if (category.nodes.length > 0) {
        const categoryNode: TreeNode = {
          id: key,
          name: `${category.name} (${category.nodes.length})`,
          type: 'category',
          children: [],
          level: 1,
          isExpanded: expandedNodes.has(key)
        }

        // Sort nodes within category
        const sortedNodes = [...category.nodes].sort((a, b) => {
          switch (sortBy) {
            case 'name':
              return a.name.localeCompare(b.name)
            case 'type':
              return a.type.localeCompare(b.type)
            case 'date':
              return 0 // Would need actual date fields
            default:
              return 0
          }
        })

        // Add individual nodes
        sortedNodes.forEach(node => {
          categoryNode.children.push({
            id: node.id,
            name: node.name,
            type: node.type,
            children: [],
            level: 2,
            isExpanded: false,
            node
          })
        })

        root.children.push(categoryNode)
      }
    })

    return root
  }, [nodes, expandedNodes, sortBy])

  // Filter tree based on search query
  const filteredTree = useMemo(() => {
    if (!searchQuery) return designTree

    const filterNode = (node: TreeNode): TreeNode | null => {
      const matchesSearch = node.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                           node.type.toLowerCase().includes(searchQuery.toLowerCase())

      const filteredChildren = node.children
        .map(child => filterNode(child))
        .filter(child => child !== null) as TreeNode[]

      if (matchesSearch || filteredChildren.length > 0) {
        return {
          ...node,
          children: filteredChildren,
          isExpanded: true // Expand matching nodes
        }
      }

      return null
    }

    return filterNode(designTree) || designTree
  }, [designTree, searchQuery])

  const toggleExpanded = (nodeId: string) => {
    setExpandedNodes(prev => {
      const newSet = new Set(prev)
      if (newSet.has(nodeId)) {
        newSet.delete(nodeId)
      } else {
        newSet.add(nodeId)
      }
      return newSet
    })
  }

  const handleNodeSelect = (node: TreeNode) => {
    if (node.node) {
      dispatch(selectNode(node.node.id))
    }
  }

  const handleNodeDelete = (node: TreeNode, event: React.MouseEvent) => {
    event.stopPropagation()
    if (node.node) {
      dispatch(deleteNode(node.node.id))
    }
  }

  const getNodeIcon = (type: string) => {
    switch (type) {
      case 'root': return '🌳'
      case 'category': return '📁'
      case 'cpu': return '🧠'
      case 'memory': return '💾'
      case 'io': return '🔌'
      case 'logic': return '⚡'
      case 'ai_accelerator': return '🤖'
      case 'interface': return '🔗'
      default: return '📦'
    }
  }

  const getStatusColor = (node: DesignNode) => {
    switch (node.verificationStatus) {
      case 'verified': return 'text-green-400'
      case 'failed': return 'text-red-400'
      case 'unverified': return 'text-yellow-400'
      default: return 'text-sis-gray-400'
    }
  }

  const renderTreeNode = (node: TreeNode) => {
    const isSelected = node.node && selectedNodes.includes(node.node.id)
    const hasChildren = node.children.length > 0
    
    return (
      <div key={node.id}>
        <div
          className={`flex items-center space-x-2 p-2 rounded cursor-pointer transition-colors ${
            isSelected 
              ? 'bg-sis-blue-600 text-white' 
              : 'hover:bg-sis-gray-700 text-sis-gray-200'
          }`}
          style={{ paddingLeft: `${node.level * 16 + 8}px` }}
          onClick={() => handleNodeSelect(node)}
        >
          {/* Expand/collapse button */}
          {hasChildren && (
            <button
              onClick={(e) => {
                e.stopPropagation()
                toggleExpanded(node.id)
              }}
              className="w-4 h-4 flex items-center justify-center text-sis-gray-400 hover:text-white"
            >
              {node.isExpanded ? '▼' : '▶'}
            </button>
          )}
          
          {/* Node icon */}
          <span className="text-sm">{getNodeIcon(node.type)}</span>
          
          {/* Node name */}
          <span className="flex-1 text-sm truncate">{node.name}</span>
          
          {/* Status indicator for actual design nodes */}
          {node.node && (
            <div className="flex items-center space-x-1">
              <div
                className={`w-2 h-2 rounded-full ${
                  node.node.verificationStatus === 'verified' ? 'bg-green-400' :
                  node.node.verificationStatus === 'failed' ? 'bg-red-400' :
                  'bg-yellow-400'
                }`}
              />
              
              {/* Delete button */}
              <button
                onClick={(e) => handleNodeDelete(node, e)}
                className="w-4 h-4 text-sis-gray-500 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity"
              >
                ✕
              </button>
            </div>
          )}
        </div>
        
        {/* Render children if expanded */}
        {node.isExpanded && node.children.map(child => renderTreeNode(child))}
      </div>
    )
  }

  // Connection view
  const renderConnectionsView = () => {
    return (
      <div className="space-y-2">
        <div className="text-sm font-medium text-white mb-3">
          Connections ({connections.length})
        </div>
        
        {connections.map(conn => {
          const sourceNode = nodes.find(n => n.id === conn.sourceId)
          const targetNode = nodes.find(n => n.id === conn.targetId)
          
          return (
            <div
              key={conn.id}
              className="p-2 bg-sis-gray-700 rounded border-l-3 border-blue-500"
            >
              <div className="text-sm text-white mb-1">
                {conn.signalName}
              </div>
              <div className="text-xs text-sis-gray-400 space-y-1">
                <div>
                  From: {sourceNode?.name || 'Unknown'} ({conn.sourcePort})
                </div>
                <div>
                  To: {targetNode?.name || 'Unknown'} ({conn.targetPort})
                </div>
                <div className="flex space-x-4">
                  <span>Type: {conn.signalType}</span>
                  <span>Width: {conn.bitWidth}-bit</span>
                  {conn.isCritical && (
                    <span className="text-red-400">Critical</span>
                  )}
                </div>
              </div>
            </div>
          )
        })}
        
        {connections.length === 0 && (
          <div className="text-center py-8 text-sis-gray-500">
            <div className="text-2xl mb-2">🔗</div>
            <div className="text-sm">No connections yet</div>
            <div className="text-xs">Connect components on the canvas</div>
          </div>
        )}
      </div>
    )
  }

  return (
    <div className={`bg-sis-gray-800 border border-sis-gray-700 rounded-lg ${className}`}>
      {/* Header */}
      <div className="p-4 border-b border-sis-gray-700">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-sm font-semibold text-white">Design Browser</h3>
          <div className="flex space-x-1">
            {(['hierarchy', 'flat', 'connections'] as const).map(mode => (
              <button
                key={mode}
                onClick={() => setViewMode(mode)}
                className={`px-2 py-1 text-xs rounded transition-colors ${
                  viewMode === mode
                    ? 'bg-sis-blue-600 text-white'
                    : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
                }`}
              >
                {mode === 'hierarchy' ? '🌳' : mode === 'flat' ? '📋' : '🔗'}
              </button>
            ))}
          </div>
        </div>

        {/* Search */}
        <input
          type="text"
          placeholder="Search components..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="input text-sm mb-3"
        />

        {/* Sort options */}
        {viewMode !== 'connections' && (
          <div className="flex space-x-1">
            {(['name', 'type', 'date'] as const).map(sort => (
              <button
                key={sort}
                onClick={() => setSortBy(sort)}
                className={`px-2 py-1 text-xs rounded transition-colors ${
                  sortBy === sort
                    ? 'bg-sis-gray-600 text-white'
                    : 'text-sis-gray-400 hover:text-white'
                }`}
              >
                Sort by {sort}
              </button>
            ))}
          </div>
        )}

        {/* Stats */}
        <div className="grid grid-cols-2 gap-2 mt-3 text-xs">
          <div className="text-center p-2 bg-sis-gray-700 rounded">
            <div className="text-white font-medium">{nodes.length}</div>
            <div className="text-sis-gray-400">Components</div>
          </div>
          <div className="text-center p-2 bg-sis-gray-700 rounded">
            <div className="text-white font-medium">{connections.length}</div>
            <div className="text-sis-gray-400">Connections</div>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="p-2 max-h-96 overflow-y-auto">
        {viewMode === 'connections' ? (
          renderConnectionsView()
        ) : viewMode === 'flat' ? (
          <div className="space-y-1">
            {nodes
              .filter(node => 
                searchQuery === '' || 
                node.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                node.type.toLowerCase().includes(searchQuery.toLowerCase())
              )
              .map(node => (
                <div
                  key={node.id}
                  className={`flex items-center space-x-2 p-2 rounded cursor-pointer transition-colors group ${
                    selectedNodes.includes(node.id)
                      ? 'bg-sis-blue-600 text-white'
                      : 'hover:bg-sis-gray-700 text-sis-gray-200'
                  }`}
                  onClick={() => dispatch(selectNode(node.id))}
                >
                  <span className="text-sm">{getNodeIcon(node.type)}</span>
                  <span className="flex-1 text-sm truncate">{node.name}</span>
                  <span className="text-xs text-sis-gray-400">{node.type}</span>
                  <div
                    className={`w-2 h-2 rounded-full ${getStatusColor(node).replace('text-', 'bg-')}`}
                  />
                  <button
                    onClick={(e) => {
                      e.stopPropagation()
                      dispatch(deleteNode(node.id))
                    }}
                    className="w-4 h-4 text-sis-gray-500 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity"
                  >
                    ✕
                  </button>
                </div>
              ))}
          </div>
        ) : (
          renderTreeNode(filteredTree)
        )}

        {nodes.length === 0 && (
          <div className="text-center py-8 text-sis-gray-500">
            <div className="text-2xl mb-2">📦</div>
            <div className="text-sm">No components yet</div>
            <div className="text-xs">Drag components from the palette</div>
          </div>
        )}
      </div>
    </div>
  )
}

export default DesignBrowser