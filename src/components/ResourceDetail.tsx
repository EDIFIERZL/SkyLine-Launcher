import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-shell'
import { Box, Typography, Card, Button, Chip, AlertBox, Select, Tabs, SnackbarAlert } from './material'
import { useDownloadStore } from '../stores/downloadStore'
import { LoaderLogo } from './LoaderLogo'
import { type VersionGroup } from './VersionIcon'
import type {
  Instance, InstallProgress, ModrinthProject, ModrinthProjectDetail, ModrinthVersion, ModrinthDependency,
  CurseForgeMod, CurseForgeFile, McmodItem, DownloadKind,
} from '../types'
import {
  ArrowLeft,
  Download as DownloadIcon,
  Globe,
  BookOpen,
  Loader2,
  Heart,
  Flame,
  Calendar,
  PackageOpen,
  Link as LinkIcon,
  ExternalLink,
  X,
  ChevronDown,
  ChevronRight,
} from 'lucide-react'

interface ResourceDetailProps {
  kind: DownloadKind
  project: ModrinthProject | null
  cfMod: CurseForgeMod | null
  mcmod: McmodItem | null
  gameVersion?: string
  onClearGameVersion?: () => void
  instances: Instance[]
  targetInstanceId: string
  onTargetChange: (id: string) => void
  onBack: () => void
  preferredInstanceId?: string | null
}

function markdownToText(md: string): string {
  return md
    .replace(/```[\s\S]*?```/g, (m) => `\n${m.replace(/```/g, '').trim()}\n`)
    .replace(/!\[[^\]]*\]\([^)]*\)/g, '')
    .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
    .replace(/^#{1,6}\s+/gm, '')
    .replace(/^\s*[-*+]\s+/gm, '• ')
    .replace(/^\s*\d+\.\s+/gm, '')
    .replace(/(\*\*|__|\*|_|`|~~)/g, '')
    .replace(/^\s*>\s?/gm, '')
    .replace(/<br\s*\/?\s*>/gi, '\n')
    .replace(/<\/?[^>]+>/g, '')
    .replace(/\n{3,}/g, '\n\n')
    .trim()
}

function formatDownloads(n: number | null): string {
  if (!n) return ''
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return String(n)
}

function formatSize(bytes: number): string {
  if (bytes >= 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB`
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${bytes} B`
}

function formatDate(d: string | null | undefined): string {
  if (!d) return ''
  const date = new Date(d)
  if (Number.isNaN(date.getTime())) return ''
  return date.toLocaleDateString('zh-CN')
}

type LoaderGroup = 'vanilla' | 'forge' | 'neoforge' | 'fabric' | 'quilt' | 'optifine' | 'other'

const LOADER_GROUPS: { key: LoaderGroup; label: string; logo: string }[] = [
  { key: 'forge', label: 'Forge', logo: 'Forge' },
  { key: 'neoforge', label: 'NeoForge', logo: 'NeoForge' },
  { key: 'fabric', label: 'Fabric', logo: 'Fabric' },
  { key: 'quilt', label: 'Quilt', logo: 'Quilt' },
  { key: 'optifine', label: 'OptiFine', logo: 'OptiFine' },
  { key: 'vanilla', label: '原版 / Vanilla', logo: 'vanilla' },
  { key: 'other', label: '其他', logo: 'vanilla' },
]

function loaderGroup(loaders: string[]): LoaderGroup {
  const l = loaders.map((x) => x.toLowerCase())
  if (l.includes('forge')) return 'forge'
  if (l.includes('neoforge')) return 'neoforge'
  if (l.includes('fabric')) return 'fabric'
  if (l.includes('quilt')) return 'quilt'
  if (l.includes('optifine')) return 'optifine'
  if (l.length === 0) return 'vanilla'
  return 'other'
}

function instanceLoaderKey(ml: Instance['modloader'] | undefined): string {
  if (!ml) return 'vanilla'
  if ('Forge' in ml) return 'forge'
  if ('NeoForge' in ml) return 'neoforge'
  if ('Fabric' in ml) return 'fabric'
  if ('Quilt' in ml) return 'quilt'
  return 'vanilla'
}

type ResourceType = 'release' | 'beta' | 'alpha'

const RESOURCE_TYPE_META: { key: ResourceType; label: string; cls: string }[] = [
  { key: 'release', label: '正式版', cls: 'bg-green-50 text-green-700 dark:bg-green-500/10 dark:text-green-400' },
  { key: 'beta', label: '测试版', cls: 'bg-amber-50 text-amber-700 dark:bg-amber-500/10 dark:text-amber-400' },
  { key: 'alpha', label: '预览版', cls: 'bg-sky-50 text-sky-700 dark:bg-sky-500/10 dark:text-sky-400' },
]

const RESOURCE_TYPE_MAP: Record<ResourceType, { label: string; cls: string }> = {
  release: RESOURCE_TYPE_META[0],
  beta: RESOURCE_TYPE_META[1],
  alpha: RESOURCE_TYPE_META[2],
}

const GAME_GROUPS: { key: VersionGroup; label: string }[] = [
  { key: 'release', label: '正式版' },
  { key: 'snapshot', label: '快照版' },
  { key: 'old', label: '远古版' },
  { key: 'april', label: '愚人节版' },
]

function parseVersionNumber(v: string): number[] {
  const cleaned = v.replace(/[^0-9.]/g, '')
  return cleaned.split('.').map((s) => {
    const n = parseInt(s, 10)
    return Number.isNaN(n) ? 0 : n
  })
}

function compareVersions(a: string, b: string): number {
  const pa = parseVersionNumber(a)
  const pb = parseVersionNumber(b)
  const len = Math.max(pa.length, pb.length)
  for (let i = 0; i < len; i++) {
    const na = pa[i] ?? 0
    const nb = pb[i] ?? 0
    if (na !== nb) return nb - na
  }
  return 0
}

function isAprilFools(v: string): boolean {
  return /(RV-Pre1|shareware|infinite|oneblockatatime|_or_b|potato)/i.test(v)
}

function classifyMcVersion(id: string): VersionGroup {
  const s = id.toLowerCase()
  if (isAprilFools(id)) return 'april'
  if (/(pre|rc|snapshot|-\d{2}w|\d{2}w\d{2}[a-z])/.test(s)) return 'snapshot'
  if (/(alpha|beta|classic|indev|infdev|rd-|c0\.|b1\.|a1\.)/.test(s)) return 'old'
  const m = s.match(/^(\d+)\.(\d+)/)
  if (m) {
    const major = parseInt(m[1], 10)
    const minor = parseInt(m[2], 10)
    if (major < 1 || (major === 1 && minor <= 6)) return 'old'
  }
  return 'release'
}

function classifyGameVersions(list: string[]): VersionGroup {
  let hasRelease = false
  for (const v of list) {
    const g = classifyMcVersion(v)
    if (g === 'april') return 'april'
    if (g === 'snapshot') return 'snapshot'
    if (g === 'old') return 'old'
    hasRelease = true
  }
  return hasRelease ? 'release' : 'release'
}

function mrResourceType(v: ModrinthVersion): ResourceType {
  const t = (v.version_type || '').toLowerCase()
  if (t === 'beta' || t === 'alpha') return t
  return 'release'
}

function cfResourceType(f: CurseForgeFile): ResourceType {
  if (f.release_type === 2) return 'beta'
  if (f.release_type === 3) return 'alpha'
  return 'release'
}

interface VersionItem {
  key: string
  name: string
  versionNumber?: string
  date: string
  gameVersions: string[]
  loaders: string[]
  fileName: string
  fileSize: number
  source: 'modrinth' | 'curseforge'
  resourceType: ResourceType
  gameType: VersionGroup
  mrVersion?: ModrinthVersion
  cfFile?: CurseForgeFile
}

export function ResourceDetail({
  kind,
  project,
  cfMod,
  mcmod,
  gameVersion,
  onClearGameVersion,
  instances,
  targetInstanceId,
  onTargetChange,
  onBack,
  preferredInstanceId,
}: ResourceDetailProps) {
  const [mrResolved, setMrResolved] = useState<ModrinthProject | null>(project)
  const [cfResolved, setCfResolved] = useState<CurseForgeMod | null>(cfMod)
  const [mrDetail, setMrDetail] = useState<ModrinthProjectDetail | null>(null)
  const [mrVersions, setMrVersions] = useState<ModrinthVersion[]>([])
  const [cfProject, setCfProject] = useState<CurseForgeMod | null>(null)
  const [cfFiles, setCfFiles] = useState<CurseForgeFile[]>([])
  const [mrLoading, setMrLoading] = useState(false)
  const [cfLoading, setCfLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [downloading, setDownloading] = useState<string | null>(null)
  const [iconErr, setIconErr] = useState(false)
  const downloadingRef = useRef<string | null>(null)
  const [activeLoader, setActiveLoader] = useState<LoaderGroup | null>(null)
  const [gameFilter, setGameFilter] = useState<VersionGroup | 'all'>('all')
  const [resFilter, setResFilter] = useState<ResourceType | 'all'>('all')
  const [toast, setToast] = useState<string | null>(null)
  const [selectedGameVersion, setSelectedGameVersion] = useState<string>('all')
  const [expandedVersions, setExpandedVersions] = useState<Set<string>>(new Set())

  const targetDir = kind === 'resourcepacks' ? 'resourcepacks' : kind === 'shaderpacks' ? 'shaderpacks' : kind === 'datapacks' || kind === 'maps' ? 'datapacks' : 'mods'
  const isModpack = kind === 'modpacks'

  const searchModrinthFn = (query: string) => {
    const limit = 5
    const offset = 0
    const gv = gameVersion || null
    if (kind === 'resourcepacks') return invoke<ModrinthProject[]>('search_resource_packs', { query, limit, offset, gameVersion: gv })
    if (kind === 'shaderpacks') return invoke<ModrinthProject[]>('search_shader_packs', { query, limit, offset, gameVersion: gv })
    if (kind === 'datapacks' || kind === 'maps') return invoke<ModrinthProject[]>('search_datapacks', { query, limit, offset, gameVersion: gv })
    if (kind === 'modpacks') return invoke<ModrinthProject[]>('search_modpacks', { query, limit, offset, gameVersion: gv })
    return invoke<ModrinthProject[]>('search_modrinth_mods', { query, limit, offset, gameVersion: gv })
  }

  const loadMr = async (p: ModrinthProject) => {
    setMrResolved(p)
    setIconErr(false)
    setMrDetail(null)
    setMrVersions([])
    setMrLoading(true)
    setError(null)
    try {
      const [detail, vers] = await Promise.all([
        invoke<ModrinthProjectDetail>('get_modrinth_project_detail', { slug: p.slug }),
        invoke<ModrinthVersion[]>('get_modrinth_versions', { projectId: p.project_id || p.slug }),
      ])
      setMrDetail(detail)
      setMrVersions(vers.sort((a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime()))
    } catch (e) {
      setError(String(e))
    }
    setMrLoading(false)
  }

  const loadCf = async (c: CurseForgeMod) => {
    setCfResolved(c)
    setIconErr(false)
    setCfProject(null)
    setCfFiles([])
    setCfLoading(true)
    setError(null)
    try {
      const [proj, files] = await Promise.all([
        invoke<CurseForgeMod>('get_curseforge_project', { modId: c.id }),
        invoke<CurseForgeFile[]>('get_curseforge_files', { modId: c.id }),
      ])
      setCfProject(proj)
      setCfFiles(files)
    } catch (e) {
      setError(String(e))
    }
    setCfLoading(false)
  }

  const resolveCfFromMr = async (p: ModrinthProject) => {
    try {
      const res = await invoke<CurseForgeMod[]>('search_curseforge_category', {
        query: p.title,
        gameVersion: gameVersion || null,
        category: kind === 'mods' ? null : kind,
      })
      if (res[0]) await loadCf(res[0])
    } catch {  }
  }

  const resolveMrFromCf = async (c: CurseForgeMod) => {
    try {
      const res = await searchModrinthFn(c.name)
      if (res[0]) await loadMr(res[0])
    } catch {  }
  }

  useEffect(() => {
    setIconErr(false)
    setMrResolved(project)
    setCfResolved(cfMod)
    setMrDetail(null)
    setMrVersions([])
    setCfProject(null)
    setCfFiles([])
    setError(null)
    if (project) {
      void loadMr(project)
      if (!cfMod) void resolveCfFromMr(project)
    } else if (cfMod) {
      void loadCf(cfMod)
      void resolveMrFromCf(cfMod)
    }
    
  }, [project?.project_id || project?.slug, cfMod?.id])

  const startTask = async (title: string, fn: () => Promise<unknown>) => {
    if (downloadingRef.current) return
    const taskId = `res-${Date.now()}-${Math.floor(Math.random() * 1e6)}`
    downloadingRef.current = taskId
    useDownloadStore.getState().addTask({
      id: taskId,
      title,
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
      await fn()
      useDownloadStore.getState().markDone(taskId)
    } catch (e) {
      useDownloadStore.getState().markError(taskId, String(e))
      setError(String(e))
    } finally {
      unsub()
      downloadingRef.current = null
      setTimeout(() => useDownloadStore.getState().removeTask(taskId), 6000)
    }
  }

  const handleDownload = async (item: VersionItem) => {
    if (downloadingRef.current) return
    setDownloading(item.key)
    try {
      if (item.source === 'modrinth' && item.mrVersion) await doMrDownload(item.mrVersion)
      else if (item.source === 'curseforge' && item.cfFile) await doCfDownload(item.cfFile)
    } finally {
      setDownloading(null)
    }
  }

  const doMrDownload = async (v: ModrinthVersion) => {
    if (isModpack) {
      await startTask(v.name, () => invoke('install_modrinth_modpack', { versionId: v.id }))
    } else {
      if (!targetInstanceId) {
        setError('请先创建或选择目标实例')
        return
      }
      await startTask(v.name, () =>
        invoke('download_modrinth_mod', { versionId: v.id, instanceId: targetInstanceId, target: targetDir }),
      )
    }
  }

  const doCfDownload = async (f: CurseForgeFile) => {
    if (isModpack) {
      await startTask(f.display_name, () =>
        invoke('install_curseforge_modpack', { fileId: f.id, fileName: f.file_name, downloadUrl: f.download_url }),
      )
    } else {
      if (!targetInstanceId) {
        setError('请先创建或选择目标实例')
        return
      }
      await startTask(f.display_name, () =>
        invoke('download_file', { url: f.download_url, filename: f.file_name, instanceId: targetInstanceId, target: targetDir }),
      )
    }
  }

  const handleDownloadDeps = async (item: VersionItem) => {
    if (downloadingRef.current) return
    if (!item.mrVersion) return
    if (!targetInstanceId) {
      setError('请先创建或选择目标实例')
      return
    }
    const inst = instances.find((i) => i.id === targetInstanceId)
    const mcVersion = inst?.version_id || gameVersion || item.gameVersions[0] || ''
    const loader = instanceLoaderKey(inst?.modloader)
    setDownloading(item.key)
    try {
      const deps = await invoke<ModrinthDependency[]>('resolve_modrinth_dependencies', {
        versionId: item.mrVersion.id,
        instanceId: targetInstanceId,
        mcVersion,
        loader,
      })
      if (deps.length === 0) {
        setToast('此版本没有需要额外下载的前置模组（已有的会自动跳过）')
        return
      }
      for (const dep of deps) {
        if (!dep.version_id) continue
        const name = dep.file_name || `前置模组 ${dep.project_id || dep.version_id}`
        await startTask(name, () =>
          invoke('download_modrinth_mod', { versionId: dep.version_id, instanceId: targetInstanceId, target: targetDir }),
        )
      }
    } catch (e) {
      setError(String(e))
    } finally {
      setDownloading(null)
    }
  }

  const headerIcon = iconErr ? null : (mrResolved?.icon_url || cfResolved?.logo_url || null)
  const headerTitle = mrDetail?.title || mrResolved?.title || cfResolved?.name || '加载中...'
  const headerAuthor =
    mrDetail?.team?.join(', ') ||
    mrResolved?.author ||
    cfResolved?.authors?.join(', ') ||
    ''
  const headerDesc = mrDetail?.description || mrResolved?.description || cfResolved?.summary || ''
  const headerCats = mrDetail?.categories?.length ? mrDetail.categories : mrResolved?.categories || []

  const allVersions: VersionItem[] = [
    ...mrVersions.map((v) => {
      const file = v.files.find((f) => f.primary) || v.files[0]
      return {
        key: `mr-${v.id}`,
        name: v.name,
        versionNumber: v.version_number,
        date: v.date_published,
        gameVersions: v.game_versions,
        loaders: v.loaders,
        fileName: file?.filename || '',
        fileSize: file?.size || 0,
        source: 'modrinth' as const,
        resourceType: mrResourceType(v),
        gameType: classifyGameVersions(v.game_versions),
        mrVersion: v,
      }
    }),
    ...cfFiles.map((f) => ({
      key: `cf-${f.id}`,
      name: f.display_name,
      versionNumber: undefined as string | undefined,
      date: f.file_date,
      gameVersions: f.game_versions,
      loaders: f.loaders,
      fileName: f.file_name,
      fileSize: f.file_length,
      source: 'curseforge' as const,
      resourceType: cfResourceType(f),
      gameType: classifyGameVersions(f.game_versions),
      cfFile: f,
    })),
  ]

  const baseVersions = gameVersion
    ? allVersions.filter((v) => v.gameVersions.some((g) => g === gameVersion))
    : allVersions

  const grouped = LOADER_GROUPS.map((g) => ({
    ...g,
    items: baseVersions.filter((v) => loaderGroup(v.loaders) === g.key),
  })).filter((g) => g.items.length > 0)

  const activeKey: LoaderGroup | null =
    grouped.some((g) => g.key === activeLoader) ? activeLoader : (grouped[0]?.key ?? null)
  const activeGroup = grouped.find((g) => g.key === activeKey) ?? null

  const filteredItems = (activeGroup?.items ?? []).filter(
    (v) =>
      (gameFilter === 'all' || v.gameType === gameFilter) &&
      (resFilter === 'all' || v.resourceType === resFilter),
  )

  const allGameVersions = Array.from(new Set(
    filteredItems.flatMap((v) => v.gameVersions.filter((gv) => classifyMcVersion(gv) !== 'april'))
  )).sort(compareVersions)

  const versionGroups = new Map<string, VersionItem[]>()
  for (const item of filteredItems) {
    const gameVers = item.gameVersions.length > 0 ? item.gameVersions : ['未知版本']
    for (const gv of gameVers) {
      if (selectedGameVersion !== 'all' && gv !== selectedGameVersion) continue
      const existing = versionGroups.get(gv) || []
      existing.push(item)
      versionGroups.set(gv, existing)
    }
  }

  const sortedVersionGroups = Array.from(versionGroups.entries())
    .sort(([a], [b]) => compareVersions(a, b))
    .map(([version, items]) => ({
      version,
      items: items.sort((a, b) => {
        if (a.versionNumber && b.versionNumber) return compareVersions(a.versionNumber, b.versionNumber)
        if (a.versionNumber) return -1
        if (b.versionNumber) return 1
        return new Date(b.date).getTime() - new Date(a.date).getTime()
      }),
    }))

  const showSpinner = (mrLoading || cfLoading) && mrVersions.length === 0 && cfFiles.length === 0

  return (
    <div className="flex flex-col gap-4">
      {}
      <div className="flex-1 min-w-0 space-y-4">
        <button onClick={onBack} className="flex items-center gap-1 text-sm text-surface-500 hover:text-surface-700 cursor-pointer w-fit">
          <ArrowLeft className="w-4 h-4" /> 返回资源列表
        </button>

        {}
        <Card>
          <Box className="flex gap-4">
            {headerIcon ? (
              <img
                src={headerIcon}
                alt=""
                onError={() => setIconErr(true)}
                className="w-20 h-20 rounded-xl object-cover shrink-0"
              />
            ) : (
              <Box className="w-20 h-20 rounded-xl bg-surface-100 dark:bg-surface-800 flex items-center justify-center shrink-0">
                <PackageOpen className="w-10 h-10 text-surface-400" />
              </Box>
            )}
            <Box className="flex-1 min-w-0">
              <Typography variant="h6" className="truncate">{headerTitle}</Typography>
              {headerAuthor && <Typography variant="caption" color="text.secondary">{headerAuthor}</Typography>}
              {headerDesc && (
                <Typography variant="body2" color="text.secondary" className="mt-1 line-clamp-2">{headerDesc}</Typography>
              )}
              <Box className="flex items-center gap-3 flex-wrap mt-2">
                {(mrDetail || mrResolved) && (
                  <>
                    {mrDetail?.downloads !== undefined && mrDetail?.downloads !== null ? (
                      <span className="flex items-center gap-1 text-xs text-surface-500">
                        <DownloadIcon className="w-3.5 h-3.5" /> {formatDownloads(mrDetail.downloads)} 下载
                      </span>
                    ) : mrResolved?.downloads ? (
                      <span className="flex items-center gap-1 text-xs text-surface-500">
                        <DownloadIcon className="w-3.5 h-3.5" /> {formatDownloads(mrResolved.downloads)} 下载
                      </span>
                    ) : null}
                    {mrDetail?.follows ? (
                      <span className="flex items-center gap-1 text-xs text-surface-500">
                        <Heart className="w-3.5 h-3.5" /> {formatDownloads(mrDetail.follows)} 关注
                      </span>
                    ) : null}
                    {(mrDetail?.updated || mrResolved?.date_modified) && (
                      <span className="flex items-center gap-1 text-xs text-surface-500">
                        <Calendar className="w-3.5 h-3.5" /> {formatDate(mrDetail?.updated || mrResolved?.date_modified)}
                      </span>
                    )}
                  </>
                )}
                {cfProject && (
                  <>
                    <span className="flex items-center gap-1 text-xs text-surface-500">
                      <Flame className="w-3.5 h-3.5" /> {formatDownloads(cfProject.downloads)} 下载
                    </span>
                    <span className="flex items-center gap-1 text-xs text-surface-500">
                      <Calendar className="w-3.5 h-3.5" /> {formatDate(cfProject.date_modified)}
                    </span>
                  </>
                )}
              </Box>
              <Box className="flex items-center gap-1.5 flex-wrap mt-2">
                {headerCats.slice(0, 6).map((cat) => (
                  <Chip key={cat} label={cat} size="small" variant="outlined" />
                ))}
                {mrDetail?.license && <Chip label={mrDetail.license} size="small" variant="outlined" color="primary" />}
              </Box>
            </Box>
          </Box>
        </Card>

        {error && <AlertBox severity="error" onClose={() => setError(null)}>{error}</AlertBox>}
        <SnackbarAlert open={!!toast} onClose={() => setToast(null)} message={toast || ''} severity="info" autoHideDuration={3000} />

        {gameVersion && (
          <Box className="flex items-center gap-2 bg-accent-50 dark:bg-accent-500/10 border border-accent-200 dark:border-accent-500/30 rounded-xl px-3.5 py-2.5">
            <Chip label={`MC ${gameVersion}`} size="small" color="primary" />
            <Typography variant="body2" color="text.secondary" className="text-xs flex-1">
              已按游戏版本筛选，下方仅显示兼容 {gameVersion} 的版本
            </Typography>
            {onClearGameVersion && (
              <Button variant="text" size="small" onClick={onClearGameVersion} startIcon={<X className="w-3.5 h-3.5" />}>
                清除筛选
              </Button>
            )}
          </Box>
        )}

        {mrDetail && (
          <Card>
            <Typography variant="subtitle2" className="mb-2">项目介绍</Typography>
            <Box className="max-h-48 overflow-y-auto text-sm text-surface-600 dark:text-surface-400 whitespace-pre-line pr-2">
              {markdownToText(mrDetail.body) || mrDetail.description || '暂无介绍'}
            </Box>
            {mrDetail.source_url && (
              <button
                onClick={() => open(mrDetail.source_url!)}
                className="mt-2 flex items-center gap-1 text-xs text-accent-600 hover:underline cursor-pointer"
              >
                <ExternalLink className="w-3.5 h-3.5" /> 访问 Modrinth 主页
              </button>
            )}
          </Card>
        )}

        {!isModpack && instances.length > 0 && (
          <Card className="!p-3">
            <Box className="flex items-center gap-2">
              <DownloadIcon className="w-4 h-4 text-accent-400 shrink-0" />
              <Typography variant="caption" className="text-surface-400 shrink-0">下载到</Typography>
              <Select
                value={targetInstanceId}
                onChange={onTargetChange}
                size="small"
                options={instances.map((inst) => ({
                  value: inst.id,
                  label: `${inst.name} (${inst.version_id})${preferredInstanceId === inst.id ? ' ⭐' : ''}`,
                }))}
                className="min-w-[200px]"
              />
            </Box>
          </Card>
        )}

        {grouped.length > 0 && (
          <Tabs
            value={activeKey!}
            onChange={(v) => setActiveLoader(v as LoaderGroup)}
            items={grouped.map((g) => ({
              value: g.key,
              label: `${g.label} (${g.items.length})`,
              icon: <LoaderLogo loader={g.logo} className="w-4 h-4" />,
            }))}
          />
        )}

        {grouped.length > 0 && (
          <Box className="bg-white dark:bg-surface-850 border border-surface-200 rounded-xl p-3 shadow-xs space-y-2.5">
            <Box className="flex items-center gap-2 flex-wrap">
              <span className="text-xs text-surface-400 shrink-0 w-16">游戏版本</span>
              <div className="flex flex-wrap gap-1">
                <FilterChip active={gameFilter === 'all'} onClick={() => setGameFilter('all')}>全部</FilterChip>
                {GAME_GROUPS.map((g) => (
                  <FilterChip key={g.key} active={gameFilter === g.key} onClick={() => setGameFilter(g.key)}>
                    {g.label}
                  </FilterChip>
                ))}
              </div>
            </Box>
            <Box className="flex items-center gap-2 flex-wrap">
              <span className="text-xs text-surface-400 shrink-0 w-16">版本类型</span>
              <div className="flex flex-wrap gap-1">
                <FilterChip active={resFilter === 'all'} onClick={() => setResFilter('all')}>全部</FilterChip>
                {RESOURCE_TYPE_META.map((t) => (
                  <FilterChip key={t.key} active={resFilter === t.key} onClick={() => setResFilter(t.key)}>
                    {t.label}
                  </FilterChip>
                ))}
              </div>
            </Box>
            {allGameVersions.length > 0 && (
              <Box className="flex items-center gap-2">
                <span className="text-xs text-surface-400 shrink-0 w-16">MC 版本</span>
                <Select
                  value={selectedGameVersion}
                  onChange={setSelectedGameVersion}
                  options={[
                    { value: 'all', label: '全部游戏版本' },
                    ...allGameVersions.map((gv) => ({ value: gv, label: gv })),
                  ]}
                  size="small"
                  fullWidth={false}
                  className="min-w-[180px]"
                />
              </Box>
            )}
          </Box>
        )}

        {showSpinner ? (
          <Box className="flex justify-center py-8">
            <Loader2 className="w-6 h-6 animate-spin text-surface-400" />
          </Box>
        ) : grouped.length === 0 ? (
          <EmptyStateDetail title="暂无可下载的版本" desc="Modrinth 与 CurseForge 均暂无可用版本" />
        ) : sortedVersionGroups.length === 0 ? (
          <EmptyStateDetail title="没有符合筛选条件的版本" desc="试试切换游戏版本、加载器或版本类型筛选" />
        ) : (
          sortedVersionGroups.map((group) => {
            const isExpanded = expandedVersions.has(group.version)
            return (
              <Box key={group.version} className="space-y-0">
                <button
                  onClick={() => {
                    setExpandedVersions((prev) => {
                      const next = new Set(prev)
                      if (next.has(group.version)) next.delete(group.version)
                      else next.add(group.version)
                      return next
                    })
                  }}
                  className="w-full flex items-center gap-2 pt-2 pb-1 px-1 rounded-lg hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors cursor-pointer"
                >
                  {isExpanded ? (
                    <ChevronDown className="w-4 h-4 text-surface-400 shrink-0" />
                  ) : (
                    <ChevronRight className="w-4 h-4 text-surface-400 shrink-0" />
                  )}
                  <Typography variant="subtitle2" className="flex-1 text-left">
                    MC {group.version}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    {group.items.length} 个版本
                  </Typography>
                </button>
                {isExpanded && (
                  <Box className="space-y-2 pl-6 pt-1">
                    {group.items.map((item) => {
                      const busy = downloading === item.key
                      const resMeta = RESOURCE_TYPE_MAP[item.resourceType]
                      return (
                        <Card key={item.key}>
                          <Box className="flex items-center gap-4">
                            <Box className="min-w-0 flex-1 pr-2">
                              <Box className="flex items-center gap-2 flex-wrap">
                                <Typography variant="body2" className="font-medium">{item.name}</Typography>
                                {item.versionNumber && (
                                  <Typography variant="caption" color="text.secondary">v{item.versionNumber}</Typography>
                                )}
                                <Typography variant="caption" color="text.secondary">
                                  {formatDate(item.date)}
                                </Typography>
                                <span className={`px-1.5 py-0.5 rounded text-[10px] font-medium shrink-0 ${resMeta.cls}`}>
                                  {resMeta.label}
                                </span>
                                <span
                                  className={`px-1.5 py-0.5 rounded text-[10px] font-medium shrink-0 ${
                                    item.source === 'modrinth'
                                      ? 'bg-[#1bd96a]/10 text-[#17b35a]'
                                      : 'bg-[#f16436]/10 text-[#d55428]'
                                  }`}
                                >
                                  {item.source === 'modrinth' ? 'Modrinth' : 'CurseForge'}
                                </span>
                              </Box>
                              <Box className="flex flex-wrap gap-1.5 mt-1.5">
                                {item.loaders.slice(0, 4).map((ld) => (
                                  <Chip key={ld} label={ld} size="small" color="success" variant="outlined" />
                                ))}
                              </Box>
                              <Typography variant="caption" color="text.secondary" className="block mt-1.5">
                                {item.fileName} · {formatSize(item.fileSize)}
                              </Typography>
                            </Box>
                            <Box className="flex flex-col items-center gap-1.5 shrink-0 pl-3 border-l border-surface-200 dark:border-surface-700/60">
                              <Button size="small" loading={busy} onClick={() => handleDownload(item)}>
                                <DownloadIcon className="w-3.5 h-3.5 mr-1" />
                                {isModpack ? '安装' : '下载'}
                              </Button>
                              {item.source === 'modrinth' && !isModpack && (
                                <Button size="small" variant="outlined" loading={busy} onClick={() => handleDownloadDeps(item)}>
                                  <PackageOpen className="w-3.5 h-3.5 mr-1" />
                                  下载前置
                                </Button>
                              )}
                            </Box>
                          </Box>
                        </Card>
                      )
                    })}
                  </Box>
                )}
              </Box>
            )
          })
        )}

        {mcmod && (
          <Card>
            <Box className="space-y-3">
              <Box className="flex items-center gap-2">
                <Typography variant="body2" className="font-medium">{mcmod.title}</Typography>
                <Chip label="MC百科" size="small" color="primary" variant="outlined" />
              </Box>
              <Typography variant="body2" color="text.secondary">
                {mcmod.description || '该条目暂无百科简介'}
              </Typography>
              <Box className="flex items-center gap-2 flex-wrap">
                {mcmod.mcmod_url && (
                  <Button size="small" variant="outlined" startIcon={<LinkIcon className="w-3.5 h-3.5" />} onClick={() => open(mcmod.mcmod_url!)}>
                    打开 MC百科
                  </Button>
                )}
                {mcmod.modrinth_url && (
                  <Button size="small" variant="outlined" startIcon={<Globe className="w-3.5 h-3.5" />} onClick={() => open(mcmod.modrinth_url!)}>
                    打开 Modrinth
                  </Button>
                )}
                {mcmod.curseforge_url && (
                  <Button size="small" variant="outlined" startIcon={<BookOpen className="w-3.5 h-3.5" />} onClick={() => open(mcmod.curseforge_url!)}>
                    打开 CurseForge
                  </Button>
                )}
              </Box>
            </Box>
          </Card>
        )}
      </div>
    </div>
  )
}

function EmptyStateDetail({ title, desc }: { title: string; desc?: string }) {
  return (
    <Box className="py-10 text-center text-sm text-surface-400">
      <PackageOpen className="w-10 h-10 mx-auto mb-2 text-surface-300 dark:text-surface-600" />
      {title}
      {desc && <Box className="text-xs mt-1">{desc}</Box>}
    </Box>
  )
}

function FilterChip({
  active,
  onClick,
  children,
}: {
  active: boolean
  onClick: () => void
  children: React.ReactNode
}) {
  return (
    <button
      onClick={onClick}
      className={`px-2.5 py-1 rounded-lg text-xs font-medium transition-all duration-150 cursor-pointer ${
        active
          ? 'bg-accent-500 text-white shadow-sm'
          : 'bg-surface-100 dark:bg-surface-800 text-surface-500 hover:text-surface-700 dark:hover:text-surface-300'
      }`}
    >
      {children}
    </button>
  )
}
