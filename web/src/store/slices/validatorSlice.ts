import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import type { ValidationReport, CheckResult } from '../../types/design'

export interface ValidatorState {
  currentReport: ValidationReport | null
  isValidating: boolean
  validationProgress: number
  
  history: ValidationReport[]
  
  // Configuration
  enabledChecks: {
    syntax: boolean
    timing: boolean
    power: boolean
    thermal: boolean
    safety: boolean
  }
  
  // Auto-validation
  autoValidate: boolean
  lastValidation: number
}

const initialState: ValidatorState = {
  currentReport: null,
  isValidating: false,
  validationProgress: 0,
  
  history: [],
  
  enabledChecks: {
    syntax: true,
    timing: true,
    power: true,
    thermal: true,
    safety: true,
  },
  
  autoValidate: true,
  lastValidation: 0,
}

const validatorSlice = createSlice({
  name: 'validator',
  initialState,
  reducers: {
    startValidation: (state) => {
      state.isValidating = true
      state.validationProgress = 0
    },
    
    updateValidationProgress: (state, action: PayloadAction<number>) => {
      state.validationProgress = action.payload
    },
    
    setValidationReport: (state, action: PayloadAction<ValidationReport>) => {
      state.currentReport = action.payload
      state.isValidating = false
      state.validationProgress = 100
      state.lastValidation = Date.now()
      
      // Add to history
      state.history.unshift(action.payload)
      if (state.history.length > 10) {
        state.history = state.history.slice(0, 10)
      }
    },
    
    setEnabledChecks: (state, action: PayloadAction<Partial<ValidatorState['enabledChecks']>>) => {
      state.enabledChecks = { ...state.enabledChecks, ...action.payload }
    },
    
    setAutoValidate: (state, action: PayloadAction<boolean>) => {
      state.autoValidate = action.payload
    },
    
    clearHistory: (state) => {
      state.history = []
    },
  },
})

export const {
  startValidation,
  updateValidationProgress,
  setValidationReport,
  setEnabledChecks,
  setAutoValidate,
  clearHistory,
} = validatorSlice.actions

export default validatorSlice.reducer