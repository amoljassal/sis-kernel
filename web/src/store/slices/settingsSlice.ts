import { createSlice, PayloadAction } from '@reduxjs/toolkit'

export interface SettingsState {
  theme: 'dark' | 'light' | 'auto'
  safetyMode: 'beginner' | 'advanced' | 'pro'
  autoSave: boolean
  autoSaveInterval: number // minutes
  
  // Editor preferences
  editor: {
    fontSize: number
    tabSize: number
    wordWrap: boolean
    minimap: boolean
  }
  
  // Hardware preferences
  hardware: {
    defaultTarget: string
    cloudProvider: string
    autoValidate: boolean
  }
  
  // Collaboration
  collaboration: {
    enabled: boolean
    showCursors: boolean
    notifications: boolean
  }
}

const initialState: SettingsState = {
  theme: 'dark',
  safetyMode: 'beginner',
  autoSave: true,
  autoSaveInterval: 5,
  
  editor: {
    fontSize: 14,
    tabSize: 2,
    wordWrap: true,
    minimap: true,
  },
  
  hardware: {
    defaultTarget: '',
    cloudProvider: 'aws',
    autoValidate: true,
  },
  
  collaboration: {
    enabled: false,
    showCursors: true,
    notifications: true,
  },
}

const settingsSlice = createSlice({
  name: 'settings',
  initialState,
  reducers: {
    setTheme: (state, action: PayloadAction<'dark' | 'light' | 'auto'>) => {
      state.theme = action.payload
    },
    setSafetyMode: (state, action: PayloadAction<'beginner' | 'advanced' | 'pro'>) => {
      state.safetyMode = action.payload
    },
    setAutoSave: (state, action: PayloadAction<boolean>) => {
      state.autoSave = action.payload
    },
    setAutoSaveInterval: (state, action: PayloadAction<number>) => {
      state.autoSaveInterval = action.payload
    },
    updateEditorSettings: (state, action: PayloadAction<Partial<SettingsState['editor']>>) => {
      state.editor = { ...state.editor, ...action.payload }
    },
    updateHardwareSettings: (state, action: PayloadAction<Partial<SettingsState['hardware']>>) => {
      state.hardware = { ...state.hardware, ...action.payload }
    },
    updateCollaborationSettings: (state, action: PayloadAction<Partial<SettingsState['collaboration']>>) => {
      state.collaboration = { ...state.collaboration, ...action.payload }
    },
  },
})

export const {
  setTheme,
  setSafetyMode,
  setAutoSave,
  setAutoSaveInterval,
  updateEditorSettings,
  updateHardwareSettings,
  updateCollaborationSettings,
} = settingsSlice.actions

export default settingsSlice.reducer