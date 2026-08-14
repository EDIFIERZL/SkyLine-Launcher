import { create } from 'zustand'
import type { AuthSession } from '../types'

const STORAGE_KEY = 'skyline-auth-session'
const COSMETICS_KEY = 'skyline-account-cosmetics'
const ACCOUNTS_KEY = 'skyline-accounts'

export interface AccountCosmetics {
  skin: string | null
  cape: string | null
}

export interface SavedAccount {
  id: string
  username: string
  user_type: string
  session: AuthSession
}

function loadSession(): AuthSession | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    return raw ? (JSON.parse(raw) as AuthSession) : null
  } catch {
    return null
  }
}

function loadCosmetics(): Record<string, AccountCosmetics> {
  try {
    const raw = localStorage.getItem(COSMETICS_KEY)
    return raw ? (JSON.parse(raw) as Record<string, AccountCosmetics>) : {}
  } catch {
    return {}
  }
}

function loadAccounts(): SavedAccount[] {
  try {
    const raw = localStorage.getItem(ACCOUNTS_KEY)
    return raw ? (JSON.parse(raw) as SavedAccount[]) : []
  } catch {
    return []
  }
}

function persistAccounts(accounts: SavedAccount[]) {
  localStorage.setItem(ACCOUNTS_KEY, JSON.stringify(accounts))
}

interface AuthState {
  session: AuthSession | null
  cosmetics: Record<string, AccountCosmetics>
  accounts: SavedAccount[]
  setSession: (session: AuthSession | null) => void
  clearSession: () => void
  saveAccount: (session: AuthSession) => void
  removeAccount: (id: string) => void
  setSkin: (uuid: string, skinB64: string | null) => void
  setCape: (uuid: string, capeB64: string | null) => void
}

export const useAuthStore = create<AuthState>((set) => ({
  session: loadSession(),
  cosmetics: loadCosmetics(),
  accounts: loadAccounts(),
  setSession: (session) => {
    if (session) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(session))
    } else {
      localStorage.removeItem(STORAGE_KEY)
    }
    set({ session })
  },
  clearSession: () => {
    localStorage.removeItem(STORAGE_KEY)
    set({ session: null })
  },
  saveAccount: (session) =>
    set((s) => {
      const id = session.uuid || session.username
      const existing = s.accounts.filter((a) => a.id !== id)
      const accounts = [
        {
          id,
          username: session.username,
          user_type: session.user_type,
          session,
        },
        ...existing,
      ]
      persistAccounts(accounts)
      return { accounts }
    }),
  removeAccount: (id) =>
    set((s) => {
      const accounts = s.accounts.filter((a) => a.id !== id)
      persistAccounts(accounts)
      return { accounts }
    }),
  setSkin: (uuid, skinB64) =>
    set((s) => {
      const cosmetics = {
        ...s.cosmetics,
        [uuid]: { skin: skinB64, cape: s.cosmetics[uuid]?.cape ?? null },
      }
      localStorage.setItem(COSMETICS_KEY, JSON.stringify(cosmetics))
      return { cosmetics }
    }),
  setCape: (uuid, capeB64) =>
    set((s) => {
      const cosmetics = {
        ...s.cosmetics,
        [uuid]: { skin: s.cosmetics[uuid]?.skin ?? null, cape: capeB64 },
      }
      localStorage.setItem(COSMETICS_KEY, JSON.stringify(cosmetics))
      return { cosmetics }
    }),
}))
