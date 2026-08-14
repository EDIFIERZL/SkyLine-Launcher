import { Box, Typography } from '@/components/material'
import { MusicPanel } from '../components/music/MusicPanel'

export function Music() {
  return (
    <Box className="h-full flex flex-col overflow-hidden">
      <Box className="shrink-0 mb-4">
        <Typography variant="h6" className="font-bold">音乐</Typography>
        <Typography variant="body2" color="text.secondary" className="text-sm">
          从本地添加音乐文件，以原始音质无损播放，支持播放进度拖拽与全局迷你播放条
        </Typography>
      </Box>
      <Box className="flex-1 overflow-y-auto pb-2">
        <MusicPanel />
      </Box>
    </Box>
  )
}
