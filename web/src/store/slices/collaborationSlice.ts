import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import type { CollaborationCursor, DesignComment, DesignVersion } from '../../types/design'

export interface CollaborationState {
  // Real-time collaboration
  isConnected: boolean
  connectedUsers: {
    id: string
    name: string
    color: string
    lastSeen: number
  }[]
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
  isConnected: false,
  connectedUsers: [],
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
    setConnectionStatus: (state, action: PayloadAction<boolean>) => {
      state.isConnected = action.payload
      if (!action.payload) {
        state.connectedUsers = []
        state.cursors = []
      }
    },
    
    setLocalPeerId: (state, action: PayloadAction<string>) => {
      state.localPeerId = action.payload
    },
    
    addConnectedUser: (state, action: PayloadAction<{
      id: string
      name: string
      color: string
    }>) => {
      const { id, name, color } = action.payload
      const existing = state.connectedUsers.find(user => user.id === id)
      if (!existing) {
        state.connectedUsers.push({
          id,
          name,
          color,
          lastSeen: Date.now(),
        })
      }
    },
    
    removeConnectedUser: (state, action: PayloadAction<string>) => {
      const userId = action.payload
      state.connectedUsers = state.connectedUsers.filter(user => user.id !== userId)
      state.cursors = state.cursors.filter(cursor => cursor.userId !== userId)
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
      const { id, updates } = action.payload\n      const index = state.comments.findIndex(comment => comment.id === id)\n      if (index !== -1) {\n        state.comments[index] = { ...state.comments[index], ...updates }\n      }\n    },\n    \n    deleteComment: (state, action: PayloadAction<string>) => {\n      const commentId = action.payload\n      state.comments = state.comments.filter(comment => comment.id !== commentId)\n    },\n    \n    addVersion: (state, action: PayloadAction<DesignVersion>) => {\n      state.versions.unshift(action.payload)\n      state.currentVersion = action.payload.id\n    },\n    \n    setCurrentVersion: (state, action: PayloadAction<string>) => {\n      state.currentVersion = action.payload\n    },\n    \n    setPeerConnection: (state, action: PayloadAction<{ peerId: string; connection: RTCPeerConnection }>) => {\n      const { peerId, connection } = action.payload\n      // Note: This won't be serialized due to serializableCheck configuration\n      state.peerConnections.set(peerId, connection)\n    },\n    \n    removePeerConnection: (state, action: PayloadAction<string>) => {\n      const peerId = action.payload\n      state.peerConnections.delete(peerId)\n    },\n    \n    addConflict: (state, action: PayloadAction<{\n      id: string\n      type: 'node' | 'connection'\n      localChange: any\n      remoteChange: any\n    }>) => {\n      state.conflicts.push({\n        ...action.payload,\n        timestamp: Date.now(),\n      })\n    },\n    \n    resolveConflict: (state, action: PayloadAction<string>) => {\n      const conflictId = action.payload\n      state.conflicts = state.conflicts.filter(conflict => conflict.id !== conflictId)\n    },\n  },\n})\n\nexport const {\n  setConnectionStatus,\n  setLocalPeerId,\n  addConnectedUser,\n  removeConnectedUser,\n  updateCursor,\n  removeCursor,\n  addComment,\n  updateComment,\n  deleteComment,\n  addVersion,\n  setCurrentVersion,\n  setPeerConnection,\n  removePeerConnection,\n  addConflict,\n  resolveConflict,\n} = collaborationSlice.actions\n\nexport default collaborationSlice.reducer"