import React from 'react'
import LoadingSpinner from '../../components/LoadingSpinner/LoadingSpinner'

const Marketplace: React.FC = () => {
  return (
    <div className="h-full flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-2xl font-bold text-white mb-4">IP Marketplace</h1>
        <p className="text-sis-gray-400 mb-6">Business platform coming in Phase 4D...</p>
        <LoadingSpinner text="Loading marketplace" />
      </div>
    </div>
  )
}

export default Marketplace