import React, { Suspense, useEffect } from 'react'
import { Routes, Route } from 'react-router-dom'
import { useDispatch } from 'react-redux'
import Layout from './components/Layout/Layout'
import ErrorBoundary from './components/ErrorBoundary/ErrorBoundary'
import LoadingSpinner from './components/LoadingSpinner/LoadingSpinner'

// Lazy load pages for code splitting
const Dashboard = React.lazy(() => import('./pages/Dashboard/Dashboard'))
const Designer = React.lazy(() => import('./pages/Designer/Designer'))
const Validator = React.lazy(() => import('./pages/Validator/Validator'))
const Hardware = React.lazy(() => import('./pages/Hardware/Hardware'))
const Marketplace = React.lazy(() => import('./pages/Marketplace/Marketplace'))
const Settings = React.lazy(() => import('./pages/Settings/Settings'))

// AURAG and MLX components - Emergency fix: Remove AURAG route temporarily
const TrainingInterface = React.lazy(() => import('./components/MLX/TrainingInterface'))

function App() {
  const dispatch = useDispatch()
  
  useEffect(() => {
    // Initialize the application
    const initializeApp = async () => {
      try {
        // Load WASM module
        await import('./wasm/sis-kernel')
        console.log('WASM module loaded')
        
        // Initialize WebGL context check
        const canvas = document.createElement('canvas')
        const gl = canvas.getContext('webgl2') || canvas.getContext('webgl')
        if (!gl) {
          console.warn('WebGL not supported, falling back to Canvas renderer')
        } else {
          console.log('WebGL supported:', gl.getParameter(gl.VERSION))
        }
        
        // Initialize collaboration system
        if (typeof window !== 'undefined' && 'WebRTC' in window) {
          console.log('WebRTC supported for real-time collaboration')
        }
        
        // Check for PWA installation
        window.addEventListener('beforeinstallprompt', (e) => {
          e.preventDefault()
          // Show install button
          console.log('PWA installation available')
        })
        
      } catch (error) {
        console.error('Failed to initialize app:', error)
      }
    }
    
    initializeApp()
  }, [dispatch])
  
  return (
    <ErrorBoundary>
      <div className="min-h-screen bg-gradient-sis">
        <Layout>
          <Suspense fallback={<LoadingSpinner />}>
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/design" element={<Designer />} />
              <Route path="/validate" element={<Validator />} />
              <Route path="/hardware" element={<Hardware />} />
              <Route path="/marketplace" element={<Marketplace />} />
              <Route path="/settings" element={<Settings />} />
              <Route path="/training" element={<TrainingInterface />} />
            </Routes>
          </Suspense>
        </Layout>
      </div>
    </ErrorBoundary>
  )
}

export default App