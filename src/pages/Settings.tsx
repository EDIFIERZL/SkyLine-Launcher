import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { open as shellOpen } from '@tauri-apps/plugin-shell'
import { useSettingsStore } from '../stores/settingsStore'
import { triggerSilentOptimize } from '../hooks/useMemoryOptimizer'
import { Box, Typography, Card, Button, Input, Select, Switch, Tabs, Chip, Slider, DialogBox } from '../components/material'
import type { JavaInfo, LauncherConfig, InstallProgress } from '../types'
import { Coffee, Palette, Monitor, Sun, Moon, Image, Sparkles, Maximize, CloudDownload, CheckCircle2, AlertCircle, Play as PlayIcon, Zap, Gamepad2, Info, GitBranch, ExternalLink, Heart, FolderOpen, ArrowRightLeft } from 'lucide-react'
import { WINDOW_PRESETS, applyWindowSize } from '../utils/windowSize'

const accentPresets = [
  { name: '天蓝', color: '#3b82f6' },
  { name: '翠绿', color: '#10b981' },
  { name: '紫色', color: '#8b5cf6' },
  { name: '橙色', color: '#f59e0b' },
  { name: '玫红', color: '#ec4899' },
  { name: '青色', color: '#06b6d4' },
  { name: '红色', color: '#ef4444' },
  { name: '靛蓝', color: '#6366f1' },
  { name: '粉色', color: '#f472b6' },
  { name: '琥珀', color: '#d97706' },
  { name: '石灰', color: '#84cc16' },
  { name: '石墨', color: '#64748b' },
]

const gradientPresets = [
  { name: '深邃蓝', value: 'linear-gradient(135deg, #0f172a 0%, #1e3a8a 55%, #312e81 100%)' },
  { name: '暮色紫', value: 'linear-gradient(135deg, #1e1b4b 0%, #6d28d9 60%, #9d174d 100%)' },
  { name: '深林绿', value: 'linear-gradient(135deg, #022c22 0%, #065f46 55%, #047857 100%)' },
  { name: '海洋蓝', value: 'linear-gradient(135deg, #0c4a6e 0%, #0e7490 50%, #0ea5e9 100%)' },
  { name: '落日橙', value: 'linear-gradient(135deg, #431407 0%, #9a3412 55%, #f97316 100%)' },
  { name: '酒红', value: 'linear-gradient(135deg, #3f0d12 0%, #8b1c37 50%, #be123c 100%)' },
  { name: '墨黑', value: 'linear-gradient(135deg, #020617 0%, #111827 55%, #1f2937 100%)' },
  { name: '森林', value: 'linear-gradient(135deg, #052e16 0%, #166534 55%, #22c55e 100%)' },
  { name: '天空之境', value: 'linear-gradient(180deg, #0ea5e9 0%, #38bdf8 45%, #e0f2fe 100%)' },
  { name: '极光', value: 'linear-gradient(135deg, #042f2e 0%, #0f766e 45%, #134e4a 100%)' },
]

const SETTINGS_TABS = [
  { value: 'appearance', label: '外观', icon: <Palette className="w-4 h-4" /> },
  { value: 'game', label: '游戏设置', icon: <Gamepad2 className="w-4 h-4" /> },
  { value: 'launcher', label: '启动器', icon: <Monitor className="w-4 h-4" /> },
  { value: 'about', label: '关于', icon: <Info className="w-4 h-4" /> },
]

const CREDIT_LIST = [
  { name: 'LiChenghao', avatar: '/LiChenghao.jpg', url: 'https://github.com/chenghaolee-2012' },
  { name: 'ESexplorerZDC', avatar: '/ESexplorerZDC.png', url: 'https://github.com/ExplorerMediaGroup' },
  { name: 'Yukino_fox', avatar: '/Yukino_fox.jpg', url: 'https://github.com/Yukino-fox' },
  { name: 'ZhouZhouo_O', avatar: '/ZhouZhouo_O.jpg', url: 'https://github.com/ZhouZhou-oO' },
  { name: 'Soloev', avatar: '/Soloev.png', url: 'https://github.com/0xarch' },
  { name: 'MC百科', avatar: '', url: 'https://www.mcmod.cn/' },
]

function CreditAvatar({ name, src }: { name: string; src: string }) {
  const [failed, setFailed] = useState(false)
  const showImg = src && !failed
  return (
    <Box className="w-10 h-10 rounded-full overflow-hidden shrink-0 bg-surface-100 dark:bg-surface-800 flex items-center justify-center">
      {showImg ? (
        <img src={src} alt={name} className="w-full h-full object-cover" onError={() => setFailed(true)} />
      ) : (
        <Typography variant="subtitle2" className="text-surface-400 font-semibold">
          {name.charAt(0)}
        </Typography>
      )}
    </Box>
  )
}

export function Settings() {
  const { config, setConfig } = useSettingsStore()
  const [javaList, setJavaList] = useState<JavaInfo[]>([])
  const [localConfig, setLocalConfig] = useState<LauncherConfig>(config)
  const [activeTab, setActiveTab] = useState('appearance')
  const [downloadingJava, setDownloadingJava] = useState<number | null>(null)
  const [javaProgress, setJavaProgress] = useState('')
  const [javaError, setJavaError] = useState<string | null>(null)
  const [totalMem, setTotalMem] = useState(16384)
  const [memUsedPercent, setMemUsedPercent] = useState(0)
  const [optimizing, setOptimizing] = useState(false)
  const [optimizeResult, setOptimizeResult] = useState('')
  const [lgConfirmOpen, setLgConfirmOpen] = useState(false)
  const [gameFolder, setGameFolder] = useState(localConfig.game_folder ?? '')
  const [migrating, setMigrating] = useState(false)
  const [migrationResult, setMigrationResult] = useState<string | null>(null)
  const [showMigrationDialog, setShowMigrationDialog] = useState(false)
  const [pendingOldFolder, setPendingOldFolder] = useState('')

  useEffect(() => {
    invoke<LauncherConfig>('load_config').then((cfg) => {
      setConfig(cfg)
      setLocalConfig(cfg)
    }).catch(() => {})
    invoke<JavaInfo[]>('detect_java').then(setJavaList).catch((e) => {
      console.error('Java detection failed:', e)
      setJavaError('Java 检测失败')
    })
    invoke<number>('get_total_memory').then((mb) => {
      if (mb > 0) setTotalMem(mb)
    }).catch(() => {})
    invoke<number>('get_memory_used_percent').then(setMemUsedPercent).catch(() => {})
    const timer = setInterval(() => {
      invoke<number>('get_memory_used_percent').then(setMemUsedPercent).catch(() => {})
    }, 2000)
    return () => clearInterval(timer)
  }, [])

  const handleOptimizeMemory = async () => {
    setOptimizing(true)
    setOptimizeResult('')
    try {
      const before = await invoke<number>('get_memory_used_percent')
      const [pct, freed] = await invoke<[number, number]>('optimize_memory')
      setMemUsedPercent(pct)
      const after = pct
      setOptimizeResult(
        after < before
          ? `内存占用由 ${before}% 降至 ${after}%，共释放 ${freed} MB`
          : `深度整理完成，当前占用 ${after}%，释放 ${freed} MB`
      )
    } catch (e) {
      setOptimizeResult(`内存优化失败: ${e}`)
    } finally {
      setOptimizing(false)
      setTimeout(() => setOptimizeResult(''), 8000)
    }
  }

  const hasJava = (major: number) =>
    javaList.some((j) => j.major_version === major && j.is_64bit)

  const handleDownloadJava = async (major: number) => {
    setDownloadingJava(major)
    setJavaProgress('准备下载...')
    const unsub = await listen<InstallProgress>('install-progress', (e) => {
      if (e.payload.stage === 'java') {
        setJavaProgress(e.payload.message)
      }
    })
    try {
      await invoke('download_java', { majorVersion: major })
      setJavaProgress('安装完成')
      const list = await invoke<JavaInfo[]>('detect_java')
      setJavaList(list)
    } catch (e) {
      setJavaProgress(`下载失败: ${e}`)
    } finally {
      unsub()
      setDownloadingJava(null)
      setTimeout(() => setJavaProgress(''), 3000)
    }
  }

  const [bgError, setBgError] = useState<string | null>(null)

  const handlePickMedia = async (kind: 'image' | 'video') => {
    setBgError(null)
    try {
      const filters =
        kind === 'video'
          ? [{ name: '视频', extensions: ['mp4', 'webm', 'mov', 'm4v', 'ogg'] }]
          : [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'svg', 'avif', 'ico'] }]
      const selected = await open({
        multiple: false,
        title: kind === 'video' ? '选择启动器背景视频' : '选择启动器背景图片',
        filters,
      })
      if (typeof selected === 'string' && selected) {
        const media = await invoke<{ kind: string; data_uri: string }>('read_background_media', { path: selected })
        updateConfig({ background_value: media.data_uri, background_type: media.kind })
      }
    } catch (e) {
      console.error('选择背景失败:', e)
      setBgError(`选择失败: ${e}`)
    }
  }

  const handleRemoveMedia = () => {
    updateConfig({ background_value: '', background_type: 'none' })
  }

  const updateConfig = (patch: Partial<LauncherConfig>) => {
    const next = { ...localConfig, ...patch }
    setLocalConfig(next)
    setConfig(next)
    invoke('save_config', { config: next }).catch(() => {})
  }

  const handleChange = <K extends keyof LauncherConfig>(key: K, value: LauncherConfig[K]) => {
    
    if (key === 'background_type' && (value === 'image' || value === 'video')) {
      if (localConfig.liquid_glass && localConfig.liquid_glass_mode === 'transparent' && !localConfig.background_value) {
        return
      }
    }
    updateConfig({ [key]: value } as Partial<LauncherConfig>)
    
    setTimeout(() => triggerSilentOptimize(), 500)
  }

  const confirmLiquidGlass = () => {
    setLgConfirmOpen(false)
    if (localConfig.background_type === 'blur') {
      updateConfig({ liquid_glass: true, background_type: 'none' })
    } else {
      updateConfig({ liquid_glass: true })
    }
  }

  return (
    <Box className="max-w-4xl space-y-6">
      <Box>
        <Typography variant="h5">设置</Typography>
        <Typography variant="body2" color="text.secondary">配置启动器行为和个性化外观</Typography>
      </Box>

      <Tabs
        items={SETTINGS_TABS}
        value={activeTab}
        onChange={setActiveTab}
      />

      {activeTab === 'appearance' && (
        <Box className="space-y-5">
          <Card>
            <Box className="space-y-4">
              <Typography variant="subtitle1" className="flex items-center gap-2">
                <Palette className="w-4 h-4 text-[var(--accent-color)]" /> 主题
              </Typography>
              <Box className="flex gap-2">
                {[
                  { value: 'light', label: '浅色', icon: <Sun className="w-5 h-5" /> },
                  { value: 'dark', label: '深色', icon: <Moon className="w-5 h-5" /> },
                  { value: 'system', label: '跟随系统', icon: <Monitor className="w-5 h-5" /> },
                ].map((opt) => (
                  <div key={opt.value} className="flex-1">
                    <Button
                      variant={localConfig.theme_mode === opt.value ? 'contained' : 'outlined'}
                      startIcon={opt.icon}
                      onClick={() => handleChange('theme_mode', opt.value)}
                      fullWidth
                    >
                      {opt.label}
                    </Button>
                  </div>
                ))}
              </Box>
            </Box>
          </Card>

          <Card>
            <Box className="space-y-4">
              <Typography variant="subtitle1" className="flex items-center gap-2">
                <Sparkles className="w-4 h-4 text-[var(--accent-color)]" /> 强调色
              </Typography>
              <Box className="flex flex-wrap gap-2.5">
                {accentPresets.map((preset) => (
                  <button
                    key={preset.color}
                    onClick={() => handleChange('accent_color', preset.color)}
                    className={`w-9 h-9 rounded-xl transition-all duration-150 cursor-pointer relative ${
                      localConfig.accent_color === preset.color
                        ? 'ring-2 ring-offset-2 ring-accent-500 scale-110'
                        : 'hover:scale-105'
                    }`}
                    style={{ backgroundColor: preset.color }}
                    title={preset.name}
                  >
                    {localConfig.accent_color === preset.color && (
                      <svg className="absolute inset-0 w-full h-full p-2 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3">
                        <polyline points="20 6 9 17 4 12" />
                      </svg>
                    )}
                  </button>
                ))}
                <div className="relative w-9 h-9">
                  <input
                    type="color"
                    value={localConfig.accent_color}
                    onChange={(e) => handleChange('accent_color', e.target.value)}
                    className="absolute inset-0 w-full h-full rounded-xl border border-surface-300 cursor-pointer opacity-0"
                  />
                  <div className="w-full h-full rounded-xl bg-surface-100 border border-surface-300 flex items-center justify-center text-xs text-surface-500 font-medium">
                    +
                  </div>
                </div>
              </Box>
            </Box>
          </Card>

          <Card>
            <Box className="space-y-4">
              <Typography variant="subtitle1" className="flex items-center gap-2">
                <Image className="w-4 h-4 text-[var(--accent-color)]" /> 背景
              </Typography>
              <Box className="flex gap-2">
                {[
                  { value: 'none', label: '无' },
                  { value: 'gradient', label: '渐变' },
                  { value: 'blur', label: '模糊' },
                  { value: 'image', label: '图片' },
                  { value: 'video', label: '视频' },
                ].map((opt) => {
                  const disabled =
                    (opt.value === 'blur' && localConfig.liquid_glass) ||
                    ((opt.value === 'image' || opt.value === 'video') && localConfig.liquid_glass && localConfig.liquid_glass_mode === 'transparent')
                  return (
                    <div key={opt.value} className="flex-1" title={disabled ? '当前模式不可用' : undefined}>
                      <Button
                        variant={localConfig.background_type === opt.value ? 'contained' : 'outlined'}
                        onClick={() => handleChange('background_type', opt.value)}
                        disabled={disabled}
                        fullWidth
                      >
                        {opt.label}
                      </Button>
                    </div>
                  )
                })}
              </Box>
              {localConfig.background_type === 'gradient' && (
                <Box className="space-y-3">
                  <Typography variant="caption" color="text.secondary">
                    选择预设渐变背景（点击应用）
                  </Typography>
                  <Box className="grid grid-cols-5 gap-2">
                    {gradientPresets.map((g) => (
                      <button
                        key={g.name}
                        onClick={() => handleChange('background_value', g.value)}
                        title={g.name}
                        className={`h-14 rounded-xl transition-all duration-150 cursor-pointer relative ${
                          localConfig.background_value === g.value
                            ? 'ring-2 ring-offset-2 ring-accent-500 scale-105'
                            : 'hover:scale-105'
                        }`}
                        style={{ backgroundImage: g.value, backgroundSize: 'cover', backgroundPosition: 'center' }}
                      >
                        {localConfig.background_value === g.value && (
                          <svg className="absolute inset-0 w-full h-full p-2 text-white drop-shadow" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3">
                            <polyline points="20 6 9 17 4 12" />
                          </svg>
                        )}
                      </button>
                    ))}
                  </Box>

                </Box>
              )}
              {localConfig.background_type === 'blur' && (
                <Typography variant="body2" color="text.secondary">
                  {localConfig.liquid_glass
                    ? '开启液态玻璃后不可使用模糊背景，请先关闭液态玻璃效果。'
                    : '背景将使用当前强调色的模糊光晕效果'}
                </Typography>
              )}
              {(localConfig.background_type === 'image' || localConfig.background_type === 'video') && (
                <Box className="space-y-3">
                  <Box className="flex gap-2">
                    {localConfig.background_type === 'image' && (
                      <Button
                        variant="contained"
                        startIcon={<Image className="w-4 h-4" />}
                        onClick={() => handlePickMedia('image')}
                      >
                        选择本地图片
                      </Button>
                    )}
                    {localConfig.background_type === 'video' && (
                      <Button
                        variant="contained"
                        startIcon={<PlayIcon className="w-4 h-4" />}
                        onClick={() => handlePickMedia('video')}
                      >
                        选择本地视频
                      </Button>
                    )}
                    {localConfig.background_value && (
                      <Button variant="outlined" onClick={handleRemoveMedia}>
                        移除
                      </Button>
                    )}
                  </Box>
                  {bgError && (
                    <Box className="flex items-center gap-2 px-3 py-2 rounded-lg bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800">
                      <AlertCircle className="w-4 h-4 text-red-500 shrink-0" />
                      <Typography variant="caption" color="error">{bgError}</Typography>
                    </Box>
                  )}
                  {localConfig.background_value && (
                    <Box className="relative rounded-xl overflow-hidden border border-surface-200 dark:border-surface-700 h-40 bg-surface-100 dark:bg-surface-800">
                      {localConfig.background_type === 'video' ? (
                        <video
                          src={localConfig.background_value}
                          className="w-full h-full object-cover"
                          autoPlay
                          muted
                          loop
                          playsInline
                        />
                      ) : (
                        <img
                          src={localConfig.background_value}
                          alt="启动器背景预览"
                          className="w-full h-full object-cover"
                        />
                      )}
                      <Box className="absolute inset-0 flex items-end p-2 bg-gradient-to-t from-black/40 to-transparent">
                        <Typography variant="caption" color="#fff">
                          {localConfig.background_type === 'video' ? '背景视频预览（循环播放）' : '启动器背景预览'}
                        </Typography>
                      </Box>
                    </Box>
                  )}
                </Box>
              )}
            </Box>
          </Card>

          <Card>
            <Box className="space-y-3">
              <Typography variant="subtitle1" className="flex items-center gap-2">
                <Sparkles className="w-4 h-4 text-[var(--accent-color)]" /> 液态玻璃效果
              </Typography>
              <Switch
                checked={localConfig.liquid_glass}
                onChange={(v) => {
                  if (v) {
                    setLgConfirmOpen(true)
                  } else {
                    updateConfig({ liquid_glass: false })
                  }
                }}
                label="启用液态玻璃界面"
              />
              {localConfig.liquid_glass && (
                <Box className="space-y-3 pl-4 border-l-2 border-[var(--accent-color)]/30">
                  <Typography variant="caption" color="text.secondary">
                    选择液态玻璃模式
                  </Typography>
                  <Box className="flex gap-2">
                    {[
                      { value: 'normal', label: '(高性能)液态玻璃', desc: '内置渐变光晕背景' },
                      { value: 'transparent', label: '透明液态玻璃', desc: '背景透明，可看到桌面' },
                    ].map((opt) => (
                      <div key={opt.value} className="flex-1" title={opt.desc}>
                        <Button
                          variant={localConfig.liquid_glass_mode === opt.value ? 'contained' : 'outlined'}
                          onClick={() => {
                            if (opt.value === 'transparent') {
                              updateConfig({ liquid_glass_mode: opt.value, background_type: 'none', background_value: '' })
                            } else {
                              updateConfig({ liquid_glass_mode: opt.value })
                            }
                          }}
                          fullWidth
                        >
                          {opt.label}
                        </Button>
                      </div>
                    ))}
                  </Box>
                  {localConfig.liquid_glass_mode === 'normal' && (
                    <Box className="flex items-start gap-2 px-3 py-2 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800">
                      <AlertCircle className="w-4 h-4 text-blue-500 shrink-0 mt-0.5" />
                      <Typography variant="caption" color="text.secondary">
                        高性能模式下可搭配图片或视频背景使用。
                        使用图片/视频背景时，首页右下角可开启全屏沉浸模式隐藏所有界面组件。低配设备慎用。
                      </Typography>
                    </Box>
                  )}
                  {localConfig.liquid_glass_mode === 'transparent' && (
                    <Box className="flex items-start gap-2 px-3 py-2 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800">
                      <AlertCircle className="w-4 h-4 text-amber-500 shrink-0 mt-0.5" />
                      <Typography variant="caption" color="text.secondary">
                        透明模式下，未设置背景图片/视频时背景将变为透明，可直接看到桌面。
                        此模式不支持图片及视频背景，低配设备慎用。
                      </Typography>
                    </Box>
                  )}
                </Box>
              )}
            </Box>
          </Card>
        </Box>
      )}

      {activeTab === 'game' && (
        <Box className="space-y-5">
          <Card>
            <Box className="space-y-5">
              <Typography variant="subtitle1" className="flex items-center gap-2">
                <Coffee className="w-4 h-4 text-[var(--accent-color)]" /> Java 运行时
              </Typography>

              <Select
                label="Java 路径"
                value={localConfig.java_path ?? 'auto'}
                onChange={(v) => handleChange('java_path', v === 'auto' ? null : v)}
                options={[
                  { value: 'auto', label: '自动检测' },
                  ...javaList.map((j) => ({
                    value: j.path,
                    label: `Java ${j.major_version} · ${j.path}`,
                  })),
                ]}
              />

              <Box>
                <Typography variant="subtitle2" className="mb-2">一键下载 Java 运行时</Typography>
                <Box className="grid grid-cols-2 sm:grid-cols-4 gap-2">
                  {[8, 11, 17, 21].map((major) => (
                    <Button
                      key={major}
                      variant={hasJava(major) ? 'outlined' : 'contained'}
                      startIcon={hasJava(major) ? <CheckCircle2 className="w-4 h-4" /> : <CloudDownload className="w-4 h-4" />}
                      onClick={() => handleDownloadJava(major)}
                      loading={downloadingJava === major}
                      disabled={hasJava(major) && downloadingJava !== major}
                      fullWidth
                    >
                      Java {major} {hasJava(major) && '已就绪'}
                    </Button>
                  ))}
                </Box>
                {downloadingJava && <Typography variant="caption" color="text.secondary" className="mt-2">{javaProgress}</Typography>}
                {javaProgress.includes('失败') && <Typography variant="caption" color="error" className="mt-1">{javaProgress}</Typography>}
                <Typography variant="caption" color="text.secondary" className="mt-2 block">
                  下载 Temurin (Eclipse Adoptium) 运行时：1.7 及以下用 Java 8，1.17+ 用 Java 17，1.20.5+ 用 Java 21。
                </Typography>
              </Box>

              {javaList.length > 0 ? (
                <Box className="space-y-1">
                  {javaList.map((j, i) => (
                    <Box key={i} className="flex items-center gap-2 px-3 py-1.5 bg-surface-50 dark:bg-surface-800 rounded-lg">
                      <Box className="w-2 h-2 rounded-full bg-green-500 shrink-0" />
                      <Typography variant="caption" className="flex-1">
                        Java {j.major_version} ({j.vendor}) - {j.path}
                      </Typography>
                      <Box className="flex gap-1.5">
                        {j.is_jdk && <Chip label="JDK" size="small" variant="outlined" />}
                        {j.is_64bit ? (
                          <Chip label="64-bit" size="small" color="success" variant="outlined" />
                        ) : (
                          <Chip label="32-bit" size="small" color="warning" variant="outlined" />
                        )}
                      </Box>
                    </Box>
                  ))}
                </Box>
              ) : (
                <Box className="flex items-center gap-2 px-3 py-2 bg-surface-50 dark:bg-surface-800 rounded-lg">
                  <AlertCircle className="w-4 h-4 text-surface-400" />
                  <Typography variant="caption" color="text.secondary">
                    {javaError || '未检测到 Java，请使用上方按钮下载'}
                  </Typography>
                </Box>
              )}
            </Box>
          </Card>

          <Card>
            <Box className="space-y-4">
              <Typography variant="subtitle1" className="flex items-center gap-2">
                <Monitor className="w-4 h-4 text-[var(--accent-color)]" /> 内存分配
              </Typography>
              <Box className="space-y-4">
                <Slider
                  label={`最大内存 (${localConfig.max_memory} MB = ${(localConfig.max_memory / 1024).toFixed(1)} GB)`}
                  value={localConfig.max_memory}
                  min={400}
                  max={totalMem}
                  step={64}
                  color="primary"
                  onChange={(v) => handleChange('max_memory', v)}
                />
                <Typography variant="caption" color="text.secondary">
                  系统总内存 {Math.round(totalMem / 1024)} GB，建议最大内存不超过总内存的 3/4。最小内存固定 400MB。
                </Typography>
              </Box>
            </Box>
          </Card>

          <Card>
            <Box className="space-y-3">
              <Typography variant="subtitle1" className="flex items-center gap-2">
                <Monitor className="w-4 h-4 text-[var(--accent-color)]" /> OpenGL 兼容模式（软件渲染）
              </Typography>
              <Switch
                checked={localConfig.opengl_compat ?? false}
                onChange={(v) => handleChange('opengl_compat', v)}
                label="开启软件渲染（无 GPU 加速环境下运行旧版本 Minecraft）"
              />
              <Typography variant="caption" color="text.secondary">
                远程桌面（RDP）、虚拟机或显卡驱动缺失环境下启动旧版 MC（≤1.12.2）会报「Pixel format not accelerated」。
                开启后启动器将自动注入软件渲染参数，使用 GDI 软渲染来启动游戏（画面帧率较低但可运行）。
                新版本（≥1.13）的 LWJGL3 不支持软渲染，建议检查显卡驱动或使用物理机。
              </Typography>
            </Box>
          </Card>

          <Card>
            <Box className="space-y-3">
              <Typography variant="subtitle1" className="flex items-center gap-2">
                <Zap className="w-4 h-4 text-[var(--accent-color)]" /> 内存优化
              </Typography>
              <Box className="flex items-center gap-3 px-3 py-2 bg-surface-50 dark:bg-surface-800 rounded-lg">
                <Box className="flex-1">
                  <Typography variant="body2">当前内存占用</Typography>
                  <Typography variant="caption" color="text.secondary">
                    总内存 {Math.round(totalMem / 1024)} GB · 占用 {memUsedPercent}%
                  </Typography>
                </Box>
                  <Button
                    variant="contained"
                    startIcon={<Zap className="w-4 h-4" />}
                    onClick={handleOptimizeMemory}
                    loading={optimizing}
                    className="shrink-0"
                  >
                    {optimizing ? '深度整理中...' : '深度优化内存'}
                  </Button>
              </Box>
              {optimizeResult && (
                <Typography variant="caption" color="success">
                  {optimizeResult}
                </Typography>
              )}
            </Box>
          </Card>
        </Box>
      )}

      {activeTab === 'launcher' && (
        <Card>
          <Box className="space-y-4">
            <Typography variant="subtitle1" className="flex items-center gap-2">
              <Monitor className="w-4 h-4 text-[var(--accent-color)]" /> 启动器设置
            </Typography>
            <Box className="space-y-4">
              <Slider
                label={`下载线程数`}
                min={1}
                max={512}
                value={localConfig.download_threads}
                onChange={(v) => handleChange('download_threads', v)}
              />
              <Box className="pt-2 border-t border-surface-200 dark:border-surface-700 mt-4">
                <Typography variant="subtitle2" className="flex items-center gap-2 mb-3">
                  <FolderOpen className="w-4 h-4 text-[var(--accent-color)]" /> 游戏文件夹
                </Typography>
                <Box className="flex items-center gap-2 mb-2">
                  <Input
                    label="游戏存储路径"
                    value={gameFolder}
                    onChange={(e) => setGameFolder(e.target.value)}
                    placeholder="例如: D:\Games\Minecraft"
                    fullWidth
                  />
                  <Button
                    variant="outlined"
                    size="small"
                    onClick={async () => {
                      const selected = await open({ directory: true, title: '选择游戏文件夹' })
                      if (typeof selected === 'string' && selected) {
                        setGameFolder(selected)
                      }
                    }}
                  >
                    浏览
                  </Button>
                </Box>
                <Box className="flex items-center gap-2 flex-wrap">
                  <Button
                    size="small"
                    variant="contained"
                    disabled={!gameFolder || gameFolder === localConfig.game_folder}
                    onClick={async () => {
                      const oldFolder = localConfig.game_folder ?? ''
                      if (oldFolder && oldFolder !== gameFolder) {
                        setPendingOldFolder(oldFolder)
                        setShowMigrationDialog(true)
                      } else {
                        handleChange('game_folder', gameFolder || null)
                        setMigrationResult(null)
                      }
                    }}
                  >
                    保存
                  </Button>
                  {migrationResult && (
                    <span className="text-xs text-green-500">{migrationResult}</span>
                  )}
                </Box>
                {localConfig.game_folder && (
                  <Typography variant="caption" color="text.secondary" className="block mt-1.5">
                    当前：{localConfig.game_folder}
                  </Typography>
                )}
              </Box>
            </Box>
            <Box className="grid grid-cols-2 gap-4">
              <Select
                label="窗口大小"
                value={localConfig.window_size}
                onChange={(v) => {
                  handleChange('window_size', v)
                  const cfg = { ...localConfig, window_size: v }
                  applyWindowSize(cfg).catch(() => {})
                }}
                options={WINDOW_PRESETS.map((p) => ({ value: p.value, label: p.label }))}
              />
              <Select
                label="游戏下载源"
                value={localConfig.download_source}
                onChange={(v) => handleChange('download_source', v)}
                options={[
                  { value: 'auto', label: '自动（官方优先，慢/失败自动切镜像）' },
                  { value: 'official', label: '官方源（Mojang 等）' },
                  { value: 'mirror', label: '国内镜像（BMCLAPI）' },
                ]}
              />
            </Box>
            <Box className="flex items-start gap-2">
              <CloudDownload className="w-4 h-4 text-surface-400 shrink-0 mt-0.5" />
              <Typography variant="caption" color="text.secondary">
                默认使用官方源下载（速度快且完整）；若下载缓慢或失败，可在上方切换为「国内镜像（BMCLAPI）」，推荐网络环境一般的用户使用。
              </Typography>
            </Box>
            {localConfig.window_size === 'custom' && (
              <Box className="grid grid-cols-2 gap-4">
                <Input label="自定义宽度" type="number" value={String(localConfig.window_width)} onChange={(e) => {
                  const w = Number(e.target.value)
                  handleChange('window_width', w)
                  applyWindowSize({ ...localConfig, window_width: w }).catch(() => {})
                }} />
                <Input label="自定义高度" type="number" value={String(localConfig.window_height)} onChange={(e) => {
                  const h = Number(e.target.value)
                  handleChange('window_height', h)
                  applyWindowSize({ ...localConfig, window_height: h }).catch(() => {})
                }} />
              </Box>
            )}
            {localConfig.window_size !== 'fullscreen' && (
              <Box className="flex items-center gap-2">
                <Maximize className="w-4 h-4 text-surface-400" />
                <Typography variant="caption" color="text.secondary">
                  当前预设：{WINDOW_PRESETS.find((p) => p.value === localConfig.window_size)?.label ?? localConfig.window_size}
                  {localConfig.window_size === 'custom' ? `（${localConfig.window_width}×${localConfig.window_height}）` : ''}
                </Typography>
              </Box>
            )}
            <Switch
              checked={localConfig.keep_launcher_open}
              onChange={(v) => handleChange('keep_launcher_open', v)}
              label="启动游戏后保持启动器打开"
            />
            <Switch
              checked={localConfig.close_after_launch}
              onChange={(v) => handleChange('close_after_launch', v)}
              label="启动游戏后最小化启动器到后台"
            />
            <Box className="pt-2 border-t border-surface-200 dark:border-surface-700">
              <Typography variant="subtitle2" className="mb-2 flex items-center gap-2">
                <Monitor className="w-4 h-4 text-[var(--accent-color)]" /> 服务器状态监控
              </Typography>
              <Box className="grid grid-cols-2 gap-3">
                <Input
                  label="服务器名称"
                  value={localConfig.server_name}
                  onChange={(e) => handleChange('server_name', e.target.value)}
                  placeholder="我的服务器"
                />
                <Input
                  label="服务器地址"
                  value={localConfig.server_address}
                  onChange={(e) => handleChange('server_address', e.target.value)}
                  placeholder="play.example.com:25565"
                />
              </Box>
              <Switch
                checked={localConfig.hide_server_card}
                onChange={(v) => handleChange('hide_server_card', v)}
                label="隐藏首页服务器卡片"
              />
              {!localConfig.hide_server_card && (
                <Box className="mt-2">
                  <Typography variant="caption" color="text.secondary" className="mb-1 block">
                    服务器卡片大小
                  </Typography>
                  <Box className="flex items-center gap-3">
                    <Typography variant="caption" color="text.secondary" className="text-[10px]">小</Typography>
                    <Slider
                      value={localConfig.server_card_size ?? 80}
                      onChange={(v) => handleChange('server_card_size', v)}
                      min={40}
                      max={160}
                      step={1}
                      className="flex-1"
                    />
                    <Typography variant="caption" color="text.secondary" className="text-[10px]">大</Typography>
                    <Typography variant="caption" color="text.secondary" className="w-10 text-right text-[10px]">{localConfig.server_card_size ?? 80}px</Typography>
                  </Box>
                </Box>
              )}
              <Typography variant="caption" color="text.secondary" className="mt-1 block">
                首页将实时显示服务器状态（每 15 秒自动刷新）
              </Typography>
              <Box className="border-t border-surface-200 dark:border-surface-700 pt-3 mt-1">
                <Switch
                  checked={localConfig.hide_mp_quick_card}
                  onChange={(v) => handleChange('hide_mp_quick_card', v)}
                  label="隐藏首页多人游戏快速进入"
                />
                <Typography variant="caption" color="text.secondary" className="mt-1 block">
                  隐藏首页侧边栏的多人游戏服务器快速进入列表
                </Typography>
              </Box>
            </Box>
          </Box>
        </Card>
      )}

      {activeTab === 'about' && (
        <Box className="space-y-5 max-w-3xl">
          <Card>
            <Box className="flex items-start gap-4">
              <Box className="flex flex-col items-center gap-1.5 shrink-0">
                <img src="/logo.png" alt="SkyLine" className="w-16 h-16 rounded-2xl object-contain" />
                <Typography variant="caption" color="text.secondary" className="text-[11px]">v1.0.0</Typography>
              </Box>
              <Box className="flex-1 min-w-0">
                <Typography variant="h6" className="font-bold">SkyLine Launcher</Typography>
                <Typography variant="body2" color="text.secondary" className="mt-0.5">
                  功能全面的MINECRAFT启动器，让玩游戏更方便
                </Typography>
              </Box>
            </Box>

            <Box className="border-t border-surface-200 dark:border-surface-700 mt-4 pt-4 flex items-center gap-3 flex-wrap">
              <img
                src="/EDIFIER_ZL.jpg"
                alt="EDIFIER_ZL"
                className="w-10 h-10 rounded-full object-cover"
              />
              <Box className="min-w-0 flex-1">
                <Typography variant="subtitle2" className="font-medium">EDIFIER_ZL</Typography>
                <Typography variant="caption" color="text.secondary">制作人</Typography>
              </Box>
              <Box className="flex items-center gap-2">
                <Button
                  size="small"
                  variant="outlined"
                  startIcon={<PlayIcon className="w-3.5 h-3.5" />}
                  onClick={() => shellOpen('https://space.bilibili.com/3546886107564779?spm_id_from=333.1007.0.0')}
                >
                  Bilibili
                </Button>
                <Button
                  size="small"
                  variant="outlined"
                  startIcon={<GitBranch className="w-3.5 h-3.5" />}
                  onClick={() => shellOpen('https://github.com/EDIFIERZL')}
                >
                  GitHub
                </Button>
              </Box>
            </Box>
          </Card>

          <Card>
            <Box className="flex items-center gap-2 mb-4">
              <Heart className="w-4 h-4 text-rose-500" />
              <Typography variant="h6" className="font-bold text-base">特别鸣谢</Typography>
              <Typography variant="caption" color="text.secondary">（排序不分贡献大小）</Typography>
            </Box>
            <Box className="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
              {CREDIT_LIST.map((c) => (
                <button
                  key={c.name}
                  onClick={() => shellOpen(c.url)}
                  className="flex items-center gap-3 px-3 py-2.5 rounded-xl bg-surface-50 dark:bg-surface-800/60 hover:bg-surface-100 dark:hover:bg-surface-700/60 transition-colors cursor-pointer text-left"
                >
                  <CreditAvatar name={c.name} src={c.avatar} />
                  <Box className="min-w-0 flex-1">
                    <Typography variant="body2" className="font-medium truncate">{c.name}</Typography>
                    <Typography variant="caption" color="text.secondary" className="block truncate max-w-[180px]">{c.url}</Typography>
                  </Box>
                  <ExternalLink className="w-3.5 h-3.5 text-surface-400 shrink-0" />
                </button>
              ))}
            </Box>
          </Card>

          <Card>
            <Box className="flex items-center gap-3">
              <img src={encodeURI('/四维空间.png')} alt="四维空间工作室" className="w-12 h-12 rounded-xl object-contain bg-surface-100 dark:bg-surface-800 p-1" />
              <Box>
                <Typography variant="subtitle2" className="font-semibold">版权所有</Typography>
                <Typography variant="body2" color="text.secondary" className="mt-0.5">
                  版权归四维空间工作室及贡献者所有
                </Typography>
              </Box>
            </Box>
          </Card>
        </Box>
      )}

      <DialogBox open={lgConfirmOpen} onClose={() => setLgConfirmOpen(false)} title="启用液态玻璃效果" maxWidth="xs">
        <Typography variant="body2" color="text.secondary" className="py-1">
          启用液态玻璃会调整界面视觉效果（如背景为模糊时将被关闭）。确认启用吗？
        </Typography>
        <Box className="flex justify-end gap-2 pt-4">
          <Button variant="contained" color="primary" onClick={confirmLiquidGlass}>
            确认启用
          </Button>
          <Button variant="contained" color="error" onClick={() => setLgConfirmOpen(false)}>
            取消
          </Button>
        </Box>
      </DialogBox>

      <DialogBox open={showMigrationDialog} onClose={() => setShowMigrationDialog(false)} title="迁移游戏文件夹">
        <Box className="space-y-3">
          <Typography variant="body2" color="text.secondary">
            检测到游戏文件夹路径已更改。是否将旧文件夹中的所有文件迁移到新位置？
          </Typography>
          {pendingOldFolder && (
            <Box className="bg-surface-100 dark:bg-surface-800 rounded-lg p-3 space-y-2 text-sm">
              <Box className="flex items-center gap-2">
                <ArrowRightLeft className="w-4 h-4 text-accent-500 shrink-0" />
                <Box className="min-w-0 flex-1">
                  <Typography variant="caption" color="text.secondary">旧路径</Typography>
                  <Typography variant="body2" className="break-all font-mono text-xs">{pendingOldFolder}</Typography>
                </Box>
              </Box>
              <Box className="flex items-center gap-2">
                <ArrowRightLeft className="w-4 h-4 text-accent-500 shrink-0" />
                <Box className="min-w-0 flex-1">
                  <Typography variant="caption" color="text.secondary">新路径</Typography>
                  <Typography variant="body2" className="break-all font-mono text-xs">{gameFolder}</Typography>
                </Box>
              </Box>
            </Box>
          )}
          <Typography variant="caption" color="warning" className="block">
            迁移过程中请勿关闭启动器。迁移完成后旧文件夹将被删除。
          </Typography>
          <Box className="flex justify-end gap-2 pt-2">
            <Button variant="outlined" onClick={() => {
              setShowMigrationDialog(false)
              handleChange('game_folder', gameFolder || null)
              setMigrationResult(null)
            }}>
              仅修改设置（不迁移）
            </Button>
            <Button
              variant="contained"
              loading={migrating}
              onClick={async () => {
                setShowMigrationDialog(false)
                setMigrating(true)
                setMigrationResult(null)
                try {
                  await invoke<{copied_count:number,copied_size:number,instance_count:number}>('migrate_game_folder', {
                    old_path: pendingOldFolder,
                    new_path: gameFolder,
                  })
                  handleChange('game_folder', gameFolder || null)
                  setMigrationResult(`迁移完成！已复制文件并更新 ${pendingOldFolder} → ${gameFolder}，更新实例数：${gameFolder}`)
                  setGameFolder(gameFolder)
                } catch (e) {
                  setMigrationResult(`迁移失败: ${e}`)
                } finally {
                  setMigrating(false)
                }
              }}
            >
              确认迁移
            </Button>
          </Box>
        </Box>
      </DialogBox>
    </Box>
  )
}
