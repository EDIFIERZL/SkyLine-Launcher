import { useEffect, useState, useCallback, useRef, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Box, Typography, Card, Button, Input, Chip, Tabs, Loading, EmptyState, AlertBox, Select } from '../components/material'
import { VersionIcon } from '../components/VersionIcon'
import { LoaderLogo } from '../components/LoaderLogo'
import { useDownloadStore } from '../stores/downloadStore'
import { useInstanceStore } from '../stores/instanceStore'
import { useSettingsStore } from '../stores/settingsStore'
import { triggerSilentOptimize } from '../hooks/useMemoryOptimizer'
import { ResourceDetail } from '../components/ResourceDetail'
import type { VersionManifest, InstallProgress, VersionEntry, McmodItem, Instance, DownloadKind, ModrinthProject, ModrinthVersion } from '../types'
import {
  Search,
  Download as DownloadIcon,
  Loader2,
  PackageOpen,
  ChevronLeft,
  ChevronRight,
  ArrowLeft,
  Sparkles,
  Image,
  Box as BoxIcon,
  Package,
  Calendar,
  X,
  RefreshCw,
  Map,
} from 'lucide-react'

const DOWNLOAD_TABS = [
  { value: 'minecraft', label: 'Minecraft', icon: <Sparkles className="w-4 h-4" /> },
  { value: 'mods', label: '模组', icon: <Package className="w-4 h-4" /> },
  { value: 'resourcepacks', label: '资源包', icon: <Image className="w-4 h-4" /> },
  { value: 'shaderpacks', label: '光影包', icon: <BoxIcon className="w-4 h-4" /> },
  { value: 'datapacks', label: '数据包', icon: <PackageOpen className="w-4 h-4" /> },
  { value: 'maps', label: '地图', icon: <Map className="w-4 h-4" /> },
  { value: 'modpacks', label: '整合包', icon: <PackageOpen className="w-4 h-4" /> },
]

const SIDEBAR_ITEMS: { value: DownloadKind | 'minecraft'; label: string; desc: string; icon: React.ReactNode }[] = [
  { value: 'minecraft', label: 'Minecraft', desc: '游戏本体下载与安装', icon: <Sparkles className="w-5 h-5" /> },
  { value: 'mods', label: '模组', desc: '游戏模组扩展', icon: <Package className="w-5 h-5" /> },
  { value: 'resourcepacks', label: '资源包', desc: '材质与纹理增强', icon: <Image className="w-5 h-5" /> },
  { value: 'shaderpacks', label: '光影包', desc: '光影与着色效果', icon: <BoxIcon className="w-5 h-5" /> },
  { value: 'datapacks', label: '数据包', desc: '游戏数据包', icon: <PackageOpen className="w-5 h-5" /> },
  { value: 'maps', label: '地图', desc: '游戏地图', icon: <Map className="w-5 h-5" /> },
  { value: 'modpacks', label: '整合包', desc: '一键整合包安装', icon: <PackageOpen className="w-5 h-5" /> },
]

interface LoaderVersion {
  version: string; mc_version: string; stable: boolean
}

interface OptiFineVer {
  mc_version: string; version: string; mirror_url: string; date: string | null
}

type LoaderKey = 'vanilla' | 'forge' | 'neoforge' | 'fabric' | 'fabric-api' | 'quilt' | 'qsl' | 'optifine'

const LOADERS: Record<LoaderKey, { name: string; desc: string; color: string; bg: string; logo: string }> = {
  vanilla: { name: '原版', desc: '不安装任何加载器', color: '#5da83f', bg: 'rgba(93, 168, 63, 0.12)', logo: 'vanilla' },
  forge: { name: 'Forge', desc: '老牌模组加载器', color: '#ff7a1a', bg: 'rgba(255, 122, 26, 0.12)', logo: 'Forge' },
  neoforge: { name: 'NeoForge', desc: 'Forge 继任者，支持新版本', color: '#ef4e27', bg: 'rgba(239, 78, 39, 0.12)', logo: 'NeoForge' },
  fabric: { name: 'Fabric', desc: '轻量级模组加载器', color: '#58b78e', bg: 'rgba(88, 183, 142, 0.12)', logo: 'Fabric' },
  'fabric-api': { name: 'Fabric API', desc: 'Fabric 模组前置库（自动安装 Fabric）', color: '#a78bfa', bg: 'rgba(167, 139, 250, 0.12)', logo: 'Fabric' },
  quilt: { name: 'Quilt', desc: 'Fabric 社区分支加载器', color: '#e5758a', bg: 'rgba(229, 117, 138, 0.12)', logo: 'Quilt' },
  qsl: { name: 'QSL / QFAPI', desc: 'Quilt 模组前置库（自动安装 Quilt）', color: '#22c55e', bg: 'rgba(34, 197, 94, 0.12)', logo: 'Quilt' },
  optifine: { name: 'OptiFine', desc: '高清修复与光影支持', color: '#fbbf24', bg: 'rgba(251, 191, 36, 0.12)', logo: 'OptiFine' },
}

function isAprilFools(v: VersionEntry): boolean {
  return /(RV-Pre1|shareware|infinite|oneblockatatime|_or_b|potato)/i.test(v.id)
}

function buildInstallKey(mcVersion: string, loaders: { loaderr: string; version: string }[]): string {
  const parts = loaders
    .filter((l) => l.loaderr !== 'vanilla')
    .map((l) => `${l.loaderr}-${l.version || 'latest'}`)
  return parts.length ? `${mcVersion}|${parts.join('+')}` : mcVersion
}

type GroupKey = 'release' | 'snapshot' | 'old' | 'april'

const GROUPS: { key: GroupKey; label: string }[] = [
  { key: 'release', label: '正式版' },
  { key: 'snapshot', label: '快照版' },
  { key: 'old', label: '远古版' },
  { key: 'april', label: '愚人节版' },
]

function classify(v: VersionEntry): GroupKey {
  if (isAprilFools(v)) return 'april'
  if (v.type === 'old_beta' || v.type === 'old_alpha') return 'old'
  if (v.type === 'snapshot') return 'snapshot'
  return 'release'
}

const TYPE_BADGE: Record<GroupKey, { label: string; cls: string }> = {
  release: { label: '正式版', cls: 'bg-green-50 text-green-700 dark:bg-green-500/10 dark:text-green-400' },
  snapshot: { label: '快照', cls: 'bg-amber-50 text-amber-700 dark:bg-amber-500/10 dark:text-amber-400' },
  old: { label: '远古', cls: 'bg-stone-100 text-stone-600 dark:bg-stone-500/10 dark:text-stone-400' },
  april: { label: '愚人节', cls: 'bg-fuchsia-50 text-fuchsia-700 dark:bg-fuchsia-500/10 dark:text-fuchsia-400' },
}

type SelectedItem =
  | { type: 'modrinth'; project: ModrinthProject }
  | null

export function Download() {
  const [tab, setTab] = useState<DownloadKind | 'minecraft'>('minecraft')
  const [manifest, setManifest] = useState<VersionManifest | null>(null)
  const [search, setSearch] = useState('')
  const [activeGroup, setActiveGroup] = useState<GroupKey>('release')

  
  const [selectedVersion, setSelectedVersion] = useState<VersionEntry | null>(null)
  const [loaderSel, setLoaderSel] = useState<Partial<Record<LoaderKey, { version: string; label: string }>>>({})
  const [pickerFor, setPickerFor] = useState<LoaderKey | null>(null)
  const [pickerRows, setPickerRows] = useState<{ id: string; label: string; sub: string; value: string }[]>([])
  const [pickerLoading, setPickerLoading] = useState(false)
  const [instanceName, setInstanceName] = useState('')
  const [installing, setInstalling] = useState(false)
  const [progress, setProgress] = useState<InstallProgress | null>(null)
  const [error, setError] = useState<string | null>(null)


  const [query, setQuery] = useState('')
  const [gameVersion, setGameVersion] = useState('')
  const [loaderFilter, setLoaderFilter] = useState('')
  const [mrResults, setMrResults] = useState<ModrinthProject[]>([])
  const [mcmodMap, setMcmodMap] = useState<Record<string, McmodItem>>({})
  const [recs, setRecs] = useState<ModrinthProject[]>([])
  const [recsLoading, setRecsLoading] = useState(false)
  const [loading, setLoading] = useState(false)
  const [offset, setOffset] = useState(0)
  const [selected, setSelected] = useState<SelectedItem>(null)
  const [instances, setInstances] = useState<Instance[]>([])
  const [targetInstanceId, setTargetInstanceId] = useState('')

  const LOADER_OPTIONS = [
    { value: '', label: '全部加载器' },
    { value: 'fabric', label: 'Fabric' },
    { value: 'forge', label: 'Forge' },
    { value: 'neoforge', label: 'NeoForge' },
    { value: 'quilt', label: 'Quilt' },
  ]

  const modloaderKey = (ml: Instance['modloader']): string => {
    if ('Fabric' in ml) return 'fabric'
    if ('Forge' in ml) return 'forge'
    if ('NeoForge' in ml) return 'neoforge'
    if ('Quilt' in ml) return 'quilt'
    return ''
  }

  useEffect(() => {
    invoke<Instance[]>('list_instances').then((list) => {
      setInstances(list)
      if (list.length > 0) {
        const storeId = useInstanceStore.getState().selectedId
        const lastId = useSettingsStore.getState().config.last_selected_instance
        const preferredId =
          (storeId && list.some((i) => i.id === storeId)) ? storeId
          : (lastId && list.some((i) => i.id === lastId)) ? lastId
          : list[0].id
        setTargetInstanceId(preferredId)
      }
    }).catch(console.error)
  }, [])

  const handleTargetChange = (id: string) => {
    setTargetInstanceId(id)
    const inst = instances.find((i) => i.id === id)
    if (inst) {
      setGameVersion(inst.version_id || '')
      const mlk = modloaderKey(inst.modloader)
      setLoaderFilter(mlk || '')
    }
  }
  const activeTaskId = useRef<string | null>(null)

  useEffect(() => {
    invoke<VersionManifest>('fetch_versions').then(setManifest).catch(console.error)
  }, [])

  useEffect(() => {
    const unsub = listen<InstallProgress>('install-progress', (e) => {
      setProgress(e.payload)
      if (activeTaskId.current) {
        useDownloadStore.getState().updateTask(activeTaskId.current, {
          stage: e.payload.stage,
          progress: e.payload.progress,
          message: e.payload.message,
        })
      }
    })
    return () => { unsub.then((f) => f()) }
  }, [])

  const gameVersionOptions = useMemo(() => {
    if (!manifest) return [{ value: '', label: '全部版本' }]
    const seen = new Set<string>()
    const list: { value: string; label: string }[] = []
    const sorted = [...manifest.versions].sort(
      (a, b) => new Date(b.release_time).getTime() - new Date(a.release_time).getTime(),
    )
    for (const v of sorted) {
      if (seen.has(v.id)) continue
      const g = classify(v)
      if (g === 'april' || g === 'old') continue
      seen.add(v.id)
      list.push({ value: v.id, label: v.id })
    }
    return [{ value: '', label: '全部版本' }, ...list]
  }, [manifest])

  const switchTab = (v: string) => {
    setTab(v as DownloadKind | 'minecraft')
    setSelected(null)
    setSelectedVersion(null)
    setLoaderSel({})
    setPickerFor(null)
    setError(null)
  }

  const selectVersion = (v: VersionEntry) => {
    setSelectedVersion(v)
    setLoaderSel({})
    setPickerFor(null)
    setError(null)
  }

  const removeLoader = (loader: LoaderKey) => {
    setLoaderSel((prev) => {
      const next = { ...prev }
      delete next[loader]
      return next
    })
  }

  const openLoaderPicker = async (loader: LoaderKey) => {
    const v = selectedVersion
    if (!v) return
    const mc = v.id
    setError(null)
    setPickerFor(loader)
    setPickerLoading(true)
    setPickerRows([])
    try {
      let rows: { id: string; label: string; sub: string; value: string }[] = []
      if (loader === 'forge') {
        const res = await invoke<LoaderVersion[]>('list_forge_versions', { mcVersion: mc })
        rows = res.map((x) => ({ id: x.version, label: x.version, sub: x.stable ? '稳定版' : '测试版', value: x.version }))
      } else if (loader === 'neoforge') {
        const res = await invoke<LoaderVersion[]>('list_neoforge_versions', { mcVersion: mc })
        rows = res.map((x) => ({ id: x.version, label: x.version, sub: '稳定版', value: x.version }))
      } else if (loader === 'fabric') {
        const res = await invoke<LoaderVersion[]>('list_fabric_versions', { mcVersion: mc })
        rows = res.map((x) => ({ id: x.version, label: x.version, sub: x.stable ? '稳定版' : '测试版', value: x.version }))
      } else if (loader === 'quilt') {
        const res = await invoke<LoaderVersion[]>('list_quilt_loader_versions', { mcVersion: mc })
        rows = res.map((x) => ({ id: x.version, label: x.version, sub: x.stable ? '稳定版' : '测试版', value: x.version }))
      } else if (loader === 'optifine') {
        const res = await invoke<OptiFineVer[]>('list_optifine_versions', { mcVersion: mc })
        rows = res.map((x) => ({ id: x.version, label: x.version, sub: `MC ${x.mc_version}`, value: x.version }))
      } else if (loader === 'fabric-api') {
        const res = await invoke<ModrinthVersion[]>('list_api_mod_versions', { project: 'fabric-api', mcVersion: mc, loaders: ['fabric'] })
        rows = res.map((x) => ({ id: x.id, label: x.version_number, sub: new Date(x.date_published).toLocaleDateString(), value: x.id }))
      } else if (loader === 'qsl') {
        const res = await invoke<ModrinthVersion[]>('list_api_mod_versions', { project: 'qsl', mcVersion: mc, loaders: ['quilt', 'fabric'] })
        rows = res.map((x) => ({ id: x.id, label: x.version_number, sub: new Date(x.date_published).toLocaleDateString(), value: x.id }))
      }
      rows.sort((a, b) => {
        if (loader === 'fabric-api' || loader === 'qsl') return 0
        const parseVer = (v: string) => {
          const parts = v.split('.').map(Number)
          return (parts[0] ?? 0) * 10000 + (parts[1] ?? 0) * 100 + (parts[2] ?? 0)
        }
        return parseVer(b.label) - parseVer(a.label)
      })
      setPickerRows(rows)
    } catch (e) {
      setError(String(e))
      setPickerFor(null)
    }
    setPickerLoading(false)
  }

  const pickLoaderVersion = (loader: LoaderKey, row: { id: string; label: string; sub: string; value: string }) => {
    setLoaderSel((prev) => ({ ...prev, [loader]: { version: row.value, label: row.label } }))
    setPickerFor(null)
  }

  useEffect(() => {
    if (!selectedVersion) return
    const names = (Object.keys(loaderSel) as LoaderKey[])
      .filter((k) => k !== 'vanilla')
      .map((k) => LOADERS[k].name)
    setInstanceName(names.length ? `${selectedVersion.id}-${names.join('+')}` : selectedVersion.id)
  }, [loaderSel, selectedVersion])

  const handleInstall = async () => {
    if (!selectedVersion) return
    const loaders = (Object.keys(loaderSel) as LoaderKey[])
      .filter((k) => k !== 'vanilla' && loaderSel[k]?.version)
      .map((k) => ({ loaderr: k, version: loaderSel[k]!.version }))
    const taskTitle = instanceName.trim() || selectedVersion.id
    const taskId = `game-${Date.now()}`
    const targetKey = buildInstallKey(selectedVersion.id, loaders)

    if (useDownloadStore.getState().hasActiveInstance(targetKey)) {
      const dup = useDownloadStore
        .getState()
        .tasks.find((t) => t.status === 'downloading' && t.instanceId === targetKey)
      if (!confirm(`已有相同的实例「${dup?.title ?? taskTitle}」正在下载中，是否继续？`)) return
    }

    activeTaskId.current = taskId
    useDownloadStore.getState().addTask({
      id: taskId,
      title: taskTitle,
      status: 'downloading',
      stage: 'downloading',
      progress: 0,
      message: '开始安装...',
      instanceId: targetKey,
    })
    setInstalling(true)
    setProgress(null)
    setError(null)
    try {
      await invoke('install_game_multi', {
        name: instanceName.trim() || selectedVersion.id,
        mcVersion: selectedVersion.id,
        loaderrs: loaders,
      })
      useDownloadStore.getState().markDone(taskId)
      setSelectedVersion(null)
      setLoaderSel({})
      setPickerFor(null)
      setTimeout(() => triggerSilentOptimize(), 300)
    } catch (e) {
      useDownloadStore.getState().markError(taskId, String(e))
      setError(String(e))
    }
    setInstalling(false)
    setProgress(null)
    activeTaskId.current = null
  }

  const searchMods = useCallback(async (newOffset = 0) => {
    if (!query.trim()) return
    setLoading(true)
    setError(null)
    const gv = gameVersion || null
    try {
      let res: ModrinthProject[]
      const ldrs = (tab === 'mods' && loaderFilter) ? [loaderFilter] : undefined
      if (tab === 'mods') {
        res = await invoke<ModrinthProject[]>('search_modrinth_mods', { query, limit: 20, offset: newOffset, gameVersion: gv, loaders: ldrs })
      } else if (tab === 'resourcepacks') {
        res = await invoke<ModrinthProject[]>('search_resource_packs', { query, limit: 20, offset: newOffset, gameVersion: gv })
      } else if (tab === 'shaderpacks') {
        res = await invoke<ModrinthProject[]>('search_shader_packs', { query, limit: 20, offset: newOffset, gameVersion: gv })
      } else if (tab === 'datapacks') {
        res = await invoke<ModrinthProject[]>('search_datapacks', { query, limit: 20, offset: newOffset, gameVersion: gv })
      } else if (tab === 'maps') {
        res = await invoke<ModrinthProject[]>('search_worlds', { query, limit: 20, offset: newOffset, gameVersion: gv })
      } else {
        res = await invoke<ModrinthProject[]>('search_modpacks', { query, limit: 20, offset: newOffset, gameVersion: gv })
      }
      setMrResults(res)
      if (tab === 'mods') {
        const titles = res.slice(0, 8).map((m) => m.title)
        invoke<McmodItem[]>('enrich_mcmod_batch', { titles }).then((enriched) => {
          const map: Record<string, McmodItem> = {}
          enriched.forEach((item, i) => {
            const project = res[i]
            if (project) map[project.slug] = item
          })
          setMcmodMap(map)
        }).catch(() => {})
      }
      setOffset(newOffset)
    } catch (e) { setError(String(e)) }
    setLoading(false)
  }, [query, tab, gameVersion])

  useEffect(() => {
    if (query.trim()) searchMods()
  }, [tab, gameVersion, loaderFilter, searchMods])

  useEffect(() => {
    if (tab === 'minecraft' || query.trim()) return
    const gv = gameVersion || null
    const ldrs = (tab === 'mods' && loaderFilter) ? [loaderFilter] : undefined
    const cmd =
      tab === 'mods' ? 'recommended_mods'
      : tab === 'resourcepacks' ? 'recommended_resource_packs'
      : tab === 'shaderpacks' ? 'recommended_shader_packs'
      : 'recommended_modpacks'
    setRecsLoading(true)
    invoke<ModrinthProject[]>(cmd, { limit: 12, gameVersion: gv, loaders: ldrs })
      .then((res) => {
        setRecs(res)
        if (tab === 'mods') {
          const titles = res.slice(0, 8).map((m) => m.title)
          return invoke<McmodItem[]>('enrich_mcmod_batch', { titles }).then((enriched) => {
            const map: Record<string, McmodItem> = {}
            enriched.forEach((item, i) => {
              const project = res[i]
              if (project) map[project.slug] = item
            })
            setMcmodMap(map)
          })
        }
      })
      .catch(() => {})
      .finally(() => setRecsLoading(false))
  }, [tab, query, gameVersion, loaderFilter])

  const selectProject = (project: ModrinthProject) => {
    setSelected({ type: 'modrinth', project })
    setError(null)
  }

  const formatDownloads = (n: number | null) => {
    if (!n) return ''
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
    return String(n)
  }

  const groupedVersions = () => {
    if (!manifest) return {} as Record<GroupKey, VersionEntry[]>
    const map = Object.create(null) as Record<GroupKey, VersionEntry[]>
    for (const g of GROUPS) map[g.key] = []
    const term = search.trim().toLowerCase()
    for (const v of manifest.versions) {
      if (term && !v.id.toLowerCase().includes(term)) continue
      const g = classify(v)
      map[g]?.push(v)
    }
    for (const g of GROUPS) {
      const list = map[g.key]
      list?.sort(((a: VersionEntry, b: VersionEntry) => new Date(b.release_time).getTime() - new Date(a.release_time).getTime()))
    }
    return map
  }

  const formatReleaseDate = (releaseTime: string) => {
    if (!releaseTime) return ''
    const d = new Date(releaseTime)
    if (Number.isNaN(d.getTime())) return ''
    return d.toLocaleDateString('zh-CN')
  }

  const renderGameList = () => {
    const grouped = groupedVersions()
    const list = grouped[activeGroup] ?? []
    return (
      <div className="space-y-3">
        <div className="flex items-center gap-3 flex-wrap">
          <div className="relative flex-1 max-w-sm">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-surface-400" />
            <input className="w-full h-10 pl-9 pr-3 rounded-lg bg-white dark:bg-surface-850 border border-surface-200 text-sm text-surface-900 placeholder:text-surface-400 focus:outline-none focus:ring-2 focus:ring-accent-500"
              placeholder="搜索版本..." value={search} onChange={(e) => setSearch(e.target.value)} />
          </div>
          <div className="flex gap-1 bg-surface-100 dark:bg-surface-800 p-1 rounded-lg w-fit">
            {GROUPS.map((g) => (
              <button key={g.key} onClick={() => setActiveGroup(g.key)}
                className={`px-3 py-1.5 rounded-md text-sm font-medium transition-all duration-150 cursor-pointer ${
                  activeGroup === g.key
                    ? 'bg-white dark:bg-surface-850 text-surface-900 dark:text-surface-100 shadow-xs'
                    : 'text-surface-500 hover:text-surface-700 dark:hover:text-surface-300'
                }`}
              >{g.label}</button>
            ))}
          </div>
          <span className="text-xs text-surface-400">{list.length} 个版本</span>
        </div>
        <div className="bg-white dark:bg-surface-850 rounded-xl shadow-xs divide-y divide-surface-100 dark:divide-transparent overflow-hidden">
          {list.length === 0 ? (
            <div className="py-10 text-center text-sm text-surface-400">暂无版本</div>
          ) : (
            list.map((v: VersionEntry) => (
              <button key={v.id} onClick={() => selectVersion(v)}
                className="w-full flex items-center gap-3 px-4 py-3 hover:bg-surface-50 dark:hover:bg-surface-800 transition-colors text-left cursor-pointer">
                <VersionIcon group={classify(v)} size={24} />
                <span className="font-medium text-sm text-surface-900 dark:text-surface-100 flex-1 min-w-0 truncate">{v.id}</span>
                <span className="flex items-center gap-1.5 text-xs text-surface-400 w-28 shrink-0">
                  <Calendar className="w-3 h-3 shrink-0" />
                  <span className="truncate">{formatReleaseDate(v.release_time) || '未知'}</span>
                </span>
                <ChevronRight className="w-4 h-4 text-surface-300 shrink-0" />
              </button>
            ))
          )}
        </div>
      </div>
    )
  }

  const renderLoaderSelect = () => {
    if (!selectedVersion) return null
    const showProgress = installing && progress
    const chosen = (Object.keys(loaderSel) as LoaderKey[]).filter((k) => k !== 'vanilla' && loaderSel[k]?.version)
    return (
      <div className="space-y-4">
        <button onClick={() => setSelectedVersion(null)} className="flex items-center gap-1 text-sm text-surface-500 hover:text-surface-700 cursor-pointer">
          <ArrowLeft className="w-4 h-4" /> 返回版本列表
        </button>
        <div className="bg-white dark:bg-surface-850 rounded-xl p-4 flex items-center gap-3 shadow-xs">
          <VersionIcon group={classify(selectedVersion)} size={32} />
          <div className="flex-1 min-w-0">
            <div className="font-bold text-surface-900 dark:text-surface-100">{selectedVersion.id}</div>
            <div className="text-xs text-surface-500 flex items-center gap-2 mt-0.5">
              <span className={`px-1.5 py-0.5 rounded text-[10px] font-medium ${TYPE_BADGE[classify(selectedVersion)].cls}`}>
                {TYPE_BADGE[classify(selectedVersion)].label}
              </span>
              <span className="flex items-center gap-1"><Calendar className="w-3 h-3" /> {formatReleaseDate(selectedVersion.release_time) || '未知'}</span>
            </div>
          </div>
        </div>
        <div>
          <div className="flex items-center justify-between mb-2">
            <h3 className="font-semibold text-sm text-surface-800 dark:text-surface-200">选择模组加载器（可多选）</h3>
            <span className="text-xs text-surface-400">
              {chosen.length === 0 ? '不安装加载器' : `已选 ${chosen.length} 个`}
            </span>
          </div>
          <div className="grid grid-cols-2 xl:grid-cols-4 gap-2.5">
            {(Object.keys(LOADERS) as LoaderKey[]).map((key) => {
              const meta = LOADERS[key]
              const sel = loaderSel[key]
              const isSelected = !!sel?.version
              return (
                <button key={key} onClick={() => {
                    if (key === 'vanilla') { setLoaderSel({}) }
                    else if (isSelected) { removeLoader(key) }
                    else { openLoaderPicker(key) }
                  }}
                  className={`flex items-center gap-2.5 p-2.5 rounded-xl border transition-all duration-150 text-left cursor-pointer ${
                    isSelected
                      ? 'border-accent-500 ring-2 ring-accent-500/20'
                      : 'border-surface-200 dark:border-surface-200 hover:border-accent-300 dark:hover:border-accent-500/40 hover:shadow-md'
                  } bg-white dark:bg-surface-850`}>
                  <div className="w-9 h-9 rounded-lg flex items-center justify-center shrink-0 p-1.5" style={{ backgroundColor: meta.bg }}>
                    <LoaderLogo loader={meta.logo} className="w-full h-full" />
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="font-medium text-sm text-surface-900 dark:text-surface-100 truncate">{meta.name}</div>
                    <div className="text-[11px] text-surface-400 truncate">
                      {key === 'vanilla' ? (isSelected ? sel.label : selectedVersion.id) : isSelected ? sel.label : '未安装'}
                    </div>
                  </div>
                  {isSelected ? (
                    <X className="w-4 h-4 text-surface-400 shrink-0 hover:text-red-500" />
                  ) : (
                    <ChevronRight className="w-4 h-4 text-surface-300 shrink-0" />
                  )}
                </button>
              )
            })}
          </div>
        </div>
        <div className="bg-white dark:bg-surface-850 rounded-xl p-4 space-y-3 shadow-xs">
          <Input label="实例名称" value={instanceName} onChange={(e) => setInstanceName(e.target.value)} placeholder="自定义实例名称" />
          <Button className="w-full" size="large" onClick={handleInstall} loading={installing}>
            {installing ? '正在安装...' : (
              <><DownloadIcon className="w-4 h-4 mr-1.5" /> 安装 {selectedVersion.id}
                {chosen.length > 0 && ` + ${chosen.map((k) => LOADERS[k].name).join(' + ')}`}
              </>
            )}
          </Button>
          {showProgress && (
            <div className="space-y-1.5">
              <div className="flex items-center justify-between text-xs">
                <span className="text-surface-500">{progress.message}</span>
                <span className="text-accent-600 font-medium">{Math.round(progress.progress * 100)}%</span>
              </div>
              <div className="w-full h-2 bg-surface-200 dark:bg-surface-700 rounded-full overflow-hidden">
                <div className="h-full bg-accent-500 rounded-full transition-all duration-300" style={{ width: `${progress.progress * 100}%` }} />
              </div>
            </div>
          )}
        </div>
        {pickerFor && (
          <div className="fixed inset-0 z-50 flex items-center justify-center p-4 fade-in">
            <div className="absolute inset-0 bg-black/40" onClick={() => setPickerFor(null)} />
            <div className="pop-in relative bg-white dark:bg-surface-850 rounded-xl shadow-xl w-full max-w-md flex flex-col max-h-[70vh]">
              <div className="flex items-center justify-between px-4 py-3 border-b border-surface-200 dark:border-surface-700">
                <span className="font-semibold text-sm text-surface-900 dark:text-surface-100">
                  {LOADERS[pickerFor].name} 版本 {selectedVersion.id}
                </span>
                <button onClick={() => setPickerFor(null)} className="p-1 rounded-md hover:bg-surface-100 dark:hover:bg-surface-800 cursor-pointer">
                  <X className="w-4 h-4 text-surface-400" />
                </button>
              </div>
              <div className="p-2 overflow-y-auto">
                {pickerLoading ? (
                  <div className="flex justify-center py-10"><Loader2 className="w-6 h-6 animate-spin text-surface-400" /></div>
                ) : pickerRows.length === 0 ? (
                  <div className="py-8 text-center text-sm text-surface-400 flex flex-col items-center gap-2">
                    <span>该版本暂未收录 {LOADERS[pickerFor].name}</span>
                    <span className="text-xs text-surface-400/70">可能是加载列表失败或网络波动，请重试</span>
                    <button onClick={() => openLoaderPicker(pickerFor)} className="text-accent-600 text-xs flex items-center gap-1 cursor-pointer hover:underline">
                      <RefreshCw className="w-3 h-3" /> 重新加载
                    </button>
                  </div>
                ) : (
                  pickerRows.map((row) => {
                    const isCurrent = loaderSel[pickerFor]?.version === row.value
                    return (
                      <button key={row.id} onClick={() => pickLoaderVersion(pickerFor, row)}
                        className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left transition-colors cursor-pointer ${
                          isCurrent ? 'bg-accent-50 dark:bg-accent-500/10' : 'hover:bg-surface-100 dark:hover:bg-surface-800'
                        }`}>
                        <span className={`w-4 h-4 rounded-full border-2 flex items-center justify-center shrink-0 ${
                          isCurrent ? 'border-accent-500' : 'border-surface-300'
                        }`}>{isCurrent && <span className="w-2 h-2 rounded-full bg-accent-500" />}</span>
                        <span className="font-medium text-sm text-surface-900 dark:text-surface-100 flex-1">{row.label}</span>
                        <span className="text-xs text-surface-400">{row.sub}</span>
                      </button>
                    )
                  })
                )}
              </div>
            </div>
          </div>
        )}
      </div>
    )
  }

  const renderMinecraft = () => {
    if (selectedVersion) return renderLoaderSelect()
    return renderGameList()
  }

  const renderDetail = () => {
    if (!selected) return null
    const mcmod = selected.type === 'modrinth' ? mcmodMap[selected.project.slug] || null : null
    return (
      <ResourceDetail
        kind={tab as DownloadKind}
        project={selected.project}
        mcmod={mcmod}
        gameVersion={gameVersion || undefined}
        onClearGameVersion={() => setGameVersion('')}
        instances={instances}
        targetInstanceId={targetInstanceId}
        onTargetChange={handleTargetChange}
        onBack={() => setSelected(null)}
        preferredInstanceId={useInstanceStore.getState().selectedId ?? useSettingsStore.getState().config.last_selected_instance}
      />
    )
  }

  const renderResourceList = () => {
    if (selected) return renderDetail()
    const categoryLabel = tab === 'mods' ? '模组' : tab === 'resourcepacks' ? '资源包' : tab === 'shaderpacks' ? '光影包' : tab === 'datapacks' ? '数据包' : tab === 'maps' ? '地图' : '整合包'
    const recTitle = tab === 'mods' ? '推荐模组' : tab === 'resourcepacks' ? '推荐资源包' : tab === 'shaderpacks' ? '推荐光影' : tab === 'datapacks' ? '推荐数据包' : tab === 'maps' ? '推荐地图' : '推荐整合包'
    return (
      <>
        <Box className="flex items-center gap-2 flex-wrap">
          {tab === 'mods' && (
            <>
               <Typography variant="caption" color="text.secondary">游戏版本</Typography>
              <Box className="w-52 shrink-0">
                <Select
                  value={gameVersion}
                  onChange={(v) => setGameVersion(v)}
                  options={gameVersionOptions}
                  size="small"
                  renderValue={(v) => v === '' ? '全部版本' : v}
                />
              </Box>
              <Typography variant="caption" color="text.secondary">加载器</Typography>
              <Box className="w-40 shrink-0">
                <Select
                  value={loaderFilter}
                  onChange={(v) => setLoaderFilter(v)}
                  options={LOADER_OPTIONS}
                  size="small"
                  renderValue={(v) => {
                    const opt = LOADER_OPTIONS.find(o => o.value === v)
                    return opt ? opt.label : '全部加载器'
                  }}
                />
              </Box>
            </>
          )}
          {tab !== 'mods' && tab !== 'minecraft' && (
            <>
              <Typography variant="caption" color="text.secondary">游戏版本</Typography>
              <Box className="w-52 shrink-0">
                <Select
                  value={gameVersion}
                  onChange={(v) => setGameVersion(v)}
                  options={gameVersionOptions}
                  size="small"
                  renderValue={(v) => v === '' ? '全部版本' : v}
                />
              </Box>
            </>
          )}
          {(gameVersion || loaderFilter) && (
            <Button variant="text" size="small" onClick={() => { setGameVersion(''); setLoaderFilter('') }} startIcon={<X className="w-3.5 h-3.5" />}>
              清除筛选
            </Button>
          )}
          <Typography variant="caption" color="text.secondary" className="text-xs">
            先选游戏版本，搜索结果将只显示兼容该版本的资源
          </Typography>
        </Box>
        <form onSubmit={(e) => { e.preventDefault(); searchMods() }} className="input-action-row">
          <Box className="flex-1">
            <Input placeholder={`搜索${categoryLabel}...`}
              value={query} onChange={(e) => setQuery(e.target.value)} />
          </Box>
          <Button type="submit" loading={loading} startIcon={<Search className="w-4 h-4" />}>搜索</Button>
        </form>
        {error && <AlertBox severity="error" onClose={() => setError(null)}>{error}</AlertBox>}
        {!query.trim() ? (
          <>
            <Box className="flex items-center justify-between">
              <Typography variant="subtitle2">{recTitle}</Typography>
              <Typography variant="caption" color="text.secondary">按下载量排序</Typography>
            </Box>
            {recsLoading ? (
              <Loading />
            ) : recs.length > 0 ? (
              <Box className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {recs.map((mod) => {
                  const cn = mcmodMap[mod.slug]
                  return (
                    <Card key={mod.slug} onClick={() => selectProject(mod)} hoverable>
                      <Box className="flex gap-3">
                        {mod.icon_url && <img src={mod.icon_url} alt="" className="w-12 h-12 rounded-lg object-cover shrink-0" />}
                        <Box className="min-w-0 flex-1">
                          <Typography variant="subtitle2" className="truncate flex items-center gap-1.5">
                            <span className="truncate">{cn && cn.title ? cn.title : mod.title}</span>
                            {cn && <Chip label="MC百科" size="small" color="primary" variant="outlined" />}
                          </Typography>
                          {mod.author && <Typography variant="caption" color="text.secondary">{mod.author}</Typography>}
                          <Typography variant="body2" color="text.secondary" className="mt-1 line-clamp-2">
                            {cn && cn.description ? cn.description : mod.description}
                          </Typography>
                          <Box className="flex items-center gap-2 mt-2">
                            {mod.downloads && <Typography variant="caption" color="text.secondary">{formatDownloads(mod.downloads)} 下载</Typography>}
                            {mod.categories.slice(0, 2).map((cat) => (
                              <Chip key={cat} label={cat} size="small" variant="outlined" />
                            ))}
                          </Box>
                        </Box>
                      </Box>
                    </Card>
                  )
                })}
              </Box>
            ) : (
              <EmptyState icon={<PackageOpen className="w-12 h-12" />} title="暂无推荐" />
            )}
          </>
        ) : loading ? (
          <div className="flex justify-center py-16"><Loader2 className="w-8 h-8 animate-spin text-surface-400" /></div>
        ) : mrResults.length > 0 ? (
          <>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {mrResults.map((mod) => {
                const cn = mcmodMap[mod.slug]
                return (
                  <div key={mod.slug} onClick={() => selectProject(mod)}
                    className="bg-white dark:bg-surface-850 border border-surface-200 rounded-xl p-4 hover:border-accent-300 dark:hover:border-accent-500/40 hover:shadow-md transition-all duration-150 cursor-pointer">
                    <div className="flex gap-3">
                      {mod.icon_url && <img src={mod.icon_url} alt="" className="w-12 h-12 rounded-lg object-cover shrink-0" />}
                      <div className="min-w-0 flex-1">
                        <h3 className="font-medium text-surface-900 dark:text-surface-100 truncate flex items-center gap-1.5">
                          <span className="truncate">{cn && cn.title ? cn.title : mod.title}</span>
                          {cn && <span className="px-1.5 py-0.5 bg-accent-50 dark:bg-accent-500/10 text-accent-600 dark:text-accent-400 rounded text-[10px] font-medium shrink-0">MC百科</span>}
                        </h3>
                        {mod.author && <p className="text-xs text-surface-400">{mod.author}</p>}
                        <p className="text-sm text-surface-500 mt-1 line-clamp-2">{cn && cn.description ? cn.description : mod.description}</p>
                        <div className="flex items-center gap-2 mt-2 text-xs text-surface-400">
                          {mod.downloads && <span>{formatDownloads(mod.downloads)} 下载</span>}
                          {mod.categories.slice(0, 2).map((cat) => (
                            <span key={cat} className="px-1.5 py-0.5 bg-surface-100 dark:bg-surface-800 rounded text-xs">{cat}</span>
                          ))}
                        </div>
                      </div>
                    </div>
                  </div>
                )
              })}
            </div>
            <div className="flex justify-center gap-2">
              <Button variant="outlined" size="small" disabled={offset === 0} onClick={() => searchMods(offset - 20)}>
                <ChevronLeft className="w-4 h-4 mr-1" /> 上一页
              </Button>
              <Button variant="outlined" size="small" onClick={() => searchMods(offset + 20)}>
                下一页 <ChevronRight className="w-4 h-4 ml-1" />
              </Button>
            </div>
          </>
        ) : (
          <EmptyState icon={<PackageOpen className="w-12 h-12" />} title="输入关键词开始搜索" />
        )}
      </>
    )
  }

  const renderResourceSelector = () => {
    const items = SIDEBAR_ITEMS.filter((i) => i.value !== 'minecraft' && i.value !== 'mods')
    return (
      <div className="pt-4 border-t border-surface-200 dark:border-surface-700/60">
        <div className="flex items-center justify-between mb-3">
          <Typography variant="subtitle2">更多资源</Typography>
          <Typography variant="caption" color="text.secondary">点击切换到对应资源下载</Typography>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          {items.map((item) => {
            const active = tab === item.value
            return (
              <button
                key={item.value}
                onClick={() => switchTab(item.value)}
                className={`flex items-center gap-3 p-3 rounded-xl border text-left transition-all duration-150 cursor-pointer ${
                  active
                    ? 'border-accent-500 ring-2 ring-accent-500/20 bg-accent-50 dark:bg-accent-500/10'
                    : 'border-surface-200 dark:border-surface-700 bg-white dark:bg-surface-850 hover:border-accent-300 dark:hover:border-accent-500/40 hover:shadow-md'
                }`}
              >
                <span className={`shrink-0 ${active ? 'text-accent-600 dark:text-accent-400' : 'text-surface-400'}`}>
                  {item.icon}
                </span>
                <span className="min-w-0 flex-1">
                  <span className="block text-sm font-medium truncate text-surface-900 dark:text-surface-100">{item.label}</span>
                  <span className="block text-[11px] text-surface-400 truncate">{item.desc}</span>
                </span>
                <ChevronRight className="w-4 h-4 text-surface-300 shrink-0" />
              </button>
            )
          })}
        </div>
      </div>
    )
  }

  return (
    <Box className="space-y-5 max-w-7xl w-full">
      <Box>
        <Typography variant="h5">资源中心</Typography>
        <Typography variant="body2" color="text.secondary" className="mt-1">下载各类游戏资源</Typography>
      </Box>
      <Box className="space-y-4">
        <Tabs items={DOWNLOAD_TABS} value={tab} onChange={switchTab} />
        {tab === 'minecraft' ? (
          <Box className="space-y-4">
            {error && <AlertBox severity="error" onClose={() => setError(null)}>{error}</AlertBox>}
            {renderMinecraft()}
          </Box>
        ) : (
          <>
            {renderResourceList()}
            {tab === 'mods' && renderResourceSelector()}
          </>
        )}
      </Box>
    </Box>
  )
}
