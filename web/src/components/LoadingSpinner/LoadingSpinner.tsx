import React from 'react'

interface LoadingSpinnerProps {
  size?: 'sm' | 'md' | 'lg'
  text?: string
  className?: string
}

const LoadingSpinner: React.FC<LoadingSpinnerProps> = ({ 
  size = 'md', 
  text = 'Loading...',
  className = ''
}) => {
  const sizeClasses = {
    sm: 'w-4 h-4',
    md: 'w-8 h-8',
    lg: 'w-12 h-12'
  }

  return (
    <div className={`flex flex-col items-center justify-center p-8 ${className}`}>
      <div 
        className={`${sizeClasses[size]} border-4 border-sis-gray-600 border-t-sis-blue-500 rounded-full animate-spin`}
        role="status"
        aria-label="Loading"
      />
      {text && (
        <p className="mt-4 text-sis-gray-400 text-sm animate-pulse">
          {text}
        </p>
      )}
    </div>
  )
}

export default LoadingSpinner