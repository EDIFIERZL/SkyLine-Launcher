import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from '../stores/settingsStore'
import { Box, Typography, Button } from '@/components/material'
import { Sun, Moon, LayoutDashboard, LayoutList, CheckCircle, ChevronRight, ChevronLeft } from 'lucide-react'
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
  })

  const update = (patch: Partial<LauncherConfig>) => {
    setLocalConfig(prev => ({ ...prev, ...patch }))
  }

  const applyTheme = (mode: string) => {
    update({ theme_mode: mode })
    if (mode === 'system') {
      document.documentElement.removeAttribute('data-theme')
    } else {
      document.documentElement.setAttribute('data-theme', mode)
    }
  }

  const applyAccent = (color: string) => {
    update({ accent_color: color })
    document.documentElement.style.setProperty('--accent-color', color)
  }

  const complete = async () => {
    const final: LauncherConfig = {
      ...config,
      ...localConfig,
      onboarding_completed: true,
    }
    setConfig(final)
    await invoke('save_config', { config: final })
    onComplete()
  }

  const steps = [
    {
      title: '选择主题',
      desc: '选择你喜欢的明暗风格',
      content: (
        <Box className="flex gap-4 justify-center mt-6">
          {[
            { value: 'light', label: '浅色（beta）', icon: <Sun className="w-8 h-8" /> },
            { value: 'dark', label: '深色', icon: <Moon className="w-8 h-8" /> },
          ].map((opt) => (
            <Box
              key={opt.value}
              onClick={() => applyTheme(opt.value)}
              className={`flex flex-col items-center gap-3 p-6 rounded-2xl cursor-pointer transition-all duration-200 border-2 ${
                localConfig.theme_mode === opt.value
                  ? 'border-[var(--accent-color)] bg-[var(--accent-color)]/10'
                  : 'border-surface-200 dark:border-surface-700 hover:border-surface-300 dark:hover:border-surface-600'
              }`}
              style={{ minWidth: 140 }}
            >
              <Box className={`p-3 rounded-xl ${localConfig.theme_mode === opt.value ? 'bg-[var(--accent-color)]/20 text-[var(--accent-color)]' : 'bg-surface-100 dark:bg-surface-800 text-surface-400'}`}>
                {opt.icon}
              </Box>
              <Typography variant="subtitle1" className="font-semibold">{opt.label}</Typography>
            </Box>
          ))}
        </Box>
      ),
    },
    {
      title: '选择强调色',
      desc: '选择启动器的主题色',
      content: (
        <Box className="flex flex-wrap gap-3 justify-center mt-6 max-w-md mx-auto">
          {accentPresets.map((preset) => (
            <button
              key={preset.color}
              onClick={() => applyAccent(preset.color)}
              className={`w-11 h-11 rounded-xl transition-all duration-150 cursor-pointer relative ${
                localConfig.accent_color === preset.color
                  ? 'ring-2 ring-offset-2 ring-offset-white dark:ring-offset-surface-900 scale-110'
                  : 'hover:scale-105'
              }`}
              style={{
                backgroundColor: preset.color,
              }}
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
            <div className="w-full h-full rounded-xl bg-surface-100 dark:bg-surface-800 border border-surface-300 dark:border-surface-600 flex items-center justify-center text-sm text-surface-400 font-medium">
              +
            </div>
          </div>
        </Box>
      ),
    },
    {
      title: '背景设置',
      desc: '你可以在设置中随时更改背景图片或视频',
      content: (
        <Box className="flex flex-col items-center justify-center mt-8 gap-4">
          <Box className="w-48 h-32 rounded-2xl bg-surface-100 dark:bg-surface-800 border-2 border-dashed border-surface-300 dark:border-surface-600 flex items-center justify-center">
            <Typography variant="body2" color="text.secondary" className="text-sm">在设置中配置</Typography>
          </Box>
          <Typography variant="body2" color="text.secondary" className="text-center max-w-xs">
            你可以在「设置 → 外观」中选择背景图片、视频或渐变色，随时个性化你的启动器
          </Typography>
        </Box>
      ),
    },
    {
      title: '首页风格',
      desc: '选择你喜欢的首页布局',
      content: (
        <Box className="flex gap-4 justify-center mt-6">
          {[
            { value: 'full', label: '完整模式', desc: '显示实例详情、模组、截图等', icon: <LayoutList className="w-8 h-8" /> },
            { value: 'minimal', label: '简洁模式', desc: '干净清爽，聚焦启动', icon: <LayoutDashboard className="w-8 h-8" /> },
          ].map((opt) => (
            <Box
              key={opt.value}
              onClick={() => update({ home_style: opt.value })}
              className={`flex flex-col items-center gap-3 p-6 rounded-2xl cursor-pointer transition-all duration-200 border-2 ${
                localConfig.home_style === opt.value
                  ? 'border-[var(--accent-color)] bg-[var(--accent-color)]/10'
                  : 'border-surface-200 dark:border-surface-700 hover:border-surface-300 dark:hover:border-surface-600'
              }`}
              style={{ minWidth: 160 }}
            >
              <Box className={`p-3 rounded-xl ${localConfig.home_style === opt.value ? 'bg-[var(--accent-color)]/20 text-[var(--accent-color)]' : 'bg-surface-100 dark:bg-surface-800 text-surface-400'}`}>
                {opt.icon}
              </Box>
              <Typography variant="subtitle1" className="font-semibold">{opt.label}</Typography>
              <Typography variant="caption" color="text.secondary" className="text-center text-xs">{opt.desc}</Typography>
            </Box>
          ))}
        </Box>
      ),
    },
  ]

  const isLast = step === steps.length - 1
  const current = steps[step]

  return (
    <Box className="h-full flex items-center justify-center bg-surface-50 dark:bg-surface-900">
      <Box className="w-full max-w-lg mx-4">
        <Box className="bg-white dark:bg-surface-800 rounded-3xl shadow-2xl overflow-hidden border border-surface-200/60 dark:border-surface-700/40">
          <Box className="px-8 pt-8 pb-4">
            <Typography variant="h5" className="font-bold text-center">{current.title}</Typography>
            <Typography variant="body2" color="text.secondary" className="text-center mt-1">{current.desc}</Typography>
          </Box>

          <Box className="px-8 pb-6 min-h-[240px] flex items-center justify-center">
            {current.content}
          </Box>

          <Box className="px-8 pb-8 flex items-center justify-between">
            <Button
              variant="text"
              size="small"
              onClick={() => step > 0 && setStep(step - 1)}
              disabled={step === 0}
              startIcon={<ChevronLeft className="w-4 h-4" />}
            >
              上一步
            </Button>

            <Box className="flex gap-1.5">
              {steps.map((_, i) => (
                <Box
                  key={i}
                  className={`w-2 h-2 rounded-full transition-colors ${i === step ? 'bg-[var(--accent-color)]' : 'bg-surface-300 dark:bg-surface-600'}`}
                />
              ))}
            </Box>

            {isLast ? (
              <Button
                variant="contained"
                size="small"
                onClick={complete}
                endIcon={<CheckCircle className="w-4 h-4" />}
              >
                完成
              </Button>
            ) : (
              <Button
                variant="contained"
                size="small"
                onClick={() => setStep(step + 1)}
                endIcon={<ChevronRight className="w-4 h-4" />}
              >
                下一步
              </Button>
            )}
          </Box>
        </Box>
      </Box>
    </Box>
  )
}
