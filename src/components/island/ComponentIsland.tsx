import { useEffect, useState } from 'react'
import { useMusicStore } from '../../stores/musicStore'
import { togglePlay, next, prev, seekTo, setVolume } from '../music/musicEngine'
import { formatTime } from '../music/format'
import { useIslandStore } from '../../stores/islandStore'
import { AiIsland } from './AiIsland'
import { RecentInstances } from './RecentInstances'
import { Music, Play, Pause, SkipBack, SkipForward, Volume2, VolumeX, ChevronDown, Repeat, Shuffle } from 'lucide-react'

const SPRING = 'cubic-bezier(0.34, 1.56, 0.64, 1)'

export function ComponentIsland() {
  const { playlist, currentId, playing, currentTime, volume, loading, mode, setMode } = useMusicStore()
  const aiTasks = useIslandStore((s) => s.aiTasks)
  const aiActive = useIslandStore((s) => s.aiActive)
  const aiOpen = useIslandStore((s) => s.aiOpen)
  const setCompactMode = useIslandStore((s) => s.setCompactMode)

  const [hoverMusic, setHoverMusic] = useState(false)
  const [hoverRecent, setHoverRecent] = useState(false)
  const [scrub, setScrub] = useState<number | null>(null)

  const current = playlist.find((t) => t.id === currentId) ?? null
  const duration = current?.duration ?? 0
  const displayTime = scrub ?? currentTime
  const pct = duration > 0 ? Math.min(100, (displayTime / duration) * 100) : 0

  useEffect(() => {
    setCompactMode(!!(aiActive && aiTasks.length > 0))
  }, [aiActive, aiTasks, setCompactMode])

  const onSeekChange = (value: number) => setScrub(value)
  const commitSeek = () => {
    if (scrub !== null) seekTo(scrub)
    setScrub(null)
  }

  // AI 执行或展开时音乐卡变成可用的播放圆卡
  const musicCollapsed = aiOpen || hoverRecent

  return (
    <div className="absolute top-0 left-1/2 -translate-x-1/2 z-40 flex items-start gap-1.5 justify-center" data-no-drag>
      {/* 音乐卡：平时同原来 hover 放大，AI 执行时变圆 */}
      <div
        onMouseEnter={() => setHoverMusic(true)}
        onMouseLeave={() => {
          setHoverMusic(false)
          setScrub(null)
        }}
      >
        {musicCollapsed ? (
          <div className="island-glass relative w-10 h-10 rounded-full border bg-white/95 dark:bg-surface-850/95 backdrop-blur-xl border-surface-200 dark:border-surface-700/60 shadow-lg overflow-hidden origin-left flex items-center justify-center">
            <button onClick={togglePlay} disabled={loading} className="w-10 h-10 rounded-full flex items-center justify-center text-[var(--accent-color)] hover:bg-surface-100 dark:hover:bg-surface-800 cursor-pointer disabled:opacity-50" title={playing ? '暂停' : '播放'}>
              {playing ? <Pause className="w-3.5 h-3.5" /> : <Play className="w-3.5 h-3.5 translate-x-[1px]" />}
            </button>
          </div>
        ) : !current ? null : (
          <div
            className={`music-player-bar rounded-2xl border bg-white/95 dark:bg-surface-850/95 backdrop-blur-xl border-surface-200 dark:border-surface-700/60 overflow-hidden origin-top music-card-pop ${
              hoverMusic ? 'shadow-2xl' : 'shadow-lg'
            }`}
            style={{
              width: hoverMusic ? 360 : 280,
              transform: hoverMusic ? 'scale(1.02)' : 'scale(1)',
              transition: `width 0.35s ${SPRING}, transform 0.35s ${SPRING}, box-shadow 0.35s ease`,
            }}
          >
            <div className="flex items-center gap-2 px-2.5 h-9 select-none">
              <button
                onClick={togglePlay}
                disabled={loading}
                className={`w-6 h-6 shrink-0 rounded-full flex items-center justify-center bg-[var(--accent-color)] text-white hover:opacity-90 active:scale-95 transition-all cursor-pointer disabled:opacity-50 ${loading ? 'animate-spin' : ''}`}
                title={playing ? '暂停' : '播放'}
              >
                {loading ? <span className="w-3 h-3 rounded-full border-2 border-white/40 border-t-white" /> : playing ? <Pause className="w-3 h-3" /> : <Play className="w-3 h-3 translate-x-[1px]" />}
              </button>
              <div className="flex items-center gap-2 flex-1 min-w-0">
                <Music className="w-3.5 h-3.5 text-[var(--accent-color)] shrink-0" />
                <span className="truncate text-xs font-medium text-surface-900 dark:text-surface-100">{current.title}</span>
              </div>
              <span className="text-[10px] text-surface-400 shrink-0 tabular-nums">{formatTime(displayTime)}</span>
              <ChevronDown className={`w-3 h-3 text-surface-400 shrink-0 transition-transform duration-300 ${hoverMusic ? 'rotate-180' : ''}`} />
            </div>

            <div
              className="overflow-hidden"
              style={{ maxHeight: hoverMusic ? 260 : 0, transition: `max-height 0.4s ${SPRING}` }}
            >
              <div
                className="px-3 pb-2.5 pt-2 space-y-2.5 border-t border-surface-200/60 dark:border-surface-700/40"
                style={{
                  opacity: hoverMusic ? 1 : 0,
                  transform: hoverMusic ? 'translateY(0)' : 'translateY(12px)',
                  transition: `opacity 0.3s ease, transform 0.4s ${SPRING}`,
                }}
              >
                <div className="flex items-center gap-2">
                  <span className="text-[10px] text-surface-400 tabular-nums shrink-0">{formatTime(displayTime)}</span>
                  <input
                    type="range"
                    className="flex-1 music-range"
                    min={0}
                    max={duration > 0 ? duration : 0}
                    step={0.1}
                    value={Math.min(displayTime, duration || 0)}
                    disabled={duration <= 0}
                    onPointerDown={() => setScrub(displayTime)}
                    onChange={(e) => onSeekChange(Number(e.target.value))}
                    onPointerUp={commitSeek}
                    onTouchEnd={commitSeek}
                    onBlur={commitSeek}
                    style={{ ['--music-progress' as string]: `${pct}%` }}
                  />
                  <span className="text-[10px] text-surface-500 tabular-nums shrink-0">{formatTime(duration)}</span>
                </div>

                <div className="flex items-center justify-center gap-3">
                  <button
                    onClick={() => setMode(mode === 'shuffle' ? 'list' : 'shuffle')}
                    className={`transition-colors cursor-pointer ${mode === 'shuffle' ? 'text-[var(--accent-color)]' : 'text-surface-400 hover:text-surface-600 dark:hover:text-surface-200'}`}
                    title={mode === 'shuffle' ? '随机播放（点击切换为列表循环）' : '列表循环（点击切换为随机播放）'}
                  >
                    <Shuffle className="w-3.5 h-3.5" />
                  </button>
                  <button onClick={prev} className="text-surface-500 hover:text-surface-800 dark:hover:text-surface-200 transition-colors cursor-pointer" title="上一首">
                    <SkipBack className="w-4 h-4" />
                  </button>
                  <button
                    onClick={togglePlay}
                    disabled={loading}
                    className="w-9 h-9 rounded-full flex items-center justify-center bg-[var(--accent-color)] text-white hover:opacity-90 active:scale-95 transition-all cursor-pointer disabled:opacity-50"
                  >
                    {playing ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4 translate-x-[1px]" />}
                  </button>
                  <button onClick={next} className="text-surface-500 hover:text-surface-800 dark:hover:text-surface-200 transition-colors cursor-pointer" title="下一首">
                    <SkipForward className="w-4 h-4" />
                  </button>
                  <button
                    onClick={() => setMode(mode === 'list' ? 'shuffle' : 'list')}
                    className={`transition-colors cursor-pointer ${mode === 'list' ? 'text-[var(--accent-color)]' : 'text-surface-400 hover:text-surface-600 dark:hover:text-surface-200'}`}
                    title={mode === 'list' ? '列表循环（点击切换为随机播放）' : '随机播放（点击切换为列表循环）'}
                  >
                    <Repeat className="w-3.5 h-3.5" />
                  </button>
                </div>

                <div className="flex items-center gap-2">
                  <button
                    onClick={() => setVolume(volume > 0 ? 0 : 0.8)}
                    className="text-surface-400 hover:text-surface-600 dark:hover:text-surface-200 transition-colors cursor-pointer shrink-0"
                  >
                    {volume > 0 ? <Volume2 className="w-3.5 h-3.5" /> : <VolumeX className="w-3.5 h-3.5" />}
                  </button>
                  <input
                    type="range"
                    className="flex-1 music-range"
                    min={0}
                    max={1}
                    step={0.01}
                    value={volume}
                    onChange={(e) => setVolume(Number(e.target.value))}
                    style={{ ['--music-progress' as string]: `${volume * 100}%` }}
                  />
                  <span className="text-[10px] text-surface-400 tabular-nums shrink-0">{Math.round(volume * 100)}%</span>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* 最近游玩实例圆卡 */}
      <RecentInstances collapsed={false} onHoverChange={setHoverRecent} />

      {/* AI 卡片 */}
      <AiIsland />
    </div>
  )
}
