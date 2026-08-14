import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

export interface MusicTrack {
  id: string
  title: string
  path: string
  duration?: number
}

interface MusicState {
  playlist: MusicTrack[]
  currentId: string | null
  playing: boolean
  currentTime: number
  volume: number
  loading: boolean
  mode: 'list' | 'shuffle'
  setPlaylist: (tracks: MusicTrack[]) => void
  reloadPlaylist: () => Promise<void>
  addTracks: (tracks: MusicTrack[]) => void
  removeTrack: (id: string) => void
  clearPlaylist: () => void
  setCurrentId: (id: string | null) => void
  setPlaying: (playing: boolean) => void
  setCurrentTime: (time: number) => void
  setVolume: (volume: number) => void
  setLoading: (loading: boolean) => void
  setMode: (mode: 'list' | 'shuffle') => void
  updateTrackMeta: (id: string | null, patch: Partial<MusicTrack>) => void
}

const LS_PLAYLIST = 'skyline.music.playlist'
const LS_CURRENT = 'skyline.music.current'
const LS_VOLUME = 'skyline.music.volume'
const LS_MODE = 'skyline.music.mode'

export function titleFromPath(path: string): string {
  const name = path.split(/[\\/]/).pop() ?? path
  return name.replace(/\.[^.]+$/, '') || name
}

function loadVolume(): number {
  try {
    const v = Number(localStorage.getItem(LS_VOLUME))
    if (Number.isFinite(v) && v >= 0 && v <= 1) return v
  } catch {  }
  return 0.8
}

function loadPlaylist(): MusicTrack[] {
  try {
    const raw = localStorage.getItem(LS_PLAYLIST)
    if (!raw) return []
    const parsed = JSON.parse(raw) as unknown
    if (!Array.isArray(parsed)) return []
    return parsed
      .filter((t): t is MusicTrack => !!t && typeof t === 'object' && typeof (t as MusicTrack).path === 'string')
      .map((t) => ({ id: t.path, title: t.title || titleFromPath(t.path), path: t.path }))
  } catch {  }
  return []
}

function loadCurrent(): string | null {
  try {
    const id = localStorage.getItem(LS_CURRENT)
    if (!id) return null
    const exists = loadPlaylist().some((t) => t.id === id)
    return exists ? id : null
  } catch {  }
  return null
}

function loadMode(): 'list' | 'shuffle' {
  try {
    const m = localStorage.getItem(LS_MODE)
    if (m === 'shuffle' || m === 'list') return m
  } catch {  }
  return 'list'
}

function persistPlaylist(playlist: MusicTrack[]) {
  try {
    localStorage.setItem(LS_PLAYLIST, JSON.stringify(playlist))
  } catch {  }
}

function persistCurrent(id: string | null) {
  try {
    if (id) localStorage.setItem(LS_CURRENT, id)
    else localStorage.removeItem(LS_CURRENT)
  } catch {  }
}

export const useMusicStore = create<MusicState>((set, get) => ({
  playlist: loadPlaylist(),
  currentId: loadCurrent(),
  playing: false,
  currentTime: 0,
  volume: loadVolume(),
  loading: false,
  mode: loadMode(),
  setPlaylist: (tracks) => {
    const next = tracks.map((t) => ({ id: t.id || t.path, title: t.title || titleFromPath(t.path), path: t.path }))
    persistPlaylist(next)
    set({ playlist: next, currentId: null, currentTime: 0, playing: false })
    persistCurrent(null)
  },
  reloadPlaylist: async () => {
    const current = get().playlist
    if (current.length === 0) return
    set({ loading: true })
    try {
      const exists = await invoke<boolean[]>('check_files_exist', { paths: current.map((t) => t.path) })
      const next = current.filter((_, i) => exists[i])
      persistPlaylist(next)
      const currentId = get().currentId
      if (currentId && !next.some((t) => t.id === currentId)) {
        persistCurrent(null)
        set({ playlist: next, currentId: null, currentTime: 0, playing: false })
      } else {
        set({ playlist: next })
      }
    } catch {
      
    } finally {
      set({ loading: false })
    }
  },
  addTracks: (tracks) => {
    const existing = new Set(get().playlist.map((t) => t.path))
    const fresh = tracks.filter((t) => !existing.has(t.path)).map((t) => ({
      id: t.id || t.path,
      title: t.title || titleFromPath(t.path),
      path: t.path,
    }))
    if (fresh.length === 0) return
    const next = [...get().playlist, ...fresh]
    persistPlaylist(next)
    set({ playlist: next })
  },
  removeTrack: (id) => {
    const next = get().playlist.filter((t) => t.id !== id)
    persistPlaylist(next)
    const currentId = get().currentId
    if (currentId === id) {
      persistCurrent(null)
      set({ playlist: next, currentId: null, currentTime: 0, playing: false })
    } else {
      set({ playlist: next })
    }
  },
  clearPlaylist: () => {
    persistPlaylist([])
    persistCurrent(null)
    set({ playlist: [], currentId: null, currentTime: 0, playing: false })
  },
  setCurrentId: (id) => {
    persistCurrent(id)
    set({ currentId: id, currentTime: 0 })
  },
  setPlaying: (playing) => set({ playing }),
  setCurrentTime: (currentTime) => set({ currentTime }),
  setVolume: (volume) => {
    try { localStorage.setItem(LS_VOLUME, String(volume)) } catch {  }
    set({ volume })
  },
  setLoading: (loading) => set({ loading }),
  setMode: (mode) => {
    try { localStorage.setItem(LS_MODE, mode) } catch {  }
    set({ mode })
  },
  updateTrackMeta: (id, patch) => {
    if (!id) return
    set((s) => ({
      playlist: s.playlist.map((t) => (t.id === id ? { ...t, ...patch } : t)),
    }))
  },
}))
