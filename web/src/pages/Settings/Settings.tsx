import React from 'react'
import LoadingSpinner from '../../components/LoadingSpinner/LoadingSpinner'

const Settings: React.FC = () => {
  return (
    <div className="h-full flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-2xl font-bold text-white mb-4">Settings</h1>
        <p className="text-sis-gray-400 mb-6">Configuration panel in development...</p>
        <LoadingSpinner text="Loading settings" />
      </div>
    </div>
  )
}

export default Settings