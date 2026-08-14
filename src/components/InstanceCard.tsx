import { LoaderLogo } from './LoaderLogo'
import { Play, Clock, Server } from 'lucide-react'
import type { Instance } from '../types'

interface InstanceCardProps {
  instance: Instance
  onLaunch: (instance: Instance) => void
  onSelect?: (instance: Instance) => void
  isRunning?: boolean
  isSelected?: boolean
  className?: string
  style?: React.CSSProperties
}

function formatPlayTime(seconds: number): string {
  if (seconds === 0) return ''
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  if (h > 0) return `${h}h ${m}m`
  return `${m}m`
}

function formatLastPlayed(dateStr: string | null): string {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  if (days === 0) return '今天'
  if (days === 1) return '昨天'
  if (days < 7) return `${days}天前`
  return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric' })
}

export function InstanceCard({ instance, onLaunch, onSelect, isRunning, isSelected, className, style }: InstanceCardProps) {
  const playTime = formatPlayTime(instance.play_time)
  const lastPlayed = formatLastPlayed(instance.last_played)

  return (
    <div
      className={`group relative bg-white dark:bg-surface-850 border rounded-2xl p-4 
        hover:border-[var(--accent-color)]/30 hover:shadow-card hover:-translate-y-1
        transition-all duration-200 cursor-pointer card-enter
        ${isSelected 
          ? 'border-[var(--accent-color)] dark:border-[var(--accent-color)] shadow-card' 
          : 'border-surface-200/80 dark:border-surface-700/40'
        } ${className || ''}`}
      style={style}
      onClick={() => onSelect?.(instance)}
    >
      <div className="flex items-start gap-3 mb-3">
        <div className="w-10 h-10 rounded-xl bg-accent-50 dark:bg-accent-500/10 flex items-center justify-center shrink-0">
          <LoaderLogo loader={instance.modloader} versionId={instance.version_id} className="w-6 h-6" />
        </div>
        <div className="flex-1 min-w-0">
          <h3 className="font-semibold text-surface-900 dark:text-surface-100 truncate text-sm">
            {instance.name}
          </h3>
          <p className="text-xs text-surface-500 dark:text-surface-400 truncate mt-0.5">
            {instance.version_id}
          </p>
        </div>
        {isRunning && (
          <div className="flex items-center gap-1.5 px-2 py-1 bg-green-500/10 rounded-full">
            <div className="w-2 h-2 rounded-full bg-green-500 status-pulse" />
            <span className="text-[10px] font-medium text-green-600 dark:text-green-400">运行中</span>
          </div>
        )}
        {isSelected && !isRunning && (
          <div className="flex items-center gap-1.5 px-2 py-1 bg-[var(--accent-color)]/10 rounded-full">
            <span className="text-[10px] font-medium text-[var(--accent-color)]">已选中</span>
          </div>
        )}
      </div>

      <div className="flex flex-wrap gap-1.5 mb-3">
        {instance.server_ip && (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 bg-surface-100 dark:bg-surface-800 rounded-md text-[10px] text-surface-500">
            <Server className="w-3 h-3" />
            {instance.server_ip}
          </span>
        )}
        {playTime && (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 bg-surface-100 dark:bg-surface-800 rounded-md text-[10px] text-surface-500">
            <Clock className="w-3 h-3" />
            {playTime}
          </span>
        )}
        {lastPlayed && (
          <span className="inline-flex items-center gap-1 px-2 py-0.5 bg-surface-100 dark:bg-surface-800 rounded-md text-[10px] text-surface-500">
            {lastPlayed}
          </span>
        )}
      </div>

      <div className="flex items-center justify-between">
        <button
          onClick={(e) => {
            e.stopPropagation()
            onLaunch(instance)
          }}
          disabled={isRunning}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all duration-200
            ${isRunning
              ? 'bg-red-500/10 text-red-500 hover:bg-red-500/20'
              : 'bg-[var(--accent-color)]/10 text-[var(--accent-color)] hover:bg-[var(--accent-color)]/20 active:scale-95'
            }`}
        >
          <Play className="w-3.5 h-3.5 fill-current" />
          {isRunning ? '停止' : '启动'}
        </button>
      </div>
    </div>
  )
}
