import React, { Component, ReactNode } from 'react'

interface Props {
  children: ReactNode
  fallback?: ReactNode
}

interface State {
  hasError: boolean
  error?: Error
  errorInfo?: string
}

class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { hasError: false }
  }

  static getDerivedStateFromError(error: Error): State {
    return { 
      hasError: true, 
      error 
    }
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    this.setState({
      error,
      errorInfo: errorInfo.componentStack
    })
    
    // Log error to monitoring service
    console.error('ErrorBoundary caught an error:', error, errorInfo)
  }

  handleReset = () => {
    this.setState({ hasError: false, error: undefined, errorInfo: undefined })
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback
      }

      return (
        <div className="min-h-screen bg-sis-gray-900 flex items-center justify-center p-4">
          <div className="max-w-md w-full card p-6 text-center">
            <div className="text-red-400 text-6xl mb-4">⚠️</div>
            <h1 className="text-2xl font-bold text-white mb-2">
              Something went wrong
            </h1>
            <p className="text-sis-gray-400 mb-6">
              An unexpected error occurred in the SIS AI-Lab interface.
            </p>
            
            {process.env.NODE_ENV === 'development' && this.state.error && (
              <details className="text-left mb-6">
                <summary className="cursor-pointer text-sis-blue-400 mb-2">
                  Error Details
                </summary>
                <pre className="text-xs bg-sis-gray-800 p-3 rounded overflow-auto text-red-400">
                  {this.state.error.toString()}
                  {this.state.errorInfo}
                </pre>
              </details>
            )}
            
            <div className="space-y-2">
              <button
                onClick={this.handleReset}
                className="btn-primary w-full"
              >
                Try Again
              </button>
              <button
                onClick={() => window.location.reload()}
                className="btn-secondary w-full"
              >
                Reload Page
              </button>
            </div>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}

export default ErrorBoundary