import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useAuthStore } from '../stores/authStore'
import { useInstanceStore } from '../stores/instanceStore'
import { useSettingsStore } from '../stores/settingsStore'
import { LaunchButton } from '../components/LaunchButton'
import { Box, Typography, Button } from '@/components/material'
import { Download, Gamepad2, Settings, User, LayoutGrid, Map, Puzzle } from 'lucide-react'
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

  const bgStyle = config.background_type === 'image' && config.background_value
    ? { backgroundImage: `url(${config.background_value})`, backgroundSize: 'cover', backgroundPosition: 'center' }
    : {}

  return (
    <Box className="h-full flex flex-col relative" style={bgStyle}>
      {config.background_type === 'video' && config.background_value && (
        <video
          autoPlay
          muted
          loop
          playsInline
          className="absolute inset-0 w-full h-full object-cover"
          src={config.background_value}
        />
      )}
      {config.background_type !== 'none' && config.background_value && (
        <Box className="absolute inset-0 bg-black/40 backdrop-blur-sm" />
      )}

      <Box className="relative z-10 flex-1 flex flex-col px-12 py-10">
        <Box className="flex items-center justify-between mb-auto">
          <Box>
            <Typography variant="body2" className="text-surface-400 dark:text-surface-500 text-sm tracking-wide">{getGreeting()}</Typography>
            <Typography variant="h4" className="font-bold mt-1">{session?.username ?? '旅行者'}</Typography>
          </Box>
          <Box className="flex items-center gap-2">
            <Button
              variant="text"
              size="small"
              className="!rounded-full !min-w-0 !p-2"
              onClick={() => navigate('/account')}
            >
              <User className="w-4 h-4" />
            </Button>
            <Button
              variant="text"
              size="small"
              className="!rounded-full !min-w-0 !p-2"
              onClick={() => navigate('/settings')}
            >
              <Settings className="w-4 h-4" />
            </Button>
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
          ) : (
            <Box className="text-center">
              <Typography variant="body2" color="text.secondary" className="text-sm mb-1">{selected.version_id} · {getLoaderName(selected.modloader)}</Typography>
              <Typography variant="h4" className="font-bold mb-6">{selected.name}</Typography>
              <LaunchButton
                onClick={handleLaunch}
                isRunning={!!runningId}
                isLoading={launching}
                className="h-14 px-12 text-base"
              />
            </Box>
          )}
        </Box>

        <Box className="mt-auto flex items-center justify-between">
          <Box className="flex items-center gap-3">
            {selected && (
              <>
                <Button
                  size="small"
                  variant="text"
                  startIcon={<Puzzle className="w-3.5 h-3.5" />}
                  onClick={() => navigate(`/instances/${selected.id}/manage?type=mods`)}
                  className="!text-surface-400 !text-xs"
                >
                  模组
                </Button>
                <Button
                  size="small"
                  variant="text"
                  startIcon={<LayoutGrid className="w-3.5 h-3.5" />}
                  onClick={() => navigate(`/instances/${selected.id}/manage?type=resourcepacks`)}
                  className="!text-surface-400 !text-xs"
                >
                  资源包
                </Button>
                <Button
                  size="small"
                  variant="text"
                  startIcon={<Map className="w-3.5 h-3.5" />}
                  onClick={() => navigate(`/instances/${selected.id}/manage?type=mods`)}
                  className="!text-surface-400 !text-xs"
                >
                  世界
                </Button>
              </>
            )}
          </Box>
          <Button
            size="small"
            variant="text"
            onClick={() => navigate('/download')}
            className="!text-surface-400 !text-xs"
          >
            更多
          </Button>
        </Box>
      </Box>
    </Box>
  )
}
