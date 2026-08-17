import { create } from 'zustand'
import type { LauncherConfig } from '../types'

interface SettingsState {
  config: LauncherConfig
  setConfig: (config: LauncherConfig) => void
}

const defaultConfig: LauncherConfig = {
  theme: 'dark',
  language: 'zh-CN',
  download_threads: 64,
  max_memory: 4096,
  min_memory: 1024,
  java_path: null,
  keep_launcher_open: true,
  close_after_launch: false,
  theme_mode: 'dark',
  accent_color: '#3b82f6',
  background_type: 'none',
  background_value: '',
  ui_scale: 1.0,
  font_size: 'normal',
  sidebar_collapsed: false,
  use_mirror: true,
  window_size: '1200x720',
  window_width: 1200,
  window_height: 720,
  version_settings: {},
  instance_folders: [],
  last_selected_instance: null,
  download_source: 'auto',
  server_address: '',
  server_name: '',
  hide_server_card: false,
  hide_mp_quick_card: false,
  server_card_size: 80,
  liquid_glass: false,
  liquid_glass_mode: 'normal',
  liquid_glass_intensity: 1.0,
  jvm_args: [],
  opengl_compat: false,
  home_style: 'full',
  onboarding_completed: false,
}

export const useSettingsStore = create<SettingsState>((set) => ({
  config: defaultConfig,
  setConfig: (config) => set({ config }),
}))
