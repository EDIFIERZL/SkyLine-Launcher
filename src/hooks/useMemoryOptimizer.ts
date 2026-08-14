import { useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from '../stores/settingsStore'

let lastOptimizeTime = 0
let optimizeLock = false

async function silentOptimize() {
  if (optimizeLock) return
  const now = Date.now()
  if (now - lastOptimizeTime < 2000) return
  optimizeLock = true
  lastOptimizeTime = now
  try {
    await invoke('optimize_memory_silent')
  } catch {
    
  } finally {
    optimizeLock = false
  }
}

async function aggressiveOptimize() {
  if (optimizeLock) return
  optimizeLock = true
  try {
    await invoke('optimize_memory_aggressive')
  } catch {
    
  } finally {
    optimizeLock = false
  }
}

async function bestOptimize() {
  if (optimizeLock) return
  optimizeLock = true
  try {
    await invoke('optimize_memory_best')
  } catch {
    
  } finally {
    optimizeLock = false
  }
}

let periodicStarted = false

export function useMemoryOptimizer() {
  const { config } = useSettingsStore()
  const liquidGlass = config.liquid_glass

  useEffect(() => {
    if (liquidGlass && !periodicStarted) {
      periodicStarted = true
      invoke('start_periodic_optimization').catch(() => {})
    } else if (!liquidGlass && periodicStarted) {
      periodicStarted = false
      invoke('stop_periodic_optimization').catch(() => {})
    }
  }, [liquidGlass])

  const onTabSwitch = useCallback(() => {
    silentOptimize()
  }, [])

  const onDownloadComplete = useCallback(() => {
    silentOptimize()
  }, [])

  const onGameLaunchPre = useCallback(() => {
    
    bestOptimize()
  }, [])

  const onGameLaunchPost = useCallback(() => {
    
    silentOptimize()
  }, [])

  return { onTabSwitch, onDownloadComplete, onGameLaunchPre, onGameLaunchPost }
}

export function triggerSilentOptimize() {
  silentOptimize()
}

export function triggerAggressiveOptimize() {
  aggressiveOptimize()
}

export function triggerBestOptimize() {
  bestOptimize()
}
