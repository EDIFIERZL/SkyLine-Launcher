import { invoke } from '@tauri-apps/api/core'
import { memo, useEffect, useState } from 'react'
import { Bug, FolderOpen, X, Zap } from 'lucide-react'

interface GameExitInfo {
  instance_id: string
  exit_code?: number | null
  reason: string
  play_time_secs: number
}

type Props = {
  open: boolean
  exitInfo: GameExitInfo | null
  instanceName?: string
  onClose: () => void
  onAnalyze: (instanceId: string) => void
}

const CrashDialog = memo(function CrashDialog({ open, exitInfo, instanceName, onClose, onAnalyze }: Props) {
  const [crashPath, setCrashPath] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (open && exitInfo?.instance_id) {
      setLoading(true)
      invoke<string | null>('get_crash_file_path', { instanceId: exitInfo.instance_id })
        .then(setCrashPath)
        .catch(() => setCrashPath(null))
        .finally(() => setLoading(false))
    }
  }, [open, exitInfo?.instance_id])

  if (!open || !exitInfo) return null

  const reasonLabels: Record<string, string> = {
    Crash: '游戏崩溃',
    NoWindow: '窗口未出现',
    Normal: '正常退出',
    Killed: '被强制终止',
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      onClick={onClose}>
      <div className="bg-surface-800 border border-red-500/30 rounded-2xl shadow-2xl w-full max-w-md mx-4 overflow-hidden"
        onClick={(e) => e.stopPropagation()}>
        <div className="bg-red-500/10 border-b border-red-500/20 px-5 py-4 flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-red-500/20 flex items-center justify-center shrink-0">
            <Bug className="w-5 h-5 text-red-400" />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium text-red-300">游戏异常退出</p>
            <p className="text-xs text-red-400/70 truncate">
              {instanceName ?? exitInfo.instance_id}
            </p>
          </div>
          <button onClick={onClose} className="p-1.5 rounded-lg hover:bg-white/10 transition-colors">
            <X className="w-4 h-4 text-red-400/70" />
          </button>
        </div>

        <div className="px-5 py-4 space-y-3">
          <div className="flex items-center justify-between text-sm">
            <span className="text-surface-400">原因</span>
            <span className="text-red-300 font-medium">
              {reasonLabels[exitInfo.reason] ?? exitInfo.reason}
            </span>
          </div>
          {exitInfo.exit_code !== undefined && exitInfo.exit_code !== null && (
            <div className="flex items-center justify-between text-sm">
              <span className="text-surface-400">退出码</span>
              <span className="text-surface-200 font-mono">{exitInfo.exit_code}</span>
            </div>
          )}
          {exitInfo.play_time_secs !== undefined && (
            <div className="flex items-center justify-between text-sm">
              <span className="text-surface-400">运行时长</span>
              <span className="text-surface-300">
                {Math.floor(exitInfo.play_time_secs / 60)}分{exitInfo.play_time_secs % 60}秒
              </span>
            </div>
          )}
          {crashPath && (
            <div className="text-xs text-surface-500 truncate bg-surface-900 rounded-lg px-3 py-2">
              {crashPath}
            </div>
          )}
          {loading && (
            <div className="text-xs text-surface-400 animate-pulse">正在查找崩溃报告...</div>
          )}
        </div>

        <div className="px-5 pb-4 flex gap-2">
          {crashPath && (
            <button
              onClick={() => invoke('open_folder_select', { path: crashPath })}
              className="flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl bg-surface-700 hover:bg-surface-600 text-surface-300 transition-colors text-sm">
              <FolderOpen className="w-4 h-4" />
              打开崩溃报告
            </button>
          )}
          <button
            onClick={() => onAnalyze(exitInfo.instance_id)}
            className="flex-1 flex items-center justify-center gap-2 py-2.5 rounded-xl bg-blue-500/20 hover:bg-blue-500/30 text-blue-300 transition-colors text-sm">
            <Zap className="w-4 h-4" />
            AI 分析
          </button>
        </div>
      </div>
    </div>
  )
})

export default CrashDialog
