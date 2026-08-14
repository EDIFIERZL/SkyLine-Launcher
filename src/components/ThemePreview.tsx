
import { Box, Typography, Card, Button, Chip } from './material'
import { useSettingsStore } from '../stores/settingsStore'

export function ThemePreview() {
  const { config } = useSettingsStore()

  return (
    <Card>
      <Box className="space-y-4">
        <Typography variant="subtitle1">主题预览</Typography>
        
        <Box className="space-y-2">
          <Typography variant="caption" color="text.secondary">强调色</Typography>
          <Box className="flex gap-2">
            <Box className="w-8 h-8 rounded-lg" style={{ backgroundColor: 'var(--accent-color)' }} />
            <Box className="w-8 h-8 rounded-lg" style={{ backgroundColor: 'var(--color-accent-400)' }} />
            <Box className="w-8 h-8 rounded-lg" style={{ backgroundColor: 'var(--color-accent-600)' }} />
          </Box>
        </Box>

        <Box className="space-y-2">
          <Typography variant="caption" color="text.secondary">按钮样式</Typography>
          <Box className="flex gap-2 flex-wrap">
            <Button variant="contained" size="small">主要按钮</Button>
            <Button variant="outlined" size="small">次要按钮</Button>
            <Button variant="text" size="small">文字按钮</Button>
          </Box>
        </Box>

        <Box className="space-y-2">
          <Typography variant="caption" color="text.secondary">标签样式</Typography>
          <Box className="flex gap-2">
            <Chip label="默认" size="small" />
            <Chip label="主要" size="small" color="primary" />
            <Chip label="成功" size="small" color="success" />
            <Chip label="警告" size="small" color="warning" />
            <Chip label="错误" size="small" color="error" />
          </Box>
        </Box>

        <Box className="space-y-2">
          <Typography variant="caption" color="text.secondary">卡片样式</Typography>
          <Card className="p-3">
            <Typography variant="body2">这是一个卡片组件示例</Typography>
          </Card>
        </Box>

        <Box className="space-y-2">
          <Typography variant="caption" color="text.secondary">当前配置</Typography>
          <Box className="text-xs space-y-1">
            <div>主题模式: {config.theme_mode}</div>
            <div>强调色: {config.accent_color}</div>
            <div>UI 缩放: {config.ui_scale * 100}%</div>
            <div>字体大小: {config.font_size}</div>
            <div>紧凑模式: {config.compact_mode ? '开启' : '关闭'}</div>
          </Box>
        </Box>
      </Box>
    </Card>
  )
}

export default ThemePreview
