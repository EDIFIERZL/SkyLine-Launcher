import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { ModInfo, ModUpdateInfo, InstallProgress } from '../types'
import { Box, Typography, Card, Button, IconButton, Input, Loading, EmptyState, AlertBox } from '../components/material'
import { useDownloadStore } from '../stores/downloadStore'
import { ArrowLeft, ToggleLeft, ToggleRight, Trash2, Puzzle, RefreshCw, Download, Upload, User, Tag } from 'lucide-react'

const MOD_LOADER_BADGES: Record<string, { label: string; color: string }> = {
  fabric: { label: 'Fabric', color: 'bg-amber-100 text-amber-700 dark:bg-amber-500/20 dark:text-amber-400' },
  forge: { label: 'Forge', color: 'bg-red-100 text-red-700 dark:bg-red-500/20 dark:text-red-400' },
  neoforge: { label: 'NeoForge', color: 'bg-purple-100 text-purple-700 dark:bg-purple-500/20 dark:text-purple-400' },
  quilt: { label: 'Quilt', color: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/20 dark:text-emerald-400' },
}

function ModIcon({ mod }: { mod: ModInfo }) {
  const [err, setErr] = useState(false)
  if (!mod.icon_url || err) {
    return (
      <Box className="w-8 h-8 rounded-lg bg-accent-50 dark:bg-accent-500/10 flex items-center justify-center shrink-0">
        <Puzzle className="w-4 h-4 text-[var(--accent-color)]" />
      </Box>
    )
  }
  return (
    <img
      src={mod.icon_url}
      alt={mod.name || mod.file_name}
      onError={() => setErr(true)}
      className="w-8 h-8 rounded-lg shrink-0 object-cover bg-surface-200 dark:bg-surface-700"
    />
  )
}

export function Mods() {
  const { instanceId } = useParams<{ instanceId: string }>()
  const navigate = useNavigate()
  const [mods, setMods] = useState<ModInfo[]>([])
  const [search, setSearch] = useState('')
  const [loading, setLoading] = useState(true)
  const [updates, setUpdates] = useState<ModUpdateInfo[]>([])
  const [checkingUpdates, setCheckingUpdates] = useState(false)
  const [selected, setSelected] = useState<Set<string>>(new Set())
  const [message, setMessage] = useState<string | null>(null)
  const [showImport, setShowImport] = useState(false)
  const [importPath, setImportPath] = useState('')
  const [importing, setImporting] = useState(false)

  const loadMods = () => {
    if (!instanceId) return
    setLoading(true)
    invoke<ModInfo[]>('scan_instance_mods', { instanceId })
      .then(setMods)
      .catch(console.error)
      .finally(() => setLoading(false))
  }

  useEffect(() => {
    loadMods()
  }, [instanceId])

  const handleToggle = async (mod: ModInfo) => {
    await invoke('toggle_mod', { path: mod.path, enable: !mod.enabled })
    loadMods()
  }

  const handleDelete = async (mod: ModInfo) => {
    await invoke('delete_mod', { path: mod.path })
    loadMods()
  }

  const handleCheckUpdates = async () => {
    if (!instanceId) return
    setCheckingUpdates(true)
    setMessage(null)
    try {
      const result = await invoke<ModUpdateInfo[]>('check_mod_updates', { instanceId })
      setUpdates(result)
      if (result.length === 0) setMessage('所有模组已是最新版本')
    } catch (e) {
      setMessage(`检查更新失败: ${e}`)
    }
    setCheckingUpdates(false)
  }

  const handleUpdateMod = async (update: ModUpdateInfo) => {
    const taskId = `mod-${Date.now()}`
    useDownloadStore.getState().addTask({
      id: taskId,
      title: `更新 ${update.filename}`,
      status: 'downloading',
      stage: 'mod',
      progress: 0,
      message: '开始下载...',
    })
    const unsub = await listen<InstallProgress>('install-progress', (e) => {
      useDownloadStore.getState().updateTask(taskId, {
        stage: e.payload.stage,
        progress: e.payload.progress,
        message: e.payload.message,
      })
    })
    try {
      await invoke('download_modrinth_mod', { versionId: update.version_id, instanceId })
      useDownloadStore.getState().markDone(taskId)
      setMessage(`已更新 ${update.filename}`)
      loadMods()
      setUpdates([])
    } catch (e) {
      useDownloadStore.getState().markError(taskId, String(e))
      setMessage(`更新失败: ${e}`)
    } finally {
      unsub()
    }
  }

  const toggleSelect = (path: string) => {
    setSelected((prev) => {
      const next = new Set(prev)
      if (next.has(path)) next.delete(path)
      else next.add(path)
      return next
    })
  }

  const toggleSelectAll = () => {
    setSelected((prev) => {
      if (mods.length > 0 && prev.size === mods.length) return new Set()
      return new Set(mods.map((m) => m.path))
    })
  }

  const handleBatchToggle = async (enabled: boolean) => {
    const paths = [...selected]
    if (paths.length === 0) return
    try {
      await invoke<string[]>('batch_toggle_mods', { paths, enabled })
    } catch (e) {
      setMessage(`批量操作失败: ${e}`)
    }
    setSelected(new Set())
    loadMods()
  }

  const handleBatchDelete = async () => {
    const paths = [...selected]
    if (paths.length === 0) return
    if (!confirm(`确定删除选中的 ${paths.length} 个模组吗？`)) return
    try {
      await invoke<string[]>('batch_delete_mods', { paths })
    } catch (e) {
      setMessage(`批量删除失败: ${e}`)
    }
    setSelected(new Set())
    loadMods()
  }

  const handleImport = async () => {
    if (!importPath) return
    setImporting(true)
    setMessage(null)
    try {
      const cmd = 'import_modrinth_pack'
      const id = await invoke<string>(cmd, { packPath: importPath })
      setMessage(`导入成功！实例 ID: ${id}`)
      setImportPath('')
      setShowImport(false)
    } catch (e) {
      setMessage(`导入失败: ${e}`)
    }
    setImporting(false)
  }

  const handleExport = async () => {
    if (!instanceId) return
    const path = prompt(`输入导出路径 (.mrpack):`, `./${instanceId}.mrpack`)
    if (!path) return
    try {
      const result = await invoke<string>('export_modrinth_pack', { instanceId, outputPath: path })
      setMessage(`导出成功: ${result}`)
    } catch (e) {
      setMessage(`导出失败: ${e}`)
    }
  }

  const filtered = mods.filter((m) =>
    !search || m.file_name.toLowerCase().includes(search.toLowerCase()) ||
    (m.name && m.name.toLowerCase().includes(search.toLowerCase())) ||
    (m.mod_id && m.mod_id.toLowerCase().includes(search.toLowerCase())) ||
    (m.author && m.author.toLowerCase().includes(search.toLowerCase()))
  )

  return (
    <Box className="space-y-4 max-w-5xl">
      <Box className="flex items-center justify-between">
        <Box className="flex items-center gap-3">
          <IconButton onClick={() => navigate(-1)}>
            <ArrowLeft className="w-5 h-5" />
          </IconButton>
          <Box>
            <Typography variant="h5">模组管理</Typography>
            <Typography variant="body2" color="text.secondary">{mods.length} 个模组已安装</Typography>
          </Box>
        </Box>
        <Box className="flex gap-2">
          <Button size="small" variant="outlined" startIcon={<Upload className="w-3.5 h-3.5" />} onClick={() => setShowImport(!showImport)}>
            导入
          </Button>
          <Button size="small" variant="outlined" startIcon={<Download className="w-3.5 h-3.5" />} onClick={handleExport}>
            导出
          </Button>
        </Box>
      </Box>

      {showImport && (
        <Card>
          <Box className="space-y-3">
            <Typography variant="subtitle2">导入模组包</Typography>
            <Box className="input-action-row">
              <Input
                value={importPath}
                onChange={(e) => setImportPath(e.target.value)}
                placeholder="输入 .mrpack 文件路径"
                className="flex-1"
              />
              <Button onClick={handleImport} loading={importing}>导入</Button>
            </Box>
          </Box>
        </Card>
      )}

      {message && (
        <AlertBox severity={message.includes('失败') ? 'error' : 'success'}>
          {message}
        </AlertBox>
      )}

      <Box className="input-action-row">
        <Box className="flex-1 max-w-xs">
          <Input
            placeholder="搜索模组..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </Box>
        <Button variant="outlined" startIcon={<RefreshCw className="w-3.5 h-3.5" />} onClick={handleCheckUpdates} loading={checkingUpdates}>
          检查更新
        </Button>
      </Box>

      {updates.length > 0 && (
        <Card className="bg-amber-50 dark:bg-amber-500/10 border-amber-200 dark:border-amber-500/30">
          <Box className="space-y-2">
            <Typography variant="subtitle2" className="flex items-center gap-2 text-amber-800 dark:text-amber-300">
              <RefreshCw className="w-4 h-4" /> {updates.length} 个模组可更新
            </Typography>
            {updates.map((u) => (
              <Box key={u.mod_path} className="flex items-center justify-between bg-white dark:bg-surface-800 rounded-lg px-3 py-2">
                <Typography variant="body2">
                  <span>{u.filename}</span>
                  <span className="text-surface-400 mx-2">→</span>
                  <span className="text-[var(--accent-color)] font-medium">{u.latest_version}</span>
                  <span className="text-surface-400 text-xs ml-2">(当前: {u.current_version})</span>
                </Typography>
                <Button size="small" startIcon={<Download className="w-3 h-3" />} onClick={() => handleUpdateMod(u)}>
                  更新
                </Button>
              </Box>
            ))}
          </Box>
        </Card>
      )}

      {selected.size > 0 && (
        <Box className="flex items-center gap-2 px-4 py-2.5 bg-accent-50 dark:bg-accent-500/10 border border-accent-200 dark:border-accent-500/30 rounded-lg">
          <Typography variant="subtitle2">已选择 {selected.size} 个模组</Typography>
          <Box className="flex-1" />
          <Button size="small" variant="outlined" startIcon={<ToggleRight className="w-3.5 h-3.5 text-accent-600" />} onClick={() => handleBatchToggle(true)}>
            启用
          </Button>
          <Button size="small" variant="outlined" startIcon={<ToggleLeft className="w-3.5 h-3.5 text-surface-400" />} onClick={() => handleBatchToggle(false)}>
            禁用
          </Button>
          <Button size="small" variant="outlined" startIcon={<Trash2 className="w-3.5 h-3.5 text-red-400" />} onClick={handleBatchDelete}>
            删除
          </Button>
        </Box>
      )}

      {loading ? (
        <Loading />
      ) : filtered.length === 0 ? (
        <EmptyState
          icon={<Puzzle className="w-12 h-12" />}
          title={search ? '没有匹配的模组' : '这个实例还没有安装模组'}
        />
      ) : (
        <Box className="space-y-2">
          <label className="flex items-center gap-3 px-4 py-2 text-xs text-surface-400 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={mods.length > 0 && selected.size === mods.length}
              onChange={toggleSelectAll}
              className="accent-accent-600 w-3.5 h-3.5"
            />
            全选
            {selected.size > 0 && <span className="text-[var(--accent-color)]">(已选 {selected.size})</span>}
          </label>
          {filtered.map((mod) => (
            <Card key={mod.path} className="px-4 py-3">
              <Box className="flex items-center justify-between">
                <Box className="flex items-center gap-3 min-w-0 flex-1">
                  <input
                    type="checkbox"
                    checked={selected.has(mod.path)}
                    onChange={() => toggleSelect(mod.path)}
                    className="accent-accent-600 w-3.5 h-3.5 shrink-0"
                  />
                  <ModIcon mod={mod} />
                  <Box className="min-w-0 flex-1">
                    <Box className="flex items-center gap-2">
                      <Typography variant="subtitle2" className="truncate">
                        {mod.name || mod.file_name}
                      </Typography>
                      {mod.mod_loader !== 'unknown' && MOD_LOADER_BADGES[mod.mod_loader] && (
                        <span className={`px-1.5 py-0.5 rounded text-[10px] font-medium ${MOD_LOADER_BADGES[mod.mod_loader].color}`}>
                          {MOD_LOADER_BADGES[mod.mod_loader].label}
                        </span>
                      )}
                      {!mod.enabled && (
                        <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-surface-200 dark:bg-surface-700 text-surface-500">
                          已禁用
                        </span>
                      )}
                    </Box>
                    <Box className="flex items-center gap-2 flex-wrap">
                      <Typography variant="caption" color="text.secondary">{mod.file_name}</Typography>
                      {mod.version && (
                        <Typography variant="caption" className="flex items-center gap-0.5 text-[var(--accent-color)]">
                          <Tag className="w-3 h-3" /> v{mod.version}
                        </Typography>
                      )}
                      {mod.author && (
                        <Typography variant="caption" color="text.secondary" className="flex items-center gap-0.5">
                          <User className="w-3 h-3" /> {mod.author}
                        </Typography>
                      )}
                      {mod.size_kb > 0 && <Typography variant="caption" color="text.secondary">{(mod.size_kb / 1024).toFixed(1)} MB</Typography>}
                    </Box>
                    {mod.description && (
                      <Typography variant="caption" color="text.secondary" className="line-clamp-2 max-w-lg block mt-0.5">
                        {mod.description}
                      </Typography>
                    )}
                    {mod.mod_id && (
                      <Typography variant="caption" color="text.secondary" className="mt-0.5 opacity-60">
                        ID: {mod.mod_id}
                      </Typography>
                    )}
                  </Box>
                </Box>
                <Box className="flex items-center gap-2 shrink-0">
                  <IconButton title={mod.enabled ? '禁用' : '启用'} onClick={() => handleToggle(mod)}>
                    {mod.enabled
                      ? <ToggleRight className="w-4 h-4 text-[var(--accent-color)]" />
                      : <ToggleLeft className="w-4 h-4 text-surface-400" />
                    }
                  </IconButton>
                  <IconButton title="删除" onClick={() => handleDelete(mod)}>
                    <Trash2 className="w-4 h-4 text-red-400" />
                  </IconButton>
                </Box>
              </Box>
            </Card>
          ))}
        </Box>
      )}
    </Box>
  )
}
