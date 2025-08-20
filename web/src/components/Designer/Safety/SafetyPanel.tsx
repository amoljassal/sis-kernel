import React, { useState } from 'react'
import { useSelector, useDispatch } from 'react-redux'
import type { RootState } from '../../../store/store'
import { setSafetyMode } from '../../../store/slices/settingsSlice'

interface SafetyPanelProps {
  className?: string
}

interface PreflightCheck {
  id: string
  name: string
  description: string
  status: 'pending' | 'checking' | 'passed' | 'failed' | 'warning'
  criticality: 'low' | 'medium' | 'high' | 'critical'
  autoCheck: boolean
}

const preflightChecks: PreflightCheck[] = [
  {
    id: 'power_budget',
    name: 'Power Budget Validation',
    description: 'Verify total power consumption is within safe limits',
    status: 'pending',
    criticality: 'critical',
    autoCheck: true
  },
  {
    id: 'timing_closure',
    name: 'Timing Closure',
    description: 'Check for setup/hold violations and negative slack',
    status: 'pending',
    criticality: 'high',
    autoCheck: true
  },
  {
    id: 'signal_integrity',
    name: 'Signal Integrity',
    description: 'Validate signal routing and cross-talk analysis',
    status: 'pending',
    criticality: 'high',
    autoCheck: true
  },
  {
    id: 'thermal_envelope',
    name: 'Thermal Envelope',
    description: 'Ensure operating temperature within device specifications',
    status: 'pending',
    criticality: 'critical',
    autoCheck: true
  },
  {
    id: 'design_rules',
    name: 'Design Rule Check (DRC)',
    description: 'Verify geometric and electrical design rules',
    status: 'pending',
    criticality: 'medium',
    autoCheck: true
  },
  {
    id: 'connectivity',
    name: 'Connectivity Check',
    description: 'Ensure all required connections are properly routed',
    status: 'pending',
    criticality: 'high',
    autoCheck: true
  },
  {
    id: 'resource_usage',
    name: 'Resource Utilization',
    description: 'Check FPGA/ASIC resource usage and availability',
    status: 'pending',
    criticality: 'medium',
    autoCheck: true
  },
  {
    id: 'two_person_approval',
    name: 'Two-Person Approval',
    description: 'Production deployments require secondary approval',
    status: 'pending',
    criticality: 'critical',
    autoCheck: false
  }
]

const SafetyPanel: React.FC<SafetyPanelProps> = ({ className = '' }) => {
  const dispatch = useDispatch()
  const { safetyMode } = useSelector((state: RootState) => state.settings)
  const { hazardScore, isSimulating } = useSelector((state: RootState) => state.designer)
  
  const [checks, setChecks] = useState<PreflightCheck[]>(preflightChecks)
  const [isExpanded, setIsExpanded] = useState(false)
  const [deploymentBlocked, setDeploymentBlocked] = useState(true)

  const handleModeToggle = (mode: 'beginner' | 'advanced' | 'pro') => {
    dispatch(setSafetyMode(mode))
  }

  const runPreflightChecks = () => {
    setChecks(prev => prev.map(check => 
      check.autoCheck ? { ...check, status: 'checking' } : check
    ))

    // Simulate checks with realistic timing
    setTimeout(() => {
      setChecks(prev => prev.map(check => {
        if (!check.autoCheck) return check
        
        // Simulate realistic check results based on hazard score
        const failureChance = hazardScore / 100 * 0.3 // 30% max failure rate
        const warningChance = hazardScore / 100 * 0.5 // 50% max warning rate
        
        const roll = Math.random()
        if (roll < failureChance) {
          return { ...check, status: 'failed' }
        } else if (roll < warningChance) {
          return { ...check, status: 'warning' }
        } else {
          return { ...check, status: 'passed' }
        }
      }))

      // Check if deployment should be unblocked
      const hasFailures = checks.some(c => c.status === 'failed' && c.criticality === 'critical')
      setDeploymentBlocked(hasFailures || hazardScore > 75)
    }, 2000)
  }

  const getStatusIcon = (status: PreflightCheck['status']) => {
    switch (status) {
      case 'passed': return '✓'
      case 'failed': return '✗'
      case 'warning': return '⚠'
      case 'checking': return '⟳'
      default: return '○'
    }
  }

  const getStatusColor = (status: PreflightCheck['status']) => {
    switch (status) {
      case 'passed': return 'text-green-400'
      case 'failed': return 'text-red-400'
      case 'warning': return 'text-yellow-400'
      case 'checking': return 'text-blue-400'
      default: return 'text-sis-gray-400'
    }
  }

  const getCriticalityColor = (criticality: PreflightCheck['criticality']) => {
    switch (criticality) {
      case 'critical': return 'border-red-500'
      case 'high': return 'border-orange-500'
      case 'medium': return 'border-yellow-500'
      default: return 'border-sis-gray-600'
    }
  }

  const getModeColor = (mode: string) => {
    switch (mode) {
      case 'beginner': return 'text-green-400 border-green-500'
      case 'advanced': return 'text-yellow-400 border-yellow-500'
      case 'pro': return 'text-red-400 border-red-500'
      default: return 'text-sis-gray-400 border-sis-gray-600'
    }
  }

  const passedChecks = checks.filter(c => c.status === 'passed').length
  const failedChecks = checks.filter(c => c.status === 'failed').length
  const warningChecks = checks.filter(c => c.status === 'warning').length

  return (
    <div className={`bg-sis-gray-800 border border-sis-gray-700 rounded-lg ${className}`}>
      {/* Header */}
      <div className="p-4 border-b border-sis-gray-700">
        <div className="flex items-center justify-between mb-3">
          <h3 className="text-lg font-semibold text-white">Safety Control</h3>
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="text-sis-gray-400 hover:text-white transition-colors"
          >
            {isExpanded ? '▼' : '▶'}
          </button>
        </div>

        {/* Safety Mode Toggle */}
        <div className="space-y-2">
          <label className="text-sm text-sis-gray-400">Safety Mode</label>
          <div className="flex space-x-2">
            {(['beginner', 'advanced', 'pro'] as const).map(mode => (
              <button
                key={mode}
                onClick={() => handleModeToggle(mode)}
                className={`px-3 py-1 text-xs rounded border transition-all ${
                  safetyMode === mode
                    ? getModeColor(mode)
                    : 'text-sis-gray-400 border-sis-gray-600 hover:border-sis-gray-500'
                }`}
              >
                {mode.toUpperCase()}
              </button>
            ))}
          </div>
        </div>

        {/* Hazard Score */}
        <div className="mt-3">
          <div className="flex items-center justify-between mb-1">
            <span className="text-sm text-sis-gray-400">Risk Score</span>
            <span className={`text-sm font-medium ${
              hazardScore <= 25 ? 'text-green-400' :
              hazardScore <= 50 ? 'text-yellow-400' :
              hazardScore <= 75 ? 'text-orange-400' :
              'text-red-400'
            }`}>
              {hazardScore}/100
            </span>
          </div>
          <div className="w-full bg-sis-gray-700 rounded-full h-2">
            <div
              className={`h-2 rounded-full transition-all duration-300 ${
                hazardScore <= 25 ? 'bg-green-500' :
                hazardScore <= 50 ? 'bg-yellow-500' :
                hazardScore <= 75 ? 'bg-orange-500' :
                'bg-red-500'
              }`}
              style={{ width: `${hazardScore}%` }}
            />
          </div>
        </div>
      </div>

      {/* Preflight Checklist - Expandable */}
      {isExpanded && (
        <div className="p-4 space-y-4">
          <div className="flex items-center justify-between">
            <h4 className="text-md font-medium text-white">Preflight Checklist</h4>
            <button
              onClick={runPreflightChecks}
              disabled={checks.some(c => c.status === 'checking')}
              className="btn-secondary text-sm px-3 py-1 disabled:opacity-50"
            >
              {checks.some(c => c.status === 'checking') ? 'Running...' : 'Run Checks'}
            </button>
          </div>

          {/* Check Summary */}
          <div className="grid grid-cols-3 gap-2 text-xs">
            <div className="text-center p-2 bg-sis-gray-700 rounded">
              <div className="text-green-400 font-medium">{passedChecks}</div>
              <div className="text-sis-gray-400">Passed</div>
            </div>
            <div className="text-center p-2 bg-sis-gray-700 rounded">
              <div className="text-yellow-400 font-medium">{warningChecks}</div>
              <div className="text-sis-gray-400">Warnings</div>
            </div>
            <div className="text-center p-2 bg-sis-gray-700 rounded">
              <div className="text-red-400 font-medium">{failedChecks}</div>
              <div className="text-sis-gray-400">Failed</div>
            </div>
          </div>

          {/* Individual Checks */}
          <div className="space-y-2 max-h-48 overflow-y-auto">
            {checks.map(check => (
              <div
                key={check.id}
                className={`p-2 rounded border-l-3 ${getCriticalityColor(check.criticality)} bg-sis-gray-700`}
              >
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <div className="flex items-center space-x-2">
                      <span className={`text-sm ${getStatusColor(check.status)}`}>
                        {getStatusIcon(check.status)}
                      </span>
                      <span className="text-sm font-medium text-white">{check.name}</span>
                      <span className={`text-xs px-1 rounded ${
                        check.criticality === 'critical' ? 'bg-red-900 text-red-300' :
                        check.criticality === 'high' ? 'bg-orange-900 text-orange-300' :
                        check.criticality === 'medium' ? 'bg-yellow-900 text-yellow-300' :
                        'bg-sis-gray-600 text-sis-gray-300'
                      }`}>
                        {check.criticality}
                      </span>
                    </div>
                    <p className="text-xs text-sis-gray-400 mt-1">{check.description}</p>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Deploy Button */}
          <div className="pt-2 border-t border-sis-gray-700">
            <button
              disabled={deploymentBlocked || isSimulating}
              className={`w-full py-2 px-4 rounded font-medium transition-all ${
                deploymentBlocked
                  ? 'bg-sis-gray-700 text-sis-gray-500 cursor-not-allowed'
                  : 'bg-green-600 hover:bg-green-700 text-white'
              }`}
            >
              {deploymentBlocked ? 'DEPLOY BLOCKED' : 'DEPLOY TO HARDWARE'}
            </button>
            
            {/* Emergency Kill Switch */}
            <button className="w-full mt-2 py-1 px-4 rounded text-xs bg-red-900 hover:bg-red-800 text-red-300 transition-colors">
              🛑 EMERGENCY STOP
            </button>
          </div>
        </div>
      )}
    </div>
  )
}

export default SafetyPanel