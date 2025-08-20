/**
 * React hooks for WebAssembly integration
 */

import { useState, useEffect, useCallback } from 'react'
import { useDispatch } from 'react-redux'
import { setWasmModule } from '../store/slices/hardwareSlice'
import { SisKernelAPI, getSisKernel } from '../wasm/sisKernel'
import type { DesignNode, DesignConnection, ValidationReport } from '../types/design'

export interface WasmState {
  isLoaded: boolean
  isLoading: boolean
  error: string | null
  kernel: SisKernelAPI | null
}

/**
 * Hook for managing WASM module lifecycle
 */
export const useWasm = () => {
  const dispatch = useDispatch()
  const [state, setState] = useState<WasmState>({
    isLoaded: false,
    isLoading: false,
    error: null,
    kernel: null
  })

  const loadWasm = useCallback(async () => {
    if (state.isLoaded || state.isLoading) return

    setState(prev => ({ ...prev, isLoading: true, error: null }))

    try {
      console.log('Initializing SIS Kernel WASM...')
      const kernel = await getSisKernel()
      
      if (kernel.isInitialized()) {
        setState({
          isLoaded: true,
          isLoading: false,
          error: null,
          kernel
        })
        
        // Update Redux store
        dispatch(setWasmModule(kernel))
        
        console.log('WASM module loaded successfully:', kernel.getVersion())
      } else {
        throw new Error('Failed to initialize kernel')
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      console.error('Failed to load WASM:', errorMessage)
      setState({
        isLoaded: false,
        isLoading: false,
        error: errorMessage,
        kernel: null
      })
    }
  }, [dispatch, state.isLoaded, state.isLoading])

  const unloadWasm = useCallback(() => {
    if (state.kernel) {
      state.kernel.destroy()
    }
    setState({
      isLoaded: false,
      isLoading: false,
      error: null,
      kernel: null
    })
  }, [state.kernel])

  // Auto-load on mount
  useEffect(() => {
    loadWasm()
    
    // Cleanup on unmount
    return () => {
      if (state.kernel) {
        state.kernel.destroy()
      }
    }
  }, []) // Only run on mount

  return {
    ...state,
    loadWasm,
    unloadWasm,
    reload: () => {
      unloadWasm()
      setTimeout(loadWasm, 100)
    }
  }
}

/**
 * Hook for design validation using WASM
 */
export const useDesignValidation = () => {
  const { kernel, isLoaded } = useWasm()
  const [isValidating, setIsValidating] = useState(false)
  const [lastReport, setLastReport] = useState<ValidationReport | null>(null)

  const validateDesign = useCallback(async (
    nodes: DesignNode[], 
    connections: DesignConnection[]
  ): Promise<ValidationReport> => {
    if (!kernel || !isLoaded) {
      throw new Error('WASM kernel not available')
    }

    setIsValidating(true)
    try {
      const report = await kernel.validateDesign(nodes, connections)
      setLastReport(report)
      return report
    } finally {
      setIsValidating(false)
    }
  }, [kernel, isLoaded])

  const runPreflightChecks = useCallback((
    nodes: DesignNode[],
    connections: DesignConnection[]
  ): number => {
    if (!kernel || !isLoaded) {
      throw new Error('WASM kernel not available')
    }
    
    return kernel.runPreflightChecks(nodes, connections)
  }, [kernel, isLoaded])

  return {
    validateDesign,
    runPreflightChecks,
    isValidating,
    lastReport,
    isAvailable: isLoaded && kernel !== null
  }
}

/**
 * Hook for HDL generation using WASM  
 */
export const useHDLGeneration = () => {
  const { kernel, isLoaded } = useWasm()
  const [isGenerating, setIsGenerating] = useState(false)
  const [lastHDL, setLastHDL] = useState<string>('')

  const generateHDL = useCallback(async (
    nodes: DesignNode[],
    connections: DesignConnection[],
    target: 'verilog' | 'vhdl' | 'systemverilog'
  ): Promise<string> => {
    if (!kernel || !isLoaded) {
      throw new Error('WASM kernel not available')
    }

    setIsGenerating(true)
    try {
      const hdl = kernel.generateHDL(nodes, connections, target)
      setLastHDL(hdl)
      return hdl
    } finally {
      setIsGenerating(false)
    }
  }, [kernel, isLoaded])

  const synthesizeDesign = useCallback(async (
    hdlCode: string,
    target: string
  ) => {
    if (!kernel || !isLoaded) {
      throw new Error('WASM kernel not available')
    }

    return await kernel.synthesizeDesign(hdlCode, target)
  }, [kernel, isLoaded])

  return {
    generateHDL,
    synthesizeDesign,
    isGenerating,
    lastHDL,
    isAvailable: isLoaded && kernel !== null
  }
}

/**
 * Hook for hardware status monitoring
 */
export const useHardwareStatus = () => {
  const { kernel, isLoaded } = useWasm()
  const [status, setStatus] = useState<any>(null)
  const [isRefreshing, setIsRefreshing] = useState(false)

  const refreshStatus = useCallback(async () => {
    if (!kernel || !isLoaded) return

    setIsRefreshing(true)
    try {
      const newStatus = kernel.getHardwareStatus()
      setStatus(newStatus)
    } catch (error) {
      console.error('Failed to get hardware status:', error)
    } finally {
      setIsRefreshing(false)
    }
  }, [kernel, isLoaded])

  // Auto-refresh on load
  useEffect(() => {
    if (isLoaded) {
      refreshStatus()
    }
  }, [isLoaded, refreshStatus])

  return {
    status,
    isRefreshing,
    refreshStatus,
    isAvailable: isLoaded && kernel !== null
  }
}

/**
 * Hook for WASM performance monitoring
 */
export const useWasmPerformance = () => {
  const { kernel, isLoaded } = useWasm()
  const [metrics, setMetrics] = useState<any>({})

  const updateMetrics = useCallback(() => {
    if (!kernel || !isLoaded) return

    try {
      const newMetrics = kernel.getPerformanceMetrics()
      setMetrics(newMetrics)
    } catch (error) {
      console.error('Failed to get performance metrics:', error)
    }
  }, [kernel, isLoaded])

  // Update metrics periodically
  useEffect(() => {
    if (!isLoaded) return

    updateMetrics()
    const interval = setInterval(updateMetrics, 5000) // Every 5 seconds

    return () => clearInterval(interval)
  }, [isLoaded, updateMetrics])

  return {
    metrics,
    updateMetrics,
    isAvailable: isLoaded && kernel !== null
  }
}