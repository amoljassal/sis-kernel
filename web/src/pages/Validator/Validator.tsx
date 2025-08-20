import React from 'react'
import LoadingSpinner from '../../components/LoadingSpinner/LoadingSpinner'

const Validator: React.FC = () => {
  return (
    <div className="h-full flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-2xl font-bold text-white mb-4">Design Validator</h1>
        <p className="text-sis-gray-400 mb-6">Real-time validation coming soon...</p>
        <LoadingSpinner text="Initializing validation framework" />
      </div>
    </div>
  )
}

export default Validator