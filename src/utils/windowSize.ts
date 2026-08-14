import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import type { LauncherConfig } from '../types'

export const WINDOW_PRESETS = [
  { value: '1200x720', label: '1200×720', width: 1200, height: 720 },
  { value: '1280x720', label: '1280×720', width: 1280, height: 720 },
  { value: '1600x900', label: '1600×900', width: 1600, height: 900 },
  { value: 'fullscreen', label: '全屏', width: 0, height: 0 },
  { value: 'custom', label: '自定义', width: 0, height: 0 },
]

export function getWindowSize(config: LauncherConfig): { width: number; height: number; fullscreen: boolean } {
  if (config.window_size === 'fullscreen') {
    return { width: 1200, height: 720, fullscreen: true }
  }
  if (config.window_size === 'custom') {
    return {
      width: Math.max(800, config.window_width || 1200),
      height: Math.max(450, config.window_height || 720),
      fullscreen: false,
    }
  }
  const preset = WINDOW_PRESETS.find((p) => p.value === config.window_size)
  return {
    width: preset?.width ?? 1200,
    height: preset?.height ?? 720,
    fullscreen: false,
  }
}

export async function applyWindowSize(config: LauncherConfig) {
  const win = getCurrentWindow()
  const { width, height, fullscreen } = getWindowSize(config)
  await win.setFullscreen(fullscreen)
  if (!fullscreen) {
    await win.setSize(new LogicalSize(width, height))
  }
}
