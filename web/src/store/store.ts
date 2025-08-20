import { configureStore } from '@reduxjs/toolkit'
import designerReducer from './slices/designerSlice'
import validatorReducer from './slices/validatorSlice'
import hardwareReducer from './slices/hardwareSlice'
import collaborationReducer from './slices/collaborationSlice'
import settingsReducer from './slices/settingsSlice'

export const store = configureStore({
  reducer: {
    designer: designerReducer,
    validator: validatorReducer,
    hardware: hardwareReducer,
    collaboration: collaborationReducer,
    settings: settingsReducer,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        // Ignore these action types for WebRTC and WASM objects
        ignoredActions: [
          'collaboration/setPeerConnection',
          'hardware/setWasmModule',
        ],
        // Ignore these field paths in all actions
        ignoredActionsPaths: ['payload.connection', 'payload.wasmModule'],
        // Ignore these paths in the state
        ignoredPaths: [
          'collaboration.peerConnections',
          'hardware.wasmModule',
        ],
      },
    }),
  devTools: process.env.NODE_ENV !== 'production',
})

export type RootState = ReturnType<typeof store.getState>
export type AppDispatch = typeof store.dispatch