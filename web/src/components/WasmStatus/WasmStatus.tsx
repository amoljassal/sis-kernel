import React from 'react'
import { useWasm, useWasmPerformance } from '../../hooks/useWasm'

interface WasmStatusProps {
  className?: string
  showDetails?: boolean
}

const WasmStatus: React.FC<WasmStatusProps> = ({ className = '', showDetails = false }) => {
  const { isLoaded, isLoading, error, kernel } = useWasm()
  const { metrics } = useWasmPerformance()

  const getStatusColor = () => {
    if (error) return 'text-red-400'
    if (isLoading) return 'text-yellow-400'
    if (isLoaded) return 'text-green-400'
    return 'text-sis-gray-400'
  }

  const getStatusIcon = () => {
    if (error) return '❌'
    if (isLoading) return '⏳'
    if (isLoaded) return '✅'
    return '⚪'
  }

  const getStatusText = () => {
    if (error) return 'WASM Error'
    if (isLoading) return 'Loading WASM...'
    if (isLoaded) return 'WASM Ready'
    return 'WASM Not Loaded'
  }

  return (
    <div className={`${className}`}>
      <div className="flex items-center space-x-2">
        <span className="text-lg">{getStatusIcon()}</span>
        <span className={`text-sm font-medium ${getStatusColor()}`}>
          {getStatusText()}
        </span>
        {isLoaded && kernel && (
          <span className="text-xs text-sis-gray-500">
            {kernel.getVersion().split(' ')[3]} {/* Extract version number */}
          </span>
        )}
      </div>

      {error && (
        <div className="mt-2 text-xs text-red-400 bg-red-900/20 px-2 py-1 rounded">
          {error}
        </div>
      )}

      {showDetails && isLoaded && Object.keys(metrics).length > 0 && (
        <div className="mt-2 text-xs text-sis-gray-400">
          <div className="grid grid-cols-2 gap-2">
            {Object.entries(metrics).map(([key, value]) => (
              <div key={key} className="flex justify-between">
                <span className="capitalize">{key.replace(/_/g, ' ')}:</span>
                <span className="text-sis-blue-400">
                  {typeof value === 'number' ? value.toFixed(2) : String(value)}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}

export default WasmStatus