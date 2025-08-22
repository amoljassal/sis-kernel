import React, { ReactNode } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { useSelector } from 'react-redux'
import { Home, Zap, CheckCircle, Cpu, ShoppingCart, Settings, Brain, Beaker } from 'lucide-react'
import type { RootState } from '../../store/store'

interface LayoutProps {
  children: ReactNode
}

const Layout: React.FC<LayoutProps> = ({ children }) => {
  const location = useLocation()
  const { safetyMode } = useSelector((state: RootState) => state.settings)
  const { hazardScore } = useSelector((state: RootState) => state.designer)
  const { collaborators, isEnabled } = useSelector((state: RootState) => state.collaboration)
  
  const navItems = [
    { path: '/', label: 'Dashboard', icon: Home },
    { path: '/design', label: 'Designer', icon: Zap },
    { path: '/validate', label: 'Validator', icon: CheckCircle },
    { path: '/hardware', label: 'Hardware', icon: Cpu },
    { path: '/marketplace', label: 'Marketplace', icon: ShoppingCart },
    { path: '/settings', label: 'Settings', icon: Settings },
    { path: '/training', label: 'Training Lab', icon: Beaker },
    { path: '/aurag', label: 'AURAG', icon: Brain },
  ]
  
  const getSafetyColor = () => {
    if (hazardScore <= 25) return 'text-green-400'
    if (hazardScore <= 50) return 'text-yellow-400'
    if (hazardScore <= 75) return 'text-orange-400'
    return 'text-red-400'
  }
  
  const getSafetyModeColor = () => {
    switch (safetyMode) {
      case 'beginner': return 'bg-green-600'
      case 'advanced': return 'bg-yellow-600'
      case 'pro': return 'bg-red-600'
      default: return 'bg-sis-gray-600'
    }
  }

  return (
    <div className="h-screen flex flex-col bg-sis-gray-900">
      {/* Header */}
      <header className="glass border-b border-sis-gray-700 px-6 py-4">
        <div className="flex items-center justify-between">
          {/* Logo */}
          <div className="flex items-center space-x-3">
            <div className="w-8 h-8 bg-gradient-to-br from-sis-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
              <span className="text-white font-bold text-sm">S</span>
            </div>
            <h1 className="text-xl font-bold text-white">
              SIS AI-Lab
            </h1>
          </div>
          
          {/* Status indicators */}
          <div className="flex items-center space-x-4">
            {/* Collaboration status */}
            {isEnabled && Object.keys(collaborators).length > 0 && (
              <div className="flex items-center space-x-2">
                <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
                <span className="text-sm text-sis-gray-300">
                  {Object.keys(collaborators).length} connected
                </span>
              </div>
            )}
            
            {/* Safety status */}
            <div className="flex items-center space-x-2">
              <div className={`px-2 py-1 rounded text-xs font-medium ${getSafetyModeColor()}`}>
                {safetyMode.toUpperCase()}
              </div>
              <div className={`text-sm font-medium ${getSafetyColor()}`}>
                Risk: {hazardScore}/100
              </div>
            </div>
          </div>
        </div>
      </header>
      
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <nav className="w-64 glass border-r border-sis-gray-700 p-4">
          <ul className="space-y-2">
            {navItems.map((item) => {
              const isActive = location.pathname === item.path
              const IconComponent = item.icon
              return (
                <li key={item.path}>
                  <Link
                    to={item.path}
                    className={`flex items-center space-x-3 px-3 py-2 rounded-lg transition-colors ${
                      isActive
                        ? 'bg-sis-blue-600 text-white'
                        : 'text-sis-gray-300 hover:bg-sis-gray-700 hover:text-white'
                    }`}
                  >
                    <IconComponent className="h-5 w-5" />
                    <span className="font-medium">{item.label}</span>
                  </Link>
                </li>
              )
            })}
          </ul>
          
          {/* Quick actions */}
          <div className="mt-8 pt-4 border-t border-sis-gray-700">
            <h3 className="text-xs font-semibold text-sis-gray-400 uppercase mb-3">
              Quick Actions
            </h3>
            <div className="space-y-2">
              <button className="w-full btn-primary text-sm py-2">
                New Design
              </button>
              <button className="w-full btn-secondary text-sm py-2">
                Import
              </button>
            </div>
          </div>
        </nav>
        
        {/* Main content */}
        <main className="flex-1 overflow-auto">
          {children}
        </main>
      </div>
    </div>
  )
}

export default Layout