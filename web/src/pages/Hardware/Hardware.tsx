import React from 'react'
import LoadingSpinner from '../../components/LoadingSpinner/LoadingSpinner'

const Hardware: React.FC = () => {
  return (
    <div className="h-full flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-2xl font-bold text-white mb-4">Hardware Integration</h1>
        <p className="text-sis-gray-400 mb-6">FPGA connectivity coming in Phase 4C...</p>
        <LoadingSpinner text="Preparing hardware interfaces" />
      </div>
    </div>
  )
}

export default Hardware