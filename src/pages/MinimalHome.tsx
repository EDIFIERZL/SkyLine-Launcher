import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useAuthStore } from '../stores/authStore'
import { useInstanceStore } from '../stores/instanceStore'
import { useSettingsStore } from '../stores/settingsStore'
import { LaunchButton } from '../components/LaunchButton'
import { SkinAvatar } from '../components/SkinAvatar'
import { LoaderLogo } from '../components/LoaderLogo'
import { sortInstances } from '../lib/instanceSort'
import { Box, Typography, Button } from '@/components/material'
import { Download, Gamepad2, LayoutDashboard, FolderOpen, X, Settings } from 'lucide-react'
import { NewsCarousel } from '../components/NewsCarousel'
import type { Instance } from '../types'

function getGreeting(): string {
  const h = new Date().getHours()
  if (h < 6) return '夜深了'
  if (h < 12) return '早上好'
  if (h < 14) return '中午好'
  if (h < 18) return '下午好'
  return '晚上好'
}

function getLoaderName(loader: Instance['modloader']): string {
  if (!loader) return 'Vanilla'
  const key = typeof loader === 'string' ? loader : Object.keys(loader)[0]
  if (key === 'Vanilla') return 'Vanilla'
  const ver = typeof loader === 'string' ? '' : Object.values(loader)[0] as string
  return ver ? `${key} ${ver}` : key
}

export function MinimalHome() {
  const navigate = useNavigate()
  const session = useAuthStore(s => s.session)
  const { instances } = useInstanceStore()
  const { config } = useSettingsStore()
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [runningId, setRunningId] = useState<string | null>(null)
  const [launching, setLaunching] = useState(false)
  const [showInstanceList, setShowInstanceList] = useState(false)

  const sorted = [...instances].sort((a, b) => new Date(b.last_played ?? 0).getTime() - new Date(a.last_played ?? 0).getTime())
  const selected = selectedId ? instances.find(i => i.id === selectedId) : sorted[0]

  useEffect(() => {
    if (config.last_selected_instance && instances.some(i => i.id === config.last_selected_instance)) {
      setSelectedId(config.last_selected_instance)
    }
  }, [config.last_selected_instance, instances])

  useEffect(() => {
    const unsub = listen<{ instance_id: string }>('game-stopped', () => setRunningId(null))
    return () => { unsub.then(f => f()) }
  }, [])

  const handleLaunch = async () => {
    if (!session) { navigate('/account'); return }
    if (!selected) { navigate('/download'); return }
    if (runningId) { try { await invoke('stop_game', { instanceId: runningId }); setRunningId(null) } catch {} return }
    setLaunching(true)
    try {
      let auth = session
      if (auth.user_type === 'msa' && auth.refresh_token) {
        try { auth = await invoke('microsoft_auth_refresh', { refreshToken: auth.refresh_token }) } catch {}
      }
      await invoke('launch_game', { instanceId: selected.id, auth, quickWorld: null, quickServer: null })
      setRunningId(selected.id)
    } catch (e) {
      console.error('Launch failed:', e)
    } finally {
      setLaunching(false)
    }
  }

  const toggleMode = () => {
    const next = config.home_style === 'minimal' ? 'full' : 'minimal'
    const nextConfig = { ...config, home_style: next }
    useSettingsStore.getState().setConfig(nextConfig)
    invoke('save_config', { config: nextConfig }).catch(() => {})
  }

  const bgStyle = config.background_type === 'image' && config.background_value
    ? { backgroundImage: `url(${config.background_value})`, backgroundSize: 'cover', backgroundPosition: 'center' }
    : {}

  const showNews = config.show_home_news !== false

  return (
    <Box className="h-full flex flex-col relative" style={bgStyle}>
      {config.background_type === 'video' && config.background_value && (
        <video autoPlay muted loop playsInline className="absolute inset-0 w-full h-full object-cover" src={config.background_value} />
      )}
      {config.background_type !== 'none' && config.background_value && !config.liquid_glass && (
        <Box className="absolute inset-0 bg-black/40 backdrop-blur-sm" />
      )}

      <Box className="relative z-10 flex-1 flex px-12 py-10 gap-6">
        <Box className="flex-1 flex flex-col min-w-0">
          <Box className="flex items-start justify-between mb-auto">
          <Box className="flex items-center gap-3">
            <button
              onClick={() => navigate('/account')}
              className="flex items-center gap-3 rounded-xl px-2 py-1 -ml-2 hover:bg-white/50 dark:hover:bg-surface-800/50 transition-colors cursor-pointer"
              title="账户管理"
            >
              {session && (
                <SkinAvatar size={36} uuid={session.uuid} username={session.username} userType={session.user_type} />
              )}
              <Box>
                <Typography variant="body2" className="text-surface-400 dark:text-surface-500 text-sm tracking-wide">{getGreeting()}</Typography>
                <Typography variant="h4" className="font-bold mt-1">{session?.username ?? '旅行者'}</Typography>
              </Box>
            </button>
          </Box>
          <Box className="flex flex-col items-end gap-3">
            <button
              onClick={toggleMode}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-white/80 dark:bg-surface-800/80 backdrop-blur-sm border border-surface-200/60 dark:border-surface-700/40 text-xs text-surface-600 dark:text-surface-300 hover:bg-white dark:hover:bg-surface-700 transition-colors"
              title={config.home_style === 'minimal' ? '切换到完整模式' : '切换到简洁模式'}
            >
              <LayoutDashboard className="w-3.5 h-3.5" />
              {config.home_style === 'minimal' ? '完整模式' : '简洁模式'}
            </button>
            {showNews && <div className="w-[min(42vw,420px)] h-44 sm:h-52"><NewsCarousel /></div>}
          </Box>
        </Box>

        <Box className="flex-1 flex flex-col items-center justify-center -mt-16">
          {!selected ? (
            <Box className="text-center">
              <Gamepad2 className="w-16 h-16 mb-4 text-surface-300 dark:text-surface-600 mx-auto" />
              <Typography variant="h6" className="mb-2">还没有游戏实例</Typography>
              <Typography variant="body2" color="text.secondary" className="mb-4">去「资源」页下载一个游戏吧</Typography>
              <Button variant="contained" startIcon={<Download className="w-4 h-4" />} onClick={() => navigate('/download')}>
                下载游戏
              </Button>
            </Box>
          ) : null}
        </Box>

        <Box className="mt-auto pt-6 w-full flex flex-col items-start">
          <Box className="flex items-center gap-3">
            <LaunchButton
              onClick={handleLaunch}
              isRunning={!!runningId}
              isLoading={launching}
              className="h-12 px-8 text-sm shrink-0"
            />
            <button
              onClick={() => setShowInstanceList(true)}
              className="h-12 w-12 rounded-xl bg-white/80 dark:bg-surface-800/60 border border-surface-200/60 dark:border-surface-700/40 text-surface-600 dark:text-surface-300 hover:bg-white dark:hover:bg-surface-700 transition-colors shrink-0 flex items-center justify-center"
              title="实例列表"
            >
              <FolderOpen className="w-5 h-5" />
            </button>
          </Box>
          {selected && (
            <Box className="mt-3">
              <Typography variant="body1" className="font-semibold text-base">{selected.name}</Typography>
            </Box>
          )}
        </Box>

        {showInstanceList && (
          <Box className="absolute inset-0 z-20 flex items-center justify-center bg-black/40 backdrop-blur-sm" onClick={() => setShowInstanceList(false)}>
            <Box className="w-80 max-h-[70vh] bg-white dark:bg-surface-900 rounded-2xl shadow-2xl border border-surface-200 dark:border-surface-700 overflow-hidden" onClick={e => e.stopPropagation()}>
              <Box className="flex items-center justify-between px-4 py-3 border-b border-surface-200 dark:border-surface-700">
                <Typography variant="subtitle1" className="font-semibold">选择实例</Typography>
                <button onClick={() => setShowInstanceList(false)} className="p-1 rounded hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors">
                  <X className="w-4 h-4 text-surface-400" />
                </button>
              </Box>
              <Box className="overflow-y-auto max-h-[60vh] p-2 space-y-1">
                {sortInstances(instances).map(inst => (
                  <div
                    key={inst.id}
                    className={`w-full px-3 py-2 rounded-xl transition-colors flex items-center gap-3 ${
                      selected?.id === inst.id
                        ? 'bg-[var(--accent-color)]/10 border border-[var(--accent-color)]/30'
                        : 'hover:bg-surface-100 dark:hover:bg-surface-800'
                    }`}
                  >
                    <button
                      onClick={() => { setSelectedId(inst.id); setShowInstanceList(false); invoke('set_last_selected_instance', { instanceId: inst.id }).catch(() => {}) }}
                      className="flex-1 flex items-center gap-3 text-left min-w-0"
                    >
                      <div className="w-8 h-8 rounded-lg bg-[var(--accent-color)]/10 flex items-center justify-center shrink-0 p-1.5">
                        <LoaderLogo loader={inst.modloader} versionId={inst.version_id} className="w-full h-full" />
                      </div>
                      <div className="min-w-0">
                        <div className="text-sm font-medium truncate">{inst.name}</div>
                        <div className="text-[10px] text-surface-400 mt-0.5">{inst.version_id} · {getLoaderName(inst.modloader)}</div>
                      </div>
                    </button>
                    <button
                      onClick={() => { setShowInstanceList(false); navigate(`/instances/${inst.id}/manage`) }}
                      className="p-1.5 rounded-lg hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors shrink-0"
                      title="管理实例"
                    >
                      <Settings className="w-3.5 h-3.5 text-surface-400" />
                    </button>
                  </div>
                ))}
              </Box>
            </Box>
          </Box>
        )}
        </Box>

      </Box>
    </Box>
  )
}
