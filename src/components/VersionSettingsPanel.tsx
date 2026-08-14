import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from '../stores/settingsStore'
import { useInstanceStore } from '../stores/instanceStore'
import { Box, Typography, Card, Button, Input, Select, Slider } from './material'
import type { JavaInfo, LauncherConfig, VersionSetting } from '../types'
import { X, Boxes } from 'lucide-react'

interface Props {
  open: boolean
  onClose: () => void
}

export function VersionSettingsPanel({ open, onClose }: Props) {
  const { config, setConfig } = useSettingsStore()
  const { instances } = useInstanceStore()
  const [javaList, setJavaList] = useState<JavaInfo[]>([])
  const [selected, setSelected] = useState('')
  const [setting, setSetting] = useState<VersionSetting>({
    java_path: null,
    min_memory: null,
    max_memory: null,
    game_dir_override: null,
  })
  const [visible, setVisible] = useState(open)
  const [closing, setClosing] = useState(false)
  const [totalMem, setTotalMem] = useState(16384)
  const globalMax = config.max_memory

  useEffect(() => {
    if (open) {
      setVisible(true)
      setClosing(false)
    } else if (visible) {
      setClosing(true)
      const t = setTimeout(() => { setVisible(false); setClosing(false) }, 180)
      return () => clearTimeout(t)
    }
  }, [open])

  useEffect(() => {
    if (instances.length > 0) setSelected(instances[0].id)
    invoke<JavaInfo[]>('detect_java').then(setJavaList).catch(() => setJavaList([]))
    invoke<number>('get_total_memory').then((mb) => setTotalMem(mb)).catch(() => {})
  }, [instances])

  const applySetting = (vs: VersionSetting | undefined) => {
    setSetting({
      java_path: vs?.java_path ?? null,
      min_memory: vs?.min_memory ?? null,
      max_memory: vs?.max_memory ?? null,
      game_dir_override: vs?.game_dir_override ?? null,
    })
  }

  useEffect(() => {
    applySetting(config.version_settings?.[selected])
  }, [selected, config.version_settings])

  const handleSave = async () => {
    const version_settings = { ...config.version_settings, [selected]: setting }
    const next: LauncherConfig = { ...config, version_settings }
    setConfig(next)
    await invoke('save_config', { config: next }).catch(console.error)
    onClose()
  }

  const javaOptions = [
    { value: '', label: '自动检测' },
    ...javaList.map((j) => ({
      value: j.path,
      label: `Java ${j.major_version} (${j.version})${j.is_64bit ? ' - 64位' : ''}`,
    })),
  ]

  const selectedInstance = instances.find(i => i.id === selected)

  if (!visible) return null

  return (
    <Box className={`absolute inset-0 z-20 flex items-center justify-center ${closing ? 'fade-out' : 'fade-in'}`} style={{ background: 'rgba(0,0,0,0.5)', backdropFilter: 'blur(4px)' }} onClick={onClose}>
      <Box
        className={`w-[480px] max-h-[85%] overflow-y-auto border border-surface-200 dark:border-surface-700 rounded-2xl shadow-2xl p-5 space-y-4 ${closing ? 'pop-out' : 'pop-in'}`}
        style={{ background: 'var(--vs-panel-bg, rgb(248,250,252))' }}
        onClick={(e) => e.stopPropagation()}
      >
        <Box className="flex items-center gap-2">
          <Boxes className="w-5 h-5 text-[var(--accent-color)]" />
          <Typography variant="h6">版本设置</Typography>
          <Box className="flex-1" />
          <button
            onClick={onClose}
            className="w-7 h-7 flex items-center justify-center rounded-lg text-surface-500 hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </Box>

        <Select
          label="选择实例"
          value={selected}
          onChange={setSelected}
          options={instances.map((inst) => ({
            value: inst.id,
            label: `${inst.name}${inst.external ? ' (外部)' : ''} - ${inst.version_id}`,
          }))}
        />

        {selectedInstance && (
          <Card className="p-3">
            <Box className="space-y-1 text-xs">
              <Typography variant="caption" color="text.secondary">
                版本: {selectedInstance.version_id} | 加载器: {JSON.stringify(selectedInstance.modloader)}
                {selectedInstance.external && ' | 来源: 外部实例文件夹'}
              </Typography>
            </Box>
          </Card>
        )}

        {selected && (
          <>
            <Select
              label="Java 路径"
              value={setting.java_path ?? ''}
              onChange={(v) => setSetting({ ...setting, java_path: v || null })}
              options={javaOptions}
            />
            <Box className="space-y-3">
              <Slider
                label={`最大内存 (${setting.max_memory ?? '使用默认'} ${setting.max_memory != null ? 'MB' : ''})`}
                value={setting.max_memory ?? globalMax}
                min={400}
                max={Math.max(400, totalMem)}
                step={64}
                onChange={(v) => setSetting({ ...setting, max_memory: v })}
              />
              <Box className="flex flex-wrap gap-1.5">
                {[
                  { name: '默认', max: null },
                  { name: '低配', max: 2048 },
                  { name: '平衡', max: 4096 },
                  { name: '高配', max: 8192 },
                  { name: '顶配', max: 16384 },
                ].map((p) => {
                  const isActive = setting.max_memory === p.max
                  return (
                    <button
                      key={p.name}
                      onClick={() => setSetting({ ...setting, max_memory: p.max })}
                      className={`px-2 py-1 rounded-lg text-xs font-medium transition-all duration-150 cursor-pointer border ${
                        isActive
                          ? 'bg-[var(--accent-color)]/15 border-[var(--accent-color)]/50 text-[var(--accent-color)]'
                          : 'bg-surface-50 dark:bg-surface-800 border-surface-200 dark:border-surface-700 text-surface-600 dark:text-surface-300 hover:bg-surface-100'
                      }`}
                    >
                      {p.name}
                    </button>
                  )
                })}
                <Typography variant="caption" color="text.secondary" className="self-center ml-1">
                  空白 = 使用全局设置
                </Typography>
              </Box>
            </Box>
            <Input
              className="mt-2"
              value={setting.game_dir_override ?? ''}
              placeholder="自定义游戏目录（可选，覆盖共享目录）"
              onChange={(e) => setSetting({ ...setting, game_dir_override: e.target.value || null })}
            />
          </>
        )}

        <Box className="flex gap-2 justify-end pt-1">
          <Button variant="text" onClick={onClose}>取消</Button>
          <Button onClick={handleSave} disabled={!selected}>保存设置</Button>
        </Box>
      </Box>
    </Box>
  )
}
