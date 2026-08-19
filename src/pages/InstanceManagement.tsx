import { useEffect, useMemo, useState } from 'react'
import { useParams, useNavigate, useSearchParams } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { Box, Typography, Card, Button, IconButton, Input, Loading, EmptyState, AlertBox, Tabs, Chip } from '../components/material'
import { ArrowLeft, Puzzle, Image, Palette, FolderOpen, Map, FileText, Settings, RefreshCw, ToggleRight, ToggleLeft, Trash2, Download, Eye, EyeOff, PackageOpen, Skull, Clock, Hash, Gamepad2 } from 'lucide-react'
import type { ModInfo, Instance } from '../types'

interface PackInfo {
  file_name: string
  path: string
  size_kb: number
  enabled: boolean
  name: string | null
  description: string | null
  pack_format: number | null
  icon_url: string | null
}

interface SchematicInfo {
  file_name: string
  path: string
  size_kb: number
  enabled: boolean
}

type TabValue = 'settings' | 'mods' | 'resourcepacks' | 'shaderpacks' | 'datapacks' | 'worlds' | 'schematics'

const TABS: { value: TabValue; label: string; icon: React.ReactNode }[] = [
  { value: 'settings', label: '实例设置', icon: <Settings className="w-4 h-4" /> },
  { value: 'mods', label: '模组管理', icon: <Puzzle className="w-4 h-4" /> },
  { value: 'resourcepacks', label: '资源包管理', icon: <Image className="w-4 h-4" /> },
  { value: 'shaderpacks', label: '光影包管理', icon: <Palette className="w-4 h-4" /> },
  { value: 'datapacks', label: '数据包管理', icon: <PackageOpen className="w-4 h-4" /> },
  { value: 'worlds', label: '世界管理', icon: <Map className="w-4 h-4" /> },
  { value: 'schematics', label: '原理图管理', icon: <FileText className="w-4 h-4" /> },
]

export function InstanceManagement() {
  const { instanceId } = useParams<{ instanceId: string }>()
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const [activeTab, setActiveTab] = useState<TabValue>(() => {
    const t = searchParams.get('type') as TabValue | null
    return t && TABS.some((x) => x.value === t) ? t : 'mods'
  })
  const [instance, setInstance] = useState<Instance | null>(null)
  const [mods, setMods] = useState<ModInfo[]>([])
  const [resourcePacks, setResourcePacks] = useState<PackInfo[]>([])
  const [shaderPacks, setShaderPacks] = useState<PackInfo[]>([])
  const [dataPacks, setDataPacks] = useState<PackInfo[]>([])
  const [worlds, setWorlds] = useState<{ name: string; path: string; game_mode: string; seed: number | null; size_kb: number; icon: string | null; is_hardcore: boolean; difficulty: string | null; play_time: number; spawn_x: number | null; spawn_z: number | null }[]>([])
  const [schematics, setSchematics] = useState<SchematicInfo[]>([])
  const [message, setMessage] = useState<string | null>(null)
  const [worldSearch, setWorldSearch] = useState('')
  const [loadingTab, setLoadingTab] = useState<Record<string, boolean>>({})

  const setTabLoading = (tab: string, v: boolean) => setLoadingTab((prev) => ({ ...prev, [tab]: v }))

  const loadInstance = () => {
    if (!instanceId) return
    invoke<Instance>('get_instance', { id: instanceId })
      .then(setInstance)
      .catch(console.error)
  }

  const refreshAll = () => {
    loadInstance()
    loadMods()
    loadResourcePacks()
    loadShaderPacks()
    loadDataPacks()
    loadWorlds()
    loadSchematics()
  }

  const loadMods = () => {
    if (!instanceId) return
    setTabLoading('mods', true)
    invoke<ModInfo[]>('scan_instance_mods', { instanceId })
      .then(setMods)
      .catch(console.error)
      .finally(() => setTabLoading('mods', false))
  }

  const loadResourcePacks = () => {
    if (!instanceId) return
    setTabLoading('resourcepacks', true)
    invoke<PackInfo[]>('scan_resource_packs', { instanceId })
      .then(setResourcePacks)
      .catch(console.error)
      .finally(() => setTabLoading('resourcepacks', false))
  }

  const loadShaderPacks = () => {
    if (!instanceId) return
    setTabLoading('shaderpacks', true)
    invoke<PackInfo[]>('scan_shader_packs', { instanceId })
      .then(setShaderPacks)
      .catch(console.error)
      .finally(() => setTabLoading('shaderpacks', false))
  }

  const loadDataPacks = () => {
    if (!instanceId) return
    setTabLoading('datapacks', true)
    invoke<PackInfo[]>('scan_data_packs', { instanceId })
      .then(setDataPacks)
      .catch(console.error)
      .finally(() => setTabLoading('datapacks', false))
  }

  const loadWorlds = () => {
    if (!instanceId) return
    setTabLoading('worlds', true)
    invoke<any[]>('list_instance_worlds', { instanceId })
      .then((list) => setWorlds(list.map((w: any) => ({
        name: w.name,
        path: w.path,
        game_mode: w.game_mode,
        seed: w.seed ?? null,
        size_kb: w.size_kb,
        icon: w.icon,
        is_hardcore: w.is_hardcore,
        difficulty: w.difficulty ?? null,
        play_time: w.play_time ?? 0,
        spawn_x: w.spawn_x ?? null,
        spawn_z: w.spawn_z ?? null,
      }))))
      .catch(console.error)
      .finally(() => setTabLoading('worlds', false))
  }

  const loadSchematics = () => {
    if (!instanceId) return
    setTabLoading('schematics', true)
    invoke<SchematicInfo[]>('scan_schematics', { instanceId })
      .then(setSchematics)
      .catch(console.error)
      .finally(() => setTabLoading('schematics', false))
  }

  const filteredWorlds = useMemo(() => {
    const query = worldSearch.trim().toLowerCase()
    if (!query) return worlds
    return worlds.filter((world) => [world.name, world.game_mode, world.seed == null ? '' : String(world.seed)]
      .some((value) => value.toLowerCase().includes(query)))
  }, [worldSearch, worlds])

  useEffect(() => {
    refreshAll()
  }, [instanceId])

  const handleToggle = async (mod: ModInfo) => {
    await invoke('toggle_mod', { path: mod.path, enable: !mod.enabled })
    loadMods()
  }

  const handleDelete = async (mod: ModInfo) => {
    if (!confirm(`确定删除模组「${mod.name || mod.file_name}」吗？`)) return
    await invoke('delete_mod', { path: mod.path })
    loadMods()
  }

  const handleTogglePack = async (pack: PackInfo, enable: boolean, kind: 'resourcepacks' | 'shaderpacks' | 'datapacks') => {
    try {
      await invoke(kind === 'datapacks' ? 'toggle_data_pack' : 'toggle_resource_pack', { path: pack.path, enable })
      if (kind === 'datapacks') loadDataPacks()
      else if (kind === 'shaderpacks') loadShaderPacks()
      else loadResourcePacks()
    } catch (e) {
      setMessage(`操作失败: ${e}`)
    }
  }

  const handleDeletePack = async (pack: PackInfo, kind: 'resourcepacks' | 'shaderpacks' | 'datapacks') => {
    if (!confirm(`确定删除「${pack.name || pack.file_name}」吗？`)) return
    try {
      await invoke(kind === 'datapacks' ? 'delete_data_pack' : 'delete_mod', { path: pack.path })
      if (kind === 'datapacks') loadDataPacks()
      else if (kind === 'shaderpacks') loadShaderPacks()
      else loadResourcePacks()
    } catch (e) {
      setMessage(`删除失败: ${e}`)
    }
  }

  const handleOpenFolder = async (subdir: string) => {
    if (!instanceId) return
    await invoke('open_instance_folder', { instanceId, subdir })
  }

  const handleImportWorld = async () => {
    if (!instanceId) return
    const picked = await open({ filters: [{ name: 'World ZIP', extensions: ['zip'] }] })
    if (typeof picked === 'string' && picked) {
      try {
        const name = await invoke<string>('import_world_zip', { instanceId, zipPath: picked })
        setMessage(`已添加世界: ${name}`)
        loadWorlds()
      } catch (e) {
        setMessage(`导入失败: ${e}`)
      }
    }
  }

  const handleRename = async (_id: string, field: keyof Instance, value: any) => {
    if (!instance) return
    const updated = { ...instance, [field]: value }
    try {
      await invoke('update_instance', { instance: updated })
      setInstance(updated)
    } catch (e) {
      setMessage(`保存失败: ${e}`)
    }
  }

  const msg = message

  const renderModsTab = () => {
    if (loadingTab['mods']) return <Loading />
    return (
      <>
        <Box className="flex items-center gap-3 mb-4 flex-wrap">
          <Button size="small" variant="outlined" startIcon={<FolderOpen className="w-3.5 h-3.5" />} onClick={() => handleOpenFolder('mods')}>
            打开模组文件夹
          </Button>
          <Box className="flex-1" />
          <Typography variant="caption" color="text.secondary">{mods.length} 个模组</Typography>
        </Box>
        {mods.length === 0 ? (
          <EmptyState icon={<Puzzle className="w-12 h-12" />} title="该实例暂无模组" description="将 .jar 文件放入 mods/ 文件夹即可" />
        ) : (
          <Box className="space-y-2">
            {mods.map((mod) => (
              <Card key={mod.path} className="px-4 py-3">
                <Box className="flex items-center justify-between">
                  <Box className="flex items-center gap-3 min-w-0 flex-1">
                    <Box className="w-8 h-8 rounded-lg bg-accent-50 dark:bg-accent-500/10 flex items-center justify-center shrink-0 overflow-hidden">
                      {mod.icon_url ? (
                        <img src={mod.icon_url} alt={mod.name || mod.file_name} onError={(e) => { (e.target as HTMLImageElement).style.display = 'none' }} className="w-full h-full object-cover" />
                      ) : (
                        <Puzzle className="w-4 h-4 text-[var(--accent-color)]" />
                      )}
                    </Box>
                    <Box className="min-w-0 flex-1">
                      <Box className="flex items-center gap-2">
                        <Typography variant="subtitle2" className="truncate">{mod.name || mod.file_name}</Typography>
                        {!mod.enabled && <Chip label="已禁用" size="small" color="warning" variant="outlined" />}
                      </Box>
                      <Box className="flex items-center gap-2 mt-0.5">
                        <Typography variant="caption" color="text.secondary">{mod.file_name}</Typography>
                        {mod.version && <Typography variant="caption" className="text-[var(--accent-color)]">v{mod.version}</Typography>}
                        {mod.author && <Typography variant="caption" color="text.secondary">by {mod.author}</Typography>}
                      </Box>
                      {mod.description && (
                        <Typography variant="caption" color="text.secondary" className="line-clamp-1 block mt-0.5">{mod.description}</Typography>
                      )}
                    </Box>
                  </Box>
                  <Box className="flex items-center gap-1 shrink-0 ml-3">
                    <IconButton title={mod.enabled ? '禁用' : '启用'} onClick={() => handleToggle(mod)}>
                      {mod.enabled ? <ToggleRight className="w-4 h-4 text-[var(--accent-color)]" /> : <ToggleLeft className="w-4 h-4 text-surface-400" />}
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
      </>
    )
  }

  const renderPackTab = (packs: PackInfo[], kind: 'resourcepacks' | 'shaderpacks' | 'datapacks') => {
    const label = kind === 'datapacks' ? '数据包' : kind === 'shaderpacks' ? '光影包' : '资源包'
    if (loadingTab[kind]) return <Loading />
    return (
      <>
        <Box className="flex items-center gap-2 mb-4">
          <Button size="small" variant="outlined" startIcon={<FolderOpen className="w-3.5 h-3.5" />} onClick={() => handleOpenFolder(kind)}>
            打开{label}文件夹
          </Button>
          <Box className="flex-1" />
          <Typography variant="caption" color="text.secondary">{packs.length} 个{label}</Typography>
        </Box>
        {packs.length === 0 ? (
          <EmptyState
            icon={<PackageOpen className="w-12 h-12" />}
            title={`暂无${label}`}
            description={`将 ${label} 放入 .minecraft/${kind} 目录`}
          />
        ) : (
          <Box className="space-y-2">
            {packs.map((pack) => (
              <Card key={pack.path} className="px-4 py-3">
                <Box className="flex items-center justify-between">
                  <Box className="flex items-center gap-3 min-w-0 flex-1">
                    <Box className={`w-8 h-8 rounded-lg flex items-center justify-center shrink-0 overflow-hidden ${pack.enabled ? 'bg-accent-50 dark:bg-accent-500/10' : 'bg-surface-100 dark:bg-surface-800'}`}>
                      {pack.icon_url ? (
                        <img src={pack.icon_url} alt={pack.name || pack.file_name} onError={(e) => { (e.target as HTMLImageElement).style.display = 'none' }} className="w-full h-full object-cover" />
                      ) : kind === 'datapacks' ? <PackageOpen className={`w-4 h-4 ${pack.enabled ? 'text-[var(--accent-color)]' : 'text-surface-400'}`} /> : <Image className={`w-4 h-4 ${pack.enabled ? 'text-emerald-500' : 'text-surface-400'}`} />}
                    </Box>
                    <Box className="min-w-0 flex-1">
                      <Box className="flex items-center gap-2">
                        <Typography variant="subtitle2" className={pack.enabled ? '' : 'opacity-50'}>{pack.name || pack.file_name}</Typography>
                        {!pack.enabled && <Chip label="已禁用" size="small" color="warning" variant="outlined" />}
                      </Box>
                      <Typography variant="caption" color="text.secondary">{pack.file_name} · {(pack.size_kb / 1024).toFixed(1)} MB</Typography>
                    </Box>
                  </Box>
                  <Box className="flex items-center gap-1 shrink-0 ml-3">
                    <IconButton title={pack.enabled ? '禁用' : '启用'} onClick={() => handleTogglePack(pack, !pack.enabled, kind)}>
                      {pack.enabled ? <Eye className="w-4 h-4 text-[var(--accent-color)]" /> : <EyeOff className="w-4 h-4 text-surface-400" />}
                    </IconButton>
                    <IconButton title="删除" onClick={() => handleDeletePack(pack, kind)}>
                      <Trash2 className="w-4 h-4 text-red-400" />
                    </IconButton>
                  </Box>
                </Box>
              </Card>
            ))}
          </Box>
        )}
      </>
    )
  }

  const GAME_MODE_CN: Record<string, string> = {
    survival: '生存', creative: '创造', adventure: '冒险', spectator: '旁观',
    '0': '生存', '1': '创造', '2': '冒险', '3': '旁观',
  }
  const formatPlayTime = (s: number) => {
    if (!s) return ''
    const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60)
    return h > 0 ? `${h}h ${m}m` : `${m}m`
  }

  const renderWorldsTab = () => {
    if (loadingTab['worlds']) return <Loading />
    return (
      <>
        <Box className="flex items-center gap-2 mb-4">
          <Button size="small" variant="outlined" startIcon={<FolderOpen className="w-3.5 h-3.5" />} onClick={() => handleOpenFolder('saves')}>
            打开存档文件夹
          </Button>
          <Button size="small" variant="outlined" startIcon={<Download className="w-3.5 h-3.5" />} onClick={handleImportWorld}>
            导入世界
          </Button>
          <Button size="small" variant="outlined" startIcon={<RefreshCw className="w-3.5 h-3.5" />} onClick={loadWorlds}>
            刷新
          </Button>
          <Box className="flex-1 min-w-4" />
          <Typography variant="caption" color="text.secondary">{worlds.length} 个世界</Typography>
        </Box>
        {worlds.length > 0 && (
          <Box className="mb-4 max-w-md">
            <Input
              placeholder="搜索世界名称、模式或种子..."
              value={worldSearch}
              onChange={(e) => setWorldSearch(e.target.value)}
            />
          </Box>
        )}
        {worlds.length === 0 ? (
          <EmptyState
            icon={<Map className="w-12 h-12" />}
            title="暂无世界"
            description="将 world.zip 拖入或点击上方按钮添加"
          />
        ) : filteredWorlds.length === 0 ? (
          <EmptyState icon={<Map className="w-10 h-10" />} title="没有匹配的世界" description="尝试修改搜索关键词" />
        ) : (
          <Box className="space-y-2">
            {filteredWorlds.map((w) => (
              <Card key={w.path} className="px-4 py-3 hover:shadow-md transition-shadow">
                <Box className="flex items-center gap-3">
                  {w.icon ? (
                    <img src={w.icon} alt="" className="w-10 h-10 rounded-lg object-cover shrink-0 border border-white/10" />
                  ) : (
                    <Box className="w-10 h-10 rounded-lg bg-cyan-50 dark:bg-cyan-500/10 flex items-center justify-center shrink-0">
                      <Map className="w-5 h-5 text-cyan-500" />
                    </Box>
                  )}
                  <Box className="flex-1 min-w-0">
                    <Box className="flex items-center gap-2">
                      <Typography variant="subtitle2" className="truncate">{w.name}</Typography>
                      {w.is_hardcore && (
                        <Chip icon={<Skull className="w-3 h-3" />} label="极限" size="small" color="error" variant="outlined" />
                      )}
                    </Box>
                    <Box className="flex items-center gap-3 mt-0.5 flex-wrap">
                      <span className="text-[11px] text-surface-500 flex items-center gap-1">
                        <Gamepad2 className="w-3 h-3" />
                        {GAME_MODE_CN[w.game_mode?.toLowerCase()] ?? w.game_mode ?? '未知'}
                      </span>
                      {w.seed != null && (
                        <span className="text-[11px] text-surface-500 flex items-center gap-1">
                          <Hash className="w-3 h-3" />
                          {w.seed}
                        </span>
                      )}
                      {w.play_time > 0 && (
                        <span className="text-[11px] text-surface-500 flex items-center gap-1">
                          <Clock className="w-3 h-3" />
                          {formatPlayTime(w.play_time)}
                        </span>
                      )}
                      <span className="text-[11px] text-surface-500">
                        {(w.size_kb / 1024).toFixed(1)} MB
                      </span>
                    </Box>
                  </Box>
                  <Box className="flex items-center gap-1 shrink-0">
                    <IconButton
                      title="查看地图"
                      onClick={() => navigate(`/worlds/${instanceId}?world=${encodeURIComponent(w.path)}`)}
                    >
                      <Map className="w-4 h-4 text-cyan-400" />
                    </IconButton>
                    <IconButton title="打开文件夹" onClick={() => handleOpenFolder('saves')}>
                      <FolderOpen className="w-4 h-4" />
                    </IconButton>
                    <IconButton
                      title="删除世界"
                      onClick={async () => {
                        if (!confirm(`确定删除世界「${w.name}」吗？`)) return
                        try {
                          await invoke('delete_world', { instanceId, worldName: w.name })
                          loadWorlds()
                        } catch (e) {
                          setMessage(`删除失败: ${e}`)
                        }
                      }}
                    >
                      <Trash2 className="w-4 h-4 text-red-400" />
                    </IconButton>
                  </Box>
                </Box>
              </Card>
            ))}
          </Box>
        )}
      </>
    )
  }

  const renderSchematicsTab = () => {
    if (loadingTab['schematics']) return <Loading />
    return (
      <>
        <Box className="flex items-center gap-2 mb-4">
          <Button size="small" variant="outlined" startIcon={<FolderOpen className="w-3.5 h-3.5" />} onClick={() => handleOpenFolder('schematics')}>
            打开原理图文件夹
          </Button>
          <Button size="small" variant="outlined" startIcon={<RefreshCw className="w-3.5 h-3.5" />} onClick={loadSchematics}>
            刷新
          </Button>
          <Box className="flex-1" />
          <Typography variant="caption" color="text.secondary">{schematics.length} 个原理图</Typography>
        </Box>
        {schematics.length === 0 ? (
          <EmptyState
            icon={<FileText className="w-12 h-12" />}
            title="暂无原理图"
            description="将 .litematic 文件放入 schematics/ 文件夹"
          />
        ) : (
          <Box className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
            {schematics.map((s) => (
              <Card
                key={s.path}
                className="!p-3 cursor-pointer hover:border-accent-500/40 transition-colors"
                onClick={() => invoke('open_file', { path: s.path }).catch(() => {})}
              >
                <Box className="w-full aspect-square rounded-lg bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center mb-2">
                  <FileText className="w-8 h-8 text-cyan-400" />
                </Box>
                <Typography variant="caption" className="block truncate font-medium text-[11px]" title={s.file_name}>{s.file_name}</Typography>
                <Typography variant="caption" color="text.secondary" className="text-[10px]">
                  {s.size_kb < 1024 ? `${s.size_kb} KB` : `${(s.size_kb / 1024).toFixed(1)} MB`}
                </Typography>
              </Card>
            ))}
          </Box>
        )}
      </>
    )
  }

  const renderSettingsTab = () => {
    if (!instance) return <Loading />
    return (
      <Box className="space-y-4 max-w-lg">
        <Card className="p-4 space-y-3">
          <Typography variant="subtitle2">实例信息</Typography>
          <Box className="space-y-2">
            <Box className="flex items-center gap-2">
              <Typography variant="body2" className="w-20 shrink-0">实例名称</Typography>
              <Input
                value={instance.name}
                onChange={(e) => handleRename(instanceId!, 'name', e.target.value)}
                placeholder="实例名称"
                fullWidth={false}
                className="flex-1"
              />
            </Box>
            <Box className="flex items-center gap-2">
              <Typography variant="body2" className="w-20 shrink-0">版本 ID</Typography>
              <Typography variant="body2" color="text.secondary" className="flex-1">{instance.version_id}</Typography>
            </Box>
            <Box className="flex items-center gap-2">
              <Typography variant="body2" className="w-20 shrink-0">加载器</Typography>
              <Typography variant="body2" color="text.secondary" className="flex-1">
                {typeof instance.modloader === 'object' && !('Vanilla' in instance.modloader)
                  ? Object.entries(instance.modloader)[0][0]
                  : 'Vanilla'}
              </Typography>
            </Box>
            <Box className="flex items-center gap-2">
              <Typography variant="body2" className="w-20 shrink-0">外部实例</Typography>
              <Typography variant="body2" color="text.secondary" className="flex-1">{instance.external ? '是' : '否'}</Typography>
            </Box>
            <Box className="flex items-center gap-2">
              <Typography variant="body2" className="w-20 shrink-0">创建时间</Typography>
              <Typography variant="body2" color="text.secondary" className="flex-1">{instance.created_at}</Typography>
            </Box>
          </Box>
        </Card>
        <Card className="p-4 space-y-3">
          <Typography variant="subtitle2">内存设置</Typography>
          <Box className="space-y-2">
            <Box className="flex items-center gap-2">
              <Typography variant="body2" className="w-20 shrink-0">最小内存</Typography>
              <Input
                type="number"
                value={String(instance.min_memory)}
                onChange={(e) => handleRename(instanceId!, 'min_memory', Number(e.target.value))}
                fullWidth={false}
                className="w-24"
              />
              <Typography variant="caption" color="text.secondary">MB</Typography>
            </Box>
            <Box className="flex items-center gap-2">
              <Typography variant="body2" className="w-20 shrink-0">最大内存</Typography>
              <Input
                type="number"
                value={String(instance.max_memory)}
                onChange={(e) => handleRename(instanceId!, 'max_memory', Number(e.target.value))}
                fullWidth={false}
                className="w-24"
              />
              <Typography variant="caption" color="text.secondary">MB</Typography>
            </Box>
          </Box>
        </Card>
        <Card className="p-4 space-y-3">
          <Typography variant="subtitle2">窗口设置</Typography>
          <Box className="space-y-2">
            <Box className="flex items-center gap-2">
              <Typography variant="body2" className="w-20 shrink-0">宽度</Typography>
              <Input
                type="number"
                value={String(instance.window_width)}
                onChange={(e) => handleRename(instanceId!, 'window_width', Number(e.target.value))}
                fullWidth={false}
                className="w-24"
              />
            </Box>
            <Box className="flex items-center gap-2">
              <Typography variant="body2" className="w-20 shrink-0">高度</Typography>
              <Input
                type="number"
                value={String(instance.window_height)}
                onChange={(e) => handleRename(instanceId!, 'window_height', Number(e.target.value))}
                fullWidth={false}
                className="w-24"
              />
            </Box>
          </Box>
        </Card>
        <Card className="p-4 space-y-3">
          <Typography variant="subtitle2">服务器</Typography>
          <Box className="space-y-2">
            <Box className="flex items-center gap-2">
              <Typography variant="body2" className="w-20 shrink-0">服务器地址</Typography>
              <Input
                value={instance.server_ip ?? ''}
                onChange={(e) => handleRename(instanceId!, 'server_ip', e.target.value)}
                placeholder="如: mc.example.com"
                fullWidth={false}
                className="flex-1"
              />
            </Box>
          </Box>
        </Card>
        <Card className="p-4 space-y-3">
          <Typography variant="subtitle2">JVM 参数</Typography>
          <Box className="space-y-2">
            <Box className="flex items-center gap-2">
              <Typography variant="body2" className="w-20 shrink-0">JVM 参数</Typography>
              <Input
                value={instance.jvm_args.join(' ')}
                onChange={(e) => {
                  const newArgs = e.target.value.split(' ').filter(Boolean)
                  handleRename(instanceId!, 'jvm_args', newArgs)
                }}
                placeholder="-Xmx4G -XX:+UseG1GC"
                fullWidth={false}
                className="flex-1"
              />
            </Box>
          </Box>
        </Card>
        <Box className="flex gap-2">
          <Button size="small" variant="contained" onClick={refreshAll}>
            <RefreshCw className="w-3.5 h-3.5 mr-1" /> 刷新所有
          </Button>
          <Button size="small" variant="outlined" onClick={() => handleOpenFolder('')}>
            <FolderOpen className="w-3.5 h-3.5 mr-1" /> 打开实例文件夹
          </Button>
        </Box>
      </Box>
    )
  }

  return (
    <Box className="space-y-4 max-w-5xl pt-1">
      <Box className="flex items-center gap-3">
        <IconButton onClick={() => navigate(-1)}>
          <ArrowLeft className="w-5 h-5" />
        </IconButton>
        <Box>
          <Typography variant="h5">实例管理</Typography>
          <Typography variant="body2" color="text.secondary">{instance?.name ?? instanceId} · {instance?.version_id ?? ''}</Typography>
        </Box>
      </Box>

      {msg && <AlertBox severity={msg.includes('失败') ? 'error' : 'success'}>{msg}</AlertBox>}

      <Tabs
        items={TABS.map(t => ({ value: t.value, label: t.label, icon: t.icon }))}
        value={activeTab}
        onChange={(v) => setActiveTab(v as TabValue)}
      />

      {activeTab === 'settings' && renderSettingsTab()}
      {activeTab === 'mods' && renderModsTab()}
      {activeTab === 'resourcepacks' && renderPackTab(resourcePacks, 'resourcepacks')}
      {activeTab === 'shaderpacks' && renderPackTab(shaderPacks, 'shaderpacks')}
      {activeTab === 'datapacks' && renderPackTab(dataPacks, 'datapacks')}
      {activeTab === 'worlds' && renderWorldsTab()}
      {activeTab === 'schematics' && renderSchematicsTab()}
    </Box>
  )
}
