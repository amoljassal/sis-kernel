import React, { useEffect, useState, useCallback } from 'react'
import { useSelector, useDispatch } from 'react-redux'
import type { RootState } from '../../../store/store'
import { updateCursorPosition, addCollaborator, removeCollaborator } from '../../../store/slices/collaborationSlice'

interface CursorPosition {
  x: number
  y: number
  timestamp: number
  visible: boolean
}

interface Collaborator {
  id: string
  name: string
  color: string
  cursor: CursorPosition
  isActive: boolean
  lastSeen: number
}

interface CollaborationCursorsProps {
  className?: string
  canvasRef: React.RefObject<HTMLElement>
}

// Individual cursor component
const CollaboratorCursor: React.FC<{ 
  collaborator: Collaborator 
  isCurrentUser?: boolean 
}> = ({ collaborator, isCurrentUser = false }) => {
  const [isMoving, setIsMoving] = useState(false)
  const [lastPosition, setLastPosition] = useState({ x: collaborator.cursor.x, y: collaborator.cursor.y })

  useEffect(() => {
    const { x, y } = collaborator.cursor
    if (x !== lastPosition.x || y !== lastPosition.y) {
      setIsMoving(true)
      setLastPosition({ x, y })
      
      const timeout = setTimeout(() => setIsMoving(false), 150)
      return () => clearTimeout(timeout)
    }
  }, [collaborator.cursor.x, collaborator.cursor.y, lastPosition])

  if (!collaborator.cursor.visible || isCurrentUser) {
    return null
  }

  const cursorStyle: React.CSSProperties = {
    position: 'absolute',
    left: collaborator.cursor.x,
    top: collaborator.cursor.y,
    pointerEvents: 'none',
    zIndex: 9999,
    transform: 'translate(-2px, -2px)',
    transition: isMoving ? 'none' : 'all 0.1s ease-out',
  }

  return (
    <div style={cursorStyle}>
      {/* Cursor pointer */}
      <div 
        className="relative"
        style={{
          transform: isMoving ? 'scale(1.2)' : 'scale(1)',
          transition: 'transform 0.1s ease-out'
        }}
      >
        <svg width="20" height="20" viewBox="0 0 20 20" className="drop-shadow-lg">
          <path
            d="M4 4L16 10L10 12L8 18L4 4Z"
            fill={collaborator.color}
            stroke="white"
            strokeWidth="1"
          />
        </svg>
        
        {/* User name label */}
        <div
          className="absolute top-5 left-3 px-2 py-1 rounded text-xs font-medium whitespace-nowrap shadow-lg"
          style={{
            backgroundColor: collaborator.color,
            color: getContrastColor(collaborator.color),
            opacity: isMoving ? 1 : 0.8,
            transform: `scale(${isMoving ? 1 : 0.9})`,
            transition: 'all 0.1s ease-out'
          }}
        >
          {collaborator.name}
        </div>
      </div>
      
      {/* Movement trail effect */}
      {isMoving && (
        <div
          className="absolute rounded-full animate-ping"
          style={{
            width: '8px',
            height: '8px',
            backgroundColor: collaborator.color,
            opacity: 0.4,
            left: '6px',
            top: '6px'
          }}
        />
      )}
    </div>
  )
}

// Get contrasting text color for cursor labels
function getContrastColor(hexColor: string): string {
  // Convert hex to RGB
  const hex = hexColor.replace('#', '')
  const r = parseInt(hex.substr(0, 2), 16)
  const g = parseInt(hex.substr(2, 2), 16)
  const b = parseInt(hex.substr(4, 2), 16)
  
  // Calculate luminance
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255
  
  return luminance > 0.5 ? '#000000' : '#FFFFFF'
}

// Generate a consistent color for a user ID
function getUserColor(userId: string): string {
  const colors = [
    '#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', '#FECA57',
    '#FF9FF3', '#54A0FF', '#5F27CD', '#00D2D3', '#FF9F43',
    '#2ED573', '#3742FA', '#F8B500', '#FF6348', '#7D5FFF'
  ]
  
  // Use a simple hash function to get consistent color
  let hash = 0
  for (let i = 0; i < userId.length; i++) {
    hash = userId.charCodeAt(i) + ((hash << 5) - hash)
  }
  
  return colors[Math.abs(hash) % colors.length]
}

const CollaborationCursors: React.FC<CollaborationCursorsProps> = ({ 
  className = '', 
  canvasRef 
}) => {
  const dispatch = useDispatch()
  const { collaborators, isEnabled, currentUserId } = useSelector(
    (state: RootState) => state.collaboration
  )
  const { showCursors } = useSelector(
    (state: RootState) => state.settings.collaboration
  )
  
  const [mousePosition, setMousePosition] = useState({ x: 0, y: 0 })
  const [isMouseInCanvas, setIsMouseInCanvas] = useState(false)

  // Handle mouse movement within the canvas
  const handleMouseMove = useCallback((event: MouseEvent) => {
    if (!canvasRef.current || !isEnabled) return

    const rect = canvasRef.current.getBoundingClientRect()
    const x = event.clientX - rect.left
    const y = event.clientY - rect.top
    
    setMousePosition({ x, y })
    
    // Dispatch cursor position update
    dispatch(updateCursorPosition({
      userId: currentUserId,
      position: {
        x,
        y,
        timestamp: Date.now(),
        visible: isMouseInCanvas
      }
    }))
  }, [canvasRef, dispatch, currentUserId, isEnabled, isMouseInCanvas])

  const handleMouseEnter = useCallback(() => {
    setIsMouseInCanvas(true)
  }, [])

  const handleMouseLeave = useCallback(() => {
    setIsMouseInCanvas(false)
    // Hide cursor when leaving canvas
    dispatch(updateCursorPosition({
      userId: currentUserId,
      position: {
        x: mousePosition.x,
        y: mousePosition.y,
        timestamp: Date.now(),
        visible: false
      }
    }))
  }, [dispatch, currentUserId, mousePosition])

  // Set up event listeners
  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas || !isEnabled) return

    canvas.addEventListener('mousemove', handleMouseMove)
    canvas.addEventListener('mouseenter', handleMouseEnter)
    canvas.addEventListener('mouseleave', handleMouseLeave)

    return () => {
      canvas.removeEventListener('mousemove', handleMouseMove)
      canvas.removeEventListener('mouseenter', handleMouseEnter)
      canvas.removeEventListener('mouseleave', handleMouseLeave)
    }
  }, [canvasRef, handleMouseMove, handleMouseEnter, handleMouseLeave, isEnabled])

  // Simulate collaborators joining/leaving (for demo purposes)
  useEffect(() => {
    if (!isEnabled) return

    const simulateCollaborators = () => {
      const demoUsers = [
        { id: 'user_alice', name: 'Alice Chen' },
        { id: 'user_bob', name: 'Bob Smith' },
        { id: 'user_carol', name: 'Carol Johnson' }
      ]

      // Randomly add/remove demo collaborators
      demoUsers.forEach(user => {
        if (Math.random() > 0.7) {
          dispatch(addCollaborator({
            id: user.id,
            name: user.name,
            color: getUserColor(user.id),
            cursor: {
              x: Math.random() * 800,
              y: Math.random() * 600,
              timestamp: Date.now(),
              visible: Math.random() > 0.3
            },
            isActive: true,
            lastSeen: Date.now()
          }))

          // Simulate cursor movement
          const moveInterval = setInterval(() => {
            dispatch(updateCursorPosition({
              userId: user.id,
              position: {
                x: Math.random() * 800,
                y: Math.random() * 600,
                timestamp: Date.now(),
                visible: Math.random() > 0.2
              }
            }))
          }, 2000 + Math.random() * 3000)

          // Clean up after some time
          setTimeout(() => {
            clearInterval(moveInterval)
            dispatch(removeCollaborator(user.id))
          }, 10000 + Math.random() * 20000)
        }
      })
    }

    // Start simulation after a delay
    const timeout = setTimeout(simulateCollaborators, 2000)
    return () => clearTimeout(timeout)
  }, [dispatch, isEnabled])

  // Clean up inactive collaborators
  useEffect(() => {
    const cleanupInterval = setInterval(() => {
      const now = Date.now()
      Object.values(collaborators).forEach(collaborator => {
        if (now - collaborator.lastSeen > 30000) { // 30 seconds timeout
          dispatch(removeCollaborator(collaborator.id))
        }
      })
    }, 5000)

    return () => clearInterval(cleanupInterval)
  }, [collaborators, dispatch])

  if (!isEnabled || !showCursors) {
    return null
  }

  return (
    <div className={`absolute inset-0 pointer-events-none ${className}`}>
      {/* Render all collaborator cursors */}
      {Object.values(collaborators).map(collaborator => (
        <CollaboratorCursor
          key={collaborator.id}
          collaborator={collaborator}
          isCurrentUser={collaborator.id === currentUserId}
        />
      ))}
      
      {/* Collaboration status indicator */}
      {Object.keys(collaborators).length > 0 && (
        <div className="absolute top-4 right-4 pointer-events-auto">
          <div className="bg-sis-gray-800 border border-sis-gray-600 rounded-lg p-2 shadow-lg">
            <div className="flex items-center space-x-2">
              <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
              <span className="text-xs text-white">
                {Object.keys(collaborators).length} user{Object.keys(collaborators).length !== 1 ? 's' : ''} online
              </span>
            </div>
            
            {/* Active collaborators list */}
            <div className="mt-2 space-y-1">
              {Object.values(collaborators).slice(0, 3).map(collaborator => (
                <div key={collaborator.id} className="flex items-center space-x-2">
                  <div
                    className="w-2 h-2 rounded-full"
                    style={{ backgroundColor: collaborator.color }}
                  />
                  <span className="text-xs text-sis-gray-300">{collaborator.name}</span>
                </div>
              ))}
              {Object.keys(collaborators).length > 3 && (
                <div className="text-xs text-sis-gray-400">
                  +{Object.keys(collaborators).length - 3} more
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default CollaborationCursors