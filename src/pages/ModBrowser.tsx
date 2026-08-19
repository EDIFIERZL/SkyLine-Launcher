import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Box, Typography, Card, Button, Input, Chip, Loading, EmptyState, AlertBox } from '../components/material'
import { Search, Download, ChevronLeft, ChevronRight, PackageOpen, ArrowLeft, ChevronDown, ChevronRight as ChevronRightIcon } from 'lucide-react'

interface ModrinthProject {
  slug: string
  title: string
  description: string
  versions: string[]
  client_side: string
  server_side: string
  categories: string[]
  license: string | null
  icon_url: string | null
  project_id: string | null
  author: string | null
  downloads: number | null
  date_modified: string | null
}

interface ModrinthVersion {
  id: string
  project_id: string
  name: string
  version_number: string
  game_versions: string[]
  loaders: string[]
  files: { url: string; filename: string; primary: boolean; size: number }[]
  date_published: string
  changelog: string | null
}

export function ModBrowser() {
  const [query, setQuery] = useState('')
  const [modrinthResults, setModrinthResults] = useState<ModrinthProject[]>([])
  const [loading, setLoading] = useState(false)
  const [offset, setOffset] = useState(0)
  const [selected, setSelected] = useState<ModrinthProject | null>(null)
  const [versions, setVersions] = useState<ModrinthVersion[]>([])
  const [versionsLoading, setVersionsLoading] = useState(false)
  const [downloading, setDownloading] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [expandedVersions, setExpandedVersions] = useState<Set<string>>(new Set())

  const search = useCallback(async (newOffset = 0) => {
    if (!query.trim()) return
    setLoading(true)
    setError(null)
    try {
      const res = await invoke<ModrinthProject[]>('search_modrinth_mods', { query, limit: 20, offset: newOffset })
      setModrinthResults(res)
      setOffset(newOffset)
    } catch (e) {
      setError(String(e))
    }
    setLoading(false)
  }, [query])

  useEffect(() => {
    if (query.trim()) search()
  }, [query])

  const selectProject = async (project: ModrinthProject) => {
    setSelected(project)
    setVersionsLoading(true)
    try {
      const v = await invoke<ModrinthVersion[]>('get_modrinth_versions', { projectId: project.project_id || project.slug })
      setVersions(v.sort((a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime()))
    } catch {
      setVersions([])
    }
    setVersionsLoading(false)
  }

  const downloadMod = async (version: ModrinthVersion) => {
    setDownloading(version.id)
    setError(null)
    try {
      await invoke('download_modrinth_mod', { versionId: version.id, instanceId: 'default' })
    } catch (e) {
      setError(String(e))
    }
    setDownloading(null)
  }

  const formatDownloads = (n: number | null) => {
    if (!n) return ''
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
    return String(n)
  }

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

  const versionGroups = new Map<string, ModrinthVersion[]>()
  for (const v of versions) {
    const gameVers = v.game_versions.length > 0 ? v.game_versions : ['未知版本']
    for (const gv of gameVers) {
      const existing = versionGroups.get(gv) || []
      existing.push(v)
      versionGroups.set(gv, existing)
    }
  }

  const sortedVersionGroups = Array.from(versionGroups.entries())
    .sort(([a], [b]) => compareVersions(a, b))
    .map(([version, vers]) => ({
      version,
      items: vers.sort((a, b) => compareVersions(a.version_number, b.version_number)),
    }))

  if (selected) {
    return (
      <Box className="space-y-4 max-w-5xl">
        <Button variant="text" startIcon={<ArrowLeft className="w-4 h-4" />} onClick={() => setSelected(null)}>
          返回
        </Button>
        <Card>
          <Box className="flex gap-4">
            {selected.icon_url && <img src={selected.icon_url} alt="" className="w-16 h-16 rounded-xl object-cover" />}
            <Box className="flex-1">
              <Typography variant="h6">{selected.title}</Typography>
              {selected.author && <Typography variant="body2" color="text.secondary">{selected.author}</Typography>}
              <Typography variant="body2" color="text.secondary" className="mt-1 line-clamp-2">{selected.description}</Typography>
            </Box>
          </Box>
        </Card>
        <Typography variant="subtitle1">可用版本</Typography>
        {versionsLoading ? (
          <Loading />
        ) : (
          <Box className="space-y-1">
            {sortedVersionGroups.length === 0 && <Typography variant="body2" color="text.secondary" className="text-center py-8">暂无版本</Typography>}
            {sortedVersionGroups.map((group) => {
              const isExpanded = expandedVersions.has(group.version)
              return (
                <Box key={group.version}>
                  <button
                    onClick={() => {
                      setExpandedVersions((prev) => {
                        const next = new Set(prev)
                        if (next.has(group.version)) next.delete(group.version)
                        else next.add(group.version)
                        return next
                      })
                    }}
                    className="w-full flex items-center gap-2 py-2 px-1 rounded-lg hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors cursor-pointer"
                  >
                    {isExpanded ? (
                      <ChevronDown className="w-4 h-4 text-surface-400 shrink-0" />
                    ) : (
                      <ChevronRightIcon className="w-4 h-4 text-surface-400 shrink-0" />
                    )}
                    <Typography variant="subtitle2" className="flex-1 text-left">
                      MC {group.version}
                    </Typography>
                    <Typography variant="caption" color="text.secondary">
                      {group.items.length} 个版本
                    </Typography>
                  </button>
                  {isExpanded && (
                    <Box className="space-y-2 pl-6 pt-1 pb-2">
                      {group.items.map((v) => {
                        const file = v.files.find(f => f.primary) || v.files[0]
                        return (
                          <Card key={v.id}>
                            <Box className="flex items-center justify-between gap-4">
                              <Box className="min-w-0 flex-1">
                                <Box className="flex items-center gap-2">
                                  <Typography variant="subtitle2">{v.name}</Typography>
                                  <Typography variant="caption" color="text.secondary">v{v.version_number}</Typography>
                                </Box>
                                <Box className="flex flex-wrap gap-1.5 mt-1.5">
                                  {v.loaders.map((loader) => (
                                    <Chip key={loader} label={loader} size="small" color="success" variant="outlined" />
                                  ))}
                                </Box>
                                {file && <Typography variant="caption" color="text.secondary" className="mt-1 block">{file.filename} ({(file.size / 1024 / 1024).toFixed(1)} MB)</Typography>}
                              </Box>
                              <Button size="small" startIcon={<Download className="w-3.5 h-3.5" />} onClick={() => downloadMod(v)} loading={downloading === v.id}>
                                下载
                              </Button>
                            </Box>
                          </Card>
                        )
                      })}
                    </Box>
                  )}
                </Box>
              )
            })}
          </Box>
        )}
        {error && <AlertBox severity="error">{error}</AlertBox>}
      </Box>
    )
  }

  return (
    <Box className="space-y-4 max-w-5xl">
      <Typography variant="h5">模组浏览</Typography>

      <form onSubmit={(e) => { e.preventDefault(); search() }} className="input-action-row">
        <Box className="flex-1">
          <Input placeholder="搜索模组..." value={query} onChange={(e) => setQuery(e.target.value)} />
        </Box>
        <Button type="submit" startIcon={<Search className="w-4 h-4" />} loading={loading}>搜索</Button>
      </form>

      {error && <AlertBox severity="error">{error}</AlertBox>}

      {loading ? (
        <Loading />
      ) : modrinthResults.length > 0 ? (
        <>
          <Box className="grid grid-cols-1 md:grid-cols-2 gap-3">
            {modrinthResults.map((mod) => (
              <Card key={mod.slug} hoverable onClick={() => selectProject(mod)}>
                <Box className="flex gap-3">
                  {mod.icon_url && <img src={mod.icon_url} alt="" className="w-12 h-12 rounded-lg object-cover shrink-0" />}
                  <Box className="min-w-0 flex-1">
                    <Typography variant="subtitle2" className="truncate">{mod.title}</Typography>
                    {mod.author && <Typography variant="caption" color="text.secondary">{mod.author}</Typography>}
                    <Typography variant="body2" color="text.secondary" className="mt-1 line-clamp-2">{mod.description}</Typography>
                    <Box className="flex items-center gap-2 mt-2">
                      {mod.downloads && <Typography variant="caption" color="text.secondary">{formatDownloads(mod.downloads)} 下载</Typography>}
                      {mod.categories.slice(0, 2).map((cat) => (
                        <Chip key={cat} label={cat} size="small" variant="outlined" />
                      ))}
                    </Box>
                  </Box>
                </Box>
              </Card>
            ))}
          </Box>
          <Box className="flex justify-center gap-2">
            <Button variant="outlined" size="small" disabled={offset === 0} startIcon={<ChevronLeft className="w-4 h-4" />} onClick={() => search(offset - 20)}>
              上一页
            </Button>
            <Button variant="outlined" size="small" endIcon={<ChevronRight className="w-4 h-4" />} onClick={() => search(offset + 20)}>
              下一页
            </Button>
          </Box>
        </>
      ) : (
        <EmptyState
          icon={<PackageOpen className="w-12 h-12" />}
          title="输入关键词开始搜索"
        />
      )}
    </Box>
  )
}
