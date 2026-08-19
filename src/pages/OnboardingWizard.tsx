import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useSettingsStore } from '../stores/settingsStore'
import { Typography } from '@/components/material'
import { Sun, Moon, LayoutDashboard, LayoutList, CheckCircle, ChevronRight, ChevronLeft, Image as ImageIcon, Film } from 'lucide-react'
import type { LauncherConfig } from '../types'

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

interface OnboardingWizardProps {
  onComplete: () => void
}

export function OnboardingWizard({ onComplete }: OnboardingWizardProps) {
  const { config, setConfig } = useSettingsStore()
  const [step, setStep] = useState(0)
  const [localConfig, setLocalConfig] = useState<Partial<LauncherConfig>>({
    theme_mode: config.theme_mode || 'dark',
    accent_color: config.accent_color || '#3b82f6',
    home_style: config.home_style || 'full',
    background_type: config.background_type || 'none',
    background_value: config.background_value || '',
  })

  const update = (patch: Partial<LauncherConfig>) => {
    setLocalConfig(prev => ({ ...prev, ...patch }))
  }

  useEffect(() => {
    const isDark = localConfig.theme_mode === 'dark'
      || (localConfig.theme_mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
    document.documentElement.classList.toggle('dark', isDark)
  }, [localConfig.theme_mode])

  const applyTheme = (mode: string) => {
    update({ theme_mode: mode })
  }

  const applyAccent = (color: string) => {
    update({ accent_color: color })
    document.documentElement.style.setProperty('--accent-color', color)
  }

  const handlePickBackground = async (type: 'image' | 'video') => {
    try {
      const files = await open({
        multiple: false,
        filters: type === 'image'
          ? [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif'] }]
          : [{ name: '视频', extensions: ['mp4', 'webm'] }],
      })
      if (!files) return
      const filePath = typeof files === 'string' ? files : files[0]
      update({ background_type: type, background_value: filePath })
    } catch (e) {
      console.error('Failed to pick background:', e)
    }
  }

  const complete = async () => {
    const finalCfg: LauncherConfig = {
      ...config,
      ...localConfig,
      onboarding_completed: true,
    }
    setConfig(finalCfg)
    await invoke('save_config', { config: finalCfg })
    onComplete()
  }

  const steps = [
    {
      title: '选择主题',
      desc: '选择你喜欢的明暗风格',
      content: (
        <div className="flex gap-4 justify-center mt-6">
          {[
            { value: 'dark', label: '深色', icon: <Moon className="w-8 h-8" /> },
            { value: 'light', label: '浅色（beta）', icon: <Sun className="w-8 h-8" /> },
          ].map((opt) => (
            <div
              key={opt.value}
              onClick={() => applyTheme(opt.value)}
              className={`flex flex-col items-center gap-3 p-6 rounded-2xl cursor-pointer transition-all duration-200 border-2 ${
                localConfig.theme_mode === opt.value
                  ? 'border-[var(--accent-color)] bg-[var(--accent-color)]/10'
                  : 'border-surface-200 dark:border-surface-700 hover:border-surface-300 dark:hover:border-surface-600'
              }`}
              style={{ minWidth: 140 }}
            >
              <div className={`p-3 rounded-xl ${localConfig.theme_mode === opt.value ? 'bg-[var(--accent-color)]/20 text-[var(--accent-color)]' : 'bg-surface-100 dark:bg-surface-800 text-surface-500 dark:text-surface-400'}`}>
                {opt.icon}
              </div>
              <Typography variant="subtitle1" className="font-semibold text-surface-900 dark:text-surface-100">{opt.label}</Typography>
            </div>
          ))}
        </div>
      ),
    },
    {
      title: '选择强调色',
      desc: '选择启动器的主题色',
      content: (
        <div className="flex flex-wrap gap-3 justify-center mt-6 max-w-md mx-auto">
          {accentPresets.map((preset) => (
            <button
              key={preset.color}
              onClick={() => applyAccent(preset.color)}
              className={`w-11 h-11 rounded-xl transition-all duration-150 cursor-pointer relative ${
                localConfig.accent_color === preset.color
                  ? 'ring-2 ring-offset-2 ring-offset-white dark:ring-offset-surface-900 scale-110'
                  : 'hover:scale-105'
              }`}
              style={{ backgroundColor: preset.color }}
              title={preset.name}
            >
              {localConfig.accent_color === preset.color && (
                <svg className="absolute inset-0 w-full h-full p-2.5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3">
                  <polyline points="20 6 9 17 4 12" />
                </svg>
              )}
            </button>
          ))}
          <div className="relative w-11 h-11">
            <input
              type="color"
              value={localConfig.accent_color || '#3b82f6'}
              onChange={(e) => applyAccent(e.target.value)}
              className="absolute inset-0 w-full h-full rounded-xl cursor-pointer opacity-0"
            />
            <div className="w-full h-full rounded-xl bg-surface-100 dark:bg-surface-800 border border-surface-300 dark:border-surface-600 flex items-center justify-center text-sm text-surface-500 dark:text-surface-400 font-medium">
              +
            </div>
          </div>
        </div>
      ),
    },
    {
      title: '背景设置',
      desc: '选择启动器背景图片',
      content: (
        <div className="flex flex-col items-center gap-4 mt-6 w-full">
          <div className="flex gap-3">
            <button
              onClick={() => handlePickBackground('image')}
              className="flex flex-col items-center gap-2 px-5 py-4 rounded-2xl border-2 border-dashed transition-all hover:scale-105 active:scale-95"
              style={{
                borderColor: localConfig.background_type === 'image' ? localConfig.accent_color : undefined,
                backgroundColor: localConfig.background_type === 'image' ? `${localConfig.accent_color}15` : undefined,
                minWidth: 120,
              }}
            >
              <ImageIcon className="w-6 h-6" style={{ color: localConfig.background_type === 'image' ? localConfig.accent_color : undefined }} />
              <span className="text-xs font-medium" style={{ color: localConfig.background_type === 'image' ? localConfig.accent_color : undefined }}>照片</span>
            </button>
            <button
              onClick={() => handlePickBackground('video')}
              className="flex flex-col items-center gap-2 px-5 py-4 rounded-2xl border-2 border-dashed transition-all hover:scale-105 active:scale-95"
              style={{
                borderColor: localConfig.background_type === 'video' ? localConfig.accent_color : undefined,
                backgroundColor: localConfig.background_type === 'video' ? `${localConfig.accent_color}15` : undefined,
                minWidth: 120,
              }}
            >
              <Film className="w-6 h-6" style={{ color: localConfig.background_type === 'video' ? localConfig.accent_color : undefined }} />
              <span className="text-xs font-medium" style={{ color: localConfig.background_type === 'video' ? localConfig.accent_color : undefined }}>视频</span>
            </button>
          </div>
          {localConfig.background_value && (
            <div className="relative w-48 h-28 rounded-xl overflow-hidden border border-surface-200 dark:border-surface-700">
              {localConfig.background_type === 'video' ? (
                <video src={localConfig.background_value} className="w-full h-full object-cover" muted loop playsInline />
              ) : (
                <img src={localConfig.background_value} alt="背景预览" className="w-full h-full object-cover" />
              )}
              <div className="absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 hover:opacity-100 transition-opacity cursor-pointer" onClick={() => update({ background_type: 'none', background_value: '' })}>
                <span className="text-white text-xs font-medium">移除</span>
              </div>
            </div>
          )}
          {!localConfig.background_value && (
            <Typography variant="body2" className="text-xs text-center text-surface-500 dark:text-surface-400">
              点击上方按钮选择图片或视频
            </Typography>
          )}
        </div>
      ),
    },
    {
      title: '首页风格',
      desc: '选择你喜欢的首页布局',
      content: (
        <div className="flex gap-4 justify-center mt-6">
          {[
            { value: 'full', label: '完整模式', desc: '显示实例详情、模组、截图等', icon: <LayoutList className="w-8 h-8" /> },
            { value: 'minimal', label: '简洁模式', desc: '干净清爽，聚焦启动', icon: <LayoutDashboard className="w-8 h-8" /> },
          ].map((opt) => (
            <div
              key={opt.value}
              onClick={() => update({ home_style: opt.value })}
              className={`flex flex-col items-center gap-3 p-6 rounded-2xl cursor-pointer transition-all duration-200 border-2 ${
                localConfig.home_style === opt.value
                  ? 'border-[var(--accent-color)] bg-[var(--accent-color)]/10'
                  : 'border-surface-200 dark:border-surface-700 hover:border-surface-300 dark:hover:border-surface-600'
              }`}
              style={{ minWidth: 160 }}
            >
              <div className={`p-3 rounded-xl ${localConfig.home_style === opt.value ? 'bg-[var(--accent-color)]/20 text-[var(--accent-color)]' : 'bg-surface-100 dark:bg-surface-800 text-surface-500 dark:text-surface-400'}`}>
                {opt.icon}
              </div>
              <Typography variant="subtitle1" className="font-semibold text-surface-900 dark:text-surface-100">{opt.label}</Typography>
              <Typography variant="caption" className="text-center text-xs text-surface-500 dark:text-surface-400">{opt.desc}</Typography>
            </div>
          ))}
        </div>
      ),
    },
  ]

  const isLast = step === steps.length - 1
  const current = steps[step]

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-surface-50 dark:bg-surface-900">
      <div className="w-full max-w-lg mx-4">
        <div className="bg-white dark:bg-surface-800 rounded-3xl shadow-2xl overflow-hidden border border-surface-200/60 dark:border-surface-700/40">
          <div className="px-8 pt-8 pb-4">
            <Typography variant="h5" className="font-bold text-center text-surface-900 dark:text-surface-100">{current.title}</Typography>
            <Typography variant="body2" className="text-center mt-1 text-surface-500 dark:text-surface-400">{current.desc}</Typography>
          </div>

          <div className="px-8 pb-6 min-h-[240px] flex items-center justify-center">
            {current.content}
          </div>

          <div className="px-8 pb-8 flex items-center justify-between">
            <button
              onClick={() => step > 0 && setStep(step - 1)}
              disabled={step === 0}
              className="flex items-center gap-1 px-4 py-2 rounded-xl text-sm font-medium transition-colors disabled:opacity-30 disabled:cursor-not-allowed text-surface-600 dark:text-surface-300"
              style={{ color: step === 0 ? undefined : localConfig.accent_color }}
            >
              <ChevronLeft className="w-4 h-4" />
              上一步
            </button>

            <div className="flex gap-1.5">
              {steps.map((_, i) => (
                <div
                  key={i}
                  className="w-2 h-2 rounded-full transition-colors"
                  style={{ backgroundColor: i === step ? localConfig.accent_color : '#94a3b8', opacity: i === step ? 1 : 0.4 }}
                />
              ))}
            </div>

            {isLast ? (
              <button
                onClick={complete}
                className="flex items-center gap-1.5 px-5 py-2 rounded-xl text-sm font-medium text-white transition-all hover:opacity-90 active:scale-95"
                style={{ backgroundColor: localConfig.accent_color }}
              >
                <CheckCircle className="w-4 h-4" />
                完成
              </button>
            ) : (
              <button
                onClick={() => setStep(step + 1)}
                className="flex items-center gap-1.5 px-5 py-2 rounded-xl text-sm font-medium text-white transition-all hover:opacity-90 active:scale-95"
                style={{ backgroundColor: localConfig.accent_color }}
              >
                下一步
                <ChevronRight className="w-4 h-4" />
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
