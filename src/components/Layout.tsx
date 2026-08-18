import { useEffect, useState, useRef, type MouseEvent as ReactMouseEvent } from 'react'
import { Outlet, useLocation, useNavigate } from 'react-router-dom'
import { useSettingsStore } from '../stores/settingsStore'
import { useKeyboardShortcuts } from '../hooks/useKeyboardShortcuts'
import { useMemoryOptimizer } from '../hooks/useMemoryOptimizer'
import { useAuthRefresh } from '../hooks/useAuthRefresh'
import { NotificationContainer } from './NotificationContainer'
import { ComponentIsland } from './island/ComponentIsland'
import { DownloadCenter } from './DownloadCenter'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  Minus,
  Square,
  X,
  Maximize,
} from 'lucide-react'
import {
  Home as MuiHome,
  Apps as MuiApps,
  LibraryBooks as MuiLibrary,
  MusicNote as MuiMusicNote,
  Settings as MuiSettings,
  Person as MuiPerson,
} from '@mui/icons-material'
import { NavigationRail } from './material'
import { Network } from 'lucide-react'

const navTopItems = [
  { id: '/', label: '首页', icon: <MuiHome fontSize="small" /> },
  { id: '/download', label: '资源', icon: <MuiApps fontSize="small" /> },
  { id: '/library', label: '库', icon: <MuiLibrary fontSize="small" /> },
  { id: '/music', label: '音乐', icon: <MuiMusicNote fontSize="small" /> },
  { id: '/multiplayer', label: '联机', icon: <Network className="w-5 h-5" /> },
]

const navBottomItems = [
  { id: '/ai', label: 'AI', icon: <span className="text-xs font-black tracking-widest select-none">AI</span> },
  { id: '/account', label: '账户', icon: <MuiPerson fontSize="small" /> },
  { id: '/settings', label: '设置', icon: <MuiSettings fontSize="small" /> },
]

function hexToRgb(hex: string) {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  return result
    ? `${parseInt(result[1], 16)}, ${parseInt(result[2], 16)}, ${parseInt(result[3], 16)}`
    : '59, 130, 246'
}

function LogoMark({ className }: { className?: string }) {
  const [imgFailed, setImgFailed] = useState(false)
  if (!imgFailed) {
    return (
      <img
        src="/logo.png"
        alt="SkyLine"
        className={`${className} object-contain rounded-md shrink-0`}
        onError={() => setImgFailed(true)}
      />
    )
  }
  return (
    <svg viewBox="0 0 32 32" className={className} aria-hidden>
      <path d="M6 4h20v20H6z" fill="none" stroke="var(--accent-color)" strokeWidth="2.2" />
      <path d="M6 8h10v10H6z" fill="color-mix(in srgb, var(--accent-color) 45%, transparent)" />
      <path d="M16 14h10v10H16z" fill="color-mix(in srgb, var(--accent-color) 30%, transparent)" />
    </svg>
  )
}

export function Layout() {
  const { config } = useSettingsStore()
  const location = useLocation()
  const navigate = useNavigate()
  const glowRef = useRef<HTMLDivElement>(null)
  const [zenMode, setZenMode] = useState(false)
  const [zenPeek, setZenPeek] = useState(false)
  const peekTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const { onTabSwitch } = useMemoryOptimizer()
  useAuthRefresh()
  const prevPathRef = useRef(location.pathname)

  const canZen = config.liquid_glass && config.liquid_glass_mode === 'normal' && ((config.background_type === 'image' && config.background_value) || config.background_type === 'video')
  
  const hideImmersiveBtn = !canZen

  useEffect(() => {
    if (prevPathRef.current !== location.pathname) {
      prevPathRef.current = location.pathname
      onTabSwitch()
    }
  }, [location.pathname, onTabSwitch])

  
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'F12' || (e.ctrlKey && e.shiftKey && e.key === 'I') || (e.metaKey && e.altKey && e.key === 'i')) {
        e.preventDefault()
      }
    }
    document.addEventListener('keydown', handler)
    return () => document.removeEventListener('keydown', handler)
  }, [])

  
  useKeyboardShortcuts()

  
  useEffect(() => {
    if (!config.liquid_glass) return
    let raf = 0
    const handleMouseMove = (e: MouseEvent) => {
      if (raf) return
      raf = requestAnimationFrame(() => {
        raf = 0
        if (glowRef.current) {
          glowRef.current.style.transform = `translate(${e.clientX - 200}px, ${e.clientY - 200}px)`
        }
        const vw = window.innerWidth
        const vh = window.innerHeight
        document.documentElement.style.setProperty('--light-x', `${(e.clientX / vw) * 100}%`)
        document.documentElement.style.setProperty('--light-y', `${(e.clientY / vh) * 100}%`)
      })
    }
    document.addEventListener('mousemove', handleMouseMove, { passive: true })
    return () => document.removeEventListener('mousemove', handleMouseMove)
  }, [config.liquid_glass])

  useEffect(() => {
    const root = document.documentElement
    const isDark = config.theme_mode === 'dark' || (config.theme_mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
    const accentColor = config.accent_color || '#3b82f6'

    root.classList.toggle('dark', isDark)
    root.style.setProperty('--accent-rgb', hexToRgb(accentColor))
    root.style.setProperty('--accent-color', accentColor)
    root.style.setProperty('--ui-scale', String(config.ui_scale))
    root.style.setProperty('--font-size-multiplier', config.font_size === 'small' ? '0.9' : config.font_size === 'large' ? '1.1' : '1')

    const solidBg = isDark ? '#000000' : '#ffffff'
    const accent = accentColor

    const isTransparentGlass =
      config.liquid_glass &&
      config.liquid_glass_mode === 'transparent' &&
      config.background_type === 'none'

    if (isTransparentGlass) {
      root.style.setProperty('--app-solid', 'transparent')
      root.style.setProperty('--app-background', 'transparent')
    } else {
      root.style.setProperty('--app-solid', solidBg)
      root.style.setProperty('--app-background', solidBg)
    }

    if (config.background_type === 'gradient' && config.background_value) {
      root.style.setProperty('--app-bg-image', config.background_value)
    } else if (config.background_type === 'blur') {
      root.style.setProperty(
        '--app-bg-image',
        `radial-gradient(circle at 50% 20%, rgba(${hexToRgb(accent)}, 0.28) 0%, transparent 55%), radial-gradient(circle at 85% 80%, rgba(${hexToRgb(accent)}, 0.18) 0%, transparent 50%)`
      )
    } else if (config.background_type === 'image' && config.background_value) {
      root.style.setProperty('--app-bg-image', `url(${config.background_value})`)
    } else {
      root.style.setProperty('--app-bg-image', 'none')
    }


    root.classList.toggle('liquid-glass', config.liquid_glass)
    root.classList.toggle('liquid-glass-transparent', isTransparentGlass)

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handler = () => {
      if (config.theme_mode === 'system') {
        root.classList.toggle('dark', mediaQuery.matches)
      }
    }
    mediaQuery.addEventListener('change', handler)
    return () => mediaQuery.removeEventListener('change', handler)
  }, [config])

  const appWindow = getCurrentWindow()

  const toggleMaximize = () => {
    appWindow.toggleMaximize()
  }

  const onHeaderMouseDown = (e: ReactMouseEvent<HTMLElement>) => {
    if (e.button !== 0) return
    const target = e.target as HTMLElement
    if (target.closest('button, input, select, a, [data-no-drag]')) return
    appWindow.startDragging().catch(() => {})
  }

  return (
    <div className={`relative flex flex-col h-screen w-screen app-bg overflow-hidden ${zenMode ? 'zen-mode' : ''} ${zenPeek ? 'zen-peek' : ''}`}
      style={{ fontSize: `calc(0.875rem * var(--font-size-multiplier))` }}
      onClick={() => {
        if (zenMode && !zenPeek) {
          setZenPeek(true)
          if (peekTimer.current) clearTimeout(peekTimer.current)
          peekTimer.current = setTimeout(() => setZenPeek(false), 3000)
        }
      }}
    >
      {}
      <NotificationContainer />

      {config.background_type === 'video' && config.background_value && (
        <video
          className="absolute inset-0 z-0 w-full h-full object-cover pointer-events-none"
          src={config.background_value}
          autoPlay
          muted
          loop
          playsInline
        />
      )}

      {}
      {config.liquid_glass && (
        <div ref={glowRef} className="glass-glow" style={{ left: 0, top: 0 }} />
      )}

      {canZen && !hideImmersiveBtn && (!zenMode || zenPeek) && (
        <button
          onClick={(e) => {
            e.stopPropagation()
            if (zenMode) {
              setZenMode(false)
              setZenPeek(false)
              if (peekTimer.current) clearTimeout(peekTimer.current)
            } else {
              setZenMode(true)
            }
          }}
          className="fixed bottom-5 right-5 z-[9999] w-10 h-10 rounded-full flex items-center justify-center bg-black/30 dark:bg-white/10 text-white/70 dark:text-surface-400 hover:bg-black/50 dark:hover:bg-white/20 transition-all duration-300 cursor-pointer"
          title={zenMode ? '退出沉浸模式' : '沉浸模式 - 隐藏界面组件'}
        >
          {zenMode ? <X className="w-4 h-4" /> : <Maximize className="w-4 h-4" />}
        </button>
      )}

      <header
        className="app-header relative z-20 h-9 shrink-0 flex items-center border-b border-surface-200/60 dark:border-surface-700/30 select-none bg-white/80 dark:bg-surface-900/80 backdrop-blur-sm"
        data-tauri-drag-region
        onMouseDown={onHeaderMouseDown}
      >
        <ComponentIsland />

        <div
          className="h-full flex items-center gap-2 px-4"
          data-tauri-drag-region
        >
          <LogoMark className="w-6 h-6" />
          <span className="text-sm font-bold tracking-wide text-surface-800 dark:text-surface-200" data-tauri-drag-region>
            SkyLine Launcher
          </span>
        </div>
        <div className="flex-1 h-full" data-tauri-drag-region onDoubleClick={toggleMaximize} />
        <div className="flex items-center h-full">
          <button
            onClick={() => appWindow.minimize()}
            className="w-10 h-full flex items-center justify-center text-surface-400 hover:text-surface-600 dark:hover:text-surface-300 hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors"
            title="最小化"
          >
            <Minus className="w-4 h-4" />
          </button>
          <button
            onClick={toggleMaximize}
            className="w-10 h-full flex items-center justify-center text-surface-400 hover:text-surface-600 dark:hover:text-surface-300 hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors"
            title="最大化"
          >
            <Square className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={() => appWindow.close()}
            className="w-10 h-full flex items-center justify-center text-surface-400 hover:text-white hover:bg-red-500 transition-colors"
            title="关闭"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </header>

      <div className="app-content relative z-10 flex flex-1 overflow-hidden">
        <NavigationRail
          topItems={navTopItems}
          bottomItems={navBottomItems}
          activeId={location.pathname}
          onNavigate={(id) => navigate(id)}
        />

        <main className="flex-1 app-main overflow-hidden">
          <div className="h-full overflow-y-auto overflow-x-hidden p-6">
            <div key={location.pathname} className="page-enter h-full">
              <Outlet />
            </div>
          </div>
        </main>
      </div>

      <DownloadCenter serverCard={!!(config.server_address && !config.hide_server_card)} />
    </div>
  )
}
