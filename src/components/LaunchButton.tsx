import { Play, Square, Loader2 } from 'lucide-react'
import { cn } from '../lib/utils'

interface LaunchButtonProps {
  onClick: () => void
  isRunning?: boolean
  isLoading?: boolean
  disabled?: boolean
  instanceName?: string
  className?: string
}

export function LaunchButton({ 
  onClick, 
  isRunning, 
  isLoading, 
  disabled, 
  instanceName,
  className 
}: LaunchButtonProps) {
  return (
    <button
      onClick={onClick}
      disabled={disabled || isLoading}
      className={cn(
        'group relative flex flex-col items-center justify-center gap-1 rounded-2xl font-bold text-lg transition-all duration-200',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-500 focus-visible:ring-offset-2',
        'disabled:opacity-50 disabled:pointer-events-none',
        'active:scale-[0.97]',
        isRunning
          ? 'bg-red-500 text-white shadow-lg hover:bg-red-600'
          : 'bg-launch-gradient text-white shadow-launch hover:brightness-110 pulse-glow',
        className,
      )}
    >
      <div className="flex items-center gap-3">
        {isLoading ? (
          <Loader2 className="w-7 h-7 animate-spin" />
        ) : isRunning ? (
          <Square className="w-7 h-7 fill-current" />
        ) : (
          <Play className="w-7 h-7 fill-current" />
        )}
        <span>
          {isLoading ? '准备中...' : isRunning ? '停止游戏' : '启动游戏'}
        </span>
      </div>
      {instanceName && !isLoading && (
        <span className="text-xs font-normal opacity-80 truncate max-w-full px-2">
          {instanceName}
        </span>
      )}
    </button>
  )
}
