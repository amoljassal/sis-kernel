import React, { useRef, useEffect, useState } from 'react'
import { Canvas, useFrame, useThree } from '@react-three/fiber'
import { OrbitControls, Text, Box, Plane } from '@react-three/drei'
import * as THREE from 'three'
import { useSelector } from 'react-redux'
import type { RootState } from '../../../store/store'

interface ChipVisualizationProps {
  className?: string
  width?: number
  height?: number
}

interface ChipComponent {
  id: string
  position: [number, number, number]
  size: [number, number, number]
  color: string
  label: string
  type: string
  powerDensity: number
  temperature: number
}

// 3D Chip Component
const Component3D: React.FC<{ 
  component: ChipComponent 
  isSelected: boolean 
  onSelect: () => void 
}> = ({ component, isSelected, onSelect }) => {
  const meshRef = useRef<THREE.Mesh>(null)
  const [hovered, setHovered] = useState(false)

  useFrame(() => {
    if (meshRef.current) {
      // Subtle animation for selected/hovered components
      const scale = isSelected ? 1.1 : hovered ? 1.05 : 1
      meshRef.current.scale.setScalar(scale)
      
      // Temperature-based color shifting
      const tempRatio = Math.min(component.temperature / 85, 1) // Max 85°C
      const baseColor = new THREE.Color(component.color)
      const heatColor = new THREE.Color(0xff4444)
      baseColor.lerp(heatColor, tempRatio * 0.3)
      
      if (meshRef.current.material instanceof THREE.MeshStandardMaterial) {
        meshRef.current.material.color = baseColor
        meshRef.current.material.emissive = baseColor.clone().multiplyScalar(tempRatio * 0.2)
      }
    }
  })

  return (
    <group position={component.position}>
      <Box
        ref={meshRef}
        args={component.size}
        onClick={onSelect}
        onPointerOver={() => setHovered(true)}
        onPointerOut={() => setHovered(false)}
      >
        <meshStandardMaterial
          color={component.color}
          metalness={0.7}
          roughness={0.3}
          transparent={true}
          opacity={isSelected ? 0.9 : 0.8}
        />
      </Box>
      
      {/* Component label */}
      {(isSelected || hovered) && (
        <Text
          position={[0, component.size[1] / 2 + 0.5, 0]}
          fontSize={0.3}
          color="white"
          anchorX="center"
          anchorY="middle"
        >
          {component.label}
        </Text>
      )}
      
      {/* Power visualization */}
      {component.powerDensity > 50 && (
        <Plane
          position={[0, component.size[1] / 2 + 0.1, 0]}
          args={[component.size[0] * 1.1, component.size[2] * 1.1]}
        >
          <meshBasicMaterial
            color={0xff6600}
            transparent={true}
            opacity={Math.min(component.powerDensity / 500, 0.5)}
            side={THREE.DoubleSide}
          />
        </Plane>
      )}
    </group>
  )
}

// Wire connection visualization
const Connection3D: React.FC<{
  from: [number, number, number]
  to: [number, number, number]
  signalType: 'clock' | 'data' | 'power' | 'ground'
  isActive: boolean
}> = ({ from, to, signalType, isActive }) => {
  const points = [
    new THREE.Vector3(...from),
    new THREE.Vector3(...to)
  ]

  const getWireColor = () => {
    switch (signalType) {
      case 'clock': return 0xffff00  // Yellow
      case 'data': return 0x00ffff   // Cyan
      case 'power': return 0xff0000  // Red
      case 'ground': return 0x00ff00 // Green
      default: return 0xffffff       // White
    }
  }

  return (
    <line>
      <bufferGeometry>
        <bufferAttribute
          attach="attributes-position"
          count={points.length}
          array={new Float32Array(points.flatMap(p => [p.x, p.y, p.z]))}
          itemSize={3}
        />
      </bufferGeometry>
      <lineBasicMaterial
        color={getWireColor()}
        linewidth={isActive ? 3 : 1}
        transparent={true}
        opacity={isActive ? 1 : 0.6}
      />
    </line>
  )
}

// Main chip substrate
const ChipSubstrate: React.FC = () => {
  return (
    <Box args={[10, 0.2, 10]} position={[0, -0.5, 0]}>
      <meshStandardMaterial
        color={0x2a4a2a}
        metalness={0.9}
        roughness={0.1}
      />
    </Box>
  )
}

// 3D Scene component
const ChipScene: React.FC<{
  components: ChipComponent[]
  selectedId: string | null
  onSelectComponent: (id: string) => void
}> = ({ components, selectedId, onSelectComponent }) => {
  const { camera } = useThree()

  useEffect(() => {
    // Set up camera position
    camera.position.set(15, 10, 15)
    camera.lookAt(0, 0, 0)
  }, [camera])

  // Generate connections between components
  const connections = components.reduce((acc, comp, i) => {
    const nextComp = components[i + 1]
    if (nextComp) {
      acc.push({
        from: comp.position,
        to: nextComp.position,
        signalType: 'data' as const,
        isActive: comp.id === selectedId || nextComp.id === selectedId
      })
    }
    return acc
  }, [] as Array<{
    from: [number, number, number]
    to: [number, number, number]
    signalType: 'clock' | 'data' | 'power' | 'ground'
    isActive: boolean
  }>)

  return (
    <>
      {/* Lighting */}
      <ambientLight intensity={0.4} />
      <pointLight position={[10, 10, 10]} intensity={1} />
      <pointLight position={[-10, -10, -10]} intensity={0.5} />
      
      {/* Chip substrate */}
      <ChipSubstrate />
      
      {/* Components */}
      {components.map(component => (
        <Component3D
          key={component.id}
          component={component}
          isSelected={component.id === selectedId}
          onSelect={() => onSelectComponent(component.id)}
        />
      ))}
      
      {/* Connections */}
      {connections.map((connection, index) => (
        <Connection3D
          key={index}
          from={connection.from}
          to={connection.to}
          signalType={connection.signalType}
          isActive={connection.isActive}
        />
      ))}
      
      {/* Controls */}
      <OrbitControls
        enablePan={true}
        enableZoom={true}
        enableRotate={true}
        minDistance={5}
        maxDistance={50}
      />
    </>
  )
}

const ChipVisualization: React.FC<ChipVisualizationProps> = ({ 
  className = '', 
  height = 300 
}) => {
  const { nodes } = useSelector((state: RootState) => state.designer)
  const [selectedComponent, setSelectedComponent] = useState<string | null>(null)
  const [viewMode, setViewMode] = useState<'3d' | 'thermal' | 'power'>('3d')

  // Convert design nodes to 3D components
  const chip3DComponents: ChipComponent[] = nodes.map((node, index) => {
    const x = (index % 3) * 3 - 3
    const z = Math.floor(index / 3) * 3 - 3
    const y = 0.5

    // Estimate temperature based on power consumption
    const basePower = node.powerConsumption || 50
    const temperature = 25 + (basePower / 10) + Math.random() * 10

    return {
      id: node.id,
      position: [x, y, z] as [number, number, number],
      size: [2, 0.5, 2] as [number, number, number],
      color: getComponentColor(node.type),
      label: node.name,
      type: node.type,
      powerDensity: basePower,
      temperature
    }
  })

  function getComponentColor(type: string): string {
    switch (type) {
      case 'cpu': return '#4f46e5'        // Indigo
      case 'memory': return '#059669'     // Green
      case 'io': return '#dc2626'         // Red
      case 'logic': return '#7c3aed'      // Purple
      case 'ai_accelerator': return '#ea580c' // Orange
      case 'interface': return '#0891b2'  // Cyan
      default: return '#6b7280'           // Gray
    }
  }

  if (nodes.length === 0) {
    return (
      <div className={`bg-sis-gray-800 border border-sis-gray-700 rounded-lg p-4 ${className}`}>
        <div className="text-center text-sis-gray-500">
          <div className="text-2xl mb-2">🔷</div>
          <div className="text-sm">Add components to see 3D visualization</div>
        </div>
      </div>
    )
  }

  return (
    <div className={`bg-sis-gray-800 border border-sis-gray-700 rounded-lg ${className}`}>
      {/* Header */}
      <div className="p-3 border-b border-sis-gray-700">
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-sm font-semibold text-white">3D Chip View</h3>
          <div className="flex space-x-1">
            {(['3d', 'thermal', 'power'] as const).map(mode => (
              <button
                key={mode}
                onClick={() => setViewMode(mode)}
                className={`px-2 py-1 text-xs rounded transition-colors ${
                  viewMode === mode
                    ? 'bg-sis-blue-600 text-white'
                    : 'bg-sis-gray-700 text-sis-gray-300 hover:bg-sis-gray-600'
                }`}
              >
                {mode.toUpperCase()}
              </button>
            ))}
          </div>
        </div>
        
        {/* Stats */}
        <div className="grid grid-cols-3 gap-2 text-xs">
          <div className="text-center">
            <div className="text-white font-medium">{nodes.length}</div>
            <div className="text-sis-gray-400">Components</div>
          </div>
          <div className="text-center">
            <div className="text-white font-medium">
              {Math.round(chip3DComponents.reduce((sum, c) => sum + c.powerDensity, 0))}mW
            </div>
            <div className="text-sis-gray-400">Total Power</div>
          </div>
          <div className="text-center">
            <div className="text-white font-medium">
              {Math.round(chip3DComponents.reduce((sum, c) => sum + c.temperature, 0) / chip3DComponents.length)}°C
            </div>
            <div className="text-sis-gray-400">Avg Temp</div>
          </div>
        </div>
      </div>

      {/* 3D Canvas */}
      <div style={{ width: '100%', height: `${height}px` }}>
        <Canvas
          gl={{ antialias: true, alpha: true }}
          camera={{ position: [15, 10, 15], fov: 45 }}
          style={{ background: 'transparent' }}
        >
          <ChipScene
            components={chip3DComponents}
            selectedId={selectedComponent}
            onSelectComponent={setSelectedComponent}
          />
        </Canvas>
      </div>

      {/* Component Info */}
      {selectedComponent && (
        <div className="p-3 border-t border-sis-gray-700 bg-sis-gray-900">
          {(() => {
            const component = chip3DComponents.find(c => c.id === selectedComponent)
            const node = nodes.find(n => n.id === selectedComponent)
            if (!component || !node) return null

            return (
              <div className="space-y-2">
                <div className="flex items-center space-x-2">
                  <div
                    className="w-3 h-3 rounded"
                    style={{ backgroundColor: component.color }}
                  />
                  <span className="text-sm font-medium text-white">{component.label}</span>
                </div>
                <div className="grid grid-cols-2 gap-2 text-xs">
                  <div>
                    <span className="text-sis-gray-400">Type: </span>
                    <span className="text-white">{node.type.toUpperCase()}</span>
                  </div>
                  <div>
                    <span className="text-sis-gray-400">Power: </span>
                    <span className="text-white">{component.powerDensity}mW</span>
                  </div>
                  <div>
                    <span className="text-sis-gray-400">Temp: </span>
                    <span className={`${
                      component.temperature > 70 ? 'text-red-400' :
                      component.temperature > 50 ? 'text-yellow-400' :
                      'text-green-400'
                    }`}>
                      {Math.round(component.temperature)}°C
                    </span>
                  </div>
                  <div>
                    <span className="text-sis-gray-400">Status: </span>
                    <span className={`${
                      node.verificationStatus === 'verified' ? 'text-green-400' :
                      node.verificationStatus === 'failed' ? 'text-red-400' :
                      'text-yellow-400'
                    }`}>
                      {node.verificationStatus}
                    </span>
                  </div>
                </div>
              </div>
            )
          })()}
        </div>
      )}
    </div>
  )
}

export default ChipVisualization