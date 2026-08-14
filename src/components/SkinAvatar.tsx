import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useAuthStore } from '../stores/authStore'

const skinCache = new Map<string, string | null>()
const inflight = new Map<string, Promise<string | null>>()

async function fetchSkin(uuid: string): Promise<string | null> {
  if (skinCache.has(uuid)) return skinCache.get(uuid)!
  if (inflight.has(uuid)) return inflight.get(uuid)!

  const p = (async () => {
    for (let attempt = 0; attempt < 2; attempt++) {
      try {
        const b64 = await invoke<string | null>('get_skin_head', { uuid })
        skinCache.set(uuid, b64 ?? null)
        return b64
      } catch {
        if (attempt === 0) await new Promise(r => setTimeout(r, 800))
      }
    }
    skinCache.set(uuid, null)
    return null
  })()
  inflight.set(uuid, p)
  try { return await p } finally { inflight.delete(uuid) }
}

export function SkinAvatar({
  size = 24,
  uuid,
  username,
  userType,
}: {
  size?: number
  uuid?: string
  username?: string
  userType?: string
}) {
  const { session, cosmetics } = useAuthStore()
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const imgRef = useRef<HTMLImageElement | null>(null)
  const [skinB64, setSkinB64] = useState<string | null>(null)

  const accountUuid = uuid ?? session?.uuid ?? null
  const accountName = username ?? session?.username ?? ''
  const accountType = userType ?? session?.user_type ?? ''

  useEffect(() => {
    if (accountUuid !== session?.uuid) {
      setSkinB64(null)
    }
  }, [accountUuid, session?.uuid])

  useEffect(() => {
    let cancelled = false
    if (!accountUuid) {
      setSkinB64(null)
      return
    }
    if (accountType === 'offline') {
      setSkinB64(cosmetics[accountUuid]?.skin ?? null)
      return
    }
    fetchSkin(accountUuid).then((b64) => {
      if (!cancelled) setSkinB64(b64)
    })
    return () => { cancelled = true }
  }, [accountUuid, accountType, cosmetics])

  useEffect(() => {
    if (!skinB64 || !canvasRef.current) return
    const canvas = canvasRef.current
    const img = new Image()
    imgRef.current = img
    img.onload = () => {
      if (imgRef.current !== img) return
      const ctx = canvas.getContext('2d')
      if (!ctx) return
      canvas.width = size
      canvas.height = size
      ctx.imageSmoothingEnabled = false
      ctx.clearRect(0, 0, size, size)
      ctx.drawImage(img, 8, 8, 8, 8, 0, 0, size, size)
      ctx.drawImage(img, 40, 8, 8, 8, 0, 0, size, size)
    }
    img.src = skinB64.startsWith('data:') ? skinB64 : `data:image/png;base64,${skinB64}`
    return () => { imgRef.current = null }
  }, [skinB64, size])

  if (!accountUuid) return null
  if (!skinB64) {
    return (
      <div
        className="rounded-lg bg-[var(--accent-color)] text-white flex items-center justify-center text-xs font-semibold shrink-0 overflow-hidden"
        style={{ width: size, height: size }}
      >
        {accountName.slice(0, 1).toUpperCase()}
      </div>
    )
  }
  return (
    <canvas
      ref={canvasRef}
      className="rounded-lg shrink-0"
      style={{ width: size, height: size }}
    />
  )
}
