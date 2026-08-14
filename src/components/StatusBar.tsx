import { Download, Wifi, HardDrive } from 'lucide-react'

interface StatusBarProps {
  instanceCount: number
  currentInstance?: string | null
  downloadCount?: number
  className?: string
}

export function StatusBar({ instanceCount, currentInstance, downloadCount, className }: StatusBarProps) {
  return (
    <div className={`flex items-center justify-between px-4 py-2 bg-white/50 dark:bg-surface-850/50 border-t border-surface-200/60 dark:border-surface-700/30 text-[11px] text-surface-500 dark:text-surface-400 ${className || ''}`}>
      <div className="flex items-center gap-4">
        <span className="flex items-center gap-1.5">
          <HardDrive className="w-3.5 h-3.5" />
          {instanceCount} 个实例
        </span>
        {currentInstance && (
          <span className="flex items-center gap-1.5">
            <Wifi className="w-3.5 h-3.5 text-green-500" />
            {currentInstance}
          </span>
        )}
      </div>
      <div className="flex items-center gap-4">
        {downloadCount && downloadCount > 0 && (
          <span className="flex items-center gap-1.5">
            <Download className="w-3.5 h-3.5 text-[var(--accent-color)]" />
            {downloadCount} 个下载中
          </span>
        )}
      </div>
    </div>
  )
}
