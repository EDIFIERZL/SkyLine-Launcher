import { invoke } from '@tauri-apps/api/core'
import { useMusicStore, type MusicTrack } from '../../stores/musicStore'

export const audio: HTMLAudioElement | null = typeof window !== 'undefined' ? new Audio() : null

let initialized = false
let currentUrl: string | null = null

const FADE_DURATION = 400
let fadeTimer: ReturnType<typeof setInterval> | null = null

function cancelFade() {
  if (fadeTimer) {
    clearInterval(fadeTimer)
    fadeTimer = null
  }
}

function getBaseVolume(): number {
  return useMusicStore.getState().volume
}

function fadeAudio(to: number, duration = FADE_DURATION, onDone?: () => void): void {
  if (!audio) {
    onDone?.()
    return
  }
  cancelFade()
  const from = audio.volume
  if (from === to) {
    onDone?.()
    return
  }
  const start = performance.now()
  fadeTimer = setInterval(() => {
    const t = Math.min(1, (performance.now() - start) / duration)
    const eased = t * t * (3 - 2 * t)
    if (audio) audio.volume = from + (to - from) * eased
    if (t >= 1) {
      cancelFade()
      onDone?.()
    }
  }, 16)
}

function dataUriToBlob(dataUri: string): Blob | null {
  try {
    const m = /^data:([^;,]+)?(;base64)?,([\s\S]*)$/i.exec(dataUri)
    if (!m) return null
    const isBase64 = !!m[2]
    const bytes = isBase64
      ? Uint8Array.from(atob(m[3]), (c) => c.charCodeAt(0))
      : new TextEncoder().encode(decodeURIComponent(m[3]))
    const type = m[1] || 'audio/mpeg'
    return new Blob([bytes], { type })
  } catch {
    return null
  }
}

function bindEvents() {
  if (!audio || initialized) return
  initialized = true
  audio.preload = 'auto'
  audio.preservesPitch = true

  audio.addEventListener('timeupdate', () => {
    if (!audio) return
    useMusicStore.getState().setCurrentTime(audio.currentTime)
  })
  audio.addEventListener('durationchange', () => {
    if (!audio) return
    const d = audio.duration
    if (d && Number.isFinite(d)) {
      useMusicStore.getState().updateTrackMeta(useMusicStore.getState().currentId, { duration: d })
    }
  })
  audio.addEventListener('play', () => useMusicStore.getState().setPlaying(true))
  audio.addEventListener('pause', () => useMusicStore.getState().setPlaying(false))
  audio.addEventListener('ended', () => next())
  audio.addEventListener('error', () => {
    useMusicStore.getState().setLoading(false)
    useMusicStore.getState().setPlaying(false)
  })
}

export async function loadTrack(track: MusicTrack): Promise<void> {
  bindEvents()
  const st = useMusicStore.getState()
  st.setLoading(true)
  st.setCurrentId(track.id)
  if (!audio) {
    st.setLoading(false)
    return
  }
  try {
    const dataUri = await invoke<string>('read_audio_file', { path: track.path })
    const blob = dataUriToBlob(dataUri)
    if (!blob) throw new Error('音频解码失败')
    const url = URL.createObjectURL(blob)
    if (currentUrl) URL.revokeObjectURL(currentUrl)
    currentUrl = url
    audio.volume = 0
    audio.src = url
    await audio.play()
    useMusicStore.getState().setPlaying(true)
    fadeAudio(getBaseVolume())
  } catch (e) {
    console.error('播放失败:', e)
    useMusicStore.getState().setPlaying(false)
  } finally {
    useMusicStore.getState().setLoading(false)
  }
}

export function togglePlay(): void {
  bindEvents()
  const st = useMusicStore.getState()
  if (!audio) return
  if (!st.currentId) {
    const first = st.playlist[0]
    if (first) void loadTrack(first)
    return
  }
  if (audio.paused) {
    if (!audio.currentSrc && st.currentId) {
      const current = st.playlist.find((t) => t.id === st.currentId)
      if (current) void loadTrack(current)
      return
    }
    audio.volume = 0
    const p = audio.play()
    if (p) p.then(() => fadeAudio(getBaseVolume())).catch(() => {})
  } else {
    fadeAudio(0, 350, () => {
      if (audio) audio.pause()
    })
  }
}

export function seekTo(seconds: number): void {
  if (!audio || !Number.isFinite(seconds)) return
  audio.currentTime = Math.max(0, seconds)
  useMusicStore.getState().setCurrentTime(seconds)
}

export function setVolume(volume: number): void {
  const v = Math.min(1, Math.max(0, volume))
  useMusicStore.getState().setVolume(v)
  if (!audio) return
  if (audio.paused || audio.volume === 0) {
    audio.volume = v
  } else {
    fadeAudio(v, 180)
  }
}

export function next(): void {
  const st = useMusicStore.getState()
  if (st.playlist.length === 0) return
  if (st.playlist.length === 1) {
    void loadTrack(st.playlist[0])
    return
  }
  const idx = st.playlist.findIndex((t) => t.id === st.currentId)
  let n: number
  if (st.mode === 'shuffle') {
    n = Math.floor(Math.random() * st.playlist.length)
    if (n === idx) n = (n + 1) % st.playlist.length
  } else {
    n = (idx + 1) % st.playlist.length
  }
  void loadTrack(st.playlist[n])
}

export function prev(): void {
  const st = useMusicStore.getState()
  if (st.playlist.length === 0) return
  const idx = st.playlist.findIndex((t) => t.id === st.currentId)
  const n = idx <= 0 ? st.playlist.length - 1 : idx - 1
  void loadTrack(st.playlist[n])
}

export function removeTrackSafe(id: string): void {
  const st = useMusicStore.getState()
  const list = st.playlist
  const idx = list.findIndex((t) => t.id === id)
  const wasCurrent = id === st.currentId
  st.removeTrack(id)
  if (wasCurrent && list.length > 1) {
    const n = list[idx + 1] ?? list[idx - 1]
    if (n) void loadTrack(n)
  }
}
