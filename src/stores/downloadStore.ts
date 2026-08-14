import { create } from 'zustand'
import { triggerSilentOptimize } from '../hooks/useMemoryOptimizer'

export interface DownloadTask {
  id: string
  title: string
  stage: string
  progress: number
  message: string
  status: 'downloading' | 'done' | 'error'
  error?: string
  kind?: 'download' | 'launch'
  instanceId?: string
}

interface DownloadState {
  tasks: DownloadTask[]
  addTask: (task: Omit<DownloadTask, 'progress' | 'stage' | 'message'> & { progress?: number; stage?: string; message?: string }) => void
  updateTask: (id: string, patch: Partial<Omit<DownloadTask, 'id'>>) => void
  removeTask: (id: string) => void
  markDone: (id: string, message?: string) => void
  markError: (id: string, error: string) => void
  clearFinished: () => void
  hasActiveInstance: (instanceId: string) => boolean
}

export const useDownloadStore = create<DownloadState>((set, get) => ({
  tasks: [],
  addTask: (task) =>
    set((s) => ({
      tasks: [
        ...s.tasks,
        { progress: 0, stage: 'pending', message: '等待开始...', ...task },
      ],
    })),
  updateTask: (id, patch) =>
    set((s) => ({
      tasks: s.tasks.map((t) => (t.id === id ? { ...t, ...patch } : t)),
    })),
  removeTask: (id) =>
    set((s) => ({ tasks: s.tasks.filter((t) => t.id !== id) })),
  markDone: (id, message) => {
    triggerSilentOptimize()
    set((s) => ({
      tasks: s.tasks.map((t) =>
        t.id === id
          ? {
              ...t,
              status: 'done',
              progress: 1,
              message: message ?? (t.kind === 'launch' ? '启动完成' : '安装完成'),
            }
          : t,
      ),
    }))
  },
  markError: (id, error) =>
    set((s) => ({
      tasks: s.tasks.map((t) => (t.id === id ? { ...t, status: 'error', error } : t)),
    })),
  clearFinished: () =>
    set((s) => ({ tasks: s.tasks.filter((t) => t.status === 'downloading') })),
  hasActiveInstance: (instanceId) =>
    get().tasks.some((t) => t.status === 'downloading' && t.instanceId === instanceId),
}))
