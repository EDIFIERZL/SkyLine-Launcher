import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { useInstanceStore } from '../stores/instanceStore'


export async function watchResourceInstall(onResourceChanged: () => void): Promise<UnlistenFn> {
  return listen<{ stage: string; progress: number; message: string }>('install-progress', (e) => {
    const { stage, progress } = e.payload
    if (stage === 'java') return
    if (progress >= 1) onResourceChanged()
  })
}


export function invalidateCatalog() {
  useInstanceStore.getState().invalidate()
}
