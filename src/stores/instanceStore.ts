import { create } from 'zustand'
import type { Instance, ModInfo } from '../types'

export interface InstanceDetailData {
  mods: ModInfo[]
  resourcepacks: { file_name: string; name: string | null; enabled: boolean }[]
  shaders: { file_name: string; name: string | null; enabled: boolean }[]
}

interface InstanceState {
  instances: Instance[]
  selectedId: string | null
  loading: boolean
  
  loaded: boolean
  
  details: Record<string, InstanceDetailData>
  folders: string[]
  activeFolder: string | null
  foldersLoaded: boolean
  setInstances: (instances: Instance[]) => void
  setSelectedId: (id: string | null) => void
  setLoading: (loading: boolean) => void
  setLoaded: (loaded: boolean) => void
  setDetails: (details: Record<string, InstanceDetailData>) => void
  setInstanceDetails: (id: string, data: InstanceDetailData) => void
  setFolders: (folders: string[]) => void
  setActiveFolder: (folder: string | null) => void
  setFoldersLoaded: (v: boolean) => void
  
  invalidate: () => void
}

export const useInstanceStore = create<InstanceState>((set) => ({
  instances: [],
  selectedId: null,
  loading: false,
  loaded: false,
  details: {},
  folders: [],
  activeFolder: null,
  foldersLoaded: false,
  setInstances: (instances) => set({ instances }),
  setSelectedId: (selectedId) => set({ selectedId }),
  setLoading: (loading) => set({ loading }),
  setLoaded: (loaded) => set({ loaded }),
  setDetails: (details) => set({ details }),
  setInstanceDetails: (id, data) =>
    set((s) => ({ details: { ...s.details, [id]: data } })),
  setFolders: (folders) => set({ folders }),
  setActiveFolder: (activeFolder) => set({ activeFolder }),
  setFoldersLoaded: (foldersLoaded) => set({ foldersLoaded }),
  invalidate: () => set({ loaded: false, foldersLoaded: false, details: {} }),
}))
