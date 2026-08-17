import { useEffect, useRef, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useAuthStore } from '../stores/authStore'
import { useInstanceStore } from '../stores/instanceStore'
import { useDownloadStore } from '../stores/downloadStore'
import { useSettingsStore } from '../stores/settingsStore'
import { watchResourceInstall, invalidateCatalog } from '../lib/catalog'
import { gameModeLabel } from '../lib/utils'
import { triggerBestOptimize, triggerSilentOptimize } from '../hooks/useMemoryOptimizer'
import { InstancePanel, type InstancePanelHandle } from '../components/InstancePanel'
import { VersionSettingsPanel } from '../components/VersionSettingsPanel'
import { DownloadCenter } from '../components/DownloadCenter'
import { SkinAvatar } from '../components/SkinAvatar'
import { LaunchButton } from '../components/LaunchButton'
import { LoaderLogo } from '../components/LoaderLogo'
import CrashDialog from '../components/CrashDialog'
import { Box, Typography, Card, Button, SnackbarAlert, Chip } from '@/components/material'
import {
  FolderOpen,
  Gamepad2,
  Download,
  Puzzle,
  Image,
  Layers,
  Map,
  Server,
  Users,
  Clock,
  RefreshCw,
  ChevronRight,
  Wifi,
  WifiOff,
  Camera,
  Trash2,
  LayoutDashboard,
} from 'lucide-react'
import type { Instance, VersionManifest, GameProcessInfo, InstallProgress, ModInfo, ServerStatus, LauncherConfig, WorldInfo, LaunchProgressEvent, MultiplayerServer, AuthSession, ScreenshotInfo } from '../types'

function formatPlayTime(s: number): string {
  if (!s) return ''
  const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60)
  return h > 0 ? `${h}h ${m}m` : `${m}m`
}

function formatSize(kb: number): string {
  if (!kb) return '0 KB'
  if (kb < 1024) return `${kb} KB`
  const mb = kb / 1024
  if (mb < 1024) return `${mb.toFixed(1)} MB`
  return `${(mb / 1024).toFixed(2)} GB`
}

function getLoaderName(loader: Instance['modloader']): string {
  if (!loader) return 'Vanilla'
  const key = typeof loader === 'string' ? loader : Object.keys(loader)[0]
  if (key === 'Vanilla') return 'Vanilla'
  const ver = typeof loader === 'string' ? '' : Object.values(loader)[0] as string
  return ver ? `${key} ${ver}` : key
}

function getLoaderColor(loader: Instance['modloader']): string {
  const key = typeof loader === 'string' ? loader : Object.keys(loader)[0]
  return { Forge: 'text-red-500', NeoForge: 'text-purple-500', Fabric: 'text-amber-500', Quilt: 'text-emerald-500', Vanilla: 'text-green-500', OptiFine: 'text-blue-500' }[key] || 'text-surface-400'
}

function normalizeBase64(s: string): string {
  return s.replace(/\s/g, '').replace(/-/g, '+').replace(/_/g, '/')
}

function resolveServerIcon(favicon: string | undefined | null): string {
  if (!favicon) return ''
  const trimmed = favicon.trim()
  if (!trimmed) return ''
  if (/^data:image\//.test(trimmed)) {
    const m = /^data:image\/[^;]+;base64,([\s\S]*)$/i.exec(trimmed)
    if (m) {
      const normalized = normalizeBase64(m[1])
      if (/^[a-zA-Z0-9+/=]+$/.test(normalized) && normalized.length > 16) {
        return `data:image/png;base64,${normalized}`
      }
    }
    return trimmed
  }
  if (/^data:/i.test(trimmed)) return trimmed
  if (/^https?:\/\//.test(trimmed)) {
    const clean = normalizeBase64(trimmed.replace(/^data:[^,]*,?/i, ''))
    if (/^[a-zA-Z0-9+/=]+$/.test(clean) && clean.length > 16) {
      return `data:image/png;base64,${clean}`
    }
    return ''
  }
  return ''
}

function dataUriToBlobUrl(dataUri: string): string {
  try {
    const m = /^data:([^;,]+)?(;base64)?,([\s\S]*)$/i.exec(dataUri)
    if (!m) return ''
    const isBase64 = !!m[2]
    const bytes = isBase64
      ? Uint8Array.from(atob(normalizeBase64(m[3])), (c) => c.charCodeAt(0))
      : new TextEncoder().encode(decodeURIComponent(m[3]))
    const type = m[1] || 'image/png'
    return URL.createObjectURL(new Blob([bytes], { type }))
  } catch {
    return ''
  }
  return ''
}

function ServerIcon({ favicon, size = 16 }: { favicon: string | undefined | null; faviconPath?: string | null; size?: number }) {
  const [src, setSrc] = useState('')
  const blobRef = useRef<string | null>(null)

  useEffect(() => {
    const resolved = resolveServerIcon(favicon)
    if (resolved) {
      const objectUrl = dataUriToBlobUrl(resolved)
      if (objectUrl) {
        if (blobRef.current) URL.revokeObjectURL(blobRef.current)
        blobRef.current = objectUrl
        setSrc(objectUrl)
        return
      }
    }
    setSrc('')
  }, [favicon])

  if (!src) {
    return (
      <Box className="rounded bg-surface-100 dark:bg-surface-800 flex items-center justify-center shrink-0" style={{ width: size, height: size }}>
        <Server className="text-surface-400" style={{ width: size * 0.5, height: size * 0.5 }} />
      </Box>
    )
  }
  return (
    <img
      key={src}
      src={src}
      alt=""
      width={size}
      height={size}
      onError={() => setSrc('')}
      style={{ borderRadius: 4, objectFit: 'cover' }}
    />
  )
}

function LatencyBars({ ms }: { ms: number }) {
  const bars = ms < 30 ? 5 : ms < 80 ? 4 : ms < 150 ? 3 : ms < 300 ? 2 : 1
  const color = ms < 50 ? 'bg-green-500' : ms < 150 ? 'bg-yellow-500' : 'bg-red-500'
  return (
    <div className="flex items-end gap-[1px] h-3">
      {[1,2,3,4,5].map(i => (
        <div key={i} className={`w-[2px] rounded-sm ${i <= bars ? color : 'bg-surface-300 dark:bg-surface-600'}`} style={{ height: `${i*2+1}px` }} />
      ))}
    </div>
  )
}

function WorldIcon({ icon, size = 40 }: { icon: string | null; size?: number }) {  const [src, setSrc] = useState('')
  const blobRef = useRef<string | null>(null)

  useEffect(() => {
    if (blobRef.current) { URL.revokeObjectURL(blobRef.current); blobRef.current = null }
    if (!icon || !icon.startsWith('data:')) { setSrc(''); return }
    const objectUrl = dataUriToBlobUrl(icon)
    if (objectUrl) {
      blobRef.current = objectUrl
      setSrc(objectUrl)
    } else {
      setSrc('')
    }
  }, [icon])

  useEffect(() => () => { if (blobRef.current) URL.revokeObjectURL(blobRef.current) }, [])

  if (!src) {
    return (
      <Box className="shrink-0 rounded-lg bg-surface-100 dark:bg-surface-800 flex items-center justify-center overflow-hidden" style={{ width: size, height: size }}>
        <Map className="text-surface-400" style={{ width: size * 0.45, height: size * 0.45 }} />
      </Box>
    )
  }
  return (
    <img src={src} alt="" width={size} height={size} onError={() => setSrc('')}
      style={{ borderRadius: 8, objectFit: 'cover', width: size, height: size }} className="shrink-0" />
  )
}

function ShotThumb({ instanceId, file_name, onOpen, onDelete }: {
  instanceId: string
  file_name: string
  onOpen: () => void
  onDelete: () => void
}) {
  const [src, setSrc] = useState('')
  const [failed, setFailed] = useState(false)

  useEffect(() => {
    let cancelled = false
    invoke<string | null>('read_screenshot_base64', { instanceId, fileName: file_name })
      .then((uri) => { if (uri && !cancelled) setSrc(uri) })
      .catch(() => { if (!cancelled) setFailed(true) })
    return () => { cancelled = true }
  }, [instanceId, file_name])

  return (
    <Box
      className="group relative rounded-lg overflow-hidden bg-surface-100 dark:bg-surface-800 border border-surface-200/60 dark:border-surface-700/40 aspect-video cursor-pointer"
      onClick={onOpen}
    >
      {src ? (
        <img src={src} alt={file_name} className="w-full h-full object-cover" onError={() => setFailed(true)} />
      ) : failed ? (
        <Box className="w-full h-full flex flex-col items-center justify-center">
          <Camera className="w-4 h-4 text-surface-400" />
          <Typography variant="caption" color="text.secondary" className="text-[9px] px-1 truncate max-w-full">{file_name}</Typography>
        </Box>
      ) : (
        <Box className="w-full h-full flex items-center justify-center">
          <RefreshCw className="w-4 h-4 text-surface-400 animate-spin" />
        </Box>
      )}
      <Box className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/70 to-transparent px-1.5 py-1 opacity-0 group-hover:opacity-100 transition-opacity">
        <Typography variant="caption" className="block text-white text-[9px] truncate">{file_name}</Typography>
      </Box>
      <Box className="absolute top-1 right-1 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
        <button
          onClick={(e) => { e.stopPropagation(); onDelete() }}
          className="p-1 rounded bg-black/60 hover:bg-red-500/80 transition-colors"
          title="删除"
        >
          <Trash2 className="w-3 h-3 text-white" />
        </button>
      </Box>
    </Box>
  )
}

interface HomeInstanceData {
  mods: ModInfo[]
  rps: { file_name: string; name: string | null; enabled: boolean }[]
  sps: { file_name: string; name: string | null; enabled: boolean }[]
  schemas: { file_name: string; path: string; size_kb: number }[]
  worlds: WorldInfo[]
  shots: ScreenshotInfo[]
  mpServers: MultiplayerServer[]
  mpStatus: Record<string, ServerStatus | null>
}



const homeDataCache: Record<string, HomeInstanceData> = {}

function clearHomeCache() {
  for (const k of Object.keys(homeDataCache)) delete homeDataCache[k]
}

async function scanHomeInstanceData(id: string): Promise<HomeInstanceData> {
  const [mods, rps, sps, schemas, worlds, shots, mpServers] = await Promise.all([
    invoke<ModInfo[]>('scan_instance_mods', { instanceId: id, includeIcons: false }).catch(() => [] as ModInfo[]),
    invoke<{ file_name: string; name: string | null; enabled: boolean }[]>('scan_resource_packs', { instanceId: id }).catch(() => []),
    invoke<{ file_name: string; name: string | null; enabled: boolean }[]>('scan_shader_packs', { instanceId: id }).catch(() => []),
    invoke<{ file_name: string; path: string; size_kb: number }[]>('scan_schematics', { instanceId: id }).catch(() => []),
    invoke<WorldInfo[]>('list_instance_worlds', { instanceId: id }).catch(() => [] as WorldInfo[]),
    invoke<ScreenshotInfo[]>('list_screenshots', { instanceId: id }).catch(() => [] as ScreenshotInfo[]),
    invoke<MultiplayerServer[]>('list_multiplayer_servers', { instanceId: id }).catch(() => [] as MultiplayerServer[]),
  ])
  const mpStatus: Record<string, ServerStatus | null> = {}
  await Promise.all(mpServers.map(async (s) => {
    const addr = s.port && s.port !== 25565 ? `${s.address}:${s.port}` : s.address
    try { mpStatus[s.address] = await invoke<ServerStatus>('query_server_status', { address: addr }) } catch { mpStatus[s.address] = null }
  }))
  const data: HomeInstanceData = { mods, rps, sps, schemas, worlds, shots, mpServers, mpStatus }
  homeDataCache[id] = data
  return data
}

function applyHomeData(d: HomeInstanceData, setters: { setMods: (v: ModInfo[]) => void; setRps: (v: HomeInstanceData['rps']) => void; setSps: (v: HomeInstanceData['sps']) => void; setSchemas: (v: HomeInstanceData['schemas']) => void; setWorlds: (v: WorldInfo[]) => void; setShots: (v: ScreenshotInfo[]) => void; setMpServers: (v: MultiplayerServer[]) => void; setMpStatus: (v: Record<string, ServerStatus | null>) => void }) {
  setters.setMods(d.mods)
  setters.setRps(d.rps)
  setters.setSps(d.sps)
  setters.setSchemas(d.schemas)
  setters.setWorlds(d.worlds)
  setters.setShots(d.shots)
  setters.setMpServers(d.mpServers)
  setters.setMpStatus(d.mpStatus)
}

export function Home() {
  const navigate = useNavigate()
  const session = useAuthStore(s => s.session)
  const setSession = useAuthStore(s => s.setSession)
  const { instances, setInstances } = useInstanceStore()
  const { config } = useSettingsStore()
  const panelRef = useRef<InstancePanelHandle>(null)
  const [panelOpen, setPanelOpen] = useState(false)
  const [vsOpen, setVsOpen] = useState(false)
  const [manifest, setManifest] = useState<VersionManifest | null>(null)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [runningId, setRunningId] = useState<string | null>(null)
  const [launching, setLaunching] = useState(false)
  const [mods, setMods] = useState<ModInfo[]>([])
  const [rps, setRps] = useState<{ file_name: string; name: string | null; enabled: boolean }[]>([])
  const [sps, setSps] = useState<{ file_name: string; name: string | null; enabled: boolean }[]>([])
  const [schemas, setSchemas] = useState<{ file_name: string; path: string; size_kb: number }[]>([])
  const [worlds, setWorlds] = useState<WorldInfo[]>([])
  const [shots, setShots] = useState<ScreenshotInfo[]>([])
  const [shotsLoading, setShotsLoading] = useState(false)
  const [mpServers, setMpServers] = useState<MultiplayerServer[]>([])
  const [mpStatus, setMpStatus] = useState<Record<string, ServerStatus | null>>({})
  const [srv, setSrv] = useState<ServerStatus | null>(null)
  const [srvLoad, setSrvLoad] = useState(false)
  const [snack, setSnack] = useState({ open: false, message: '', severity: 'info' as 'success'|'error'|'info' })
  const [crashInfo, setCrashInfo] = useState<{ instance_id: string; exit_code?: number | null; reason: string; play_time_secs: number } | null>(null)

  const [refreshTick, setRefreshTick] = useState(0)

  useEffect(() => {
    invoke<VersionManifest>('fetch_versions').then(setManifest).catch(() => {})
    const reloadInstances = () => {
      invoke<Instance[]>('list_home_instances').then((list) => {
        setInstances(list)
        useInstanceStore.getState().setLoaded(true)
        invoke<LauncherConfig>('load_config').then((cfg) => {
          const last = cfg.last_selected_instance
          if (last && list.some((i) => i.id === last)) {
            setSelectedId(last)
            useInstanceStore.getState().setSelectedId(last)
          }
        }).catch(() => {})
      }).catch(() => {})
    }
    
    if (!useInstanceStore.getState().loaded) {
      reloadInstances()
    } else {
      invoke<LauncherConfig>('load_config').then((cfg) => {
        const last = cfg.last_selected_instance
        if (last && useInstanceStore.getState().instances.some((i) => i.id === last)) {
          setSelectedId(last)
          useInstanceStore.getState().setSelectedId(last)
        }
      }).catch(() => {})
    }
    let unsubStop: (() => void) | null = null
    listen<{ instance_id: string; exit_code?: number | null; reason: string; play_time_secs: number }>('game-stopped', (e) => {
      setRunningId(null)
      const info = e.payload
      if (info.reason === 'Crash' || info.reason === 'NoWindow') {
        setCrashInfo(info)
      } else {
        setSnack({ open: true, message: '游戏已退出', severity: 'info' })
      }
    }).then(f => { unsubStop = f })
    
    let unsubInstall: (() => void) | null = null
    watchResourceInstall(() => {
      clearHomeCache()
      invalidateCatalog()
      reloadInstances()
      setRefreshTick((t) => t + 1)
    }).then((f) => { unsubInstall = f }).catch(() => {})
    return () => { unsubStop?.(); unsubInstall?.() }
  }, [])

  
  useEffect(() => {
    const refreshSession = async () => {
      const s = useAuthStore.getState().session
      if (!s || s.user_type !== 'msa' || !s.refresh_token) return
      const now = Date.now()
      
      if (s.expires_at && now < s.expires_at - 10 * 60 * 1000) return
      try {
        const refreshed = await invoke<AuthSession>('microsoft_auth_refresh', { refreshToken: s.refresh_token })
        setSession(refreshed)
        useAuthStore.getState().saveAccount(refreshed)
      } catch {
        
      }
    }
    refreshSession()
  }, [])

  const sorted = [...instances].sort((a, b) => new Date(b.last_played ?? 0).getTime() - new Date(a.last_played ?? 0).getTime())
  const selected = selectedId ? instances.find(i => i.id === selectedId) : sorted[0]

  
  useEffect(() => {
    if (!selected) {
      setMods([]); setRps([]); setSps([]); setSchemas([]); setWorlds([]); setShots([]); setMpServers([]); setMpStatus({})
      return
    }
    const cached = homeDataCache[selected.id]
    if (cached) {
      applyHomeData(cached, { setMods, setRps, setSps, setSchemas, setWorlds, setShots, setMpServers, setMpStatus })
      return
    }
    let cancelled = false
    setShotsLoading(true)
    scanHomeInstanceData(selected.id).then((d) => {
      if (!cancelled) {
        applyHomeData(d, { setMods, setRps, setSps, setSchemas, setWorlds, setShots, setMpServers, setMpStatus })
        setShotsLoading(false)
      }
    })
    return () => { cancelled = true }
  }, [selected?.id, refreshTick])

  
  useEffect(() => {
    if (config.hide_mp_quick_card) { setMpServers([]); setMpStatus({}) }
  }, [config.hide_mp_quick_card])

  const loadShots = async () => {
    if (!selected) { setShots([]); return }
    setShotsLoading(true)
    invoke<ScreenshotInfo[]>('list_screenshots', { instanceId: selected.id })
      .then((list) => {
        setShots(list)
        const c = homeDataCache[selected.id]
        if (c) c.shots = list
      })
      .catch(() => setShots([]))
      .finally(() => setShotsLoading(false))
  }

  const deleteShot = async (name: string) => {
    if (!selected || !confirm(`确定删除截图「${name}」吗？`)) return
    try {
      await invoke('delete_screenshot', { instanceId: selected.id, fileName: name })
      setShots(prev => prev.filter(s => s.file_name !== name))
    } catch (e) {
      setSnack({ open: true, message: `删除失败: ${e}`, severity: 'error' })
    }
  }

  const joinServer = async (addr: string) => {
    if (!session) { navigate('/account'); return }
    if (!selected) { navigate('/download'); return }
    if (runningId) {
      try { await invoke('stop_game', { instanceId: runningId }); setRunningId(null) } catch {}
      return
    }
    setLaunching(true)
    const tid = `launch-${selected.id}-${Date.now()}`
    let unsub: (() => void) | null = null
    const launchUnsub: { f: (() => void) | null } = { f: null }
    useDownloadStore.getState().addTask({ id: tid, title: `启动 ${selected.name}`, status: 'downloading', kind: 'launch', instanceId: selected.id, stage: 'java', progress: 0, message: '准备启动...' })
    try {
      unsub = await listen<InstallProgress>('install-progress', e => useDownloadStore.getState().updateTask(tid, e.payload))
      let auth = session
      if (auth.user_type === 'msa' && auth.refresh_token) { try { auth = await invoke('microsoft_auth_refresh', { refreshToken: auth.refresh_token }); setSession(auth) } catch {} }
      await invoke<GameProcessInfo>('launch_game', { instanceId: selected.id, auth, quickWorld: null, quickServer: addr })
      setRunningId(selected.id)
      const windowAppeared = await new Promise<boolean>((resolve) => {
        let settled = false
        const finish = (ok: boolean) => { if (!settled) { settled = true; resolve(ok) } }
        listen<LaunchProgressEvent>('launch-progress', (e) => {
          if (e.payload.instance_id !== selected.id) return
          if (e.payload.stage === 'running') finish(true)
        }).then((f) => { launchUnsub.f = f })
        setTimeout(() => finish(true), 90000)
      })
      if (windowAppeared) {
        useDownloadStore.getState().markDone(tid)
        setSnack({ open: true, message: `${selected.name} 启动成功！`, severity: 'success' })
      }
    } catch (e) {
      const msg = String(e); useDownloadStore.getState().markError(tid, msg)
      const lines = msg.split('\n').filter(l => l.trim())
      const key = lines.find(l => /ERROR|Exception|Error|Incompatible|Could not|找不到|无法|失败|退出/.test(l)) ?? lines[0]
      setSnack({ open: true, message: `启动失败: ${key ?? msg}`, severity: 'error' })
      if (msg.includes('[launch-crash]')) {
        setSnack({ open: true, message: '游戏启动时崩溃，正在进入 AI 错误分析...', severity: 'info' })
        navigate(`/ai?instance=${selected.id}&auto_analyze=1`)
      }
    }
    finally { if (unsub) unsub(); if (launchUnsub.f) launchUnsub.f(); setLaunching(false); setTimeout(() => useDownloadStore.getState().removeTask(tid), 3000) }
  }

  const querySrv = async () => {
    if (!config.server_address) { setSrv(null); return }
    setSrvLoad(true)
    try { setSrv(await invoke<ServerStatus>('query_server_status', { address: config.server_address })) }
    catch {
      setSrv((prev) => ({
        online: false,
        host: config.server_address,
        port: 25565,
        description: '',
        version_name: '',
        version_protocol: 0,
        players_online: 0,
        players_max: 0,
        player_names: [],
        favicon: prev?.favicon ?? null,
        favicon_path: prev?.favicon_path ?? null,
        latency_ms: 0,
        mod_info: null,
        error: '查询失败',
      }))
    }
    setSrvLoad(false)
  }

  useEffect(() => { querySrv(); if (!config.server_address) return; const t = setInterval(querySrv, 15000); return () => clearInterval(t) }, [config.server_address])

  const handleLaunch = async (target?: string) => {
    if (!session) { navigate('/account'); return }
    if (!selected) { navigate('/download'); return }
    if (runningId) { try { await invoke('stop_game', { instanceId: runningId }); setRunningId(null) } catch {} return }
    setLaunching(true)
    const tid = `launch-${selected.id}-${Date.now()}`
    let unsub: (() => void) | null = null
    const launchUnsub: { f: (() => void) | null } = { f: null }
    useDownloadStore.getState().addTask({ id: tid, title: `启动 ${selected.name}`, status: 'downloading', kind: 'launch', instanceId: selected.id, stage: 'java', progress: 0, message: '准备启动...' })
    try {
      unsub = await listen<InstallProgress>('install-progress', e => useDownloadStore.getState().updateTask(tid, e.payload))
      let auth = session
      if (auth.user_type === 'msa' && auth.refresh_token) { try { auth = await invoke('microsoft_auth_refresh', { refreshToken: auth.refresh_token }); setSession(auth) } catch {} }
      const t = target ?? ''
      const quickWorld = t.startsWith('world:') ? t.slice(6) : null
      const quickServer = t.startsWith('server:') ? t.slice(7) : null
      
      triggerBestOptimize()
      await invoke<GameProcessInfo>('launch_game', { instanceId: selected.id, auth, quickWorld, quickServer })
      setRunningId(selected.id)
      
      triggerSilentOptimize()
      const windowAppeared = await new Promise<boolean>((resolve) => {
        let settled = false
        const finish = (ok: boolean) => { if (!settled) { settled = true; resolve(ok) } }
        listen<LaunchProgressEvent>('launch-progress', (e) => {
          if (e.payload.instance_id !== selected.id) return
          if (e.payload.stage === 'running') finish(true)
        }).then((f) => { launchUnsub.f = f })
        setTimeout(() => finish(true), 90000)
      })
      if (windowAppeared) {
        useDownloadStore.getState().markDone(tid)
        setSnack({ open: true, message: `${selected.name} 启动成功！`, severity: 'success' })
      }
    } catch (e) {
      const msg = String(e); useDownloadStore.getState().markError(tid, msg)
      const lines = msg.split('\n').filter(l => l.trim())
      const key = lines.find(l => /ERROR|Exception|Error|Incompatible|Could not|找不到|无法|失败|退出/.test(l)) ?? lines[0]
      setSnack({ open: true, message: `启动失败: ${key ?? msg}`, severity: 'error' })
      if (msg.includes('[launch-crash]')) {
        setSnack({ open: true, message: '游戏启动时崩溃，正在进入 AI 错误分析...', severity: 'info' })
        navigate(`/ai?instance=${selected.id}&auto_analyze=1`)
      }
    }
    finally { if (unsub) unsub(); if (launchUnsub.f) launchUnsub.f(); setLaunching(false); setTimeout(() => useDownloadStore.getState().removeTask(tid), 3000) }
  }

  const em = mods.filter(m => m.enabled), er = rps.filter(p => p.enabled), es = sps.filter(p => p.enabled)

  return (
    <Box className="h-full flex flex-col relative overflow-hidden">
      <Box
        className="flex-1 flex overflow-hidden transition-[filter] duration-200"
        style={panelOpen ? { filter: 'blur(2px)' } : undefined}
      >
        <aside className="w-60 shrink-0 flex flex-col gap-2.5 px-3.5 py-4 border-r border-surface-200/60 dark:border-surface-700/30 overflow-y-auto">
          <Card onClick={() => navigate('/account')} hoverable className="cursor-pointer shrink-0">
            <Box className="flex items-center gap-2">
              <SkinAvatar size={36} />
              <Box className="flex-1 min-w-0">
                <Typography variant="subtitle2" className="font-semibold truncate text-sm">{session?.username ?? '未登录'}</Typography>
                <Typography variant="caption" color="text.secondary" className="text-[10px]">{session ? '管理账户' : '点击登录'}</Typography>
              </Box>
            </Box>
          </Card>

          <Box className="shrink-0">
            <LaunchButton onClick={() => handleLaunch()} isRunning={!!runningId} isLoading={launching} className="w-full h-12 text-sm" />
          </Box>

          <Box className="shrink-0">
            <Button size="medium" variant="outlined" startIcon={<FolderOpen className="w-4 h-4" />} onClick={() => setPanelOpen(true)} fullWidth className="h-11 text-sm font-medium">实例列表</Button>
          </Box>

          {selected && (
            <Box className="shrink-0 space-y-0.5">
              <button onClick={() => navigate(`/instances/${selected.id}/manage?type=mods`)} className="w-full flex items-center gap-1.5 px-2 py-1.5 rounded-lg bg-surface-50 dark:bg-surface-800 hover:bg-surface-100 dark:hover:bg-surface-700 transition-colors text-left">
                <Puzzle className="w-3 h-3 text-[var(--accent-color)]" /><Typography variant="caption" className="font-medium flex-1 text-[11px]">模组管理</Typography><Typography variant="caption" color="text.secondary" className="text-[10px]">{em.length}</Typography>
              </button>
              <button onClick={() => navigate(`/instances/${selected.id}/manage?type=resourcepacks`)} className="w-full flex items-center gap-1.5 px-2 py-1.5 rounded-lg bg-surface-50 dark:bg-surface-800 hover:bg-surface-100 dark:hover:bg-surface-700 transition-colors text-left">
                <Image className="w-3 h-3 text-emerald-500" /><Typography variant="caption" className="font-medium flex-1 text-[11px]">资源包管理</Typography><Typography variant="caption" color="text.secondary" className="text-[10px]">{er.length}</Typography>
              </button>
            </Box>
          )}

          {selected && !config.hide_mp_quick_card && (
            <Box className="shrink-0 space-y-1">
              <Box className="flex flex-col gap-0.5 px-1">
                <Box className="flex items-center gap-1.5">
                  <Server className="w-3 h-3 text-sky-500" /><Typography variant="caption" className="font-medium text-[11px]">多人游戏</Typography><Chip label={`${mpServers.length}`} size="small" variant="outlined" />
                </Box>
                <Typography variant="caption" color="text.secondary" className="text-[9px]">(低版本可能无法使用快捷加入)</Typography>
              </Box>
              {mpServers.length === 0 ? (
                <Typography variant="caption" color="text.secondary" className="block px-2 text-[10px]">暂无服务器</Typography>
              ) : (
                <Box className="space-y-1 max-h-48 overflow-y-auto pr-0.5">
                {mpServers.map((s) => {
                  const st = mpStatus[s.address]
                  const online = st?.online
                  return (
                    <Box key={s.address} className="flex items-center gap-1.5 px-1.5 py-1 rounded-lg bg-surface-50 dark:bg-surface-800/60">
                      <ServerIcon favicon={st?.favicon} faviconPath={st?.favicon_path} size={22} />
                      <Box className="min-w-0 flex-1">
                        <Typography variant="caption" className="block text-[11px] font-medium leading-tight whitespace-normal break-words">{s.name}</Typography>
                        <Box className="flex items-center gap-1 mt-0.5">
                          {st ? (
                            online ? (
                              <Wifi className="w-2.5 h-2.5 text-green-500 shrink-0" />
                            ) : (
                              <WifiOff className="w-2.5 h-2.5 text-red-400 shrink-0" />
                            )
                          ) : (
                            <RefreshCw className="w-2.5 h-2.5 text-surface-400 animate-spin shrink-0" />
                          )}
                          <Typography variant="caption" color="text.secondary" className="block text-[9px] leading-tight truncate">
                            {st ? (online ? `${st.players_online}/${st.players_max} · ${st.latency_ms}ms` : '离线') : '查询中...'}
                          </Typography>
                        </Box>
                      </Box>
                      <Button
                        size="small"
                        variant="text"
                        className="!min-w-0 !px-1.5 !text-[10px] shrink-0"
                        onClick={() => joinServer(s.address)}
                      >
                        加入
                      </Button>
                    </Box>
                  )
                })}
                </Box>
              )}
            </Box>
          )}

          <Box className="flex-1" />

          <Card className="shrink-0 !py-1.5 !px-2.5">
            <Typography variant="caption" color="text.secondary" className="text-center block text-[10px]">
              {manifest ? `${manifest.latest.release} / ${manifest.latest.snapshot}` : '...'}
            </Typography>
          </Card>
        </aside>

        <main className="flex-1 overflow-y-auto p-5">
          {!selected ? (
            <Box className="h-full flex flex-col items-center justify-center">
              <Gamepad2 className="w-20 h-20 mb-3 text-surface-300 dark:text-surface-600" />
              <Typography variant="h6" className="mb-1">欢迎使用 SkyLine Launcher</Typography>
              <Typography variant="body2" color="text.secondary" className="mb-5">还没有实例，去「资源」页下载一个游戏吧</Typography>
              <Button variant="contained" startIcon={<Download className="w-4 h-4" />} onClick={() => navigate('/download')}>下载游戏</Button>
            </Box>
          ) : (
            <Box className="h-full flex flex-col relative">
              <Box className="flex justify-center mb-4">
                <Card className="w-full max-w-2xl">
                  <Box className="flex items-center gap-3">
                    <Box className="w-11 h-11 rounded-xl bg-accent-50 dark:bg-accent-500/10 flex items-center justify-center shrink-0">
                      <LoaderLogo loader={selected.modloader} versionId={selected.version_id} className="w-7 h-7" />
                    </Box>
                    <Box className="flex-1 min-w-0">
                      <Typography variant="subtitle1" className="font-bold truncate">{selected.name}</Typography>
                      <Box className="flex items-center gap-2 mt-0.5">
                        <Chip label={selected.version_id} size="small" variant="outlined" />
                        <span className={`text-xs font-medium ${getLoaderColor(selected.modloader)}`}>{getLoaderName(selected.modloader)}</span>
                        {selected.external && <Chip label="外部" size="small" color="warning" variant="outlined" />}
                      </Box>
                    </Box>
                    <Box className="flex items-center gap-3 text-surface-400 shrink-0 text-xs">
                      {selected.play_time > 0 && <span className="flex items-center gap-1"><Clock className="w-3 h-3" />{formatPlayTime(selected.play_time)}</span>}
                      {selected.last_played && <span>{new Date(selected.last_played).toLocaleDateString()}</span>}
                    </Box>
                  </Box>
                </Card>
              </Box>

              <Box className="flex-1 overflow-y-auto space-y-3 pb-4">
                <Card>
                  <Box className="flex items-center justify-between mb-2">
                    <Box className="flex items-center gap-1.5"><Puzzle className="w-3.5 h-3.5 text-[var(--accent-color)]" /><Typography variant="subtitle2" className="font-semibold">模组</Typography><Chip label={`${em.length}`} size="small" variant="outlined" /></Box>
                    <Button size="small" variant="text" endIcon={<ChevronRight className="w-3 h-3" />} onClick={() => navigate(`/instances/${selected.id}/manage?type=mods`)}>管理</Button>
                  </Box>
                  {mods.length === 0 ? <Typography variant="caption" color="text.secondary" className="py-3 text-center block">暂无模组</Typography> : (
                    <Box className="grid grid-cols-4 gap-1.5 max-h-40 overflow-y-auto pr-1">
                      {mods.map(m => (
                        <Box
                          key={m.path}
                          onClick={() => navigate(`/instances/${selected.id}/manage?type=mods`)}
                          className="flex items-center gap-1.5 px-1.5 py-1 rounded-md bg-surface-50 dark:bg-surface-800/60 hover:bg-surface-100 dark:hover:bg-surface-700 cursor-pointer transition-colors"
                        >
                          <Box className={`w-1.5 h-1.5 rounded-full shrink-0 ${m.enabled ? 'bg-green-500' : 'bg-surface-300'}`} />
                          <Box className="min-w-0 flex-1">
                            <Typography variant="caption" className={`block truncate text-[10px] leading-tight ${!m.enabled ? 'opacity-50' : ''}`}>{m.name || m.file_name}</Typography>
                            {m.version && <Typography variant="caption" color="text.secondary" className="block truncate text-[9px] leading-tight">v{m.version}</Typography>}
                          </Box>
                        </Box>
                      ))}
                    </Box>
                  )}
                </Card>

                <Card>
                  <Box className="flex items-center justify-between mb-2">
                    <Box className="flex items-center gap-1.5"><Image className="w-3.5 h-3.5 text-emerald-500" /><Typography variant="subtitle2" className="font-semibold">资源包</Typography><Chip label={`${er.length}`} size="small" variant="outlined" /></Box>
                    <Button size="small" variant="text" endIcon={<ChevronRight className="w-3 h-3" />} onClick={() => navigate(`/instances/${selected.id}/manage?type=resourcepacks`)}>管理</Button>
                  </Box>
                  {rps.length === 0 ? <Typography variant="caption" color="text.secondary" className="py-3 text-center block">暂无资源包</Typography> : (
                    <Box className="space-y-0.5 max-h-32 overflow-y-auto">
                      {rps.slice(0, 15).map(p => (
                        <Box key={p.file_name} className="flex items-center gap-1.5 px-2 py-1 rounded-md hover:bg-surface-50 dark:hover:bg-surface-800">
                          <Box className={`w-1.5 h-1.5 rounded-full shrink-0 ${p.enabled ? 'bg-green-500' : 'bg-surface-300'}`} />
                          <Typography variant="caption" className={`flex-1 truncate text-[11px] ${!p.enabled ? 'opacity-50' : ''}`}>{p.name || p.file_name}</Typography>
                        </Box>
                      ))}
                    </Box>
                  )}
                </Card>

                <Card>
                  <Box className="flex items-center justify-between mb-2">
                    <Box className="flex items-center gap-1.5"><Map className="w-3.5 h-3.5 text-cyan-500" /><Typography variant="subtitle2" className="font-semibold">世界</Typography><Chip label={`${worlds.length}`} size="small" variant="outlined" /></Box>
                    <Button size="small" variant="text" endIcon={<ChevronRight className="w-3 h-3" />} onClick={() => invoke('open_instance_folder', { instanceId: selected.id, subdir: 'saves' })}>打开文件夹</Button>
                  </Box>
                  {worlds.length === 0 ? <Typography variant="caption" color="text.secondary" className="py-3 text-center block">暂无世界</Typography> : (
                    <Box className="space-y-1 max-h-56 overflow-y-auto pr-1">
                      {worlds.map(w => (
                        <Box
                          key={w.path}
                          className="flex items-center gap-2.5 px-2 py-1.5 rounded-lg bg-surface-50 dark:bg-surface-800/60 hover:bg-surface-100 dark:hover:bg-surface-700 transition-colors"
                        >
                          <WorldIcon icon={w.icon} />
                          <Box className="min-w-0 flex-1">
                            <Typography variant="body2" className="font-medium truncate text-[13px]">{w.name}</Typography>
                            <Typography variant="caption" color="text.secondary" className="block text-[10px]">
                              {w.play_time > 0 ? `${formatPlayTime(w.play_time)} · ` : ''}{w.is_hardcore ? '极限 ' : ''}{gameModeLabel(w.game_mode) ? `${gameModeLabel(w.game_mode)} · ` : ''}{formatSize(w.size_kb)}
                            </Typography>
                          </Box>
                          <Button
                            size="small"
                            variant="text"
                            className="!min-w-0 !px-2"
                            onClick={() => handleLaunch(`world:${w.path}`)}
                          >
                            进入
                          </Button>
                           <Button
                             size="small"
                             variant="text"
                             className="!min-w-0 !px-2 text-surface-400 hover:text-blue-400"
                              onClick={() => selected && navigate(`/worlds/${selected.id}`)}
                            >
                             预览
                          </Button>
                        </Box>
                      ))}
                    </Box>
                  )}
                </Card>

                <Box className="grid grid-cols-2 gap-3">
                  <Card>
                    <Box className="flex items-center justify-between gap-1 mb-1">
                      <Box className="flex items-center gap-1.5"><Layers className="w-3.5 h-3.5 text-violet-500" /><Typography variant="subtitle2" className="font-semibold">光影包</Typography><Chip label={`${es.length}`} size="small" variant="outlined" /></Box>
                      <Button size="small" variant="text" endIcon={<ChevronRight className="w-3 h-3" />} onClick={() => navigate(`/instances/${selected.id}/manage?type=shaderpacks`)}>管理</Button>
                    </Box>
                    {sps.length === 0 ? <Typography variant="caption" color="text.secondary" className="py-2 text-center block">暂无</Typography> : (
                      <Box className="space-y-0.5 max-h-28 overflow-y-auto">
                        {sps.slice(0, 8).map(p => (
                          <Box key={p.file_name} className="flex items-center gap-1.5 px-1.5 py-0.5 rounded hover:bg-surface-50 dark:hover:bg-surface-800">
                            <Box className={`w-1 h-1 rounded-full shrink-0 ${p.enabled ? 'bg-green-500' : 'bg-surface-300'}`} />
                            <Typography variant="caption" className={`truncate text-[10px] ${!p.enabled ? 'opacity-50' : ''}`}>{p.name || p.file_name}</Typography>
                          </Box>
                        ))}
                      </Box>
                    )}
                  </Card>
                  <Card>
                    <Box className="flex items-center justify-between gap-1 mb-1">
                    <Box className="flex items-center gap-1.5"><Map className="w-3.5 h-3.5 text-cyan-500" /><Typography variant="subtitle2" className="font-semibold">原理图</Typography><Chip label={`${schemas.length}`} size="small" variant="outlined" /></Box>
                      <Button size="small" variant="text" endIcon={<ChevronRight className="w-3 h-3" />} onClick={() => navigate(`/instances/${selected.id}/manage?type=schematics`)}>管理</Button>
                    </Box>
                    {schemas.length === 0 ? (
                      <Typography variant="caption" color="text.secondary" className="py-2 text-center block">暂无</Typography>
                    ) : (
                      <Box className="space-y-0.5 max-h-28 overflow-y-auto">
                        {schemas.slice(0, 8).map(p => (
                    <Box key={p.path} className="flex items-center gap-1.5 px-1.5 py-0.5 rounded hover:bg-surface-50 dark:hover:bg-surface-800">
                          <Box className="w-1 h-1 rounded-full shrink-0 bg-cyan-400" />
                          <Typography variant="caption" className="truncate text-[10px]">{p.file_name}</Typography>
                          <Typography variant="caption" color="text.secondary" className="text-[9px] ml-auto">
                            {p.size_kb < 1024 ? `${p.size_kb} KB` : `${(p.size_kb / 1024).toFixed(1)} MB`}
                          </Typography>
                        </Box>
                        ))}
                      </Box>
                    )}
                  </Card>
                </Box>

                <Card>
                  <Box className="flex items-center justify-between mb-2">
                    <Box className="flex items-center gap-1.5"><Camera className="w-3.5 h-3.5 text-sky-500" /><Typography variant="subtitle2" className="font-semibold">截图</Typography><Chip label={`${shots.length}`} size="small" variant="outlined" /></Box>
                    <Box className="flex items-center gap-1">
                      <Button size="small" variant="text" onClick={() => loadShots()}>
                        <RefreshCw className={`w-3.5 h-3.5 ${shotsLoading ? 'animate-spin' : ''}`} />
                      </Button>
                      <Button size="small" variant="text" endIcon={<FolderOpen className="w-3.5 h-3.5" />} onClick={() => invoke('open_instance_folder', { instanceId: selected.id, subdir: 'screenshots' })}>打开文件夹</Button>
                    </Box>
                  </Box>
                  {shots.length === 0 ? (
                    <Typography variant="caption" color="text.secondary" className="py-3 text-center block">暂无截图，游戏中按 F2 拍摄</Typography>
                  ) : (
                    <Box className="grid grid-cols-3 gap-2 max-h-60 overflow-y-auto pr-1">
                      {shots.map(s => (
                        <ShotThumb
                          key={s.path}
                          instanceId={selected.id}
                          file_name={s.file_name}
                          onOpen={() => invoke('open_screenshot', { instanceId: selected.id, fileName: s.file_name }).catch((e) => setSnack({ open: true, message: `打开失败: ${e}`, severity: 'error' }))}
                          onDelete={() => deleteShot(s.file_name)}
                        />
                      ))}
                    </Box>
                  )}
                </Card>
              </Box>

              {config.server_address && !config.hide_server_card && (
                <Box className="fixed bottom-1 right-1 z-50">
                  <Card className="!rounded-lg" style={{ padding: `${Math.max(4, (config.server_card_size ?? 80) / 20)}px ${Math.max(6, (config.server_card_size ?? 80) / 10)}px` }}>
                    <Box className="flex items-center gap-1.5">
                      <ServerIcon favicon={srv?.favicon} faviconPath={srv?.favicon_path} size={Math.max(12, (config.server_card_size ?? 80) / 3)} />
                      <Box className="min-w-0">
                        <Typography variant="caption" className="font-semibold truncate block leading-tight" style={{ fontSize: Math.max(7, (config.server_card_size ?? 80) / 10), maxWidth: (config.server_card_size ?? 80) / 2 }}>{config.server_name || ''}</Typography>
                        <Box className="flex items-center gap-0.5 mt-px">
                          {srv?.online ? (
                            <>
                              <Users style={{ width: Math.max(6, (config.server_card_size ?? 80) / 14), height: Math.max(6, (config.server_card_size ?? 80) / 14) }} className="text-surface-400" />
                              <span style={{ fontSize: Math.max(6, (config.server_card_size ?? 80) / 14) }} className="text-[var(--accent-color)] font-medium">{srv.players_online}</span>
                              <span style={{ fontSize: Math.max(6, (config.server_card_size ?? 80) / 14) }} className="text-surface-400">/{srv.players_max}</span>
                              <span className="text-surface-300 dark:text-surface-600" style={{ fontSize: Math.max(5, (config.server_card_size ?? 80) / 16) }}>·</span>
                              <LatencyBars ms={srv.latency_ms} />
                              <span className={`font-mono ${srv.latency_ms < 50 ? 'text-green-500' : srv.latency_ms < 150 ? 'text-yellow-500' : 'text-red-500'}`} style={{ fontSize: Math.max(6, (config.server_card_size ?? 80) / 14) }}>{srv.latency_ms}ms</span>
                            </>
                          ) : srv ? (
                            <span className="text-red-400" style={{ fontSize: Math.max(6, (config.server_card_size ?? 80) / 14) }} title={srv.error || '无法连接'}>离线</span>
                          ) : (
                            <RefreshCw className="animate-spin text-surface-400" style={{ width: Math.max(6, (config.server_card_size ?? 80) / 14), height: Math.max(6, (config.server_card_size ?? 80) / 14) }} />
                          )}
                        </Box>
                      </Box>
                      <button onClick={querySrv} disabled={srvLoad} className="p-0.5 rounded hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors shrink-0">
                        <RefreshCw className={`text-surface-400 ${srvLoad ? 'animate-spin' : ''}`} style={{ width: Math.max(6, (config.server_card_size ?? 80) / 14), height: Math.max(6, (config.server_card_size ?? 80) / 14) }} />
                      </button>
                    </Box>
                  </Card>
                </Box>
              )}
            </Box>
          )}
        </main>
      </Box>

      <Box
        className={`absolute inset-0 z-[9] transition-opacity duration-300 ease-in-out ${panelOpen ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none'}`}
        style={{ background: 'rgba(0,0,0,0.25)', backdropFilter: 'blur(2px)' }}
        onClick={() => { setPanelOpen(false); invoke<Instance[]>('list_home_instances').then(setInstances).catch(console.error) }}
      />
      <Box
        className={`absolute inset-y-0 right-0 w-[380px] max-w-[72%] z-10 transition-transform duration-300 ease-in-out ${panelOpen ? 'translate-x-0' : 'translate-x-full'}`}
        onClick={(e) => e.stopPropagation()}
      >
              <InstancePanel ref={panelRef} selectedId={selected?.id} onSelect={(id) => { setSelectedId(id); useInstanceStore.getState().setSelectedId(id); invoke('set_last_selected_instance', { instanceId: id }).catch(() => {}) }} onCollapse={() => { setPanelOpen(false); invoke<Instance[]>('list_home_instances').then(setInstances).catch(console.error) }} />
      </Box>

      <DownloadCenter serverCard={!!(config.server_address && !config.hide_server_card)} />
      <VersionSettingsPanel open={vsOpen} onClose={() => setVsOpen(false)} />
      <CrashDialog
        open={!!crashInfo}
        exitInfo={crashInfo}
        instanceName={selected?.name ?? selected?.id}
        onClose={() => setCrashInfo(null)}
        onAnalyze={(instanceId) => { setCrashInfo(null); navigate(`/ai?instance=${instanceId}&auto_analyze=1`) }}
      />
      <SnackbarAlert open={snack.open} onClose={() => setSnack({ ...snack, open: false })} message={snack.message} severity={snack.severity} />

      <Box className="fixed bottom-4 right-4 z-50" title={config.home_style === 'minimal' ? '切换到完整模式' : '切换到简洁模式'}>
        <Button
          variant="outlined"
          size="small"
          className="!rounded-full !min-w-0 !p-2 !bg-white/80 dark:!bg-surface-800/80 backdrop-blur-sm !border-surface-200/60 dark:!border-surface-700/40"
          onClick={() => {
            const next = config.home_style === 'minimal' ? 'full' : 'minimal'
            const nextConfig = { ...config, home_style: next }
            useSettingsStore.getState().setConfig(nextConfig)
            invoke('save_config', { config: nextConfig }).catch(() => {})
          }}
        >
          <LayoutDashboard className="w-4 h-4" />
        </Button>
      </Box>
    </Box>
  )
}