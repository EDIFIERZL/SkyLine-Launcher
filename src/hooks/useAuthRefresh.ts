import { useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useAuthStore } from '../stores/authStore'

const REFRESH_INTERVAL = 2 * 60 * 1000 


export function useAuthRefresh() {
  const { session, setSession } = useAuthStore()

  useEffect(() => {
    if (!session?.refresh_token || session.user_type !== 'msa') return

    const timer = setInterval(() => {
      invoke('microsoft_auth_refresh', { refreshToken: session.refresh_token })
        .then((auth: any) => {
          setSession(auth)
          
          const accounts = JSON.parse(localStorage.getItem('skyline-auth-accounts') || '[]')
          const idx = accounts.findIndex((a: any) => a.session?.uuid === auth.uuid)
          if (idx >= 0) {
            accounts[idx].session = auth
            localStorage.setItem('skyline-auth-accounts', JSON.stringify(accounts))
          }
        })
        .catch(() => {})
    }, REFRESH_INTERVAL)

    return () => clearInterval(timer)
  }, [session?.refresh_token, session?.user_type, setSession])
}
