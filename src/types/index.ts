export interface Instance {
  id: string
  name: string
  version_id: string
  modloader: ModLoader
  created_at: string
  last_played: string | null
  play_time: number
  icon_path: string | null
  java_path: string | null
  min_memory: number
  max_memory: number
  jvm_args: string[]
  game_args: string[]
  window_width: number
  window_height: number
  custom_resolution: boolean
  server_ip: string | null
  isolation_mode: IsolationMode
  version_isolation?: boolean
  external: boolean
  minecraft_root: string | null
  game_dir_override: string | null
}

export type IsolationMode = 'always' | 'modded' | 'never'

export type ModLoader =
  | { Vanilla: Record<string, never> }
  | { Forge: string }
  | { Fabric: string }
  | { Quilt: string }
  | { NeoForge: string }

export interface JavaInfo {
  path: string
  version: string
  major_version: number
  is_64bit: boolean
  architecture: string
  vendor: string
  is_jdk: boolean
}

export interface AuthSession {
  access_token: string
  username: string
  uuid: string
  user_type: string
  refresh_token?: string | null
  expires_at?: number
}

export interface MicrosoftDeviceCode {
  device_code: string
  user_code: string
  verification_uri: string
  verification_uri_complete?: string
  interval: number
  expires_in: number
}

export interface LittleSkinDeviceCode {
  device_code: string
  user_code: string
  verification_uri: string
  verification_uri_complete?: string
  interval: number
  expires_in: number
}

export interface NideServerInfo {
  server_id: string
  name: string
  server_ip: string | null
  server_port: number | null
}

export interface AuthlibInjectorServer {
  name: string
  url: string
  register_url: string | null
}

export type AuthType = 'Microsoft' | 'Mojang' | 'Offline' | 'AuthlibInjector' | 'Nide'

export interface GameLogEntry {
  level: 'info' | 'warn' | 'error' | 'debug'
  message: string
  timestamp: number
  source: LogSource
}

export interface GameLogEvent {
  instance_id: string
  level: 'info' | 'warn' | 'error' | 'debug'
  message: string
  timestamp: number
  source: LogSource
}

export interface GameProcessInfo {
  pid: number
  running: boolean
}

export interface GameExitInfo {
  instance_id: string
  exit_code: number | null
  reason: ExitReason
  play_time_secs: number
}

export type ExitReason = 'normal' | 'crash' | 'killed' | 'nowindow'

export type LogSource = 'game' | 'watcher'

export interface LaunchProgressEvent {
  instance_id: string
  stage: LaunchStage
  message: string
}

export type LaunchStage = 'jvmstarting' | 'gameloading' | 'waitingwindow' | 'running'

export interface InstallProgress {
  stage: string
  progress: number
  message: string
}

export interface LauncherConfig {
    theme: string
    language: string
    download_threads: number
    max_memory: number
    min_memory: number
    java_path: string | null
    keep_launcher_open: boolean
    close_after_launch: boolean
    theme_mode: string
    accent_color: string
    background_type: string
    background_value: string
    ui_scale: number
    font_size: string
    sidebar_collapsed: boolean
    use_mirror: boolean
    window_size: string
    window_width: number
    window_height: number
    version_settings: Record<string, VersionSetting>
    instance_folders: string[]
    active_instance_folder?: string | null
    last_selected_instance?: string | null
    download_source: string
    server_address: string
    server_name: string
    hide_server_card: boolean
    hide_mp_quick_card: boolean
    server_card_size: number
    liquid_glass: boolean
    liquid_glass_mode: string
    liquid_glass_intensity: number
    compact_mode?: boolean
    jvm_args: string[]
    opengl_compat?: boolean
    game_folder?: string | null
    home_style?: string
    show_home_news?: boolean
    onboarding_completed?: boolean
  }

export interface VersionSetting {
  java_path: string | null
  min_memory: number | null
  max_memory: number | null
  game_dir_override?: string | null
  isolation_mode?: 'always' | 'modded' | 'never'
}

export interface McmodItem {
  id: string
  title: string
  description: string
  mcmod_url: string
  modrinth_url: string | null
  curseforge_url: string | null
}

export interface InstalledVersion {
  id: string
  has_jar: boolean
}

export interface VersionEntry {
  id: string
  type: string
  url: string
  time: string
  release_time: string
}

export interface VersionManifest {
  latest: { release: string; snapshot: string }
  versions: VersionEntry[]
}

export interface ModUpdateInfo {
  mod_path: string
  current_version: string
  latest_version: string
  download_url: string
  filename: string
  project_id: string
  version_id: string
}

export interface ModInfo {
  file_name: string
  path: string
  size_kb: number
  enabled: boolean
  mod_id: string | null
  name: string | null
  description: string | null
  version: string | null
  mc_versions: string[] | null
  side: string | null
  url: string | null
  author: string | null
  mod_loader: ModLoaderType
  has_update: boolean
  latest_version: string | null
  update_url: string | null
  icon_url?: string | null
}

export type ModLoaderType = 'fabric' | 'forge' | 'neoforge' | 'quilt' | 'unknown'

export interface OptiFineVersion {
  mc_version: string
  version: string
  mirror_url: string
  date: string | null
}

export interface LoaderVersion {
  version: string
  mc_version: string
  stable: boolean
}

export interface CrashAnalysis {
  stage: string
  severity: CrashSeverity
  exception: string | null
  description: string | null
  suggestions: string[]
  report_path: string | null
  conflicting_mods: string[]
  detected_mods: string[]
  is_abnormal: boolean
}

export type CrashSeverity = 'critical' | 'error' | 'warning' | 'info'

export interface Account {
  id: string
  username: string
  auth_type: AuthType
  access_token: string | null
  refresh_token: string | null
  uuid: string | null
  authlib_server_url: string | null
  nide_server_id: string | null
  client_token: string | null
}

export interface WorldInfo {
  name: string
  path: string
  game_mode: string
  seed: string | null
  version: string | null
  last_played: string | null
  play_time: number
  size_kb: number
  icon: string | null
  is_hardcore: boolean
  difficulty: string | null
}

export interface ScreenshotInfo {
  path: string
  file_name: string
  size_kb: number
  modified_at: string | null
}

export interface ServerStatus {
  online: boolean
  host: string
  port: number
  description: string
  version_name: string
  version_protocol: number
  players_online: number
  players_max: number
  player_names: string[]
  favicon: string | null
  favicon_path: string | null
  latency_ms: number
  mod_info: {
    mod_type: string
    mod_list: string[]
  } | null
  error: string | null
}

export interface MultiplayerServer {
  name: string
  address: string
  ip: string
  port: number
}

export interface ModrinthProject {
  slug: string
  title: string
  description: string
  versions: string[]
  client_side: string
  server_side: string
  categories: string[]
  license: string | null
  icon_url: string | null
  project_id: string | null
  author: string | null
  downloads: number | null
  date_modified: string | null
}

export interface ModrinthProjectDetail {
  slug: string
  title: string
  description: string
  body: string
  downloads: number
  follows: number
  published: string
  updated: string
  license: string | null
  categories: string[]
  client_side: string
  server_side: string
  project_type: string
  icon_url: string | null
  game_versions: string[]
  loaders: string[]
  source_url: string | null
  wiki_url: string | null
  issues_url: string | null
  team: string[]
}

export interface ModrinthVersion {
  id: string
  project_id: string
  name: string
  version_number: string
  version_type?: string | null
  game_versions: string[]
  loaders: string[]
  files: { url: string; filename: string; primary: boolean; size: number }[]
  dependencies?: ModrinthDependency[]
  date_published: string
  changelog: string | null
}

export interface ModrinthDependency {
  project_id?: string | null
  version_id?: string | null
  dependency_type: string
  file_name?: string | null
}

export interface CurseForgeMod {
  id: number
  name: string
  slug: string
  summary: string
  downloads: number
  category: string | null
  logo_url: string | null
  authors: string[]
  game_versions: string[]
  date_modified: string
}

export interface CurseForgeFile {
  id: number
  display_name: string
  file_name: string
  file_date: string
  game_versions: string[]
  loaders: string[]
  release_type?: number | null
  file_length: number
  download_url: string
}

export type DownloadKind = 'mods' | 'resourcepacks' | 'shaderpacks' | 'datapacks' | 'maps' | 'modpacks'
