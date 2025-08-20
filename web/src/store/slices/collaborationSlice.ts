import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import type { CollaborationCursor, DesignComment, DesignVersion } from '../../types/design'

export interface CollaborationState {
  // Real-time collaboration
  isEnabled: boolean
  currentUserId: string
  collaborators: Record<string, {
    id: string
    name: string
    color: string
    cursor: {
      x: number
      y: number
      timestamp: number
      visible: boolean
    }
    isActive: boolean
    lastSeen: number
  }>
  cursors: CollaborationCursor[]
  
  // Comments
  comments: DesignComment[]
  
  // Version control
  versions: DesignVersion[]
  currentVersion: string | null
  
  // Connection management
  peerConnections: Map<string, RTCPeerConnection>
  localPeerId: string | null
  
  // Conflict resolution
  conflicts: {
    id: string
    type: 'node' | 'connection'
    localChange: any
    remoteChange: any
    timestamp: number
  }[]
}

const initialState: CollaborationState = {
  isEnabled: false,
  currentUserId: `user_${Math.random().toString(36).substr(2, 9)}`,
  collaborators: {},
  cursors: [],
  
  comments: [],
  
  versions: [],
  currentVersion: null,
  
  peerConnections: new Map(),
  localPeerId: null,
  
  conflicts: [],
}

const collaborationSlice = createSlice({
  name: 'collaboration',
  initialState,
  reducers: {
    setCollaborationEnabled: (state, action: PayloadAction<boolean>) => {
      state.isEnabled = action.payload
      if (!action.payload) {
        state.collaborators = {}
        state.cursors = []
      }
    },
    
    setLocalPeerId: (state, action: PayloadAction<string>) => {
      state.localPeerId = action.payload
    },
    
    addCollaborator: (state, action: PayloadAction<{
      id: string
      name: string
      color: string
      cursor: {
        x: number
        y: number
        timestamp: number
        visible: boolean
      }
      isActive: boolean
      lastSeen: number
    }>) => {
      const collaborator = action.payload
      state.collaborators[collaborator.id] = collaborator
    },
    
    removeCollaborator: (state, action: PayloadAction<string>) => {
      const userId = action.payload
      delete state.collaborators[userId]
      state.cursors = state.cursors.filter(cursor => cursor.userId !== userId)
    },
    
    updateCursorPosition: (state, action: PayloadAction<{
      userId: string
      position: {
        x: number
        y: number
        timestamp: number
        visible: boolean
      }
    }>) => {
      const { userId, position } = action.payload
      if (state.collaborators[userId]) {
        state.collaborators[userId].cursor = position
        state.collaborators[userId].lastSeen = Date.now()
      }
    },
    
    updateCursor: (state, action: PayloadAction<CollaborationCursor>) => {
      const cursor = action.payload
      const index = state.cursors.findIndex(c => c.userId === cursor.userId)
      if (index !== -1) {
        state.cursors[index] = cursor
      } else {
        state.cursors.push(cursor)
      }
    },
    
    removeCursor: (state, action: PayloadAction<string>) => {
      const userId = action.payload
      state.cursors = state.cursors.filter(cursor => cursor.userId !== userId)
    },
    
    addComment: (state, action: PayloadAction<DesignComment>) => {
      state.comments.push(action.payload)
    },
    
    updateComment: (state, action: PayloadAction<{ id: string; updates: Partial<DesignComment> }>) => {
      const { id, updates } = action.payload
      const index = state.comments.findIndex(comment => comment.id === id)
      if (index !== -1) {
        state.comments[index] = { ...state.comments[index], ...updates }
      }
    },
    
    deleteComment: (state, action: PayloadAction<string>) => {
      const commentId = action.payload
      state.comments = state.comments.filter(comment => comment.id !== commentId)
    },
    
    addVersion: (state, action: PayloadAction<DesignVersion>) => {
      state.versions.unshift(action.payload)
      state.currentVersion = action.payload.id
    },
    
    setCurrentVersion: (state, action: PayloadAction<string>) => {
      state.currentVersion = action.payload
    },
    
    setPeerConnection: (state, action: PayloadAction<{ peerId: string; connection: RTCPeerConnection }>) => {
      const { peerId, connection } = action.payload
      // Note: This won't be serialized due to serializableCheck configuration
      state.peerConnections.set(peerId, connection)
    },
    
    removePeerConnection: (state, action: PayloadAction<string>) => {
      const peerId = action.payload
      state.peerConnections.delete(peerId)
    },
    
    addConflict: (state, action: PayloadAction<{
      id: string
      type: 'node' | 'connection'
      localChange: any
      remoteChange: any
    }>) => {
      state.conflicts.push({
        ...action.payload,
        timestamp: Date.now(),
      })
    },
    
    resolveConflict: (state, action: PayloadAction<string>) => {
      const conflictId = action.payload
      state.conflicts = state.conflicts.filter(conflict => conflict.id !== conflictId)
    },
  },
})

export const {
  setCollaborationEnabled,
  setLocalPeerId,
  addCollaborator,
  removeCollaborator,
  updateCursorPosition,
  updateCursor,
  removeCursor,
  addComment,
  updateComment,
  deleteComment,
  addVersion,
  setCurrentVersion,
  setPeerConnection,
  removePeerConnection,
  addConflict,
  resolveConflict,
} = collaborationSlice.actions

export default collaborationSlice.reducer