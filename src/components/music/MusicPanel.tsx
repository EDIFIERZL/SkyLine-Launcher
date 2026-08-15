import { useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { useMusicStore, titleFromPath } from '../../stores/musicStore'
import { loadTrack, togglePlay, removeTrackSafe } from './musicEngine'
import { formatTime } from './format'
import { Music, Play, Pause, Trash2, Plus, Music2, Volume2, Repeat, Shuffle } from 'lucide-react'
import { Box, Typography, Button, Card, EmptyState } from '../material'

const AUDIO_EXTENSIONS = ['mp3', 'flac', 'wav', 'ogg', 'oga', 'opus', 'm4a', 'aac', 'wma', 'aiff', 'aif', 'ape', 'webm']

export function MusicPanel() {
  const { playlist, currentId, playing, loading, mode, setMode } = useMusicStore()
  const [adding, setAdding] = useState(false)

  const addMusic = async () => {
    try {
      const picked = await open({
        multiple: true,
        filters: [{ name: '音频文件', extensions: AUDIO_EXTENSIONS }],
        title: '选择本地音乐',
      })
      if (!picked) return
      setAdding(true)
      const paths = Array.isArray(picked) ? picked : [picked]
      const tracks = paths.map((p) => ({ id: p, title: titleFromPath(p), path: p }))
      useMusicStore.getState().addTracks(tracks)
    } catch (e) {
      console.error(e)
    } finally {
      setAdding(false)
    }
  }

  const onRowClick = (id: string) => {
    if (id === currentId) {
      togglePlay()
      return
    }
    const track = playlist.find((t) => t.id === id)
    if (track) void loadTrack(track)
  }

  return (
    <Card className="!p-4">
      <Box className="flex items-center justify-between mb-3">
        <Typography variant="subtitle1" className="flex items-center gap-2 font-semibold">
          <Music className="w-4 h-4 text-[var(--accent-color)]" /> 本地音乐
        </Typography>
        <Box className="flex items-center gap-2">
          <span className="flex items-center gap-1 text-[10px] text-surface-400">
            <Volume2 className="w-3 h-3" /> 原始音质 · 无损播放
          </span>
          <button
            onClick={() => setMode('list')}
            title="列表循环"
            className={`w-7 h-7 rounded-lg flex items-center justify-center transition-colors cursor-pointer ${
              mode === 'list' ? 'text-[var(--accent-color)] bg-accent-50 dark:bg-accent-500/10' : 'text-surface-400 hover:text-surface-600 dark:hover:text-surface-300'
            }`}
          >
            <Repeat className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={() => setMode('shuffle')}
            title="随机播放"
            className={`w-7 h-7 rounded-lg flex items-center justify-center transition-colors cursor-pointer ${
              mode === 'shuffle' ? 'text-[var(--accent-color)] bg-accent-50 dark:bg-accent-500/10' : 'text-surface-400 hover:text-surface-600 dark:hover:text-surface-300'
            }`}
          >
            <Shuffle className="w-3.5 h-3.5" />
          </button>
          <Button size="small" variant="contained" startIcon={<Plus className="w-3.5 h-3.5" />} onClick={addMusic} loading={adding}>
            添加音乐
          </Button>
        </Box>
      </Box>

      {playlist.length === 0 ? (
        <EmptyState
          icon={<Music2 className="w-10 h-10" />}
          title="音乐库为空"
          description="点击「添加音乐」选择本地音频文件。"
          action={<Button variant="contained" startIcon={<Plus className="w-3.5 h-3.5" />} onClick={addMusic}>添加音乐</Button>}
        />
      ) : (
        <Box className="space-y-1.5 max-h-80 overflow-y-auto pr-1">
          {playlist.map((t, idx) => {
            const isCurrent = t.id === currentId
            return (
              <Box
                key={t.id}
                className={`flex items-center gap-3 px-3 py-2 rounded-lg border transition-colors cursor-pointer ${
                  isCurrent
                    ? 'bg-accent-50 dark:bg-accent-500/10 border-accent-200 dark:border-accent-500/30'
                    : 'bg-surface-50 dark:bg-surface-800 border-transparent hover:bg-surface-100 dark:hover:bg-surface-700/60'
                }`}
                onClick={() => onRowClick(t.id)}
              >
                <span className="w-5 text-center text-[10px] text-surface-400 tabular-nums shrink-0">{idx + 1}</span>
                <button
                  className="w-7 h-7 shrink-0 rounded-full flex items-center justify-center text-white bg-[var(--accent-color)] hover:opacity-90 active:scale-95 transition-all cursor-pointer disabled:opacity-50"
                  disabled={loading && isCurrent}
                  onClick={(e) => {
                    e.stopPropagation()
                    onRowClick(t.id)
                  }}
                >
                  {isCurrent && playing ? <Pause className="w-3 h-3" /> : <Play className="w-3 h-3 translate-x-[0.5px]" />}
                </button>
                <span className={`flex-1 min-w-0 truncate text-sm ${isCurrent ? 'font-semibold text-[var(--accent-color)]' : 'font-medium text-surface-800 dark:text-surface-200'}`}>
                  {t.title}
                </span>
                {isCurrent && <span className="text-[10px] text-[var(--accent-color)] shrink-0">{playing ? '播放中' : '已暂停'}</span>}
                <span className="text-[10px] text-surface-400 tabular-nums shrink-0">{t.duration ? formatTime(t.duration) : '--:--'}</span>
                <button
                  className="text-surface-400 hover:text-red-500 transition-colors shrink-0 cursor-pointer"
                  onClick={(e) => {
                    e.stopPropagation()
                    removeTrackSafe(t.id)
                  }}
                  title="从列表移除"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </Box>
            )
          })}
        </Box>
      )}
    </Card>
  )
}
