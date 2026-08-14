import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useDownloadStore } from '../stores/downloadStore'
import { Download, ChevronUp, CheckCircle2, XCircle, Loader2, X, Rocket, Trash2 } from 'lucide-react'

interface DownloadCenterProps {
  serverCard?: boolean
}

const STAGE_LABELS: Record<string, string> = {
  pending: '等待开始',
  downloading: '下载中',
  fetching_profile: '获取版本信息',
  downloading_client: '下载游戏核心',
  downloading_asset_index: '下载资源索引',
  downloading_libraries: '下载依赖库',
  downloading_assets: '校验资源文件',
  loader: '安装加载器',
  complete: '安装完成',
  java: 'Java 环境',
  launch: '启动游戏',
}

export function DownloadCenter({ serverCard }: DownloadCenterProps) {
  const { tasks, removeTask, clearFinished } = useDownloadStore()
  const [expanded, setExpanded] = useState(false)

  const active = tasks.filter((t) => t.status === 'downloading')
  const finished = tasks.filter((t) => t.status !== 'downloading')
  const launching = active.filter((t) => t.kind === 'launch').length
  const downloadActive = active.length - launching

  if (tasks.length === 0) return null

  const overall = tasks.reduce(
    (acc, t) => acc + (t.status === 'downloading' ? t.progress : 1),
    0,
  ) / tasks.length

  const finishedCount = finished.length

  return (
    <div className={`absolute right-3 z-10 flex flex-col items-end gap-2 ${serverCard ? 'bottom-16' : 'bottom-3'}`}>
      <button
        onClick={() => setExpanded((v) => !v)}
        className="flex items-center gap-2.5 rounded-xl bg-white dark:bg-surface-850 border border-surface-200 dark:border-surface-700/60 shadow-lg px-3.5 py-2.5 hover:shadow-xl transition-all active:scale-[0.98]"
      >
        {launching > 0 && downloadActive === 0 ? (
          <Rocket className="w-4 h-4 text-[var(--accent-color)] shrink-0" />
        ) : active.length > 0 ? (
          <Loader2 className="w-4 h-4 text-[var(--accent-color)] animate-spin shrink-0" />
        ) : finishedCount > 0 ? (
          <CheckCircle2 className="w-4 h-4 text-green-500 shrink-0" />
        ) : (
          <Download className="w-4 h-4 text-surface-400 shrink-0" />
        )}
        <div className="min-w-0">
          <div className="text-xs font-medium text-surface-900 dark:text-surface-100 text-left whitespace-nowrap">
            {launching > 0 && downloadActive === 0
              ? `正在启动 ${launching} 个任务`
              : active.length > 0
                ? `正在下载 ${active.length} 个任务`
                : finishedCount > 0
                  ? `${finishedCount} 个任务已完成`
                  : '下载'}
          </div>
          <div className="w-28 h-1 bg-surface-100 dark:bg-surface-700 rounded-full overflow-hidden mt-1">
            <div
              className="h-full bg-[var(--accent-color)] rounded-full transition-all duration-300"
              style={{ width: `${Math.round(overall * 100)}%` }}
            />
          </div>
        </div>
        <ChevronUp
          className={`w-4 h-4 text-surface-400 shrink-0 transition-transform duration-200 ${expanded ? '' : 'rotate-180'}`}
        />
      </button>

      {expanded && (
        <div className="slide-up w-80 max-h-80 overflow-y-auto rounded-xl bg-white dark:bg-surface-850 border border-surface-200 dark:border-surface-700/60 shadow-xl p-2 space-y-1.5">
          <div className="flex items-center justify-between px-1.5 pt-1 pb-0.5">
            <span className="text-[11px] font-medium text-surface-500 dark:text-surface-400">
              下载任务 ({tasks.length})
            </span>
            {finishedCount > 0 && (
              <button
                onClick={clearFinished}
                className="flex items-center gap-1 text-[11px] text-surface-400 hover:text-surface-600 dark:hover:text-surface-200 transition-colors px-1 py-0.5 rounded"
              >
                <Trash2 className="w-3 h-3" />
                清除已完成
              </button>
            )}
          </div>
          {tasks.map((t) => {
            const stageLabel = STAGE_LABELS[t.stage] ?? t.stage
            return (
              <div key={t.id} className="rounded-lg p-2.5 bg-surface-50 dark:bg-surface-800/60 space-y-1">
                <div className="flex items-center gap-2">
                  {t.status === 'downloading' ? (
                    t.kind === 'launch' ? (
                      <Rocket className="w-3.5 h-3.5 text-[var(--accent-color)] shrink-0" />
                    ) : (
                      <Loader2 className="w-3.5 h-3.5 text-[var(--accent-color)] animate-spin shrink-0" />
                    )
                  ) : t.status === 'done' ? (
                    <CheckCircle2 className="w-3.5 h-3.5 text-green-500 shrink-0" />
                  ) : (
                    <XCircle className="w-3.5 h-3.5 text-red-500 shrink-0" />
                  )}
                  <span className="flex-1 text-xs font-medium text-surface-900 dark:text-surface-100 truncate">
                    {t.title}
                  </span>
                  <span className="text-xs text-surface-400 shrink-0">
                    {t.status === 'downloading'
                      ? t.kind === 'launch'
                        ? '启动中'
                        : `${Math.round(t.progress * 100)}%`
                      : t.status === 'done' ? '完成' : '失败'}
                  </span>
                  <button
                    onClick={() => {
                      if (t.kind === 'launch' && t.instanceId) {
                        invoke('cancel_game_launch', { instanceId: t.instanceId }).catch(() => {})
                      }
                      removeTask(t.id)
                    }}
                    className="shrink-0 p-0.5 rounded text-surface-400 hover:text-surface-600 dark:hover:text-surface-200 transition-colors"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </div>
                {t.status === 'downloading' ? (
                  <>
                    <div className="text-[11px] text-surface-500 truncate">
                      <span className="text-[var(--accent-color)] font-medium mr-1">{stageLabel}</span>
                      {t.message}
                    </div>
                    <div className="w-full h-1.5 bg-surface-100 dark:bg-surface-700 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-[var(--accent-color)] rounded-full transition-all duration-300"
                        style={{ width: `${Math.round(t.progress * 100)}%` }}
                      />
                    </div>
                  </>
                ) : (
                  <div className={`text-[11px] truncate ${t.status === 'error' ? 'text-red-500' : 'text-green-600 dark:text-green-400'}`}>
                    {t.status === 'error' ? (t.error || '下载失败') : `${stageLabel} · ${t.message}`}
                  </div>
                )}
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}
