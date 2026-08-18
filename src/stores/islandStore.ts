import { create } from 'zustand'

export interface IslandTask {
  id: string
  title: string
  status: 'pending' | 'running' | 'done' | 'failed'
  detail?: string
}

export interface IslandChatEntry {
  role: 'user' | 'assistant'
  content: string
  ts: number
}

interface IslandState {
  aiActive: boolean
  aiThinking: boolean
  aiTasks: IslandTask[]
  aiMessage: string
  aiOpen: boolean
  compactMode: boolean
  islandChatHistory: IslandChatEntry[]
  setAiActive: (v: boolean) => void
  setAiThinking: (v: boolean) => void
  setAiTasks: (tasks: IslandTask[]) => void
  updateTask: (id: string, patch: Partial<IslandTask>) => void
  addTask: (task: IslandTask) => void
  clearTasks: () => void
  setAiMessage: (m: string) => void
  setAiOpen: (v: boolean) => void
  setCompactMode: (v: boolean) => void
  addIslandChatEntry: (entry: IslandChatEntry) => void
  clearIslandChatHistory: () => void
}

export const useIslandStore = create<IslandState>((set) => ({
  aiActive: false,
  aiThinking: false,
  aiTasks: [],
  aiMessage: '',
  aiOpen: false,
  compactMode: false,
  islandChatHistory: [],
  setAiActive: (aiActive) => set({ aiActive }),
  setAiThinking: (aiThinking) => set({ aiThinking }),
  setAiTasks: (aiTasks) => set({ aiTasks }),
  updateTask: (id, patch) =>
    set((s) => ({ aiTasks: s.aiTasks.map((t) => (t.id === id ? { ...t, ...patch } : t)) })),
  addTask: (task) => set((s) => ({ aiTasks: [...s.aiTasks, task] })),
  clearTasks: () => set({ aiTasks: [] }),
  setAiMessage: (aiMessage) => set({ aiMessage }),
  setAiOpen: (aiOpen) => set({ aiOpen }),
  setCompactMode: (compactMode) => set({ compactMode }),
  addIslandChatEntry: (entry) =>
    set((s) => ({ islandChatHistory: [...s.islandChatHistory, entry].slice(-20) })),
  clearIslandChatHistory: () => set({ islandChatHistory: [] }),
}))