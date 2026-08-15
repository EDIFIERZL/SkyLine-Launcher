import { forwardRef, useEffect, useImperativeHandle, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useInstanceStore } from '../stores/instanceStore'
import { useAuthStore } from '../stores/authStore'
import { listen } from '@tauri-apps/api/event'
import { triggerAggressiveOptimize } from '../hooks/useMemoryOptimizer'
import { Box, Typography, Card, Button, IconButton, Chip } from './material'
import { LoaderLogo } from './LoaderLogo'
import {
  Play,
  Trash2,
  Square,
  Gamepad2,
  Server,
  Clock,
  Puzzle,
  ChevronRight,
  FolderOpen,
  FolderPlus,
  X,
  AlertTriangle,
  ClipboardCopy,
  AlertCircle,
  Info,
  ShieldAlert,
  Skull,
  CheckCircle2,
  Settings,
} from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import { useDownloadStore } from '../stores/downloadStore'
import type { AuthSession, CrashAnalysis, CrashSeverity, GameLogEvent, GameExitInfo, LaunchProgressEvent, Instance, GameProcessInfo, InstallProgress } from '../types'
import { sortInstances } from '../lib/instanceSort'

function formatPlayTime(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  if (h > 0) return `${h}h ${m}m`
  return `${m}m`
}

const SEVERITY_CONFIG: Record<CrashSeverity, { icon: typeof AlertTriangle; label: string; border: string; bg: string; text: string }> = {
  critical: { icon: Skull, label: '致命', border: 'border-red-300 dark:border-red-500/30', bg: 'bg-red-50 dark:bg-red-500/10', text: 'text-red-800 dark:text-red-400' },
  error: { icon: AlertCircle, label: '错误', border: 'border-orange-300 dark:border-orange-500/30', bg: 'bg-orange-50 dark:bg-orange-500/10', text: 'text-orange-800 dark:text-orange-400' },
  warning: { icon: AlertTriangle, label: '警告', border: 'border-amber-300 dark:border-amber-500/30', bg: 'bg-amber-50 dark:bg-amber-500/10', text: 'text-amber-800 dark:text-amber-400' },
  info: { icon: Info, label: '信息', border: 'border-blue-300 dark:border-blue-500/30', bg: 'bg-blue-50 dark:bg-blue-500/10', text: 'text-blue-800 dark:text-blue-400' },
}

export interface InstancePanelHandle {
  launch: (id: string) => Promise<void>
}

interface Props {
  onCollapse: () => void
  selectedId?: string | null
  onSelect?: (id: string) => void
}

export const InstancePanel = forwardRef<InstancePanelHandle, Props>(function InstancePanel(
  { onCollapse, selectedId, onSelect },
  ref,
) {
  const navigate = useNavigate()
  const { instances, setInstances } = useInstanceStore()
  const session = useAuthStore((s) => s.session)
  const setSession = useAuthStore((s) => s.setSession)
  const [folders, setFolders] = useState<string[]>([])
  const [showFolders, setShowFolders] = useState(false)
  const [runningId, setRunningId] = useState<string | null>(null)
  const [logsMap, setLogsMap] = useState<Record<string, string[]>>({})
  const [logVisible, setLogVisible] = useState<string | null>(null)
  const [crash, setCrash] = useState<CrashAnalysis | null>(null)
  const [analyzing, setAnalyzing] = useState(false)
  const logBoxRef = useRef<HTMLDivElement>(null)

  const copyLogs = async (id: string) => {
    const text = (logsMap[id] ?? []).join('\n')
    try {
      await navigator.clipboard.writeText(text)
    } catch {
      alert('复制失败')
    }
  }

  const handleAnalyzeCrash = async (id: string) => {
    setAnalyzing(true)
    setCrash(null)
    try {
      const result = await invoke<CrashAnalysis | null>('analyze_crash', { instanceId: id })
      setCrash(result)
      if (!result) alert('未找到崩溃报告或日志')
    } catch (e) {
      alert(`分析失败: ${e}`)
    }
    setAnalyzing(false)
  }

  useEffect(() => {
    if (logBoxRef.current) {
      logBoxRef.current.scrollTop = logBoxRef.current.scrollHeight
    }
  }, [logsMap])

  const loadInstances = () => {
    invoke<Instance[]>('list_instances').then(setInstances).catch(console.error)
  }

  const loadFolders = () => {
    invoke<string[]>('list_instance_folders').then(setFolders).catch(console.error)
  }

  useEffect(() => {
    loadInstances()
    loadFolders()

    const unsubLog = listen<GameLogEvent>('game-log', (e) => {
      const { instance_id, level, message } = e.payload
      setLogsMap((prev) => ({
        ...prev,
        [instance_id]: [...(prev[instance_id] ?? []).slice(-500), `[${level}] ${message}`],
      }))
    })
    const unsubStop = listen<GameExitInfo>('game-stopped', (e) => {
      const exitInfo = e.payload
      setRunningId(null)
      if (exitInfo.reason === 'crash' || exitInfo.reason === 'nowindow') {
        handleAnalyzeCrash(exitInfo.instance_id)
      }
    })
    const unsubProgress = listen<LaunchProgressEvent>('launch-progress', () => {})
    return () => {
      unsubLog.then((f) => f())
      unsubStop.then((f) => f())
      unsubProgress.then((f) => f())
    }
  }, [])

  const launch = async (inst: Instance) => {
    if (!session) {
      alert('请先登录账户')
      return
    }
    setRunningId(inst.id)
    setLogVisible(inst.id)
    setLogsMap((prev) => ({ ...prev, [inst.id]: [] }))
    const taskId = `launch-${inst.id}-${Date.now()}`
    let unsub: (() => void) | null = null
    const launchUnsub: { f: (() => void) | null } = { f: null }
    useDownloadStore.getState().addTask({
      id: taskId,
      title: `准备启动 ${inst.name}`,
      status: 'downloading',
      kind: 'launch',
      instanceId: inst.id,
      stage: 'java',
      progress: 0,
      message: '检查 Java 环境...',
    })
    try {
      unsub = await listen<InstallProgress>('install-progress', (e) => {
        if (e.payload.stage === 'java') {
          useDownloadStore.getState().updateTask(taskId, {
            stage: e.payload.stage,
            progress: e.payload.progress,
            message: e.payload.message,
          })
        }
      })
      let auth = session
      if (auth.user_type === 'msa' && auth.refresh_token) {
        try {
          auth = await invoke<AuthSession>('microsoft_auth_refresh', { refreshToken: auth.refresh_token })
          setSession(auth)
        } catch (e) {
          console.error('自动刷新登录失败，使用原令牌尝试启动', e)
          auth = session
        }
      }
      triggerAggressiveOptimize()
      await invoke<GameProcessInfo>('launch_game', {
        instanceId: inst.id,
        auth,
      })
      await new Promise<void>((resolve) => {
        let settled = false
        const finish = () => { if (!settled) { settled = true; resolve() } }
        listen<LaunchProgressEvent>('launch-progress', (e) => {
          if (e.payload.instance_id !== inst.id) return
          if (e.payload.stage === 'running') finish()
        }).then((f) => { launchUnsub.f = f })
        setTimeout(() => finish(), 90000)
      })
      useDownloadStore.getState().markDone(taskId)
    } catch (e) {
      console.error(e)
      useDownloadStore.getState().markError(taskId, String(e))
      setRunningId(null)
    } finally {
      if (unsub) unsub()
      if (launchUnsub.f) launchUnsub.f()
      setTimeout(() => useDownloadStore.getState().removeTask(taskId), 3000)
    }
  }

  useImperativeHandle(
    ref,
    () => ({
      launch: (id: string) => {
        const inst = instances.find((i) => i.id === id)
        if (inst) return launch(inst)
        return Promise.resolve()
      },
    }),
    [instances, session, launch],
  )

  const handleStop = async (id: string) => {
    await invoke('stop_game', { instanceId: id })
    setRunningId(null)
  }

  const handleDelete = async (id: string) => {
    if (!confirm('确定删除实例吗？该操作会删除实例目录下的所有文件。')) return
    try {
      await invoke('delete_instance', { id })
      loadInstances()
    } catch (e) {
      alert(String(e))
    }
  }

  const handleAddFolder = async () => {
    try {
      const picked = await open({ directory: true, multiple: false })
      if (typeof picked === 'string' && picked) {
        await invoke('add_instance_folder', { folder: picked })
        loadFolders()
        loadInstances()
      }
    } catch (e) {
      alert(String(e))
    }
  }

  const handleRemoveFolder = async (folder: string) => {
    try {
      await invoke('remove_instance_folder', { folder })
      loadFolders()
      loadInstances()
    } catch (e) {
      alert(String(e))
    }
  }

  const externalCount = instances.filter((i) => i.external).length
  const sortedInstances = sortInstances(instances)

  return (
    <Box className="h-full flex flex-col bg-white dark:bg-surface-900 border-l border-surface-200 dark:border-surface-700">
      <Box className="flex items-center gap-2 px-3 py-2.5 border-b border-surface-200 dark:border-surface-700">
        <button
          onClick={onCollapse}
          className="w-7 h-7 flex items-center justify-center rounded-lg text-surface-500 hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors"
          title="收起实例管理"
        >
          <ChevronRight className="w-4 h-4" />
        </button>
        <Typography variant="subtitle2" className="flex items-center gap-2">
          <FolderOpen className="w-4 h-4 text-[var(--accent-color)]" /> 实例管理
        </Typography>
        <Box className="flex-1" />
        <Button size="small" variant="outlined" onClick={() => setShowFolders((v) => !v)}>
          <FolderPlus className="w-3.5 h-3.5 mr-1" /> 实例文件夹
        </Button>
      </Box>

      <Box className="flex-1 overflow-y-auto p-3 space-y-3">
        {showFolders && (
          <Card className="p-4 space-y-3">
            <Box className="flex items-center justify-between">
              <Typography variant="subtitle2">实例文件夹</Typography>
              <Button size="small" onClick={handleAddFolder}>
                <FolderPlus className="w-3.5 h-3.5 mr-1" /> 选择文件夹
              </Button>
            </Box>
            <Typography variant="caption" color="text.secondary">
              选择实例文件夹（如 PCL / HMCL 的 .minecraft 目录）。
            </Typography>
            {folders.length === 0 ? (
              <Typography variant="body2" color="text.secondary">尚未添加任何文件夹</Typography>
            ) : (
              <Box className="space-y-2">
                {folders.map((folder) => (
                  <Box key={folder} className="flex items-center gap-2 bg-surface-50 dark:bg-surface-850 rounded-lg px-3 py-2">
                    <FolderOpen className="w-4 h-4 text-[var(--accent-color)] shrink-0" />
                    <Typography variant="caption" color="text.secondary" className="flex-1 min-w-0 truncate">{folder}</Typography>
                    <IconButton title="移除文件夹" onClick={() => handleRemoveFolder(folder)}>
                      <X className="w-3.5 h-3.5" />
                    </IconButton>
                  </Box>
                ))}
              </Box>
            )}
          </Card>
        )}

        {instances.length === 0 ? (
          <Box className="flex flex-col items-center justify-center py-16">
            <Gamepad2 className="w-12 h-12 mb-3 text-surface-300 dark:text-surface-600" />
            <Typography variant="body2" color="text.secondary">还没有实例，去「资源」页下载一个游戏吧</Typography>
          </Box>
        ) : (
          sortedInstances.map((inst) => (
            <Card key={inst.id} className="p-3.5">
              <Box className="space-y-2.5">
                <Box className="flex items-center gap-3">
                  <Box className="w-9 h-9 rounded-lg bg-accent-50 dark:bg-accent-500/10 flex items-center justify-center shrink-0 p-1.5">
                    <LoaderLogo loader={inst.modloader} versionId={inst.version_id} className="w-full h-full" />
                  </Box>
                  <Box className="flex-1 min-w-0">
                    <Box className="flex items-center gap-1.5">
                      <Typography variant="subtitle2" className="truncate">{inst.name}</Typography>
                      {inst.external && <Chip label="外部" size="small" color="warning" variant="outlined" />}
                    </Box>
                    <Box className="flex items-center gap-2">
                      <Typography variant="caption" color="text.secondary" className="truncate">
                        {inst.version_id || '未知版本'}
                      </Typography>
                      {inst.play_time > 0 && (
                        <Typography variant="caption" color="text.secondary" className="flex items-center gap-0.5 shrink-0">
                          <Clock className="w-3 h-3" /> {formatPlayTime(inst.play_time)}
                        </Typography>
                      )}
                    </Box>
                  </Box>
                </Box>
                {inst.server_ip && (
                  <Box className="flex items-center gap-1.5">
                    <Server className="w-3 h-3 text-surface-400" />
                    <Typography variant="caption" color="text.secondary">服务器: {inst.server_ip}</Typography>
                  </Box>
                )}
                <Box className="flex items-center gap-2">
                  {runningId === inst.id ? (
                    <Button size="small" variant="contained" color="error" startIcon={<Square className="w-3.5 h-3.5" />} onClick={() => handleStop(inst.id)}>
                      停止
                    </Button>
                  ) : selectedId === inst.id ? (
                    <Button size="small" variant="contained" color="success" startIcon={<CheckCircle2 className="w-3.5 h-3.5" />}>
                      已选择
                    </Button>
                  ) : (
                    <Button size="small" variant="outlined" startIcon={<Play className="w-3.5 h-3.5" />} onClick={() => onSelect?.(inst.id)}>
                      选择
                    </Button>
                  )}
                  <IconButton title="管理" onClick={() => navigate(`/instances/${inst.id}/manage`)}>
                    <Settings className="w-4 h-4" />
                  </IconButton>
                  <Box className="flex-1" />
                  {!inst.external && (
                    <IconButton title="删除" onClick={() => handleDelete(inst.id)}>
                      <Trash2 className="w-4 h-4 text-red-400" />
                    </IconButton>
                  )}
                </Box>
                {logVisible === inst.id && (logsMap[inst.id]?.length ?? 0) > 0 && (
                  <Box className="space-y-2">
                    <Box className="flex items-center gap-2">
                      <Typography variant="caption" color="text.secondary">
                        日志 ({(logsMap[inst.id]?.length ?? 0)} 条)
                      </Typography>
                    </Box>
                    <Box
                      ref={logBoxRef}
                      className="bg-surface-900 rounded-lg p-3 max-h-40 overflow-y-auto font-mono text-xs space-y-0.5"
                    >
                      {logsMap[inst.id].map((line, i) => (
                        <div
                          key={i}
                          className={
                            line.includes('[Watcher]')
                              ? 'text-purple-400'
                              : line.includes('[error]') || line.includes('ERROR') || line.includes('Exception')
                                ? 'text-red-400'
                                : line.includes('[warn]') || line.includes('WARN')
                                  ? 'text-amber-400'
                                  : 'text-surface-400'
                          }
                        >
                          {line}
                        </div>
                      ))}
                    </Box>
                    <Box className="flex gap-2">
                      <Button size="small" variant="outlined" onClick={() => copyLogs(inst.id)}>
                        <ClipboardCopy className="w-3.5 h-3.5 mr-1" /> 复制日志
                      </Button>
                      <Button size="small" variant="outlined" onClick={() => handleAnalyzeCrash(inst.id)} loading={analyzing}>
                        <AlertTriangle className="w-3.5 h-3.5 mr-1" /> 分析崩溃原因
                      </Button>
                      {crash && (
                        <Button size="small" variant="text" onClick={() => setCrash(null)}>
                          <X className="w-3.5 h-3.5 mr-1" /> 关闭分析
                        </Button>
                      )}
                    </Box>
                    {crash && (() => {
                      const sev = SEVERITY_CONFIG[crash.severity] ?? SEVERITY_CONFIG.error
                      const SevIcon = sev.icon
                      return (
                        <Box className={`${sev.bg} border ${sev.border} rounded-lg p-3 space-y-2`}>
                          <Box className="flex items-center gap-2">
                            <SevIcon className={`w-4 h-4 ${sev.text}`} />
                            <Typography variant="subtitle2" className={`font-medium ${sev.text}`}>
                              {sev.label}：
                              {crash.stage === 'launch' ? '启动阶段' :
                                crash.stage === 'mods' ? '模组加载阶段' :
                                crash.stage === 'world' ? '世界加载阶段' :
                                crash.stage === 'game' ? '游戏运行阶段' : '未知阶段'} 崩溃
                            </Typography>
                            {crash.is_abnormal && (
                              <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-surface-200 dark:bg-surface-700 text-surface-500">
                                非正常退出
                              </span>
                            )}
                          </Box>
                          {crash.description && (
                            <Typography variant="caption" color="text.secondary">描述: {crash.description}</Typography>
                          )}
                          {crash.exception && (
                            <Typography variant="caption" color="text.secondary" className="font-mono break-all text-[11px] block bg-surface-100 dark:bg-surface-800 rounded px-2 py-1">
                              {crash.exception}
                            </Typography>
                          )}
                          {crash.conflicting_mods.length > 0 && (
                            <Box className="flex items-center gap-1.5 flex-wrap">
                              <ShieldAlert className="w-3.5 h-3.5 text-orange-500" />
                              <Typography variant="caption" color="text.secondary">冲突模组:</Typography>
                              {crash.conflicting_mods.map((m, i) => (
                                <span key={i} className="px-1.5 py-0.5 rounded text-[10px] bg-orange-100 dark:bg-orange-500/20 text-orange-700 dark:text-orange-400 font-medium">
                                  {m}
                                </span>
                              ))}
                            </Box>
                          )}
                          {crash.detected_mods.length > 0 && (
                            <Box className="flex items-center gap-1.5 flex-wrap">
                              <Puzzle className="w-3.5 h-3.5 text-surface-400" />
                              <Typography variant="caption" color="text.secondary">相关模组:</Typography>
                              {crash.detected_mods.map((m, i) => (
                                <span key={i} className="px-1.5 py-0.5 rounded text-[10px] bg-surface-200 dark:bg-surface-700 text-surface-600 dark:text-surface-400 font-medium">
                                  {m}
                                </span>
                              ))}
                            </Box>
                          )}
                          {crash.suggestions.length > 0 && (
                            <Box component="ul" className="space-y-1">
                              {crash.suggestions.map((s, i) => (
                                <Box component="li" key={i} className="flex gap-1.5">
                                  <span className="shrink-0 text-[var(--accent-color)]">▸</span>
                                  <Typography variant="caption" color="text.secondary">{s}</Typography>
                                </Box>
                              ))}
                            </Box>
                          )}
                          {crash.report_path && (
                            <Typography variant="caption" color="text.secondary" className="break-all text-[10px] block opacity-60">
                              报告: {crash.report_path}
                            </Typography>
                          )}
                        </Box>
                      )
                    })()}
                  </Box>
                )}
              </Box>
            </Card>
          ))
        )}
        {externalCount > 0 && (
          <Typography variant="caption" color="text.secondary" className="text-center pt-1 block">
            共 {instances.length} 个实例（其中 {externalCount} 个来自外部文件夹）
          </Typography>
        )}
      </Box>
    </Box>
  )
})
