import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useNavigate } from 'react-router-dom'
import { Gamepad2 } from 'lucide-react'
import { useIslandStore } from '../../stores/islandStore'
import { useAuthStore } from '../../stores/authStore'
import { useSettingsStore } from '../../stores/settingsStore'
import { LoaderLogo } from '../LoaderLogo'
import type { Instance } from '../../types'

interface RecentInstancesProps {
  collapsed: boolean
  onHoverChange: (hover: boolean) => void
}

export function RecentInstances({ collapsed, onHoverChange }: RecentInstancesProps) {
  const navigate = useNavigate()
  const [hover, setHover] = useState(false)
  const [recent, setRecent] = useState<Instance[]>([])
  const [launchingId, setLaunchingId] = useState<string | null>(null)

  useEffect(() => {
    let alive = true
    const load = () => {
      invoke<Instance[]>('list_home_instances')
        .then((list) => {
          if (!alive) return
          const selectedId = useSettingsStore.getState().config.last_selected_instance
          const seen = new Set<string>()
          const result: Instance[] = []

          // 首页选中的实例优先
          if (selectedId) {
            const sel = list.find(i => i.id === selectedId)
            if (sel) { result.push(sel); seen.add(sel.id) }
          }

          // 再按最近游玩排序
          const sorted = [...list]
            .filter((i) => !!i.last_played && !seen.has(i.id))
            .sort((a, b) => new Date(b.last_played!).getTime() - new Date(a.last_played!).getTime())
            .filter((i) => {
              if (seen.has(i.id)) return false
              seen.add(i.id)
              return true
            })
          result.push(...sorted)

          setRecent(result.slice(0, 3))
        })
        .catch(() => {})
    }
    load()
    const t = setInterval(load, 20000)
    return () => { alive = false; clearInterval(t) }
  }, [])

  const launch = async (inst: Instance) => {
    if (launchingId) return
    setLaunchingId(inst.id)
    try {
      const session = useAuthStore.getState().session
      if (!session) {
        navigate('/account')
        return
      }
      let auth = session
      if (auth.user_type === 'msa' && auth.refresh_token) {
        try {
          auth = await invoke('microsoft_auth_refresh', { refreshToken: auth.refresh_token })
          useAuthStore.getState().setSession(auth)
        } catch {}
      }
      await invoke('launch_game', { instanceId: inst.id, auth, quickWorld: null, quickServer: null })
      useIslandStore.getState().setAiMessage('')
    } catch (e) {
      const msg = String(e)
      if (msg.includes('[launch-crash]')) {
        navigate(`/ai?instance=${inst.id}&auto_analyze=1`)
      } else {
        useIslandStore.getState().setAiMessage(`启动失败: ${msg}`)
      }
    } finally {
      setLaunchingId(null)
      setHover(false)
      onHoverChange(false)
    }
  }

  if (collapsed) {
    return (
      <div className="relative z-40" data-no-drag style={{ width: 0, overflow: 'hidden' }}>
        <div className="w-10 h-10" />
      </div>
    )
  }

  return (
    <div
      className="relative z-40"
      data-no-drag
      onMouseEnter={() => { setHover(true); onHoverChange(true) }}
      onMouseLeave={() => { setHover(false); onHoverChange(false) }}
    >
      {/* 圆卡常驻 40px flex 槽位，不改变布局 */}
      <button
        className="island-glass relative w-10 h-10 rounded-full border bg-white/95 dark:bg-surface-850/95 backdrop-blur-xl border-surface-200 dark:border-surface-700/60 shadow-lg flex items-center justify-center text-[var(--accent-color)] hover:opacity-90 active:scale-95 transition-all cursor-pointer origin-center"
        title="最近游玩"
        onClick={() => setHover(true)}
      >
        <Gamepad2 className="w-4 h-4" />
      </button>

      {/* 展开卡片：absolute 向下覆盖，不影响 flex 布局 */}
      {hover && (
        <div
          className="absolute top-full mt-1.5 left-1/2 -translate-x-1/2 flex items-center gap-1.5 py-1.5 px-2 rounded-full border bg-white/95 dark:bg-surface-850/95 backdrop-blur-xl border-surface-200 dark:border-surface-700/60 shadow-2xl whitespace-nowrap overflow-hidden"
          style={{
            width: 'auto',
            maxWidth: 360,
            animation: 'ai-pop-in 0.3s cubic-bezier(0.22, 1, 0.36, 1) both',
          }}
        >
          {recent.length === 0 ? (
            <span className="text-xs text-surface-400 px-2">暂无游玩记录</span>
          ) : (
            recent.map((inst) => (
              <button
                key={inst.id}
                onClick={() => launch(inst)}
                disabled={!!launchingId}
                className="flex items-center gap-1.5 px-2 py-1 rounded-full hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors cursor-pointer disabled:opacity-60 shrink-0"
                title={`启动 ${inst.name}`}
              >
                <div className="w-7 h-7 rounded-lg bg-[var(--accent-color)]/10 flex items-center justify-center shrink-0 p-1">
                  <LoaderLogo loader={inst.modloader} versionId={inst.version_id} className="w-full h-full" />
                </div>
                <span className="text-[11px] font-medium text-surface-700 dark:text-surface-200 truncate max-w-[90px]">{inst.name}</span>
                {launchingId === inst.id && <span className="w-3 h-3 rounded-full border-2 border-[var(--accent-color)] border-t-transparent animate-spin" />}
              </button>
            ))
          )}
        </div>
      )}
    </div>
  )
}