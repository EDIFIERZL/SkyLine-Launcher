import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useInstanceStore, type InstanceDetailData } from '../stores/instanceStore'
import { watchResourceInstall, invalidateCatalog } from '../lib/catalog'
import { LoaderLogo } from '../components/LoaderLogo'
import { Box, Typography, Card, Chip, Button, SnackbarAlert, Progress } from '@/components/material'
import {
  RefreshCw,
  Puzzle,
  Image,
  Layers,
  Gamepad2,
  ChevronRight,
  Clock,
  FolderOpen,
  FolderPlus,
  Trash,
  CheckCircle2,
  Circle,
} from 'lucide-react'
import type { Instance, ModInfo } from '../types'
import { sortInstances } from '../lib/instanceSort'

interface PackInfo {
  file_name: string
  path: string
  size_kb: number
  enabled: boolean
  name: string | null
  description: string | null
  pack_format: number | null
}

function getLoaderInfo(loader: Instance['modloader']): { name: string; color: string } {
  if (!loader) return { name: 'Vanilla', color: 'text-green-500' }
  const key = typeof loader === 'string' ? loader : Object.keys(loader)[0]
  const ver = typeof loader === 'string' ? '' : Object.values(loader)[0] as string
  const color = {
    Forge: 'text-red-500',
    NeoForge: 'text-purple-500',
    Fabric: 'text-amber-500',
    Quilt: 'text-emerald-500',
    Vanilla: 'text-green-500',
    OptiFine: 'text-blue-500',
  }[key] || 'text-surface-400'
  return { name: ver ? `${key} ${ver}` : key, color }
}

interface InstanceDetail {
  mods: ModInfo[]
  resourcepacks: { file_name: string; name: string | null; enabled: boolean }[]
  shaders: { file_name: string; name: string | null; enabled: boolean }[]
}

function formatPlayTime(s: number): string {
  if (!s) return ''
  const h = Math.floor(s / 3600)
  const m = Math.floor((s % 3600) / 60)
  return h > 0 ? `${h}h ${m}m` : `${m}m`
}

export function Library() {
  const navigate = useNavigate()
  const { instances, setInstances } = useInstanceStore()
  const [details, setDetails] = useState<Record<string, InstanceDetail>>({})
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [snack, setSnack] = useState({ open: false, message: '', severity: 'info' as 'success' | 'error' | 'info' })

  const [folders, setFolders] = useState<string[]>([])
  const [activeFolder, setActiveFolder] = useState<string | null>(null)
  const [foldersLoading, setFoldersLoading] = useState(false)

  const loadFolders = async () => {
    const st = useInstanceStore.getState()
    if (st.foldersLoaded) {
      setFolders(st.folders)
      setActiveFolder(st.activeFolder)
      return
    }
    try {
      const [folderList, active] = await Promise.all([
        invoke<string[]>('list_instance_folders').catch(() => [] as string[]),
        invoke<string | null>('get_active_instance_folder').catch(() => null),
      ])
      setFolders(folderList)
      useInstanceStore.getState().setFolders(folderList)
      if (!active && folderList.length === 1) {
        try {
          await invoke<string | null>('set_active_instance_folder', { folder: folderList[0] })
          setActiveFolder(folderList[0])
          useInstanceStore.getState().setActiveFolder(folderList[0])
        } catch { setActiveFolder(null) }
      } else {
        setActiveFolder(active ?? null)
        useInstanceStore.getState().setActiveFolder(active ?? null)
      }
      useInstanceStore.getState().setFoldersLoaded(true)
    } catch {  }
  }

  
  const scanMissingDetails = async (list: Instance[]) => {
    const { details, setInstanceDetails } = useInstanceStore.getState()
    const missing = list.filter(i => !details[i.id])
    if (missing.length === 0) return
    const results = await Promise.all(missing.map(async (inst) => {
      const [mods, resources, shaders] = await Promise.all([
        invoke<ModInfo[]>('scan_instance_mods', { instanceId: inst.id, includeIcons: false }).catch(() => [] as ModInfo[]),
        invoke<PackInfo[]>('scan_resource_packs', { instanceId: inst.id }).catch(() => [] as PackInfo[]),
        invoke<PackInfo[]>('scan_shader_packs', { instanceId: inst.id }).catch(() => [] as PackInfo[]),
      ])
      const data: InstanceDetailData = {
        mods: mods.filter(m => m.enabled),
        resourcepacks: resources.filter(p => p.enabled),
        shaders: shaders.filter(p => p.enabled),
      }
      setInstanceDetails(inst.id, data)
      return { id: inst.id, data }
    }))
    setDetails(prev => {
      const next = { ...prev }
      for (const { id, data } of results) next[id] = data
      return next
    })
  }

  const load = async (quiet = false) => {
    setRefreshing(true)
    if (!quiet) setLoading(true)
    try {
      const list = await invoke<Instance[]>('list_instances')
      setInstances(list)
      useInstanceStore.getState().setLoaded(true)
      await scanMissingDetails(list)
    } catch (e) {
      setSnack({ open: true, message: `加载实例失败: ${e}`, severity: 'error' })
    } finally {
      setLoading(false)
      setRefreshing(false)
    }
  }

  const addFolder = async () => {
    try {
      const picked = await open({ directory: true, multiple: false })
      if (typeof picked === 'string' && picked) {
        setFoldersLoading(true)
        const list = await invoke<string[]>('add_instance_folder', { folder: picked })
        setFolders(list)
        useInstanceStore.getState().setFolders(list)
        if (!activeFolder) {
          await invoke<string | null>('set_active_instance_folder', { folder: picked })
          setActiveFolder(picked)
          useInstanceStore.getState().setActiveFolder(picked)
        }
        invalidateCatalog()
        await load(true)
        setFoldersLoading(false)
      }
    } catch (e) {
      setFoldersLoading(false)
      setSnack({ open: true, message: `添加文件夹失败: ${e}`, severity: 'error' })
    }
  }

  const removeFolder = async (folder: string) => {
    try {
      setFoldersLoading(true)
      const list = await invoke<string[]>('remove_instance_folder', { folder })
      setFolders(list)
      useInstanceStore.getState().setFolders(list)
      if (activeFolder === folder) {
        const active = await invoke<string | null>('get_active_instance_folder').catch(() => null)
        setActiveFolder(active ?? null)
        useInstanceStore.getState().setActiveFolder(active ?? null)
      }
      invalidateCatalog()
      await load(true)
      setFoldersLoading(false)
    } catch (e) {
      setFoldersLoading(false)
      setSnack({ open: true, message: `移除文件夹失败: ${e}`, severity: 'error' })
    }
  }

  const setActive = async (folder: string) => {
    try {
      await invoke<string | null>('set_active_instance_folder', { folder })
      setActiveFolder(folder)
      useInstanceStore.getState().setActiveFolder(folder)
      invalidateCatalog()
      setSnack({ open: true, message: '已设置为首页显示文件夹', severity: 'success' })
    } catch (e) {
      setSnack({ open: true, message: `设置失败: ${e}`, severity: 'error' })
    }
  }

  const clearActive = async () => {
    try {
      await invoke<string | null>('set_active_instance_folder', { folder: null })
      setActiveFolder(null)
      useInstanceStore.getState().setActiveFolder(null)
      invalidateCatalog()
      setSnack({ open: true, message: '已取消激活，首页将显示所有实例', severity: 'success' })
    } catch (e) {
      setSnack({ open: true, message: `取消失败: ${e}`, severity: 'error' })
    }
  }

  useEffect(() => {
    loadFolders()
    const st = useInstanceStore.getState()
    
    if (st.loaded && st.instances.length > 0) {
      setDetails(st.details)
      void scanMissingDetails(st.instances)
    } else {
      load()
    }
    let unsubInstall: (() => void) | null = null
    watchResourceInstall(() => {
      invalidateCatalog()
      load(true)
    }).then((f) => { unsubInstall = f }).catch(() => {})
    return () => { unsubInstall?.() }
    
  }, [])

  const sorted = sortInstances(instances)

  return (
    <Box className="h-full flex flex-col p-6 overflow-hidden">
      <Box className="flex items-center justify-between mb-4 shrink-0">
        <Box>
          <Typography variant="h6" className="font-bold">游戏库</Typography>
          <Typography variant="body2" color="text.secondary" className="text-sm">
            管理实例文件夹，选择一个激活后首页将只显示该文件夹的实例
          </Typography>
        </Box>
        <Button
          size="small"
          variant="outlined"
          startIcon={<RefreshCw className={`w-3.5 h-3.5 ${refreshing ? 'animate-spin' : ''}`} />}
          onClick={() => { load(true); loadFolders() }}
          disabled={refreshing}
        >
          刷新
        </Button>
      </Box>

      <Box className="flex-1 overflow-y-auto space-y-4 pb-2">
        <Card className="!p-4">
          <Box className="flex items-center justify-between mb-3">
            <Typography variant="subtitle1" className="flex items-center gap-2 font-semibold">
              <FolderOpen className="w-4 h-4 text-[var(--accent-color)]" /> 实例文件夹
            </Typography>
            <Button size="small" variant="outlined" startIcon={<FolderPlus className="w-3.5 h-3.5" />} onClick={addFolder} loading={foldersLoading}>
              添加文件夹
            </Button>
          </Box>
          <Typography variant="body2" color="text.secondary" className="text-xs mb-3">
            添加 PCL2 / HMCL 等启动器的实例文件夹，点击「激活」后该文件夹的实例将显示在首页。
          </Typography>

          {folders.length === 0 ? (
            <Box className="text-center py-4">
              <Typography variant="body2" color="text.secondary" className="text-sm">尚未添加任何文件夹，点击上方按钮添加</Typography>
            </Box>
          ) : (
            <Box className="space-y-2">
              {folders.map((folder) => {
                const isActive = activeFolder === folder
                return (
                  <Box
                    key={folder}
                    className={`flex items-center justify-between px-3 py-2.5 rounded-lg border transition-colors ${
                      isActive
                        ? 'bg-accent-50 dark:bg-accent-500/10 border-accent-200 dark:border-accent-500/30'
                        : 'bg-surface-50 dark:bg-surface-800 border-surface-200 dark:border-surface-700'
                    }`}
                  >
                    <Box className="flex items-center gap-2 min-w-0 flex-1">
                      <Box className={`w-5 h-5 rounded-full flex items-center justify-center shrink-0 ${isActive ? 'bg-[var(--accent-color)]' : 'bg-surface-200 dark:bg-surface-700'}`}>
                        {isActive
                          ? <CheckCircle2 className="w-3 h-3 text-white" />
                          : <Circle className="w-3 h-3 text-surface-400" />}
                      </Box>
                      <Typography variant="body2" className="truncate text-sm" title={folder}>
                        {folder.split(/[\\/]/).pop() ?? folder}
                      </Typography>
                      <Typography variant="caption" color="text.secondary" className="truncate text-xs hidden sm:inline" title={folder}>
                        {folder}
                      </Typography>
                    </Box>
                    <Box className="flex items-center gap-2 shrink-0 ml-3">
                      {isActive ? (
                        <Button size="small" variant="text" onClick={clearActive} className="!text-xs">
                          取消激活
                        </Button>
                      ) : (
                        <Button size="small" variant="outlined" onClick={() => setActive(folder)} className="!text-xs">
                          激活
                        </Button>
                      )}
                      <Button size="small" variant="text" onClick={() => removeFolder(folder)} className="!text-xs">
                        <Trash className="w-3.5 h-3.5 text-red-400" />
                      </Button>
                    </Box>
                  </Box>
                )
              })}
            </Box>
          )}

          {activeFolder && (
            <Typography variant="caption" color="text.secondary" className="mt-2 block text-xs">
              当前激活的首页文件夹: <span className="font-medium text-[var(--accent-color)]">{activeFolder.split(/[\\/]/).pop()}</span>
            </Typography>
          )}
        </Card>

        {loading ? (
          <Box className="flex items-center justify-center py-8">
            <Progress />
          </Box>
        ) : sorted.length === 0 ? (
          <Box className="flex flex-col items-center justify-center py-12">
            <Gamepad2 className="w-16 h-16 mb-3 text-surface-300 dark:text-surface-600" />
            <Typography variant="h6" className="mb-1">你的游戏库还是空的</Typography>
            <Typography variant="body2" color="text.secondary" className="mb-5">去「资源」页下载一个游戏版本吧</Typography>
            <Button variant="contained" onClick={() => navigate('/download')}>下载游戏</Button>
          </Box>
        ) : (
          sorted.map(inst => {
            const info = getLoaderInfo(inst.modloader)
            const d = details[inst.id]
            const modCount = d?.mods.length ?? 0
            const rpCount = d?.resourcepacks.length ?? 0
            const shCount = d?.shaders.length ?? 0
            return (
              <Card key={inst.id} className="!p-4">
                <Box className="flex items-center gap-4">
                  <Box className="w-12 h-12 rounded-xl bg-accent-50 dark:bg-accent-500/10 flex items-center justify-center shrink-0">
                    <LoaderLogo loader={inst.modloader} versionId={inst.version_id} className="w-8 h-8" />
                  </Box>
                  <Box className="flex-1 min-w-0">
                    <Box className="flex items-center gap-2">
                      <Typography variant="subtitle1" className="font-bold truncate">{inst.name}</Typography>
                      {inst.external && <Chip label="外部" size="small" color="warning" variant="outlined" />}
                    </Box>
                    <Box className="flex items-center gap-2 mt-1">
                      <Chip label={inst.version_id} size="small" variant="outlined" />
                      <span className={`text-xs font-medium ${info.color}`}>{info.name}</span>
                      {inst.play_time > 0 && (
                        <span className="flex items-center gap-1 text-xs text-surface-400">
                          <Clock className="w-3 h-3" />
                          {formatPlayTime(inst.play_time)}
                        </span>
                      )}
                    </Box>
                  </Box>
                  <Box className="flex items-center gap-2 shrink-0">
                    <Button size="small" variant="text" endIcon={<ChevronRight className="w-3 h-3" />} onClick={() => navigate(`/instances/${inst.id}/mods`)}>模组 ({modCount})</Button>
                    <Button size="small" variant="text" endIcon={<ChevronRight className="w-3 h-3" />} onClick={() => navigate(`/instances/${inst.id}/resourcepacks`)}>资源包 ({rpCount})</Button>
                    <Button size="small" variant="text" startIcon={<FolderOpen className="w-3.5 h-3.5" />} onClick={() => invoke('open_instance_folder', { instanceId: inst.id })}>打开文件夹</Button>
                  </Box>
                </Box>

                <Box className="mt-3 pt-3 border-t border-surface-200/60 dark:border-surface-700/30 grid grid-cols-3 gap-3">
                  <Box>
                    <Box className="flex items-center gap-1 mb-1">
                      <Puzzle className="w-3 h-3 text-[var(--accent-color)]" />
                      <Typography variant="caption" className="font-semibold text-[11px]">模组 {modCount}</Typography>
                    </Box>
                    <Box className="flex flex-wrap gap-1.5">
                      {d?.mods.slice(0, 6).map(m => (
                        <Chip key={m.path} label={m.name || m.file_name} size="small" variant="outlined" className="!h-5 !text-[10px]" />
                      ))}
                      {d && modCount > 6 && <Chip label={`+${modCount - 6}`} size="small" variant="outlined" className="!h-5 !text-[10px]" />}
                      {d && modCount === 0 && <Typography variant="caption" color="text.secondary" className="text-[10px]">暂无模组</Typography>}
                    </Box>
                  </Box>
                  <Box>
                    <Box className="flex items-center gap-1 mb-1">
                      <Image className="w-3 h-3 text-emerald-500" />
                      <Typography variant="caption" className="font-semibold text-[11px]">资源包 {rpCount}</Typography>
                    </Box>
                    <Box className="flex flex-wrap gap-1.5">
                      {d?.resourcepacks.slice(0, 4).map(p => (
                        <Chip key={p.file_name} label={p.name || p.file_name} size="small" variant="outlined" className="!h-5 !text-[10px]" />
                      ))}
                      {d && rpCount > 4 && <Chip label={`+${rpCount - 4}`} size="small" variant="outlined" className="!h-5 !text-[10px]" />}
                      {d && rpCount === 0 && <Typography variant="caption" color="text.secondary" className="text-[10px]">暂无资源包</Typography>}
                    </Box>
                  </Box>
                  <Box>
                    <Box className="flex items-center gap-1 mb-1">
                      <Layers className="w-3 h-3 text-violet-500" />
                      <Typography variant="caption" className="font-semibold text-[11px]">光影 {shCount}</Typography>
                    </Box>
                    <Box className="flex flex-wrap gap-1.5">
                      {d?.shaders.slice(0, 4).map(p => (
                        <Chip key={p.file_name} label={p.name || p.file_name} size="small" variant="outlined" className="!h-5 !text-[10px]" />
                      ))}
                      {d && shCount > 4 && <Chip label={`+${shCount - 4}`} size="small" variant="outlined" className="!h-5 !text-[10px]" />}
                      {d && shCount === 0 && <Typography variant="caption" color="text.secondary" className="text-[10px]">暂无光影</Typography>}
                    </Box>
                  </Box>
                </Box>
              </Card>
            )
          })
        )}
      </Box>

      <SnackbarAlert open={snack.open} onClose={() => setSnack({ ...snack, open: false })} message={snack.message} severity={snack.severity} />
    </Box>
  )
}