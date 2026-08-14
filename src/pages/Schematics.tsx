import { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { invoke } from '@tauri-apps/api/core'
import { Box, Typography, Card, IconButton } from '@/components/material'
import { ArrowLeft, FileText, FolderOpen } from 'lucide-react'

interface SchematicInfo {
  file_name: string
  path: string
  size_kb: number
  enabled: boolean
}

export function Schematics() {
  const { instanceId } = useParams()
  const navigate = useNavigate()
  const [schemas, setSchemas] = useState<SchematicInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const loadSchemas = async () => {
    if (!instanceId) return
    setLoading(true)
    setError(null)
    try {
      const res = await invoke<SchematicInfo[]>('scan_schematics', { instanceId })
      setSchemas(res)
    } catch (e) {
      setError(String(e))
    }
    setLoading(false)
  }

  useEffect(() => { loadSchemas() }, [instanceId])

  const formatSize = (kb: number): string => {
    if (kb < 1024) return `${kb} KB`
    return `${(kb / 1024).toFixed(1)} MB`
  }

  return (
    <Box className="h-full flex flex-col">
      {}
      <div className="shrink-0 px-4 py-3 border-b border-white/5 flex items-center gap-3">
        <IconButton onClick={() => navigate(-1)}>
          <ArrowLeft className="w-4 h-4 text-surface-400" />
        </IconButton>
        <div className="flex-1 min-w-0">
          <h2 className="text-sm font-medium text-surface-200">原理图管理</h2>
          <p className="text-[11px] text-surface-500 mt-0.5">
            存放 .litematic 文件（WorldEdit 导出格式）
          </p>
        </div>
        <IconButton
          onClick={() => invoke('open_instance_folder', { instanceId: instanceId!, subdir: 'schematics' })}
          title="打开文件夹"
        >
          <FolderOpen className="w-4 h-4 text-surface-400" />
        </IconButton>
      </div>

      {}
      <div className="flex-1 overflow-y-auto p-4">
        {loading ? (
          <div className="flex items-center justify-center py-16">
            <Typography variant="body2" color="text.secondary">加载中...</Typography>
          </div>
        ) : error ? (
          <Card className="py-8">
            <Typography variant="body2" color="error" className="text-center">{error}</Typography>
          </Card>
        ) : schemas.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16 gap-3">
            <FileText className="w-12 h-12 text-surface-700" />
            <Typography variant="body2" color="text.secondary" className="text-center max-w-md">
              该实例暂无原理图文件<br />
              <span className="text-xs text-surface-500">将 .litematic 文件放入实例目录的 schematics/ 文件夹</span>
            </Typography>
            <IconButton
              onClick={() => invoke('open_instance_folder', { instanceId: instanceId!, subdir: 'schematics' })}
            >
              <FolderOpen className="w-4 h-4" />
              <span className="text-xs">打开 schematics 文件夹</span>
            </IconButton>
          </div>
        ) : (
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3">
            {schemas.map((s) => (
              <Card
                key={s.path}
                className="!p-3 cursor-pointer hover:border-accent-500/40 transition-colors group"
                onClick={() => {
                  
                  invoke('open_file', { path: s.path }).catch(() => {})
                }}
              >
                <div className="flex flex-col gap-2">
                  <div className="w-full aspect-square rounded-lg bg-cyan-500/10 border border-cyan-500/20 flex items-center justify-center group-hover:bg-cyan-500/20 transition-colors">
                    <FileText className="w-8 h-8 text-cyan-400" />
                  </div>
                  <div className="min-w-0">
                    <Typography variant="caption" className="block truncate font-medium text-surface-200 text-[11px]" title={s.file_name}>
                      {s.file_name}
                    </Typography>
                    <Typography variant="caption" color="text.secondary" className="text-[10px] mt-0.5">
                      {formatSize(s.size_kb)}
                    </Typography>
                  </div>
                </div>
              </Card>
            ))}
          </div>
        )}
      </div>
    </Box>
  )
}
