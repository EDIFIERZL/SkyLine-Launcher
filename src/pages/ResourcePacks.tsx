import { useEffect, useState } from 'react'
import { useParams, useSearchParams, useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { Box, Typography, Card, IconButton, Input, Chip, Loading, EmptyState, AlertBox, Tabs } from '../components/material'
import { Eye, EyeOff, PackageOpen, Image, Box as BoxIcon, ArrowLeft } from 'lucide-react'

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

type PackType = 'resourcepacks' | 'shaderpacks'

const PACK_TABS = [
  { value: 'resourcepacks', label: '资源包', icon: <Image className="w-4 h-4" /> },
  { value: 'shaderpacks', label: '光影包', icon: <BoxIcon className="w-4 h-4" /> },
]

export function ResourcePacks() {
  const { instanceId } = useParams()
  const navigate = useNavigate()
  const [searchParams] = useSearchParams()
  const [packType, setPackType] = useState<PackType>(
    searchParams.get('type') === 'shaderpacks' ? 'shaderpacks' : 'resourcepacks'
  )
  const [packs, setPacks] = useState<PackInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [search, setSearch] = useState('')
  const [error, setError] = useState<string | null>(null)

  const loadPacks = async () => {
    if (!instanceId) return
    setLoading(true)
    setError(null)
    try {
      const cmd = packType === 'resourcepacks' ? 'scan_resource_packs' : 'scan_shader_packs'
      const res = await invoke<PackInfo[]>(cmd, { instanceId })
      setPacks(res)
    } catch (e) {
      setError(String(e))
    }
    setLoading(false)
  }

  useEffect(() => { loadPacks() }, [instanceId, packType])

  const togglePack = async (pack: PackInfo, enable: boolean) => {
    try {
      await invoke('toggle_resource_pack', { path: pack.path, enable })
      await loadPacks()
    } catch (e) {
      setError(String(e))
    }
  }

  const filtered = packs.filter((p) =>
    (p.name || p.file_name).toLowerCase().includes(search.toLowerCase())
  )

  return (
    <Box className="space-y-5 max-w-5xl pt-1">
      <Box className="flex items-center gap-3">
        <IconButton onClick={() => navigate(-1)}>
          <ArrowLeft className="w-5 h-5" />
        </IconButton>
        <Box>
          <Typography variant="h5">
            {packType === 'resourcepacks' ? '资源包' : '光影包'} 管理
          </Typography>
          <Typography variant="body2" color="text.secondary" className="mt-0.5">
            管理实例中的{packType === 'resourcepacks' ? '资源包' : '光影包'}
          </Typography>
        </Box>
      </Box>

      <Tabs
        items={PACK_TABS}
        value={packType}
        onChange={(v) => setPackType(v as PackType)}
      />

      <Input
        placeholder="搜索..."
        value={search}
        onChange={(e) => setSearch(e.target.value)}
      />

      {error && <AlertBox severity="error">{error}</AlertBox>}

      {loading ? (
        <Loading />
      ) : filtered.length > 0 ? (
        <Box className="space-y-3">
          {filtered.map((pack) => (
            <Card key={pack.path} className="p-5">
              <Box className="flex items-center gap-4">
                <Box className={`w-11 h-11 rounded-lg flex items-center justify-center text-lg shrink-0 overflow-hidden ${
                  pack.enabled ? 'bg-accent-50 dark:bg-accent-500/10 text-[var(--accent-color)]' : 'bg-surface-100 dark:bg-surface-800 text-surface-400'
                }`}>
                  {pack.icon_url ? (
                    <img src={pack.icon_url} alt={pack.name || pack.file_name} className="w-full h-full object-cover" />
                  ) : (
                    packType === 'resourcepacks' ? '📦' : '✨'
                  )}
                </Box>
                <Box className="flex-1 min-w-0 space-y-1">
                  <Box className="flex items-center gap-2 flex-wrap">
                    <Typography variant="subtitle2" className={pack.enabled ? '' : 'line-through opacity-50'}>
                      {pack.name || pack.file_name}
                    </Typography>
                    {pack.pack_format && (
                      <Chip label={`格式 ${pack.pack_format}`} size="small" variant="outlined" />
                    )}
                    <Typography variant="caption" color="text.secondary">
                      {(pack.size_kb / 1024).toFixed(1)} MB
                    </Typography>
                  </Box>
                  {pack.description && (
                    <Typography variant="body2" color="text.secondary" className="line-clamp-1">
                      {pack.description}
                    </Typography>
                  )}
                </Box>
                <Box className="flex gap-2 shrink-0">
                  <IconButton
                    title={pack.enabled ? '禁用' : '启用'}
                    onClick={() => togglePack(pack, !pack.enabled)}
                  >
                    {pack.enabled ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </IconButton>
                </Box>
              </Box>
            </Card>
          ))}
        </Box>
      ) : (
        <EmptyState
          icon={<PackageOpen className="w-12 h-12" />}
          title={`暂无${packType === 'resourcepacks' ? '资源包' : '光影包'}`}
          description={`将${packType === 'resourcepacks' ? '资源包' : '光影包'}放入 .minecraft/${packType} 目录`}
        />
      )}
    </Box>
  )
}
