import { useEffect, useRef, useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface SkinViewer3DProps {
  uuid: string
  username: string
  userType: string
  skinDataUrl?: string | null
  customSkinDataUrl?: string | null
  authlibServerUrl?: string | null
  capeDataUrl?: string | null
  customCapeDataUrl?: string | null
}

type AnimType = 'idle' | 'walk' | 'run' | 'fly' | 'wave' | 'crouch' | 'hit' | 'swim'
type BgType = 'white' | 'black' | 'blue' | 'green' | 'purple' | 'orange' | 'gray' | 'pink' | 'cyan' | 'red' | 'gold' | 'slate'

const BG_COLORS: Record<BgType, number> = {
  white: 0xffffff,
  black: 0x111111,
  blue: 0x4c7bd4,
  green: 0x3f9d58,
  purple: 0x7a5fb0,
  orange: 0xd97b32,
  gray: 0x6b7280,
  pink: 0xc96a9a,
  cyan: 0x22aabb,
  red: 0xaa3344,
  gold: 0xd4a843,
  slate: 0x3a4a5a,
}

const BG_OPTIONS: { value: BgType; label: string; color: string }[] = [
  { value: 'white', label: '白色', color: '#ffffff' },
  { value: 'black', label: '黑色', color: '#111111' },
  { value: 'blue', label: '蓝色', color: '#4c7bd4' },
  { value: 'green', label: '绿色', color: '#3f9d58' },
  { value: 'purple', label: '紫色', color: '#7a5fb0' },
  { value: 'orange', label: '橙色', color: '#d97b32' },
  { value: 'gray', label: '灰色', color: '#6b7280' },
  { value: 'pink', label: '粉色', color: '#c96a9a' },
  { value: 'cyan', label: '青色', color: '#22aabb' },
  { value: 'red', label: '红色', color: '#aa3344' },
  { value: 'gold', label: '金色', color: '#d4a843' },
  { value: 'slate', label: '石板', color: '#3a4a5a' },
]

const ANIM_OPTIONS: { value: AnimType; label: string }[] = [
  { value: 'idle', label: '待机' },
  { value: 'walk', label: '走路' },
  { value: 'run', label: '跑步' },
  { value: 'fly', label: '飞行' },
  { value: 'wave', label: '招手' },
  { value: 'crouch', label: '蹲下' },
  { value: 'hit', label: '攻击' },
  { value: 'swim', label: '游泳' },
]

function createSteveSkin(): HTMLCanvasElement {
  const W = 64
  const H = 64
  const canvas = document.createElement('canvas')
  canvas.width = W
  canvas.height = H
  const ctx = canvas.getContext('2d')!
  ctx.imageSmoothingEnabled = false
  const SKIN = '#C8A57B'
  const SKIN_S = '#9C7A52'
  const HAIR = '#312019'
  const SHIRT = '#3D8EC4'
  const SHIRT_S = '#2A6CA0'
  const PANTS = '#3B3DA8'
  const PANTS_S = '#2B2E7C'
  const EYE = '#F2F2F2'
  const PUPIL = '#312019'
  const SHOES = '#2E2E2E'
  const fill = (x: number, y: number, w: number, h: number, c: string) => {
    ctx.fillStyle = c
    ctx.fillRect(x, y, w, h)
  }

  
  fill(8, 0, 8, 8, HAIR)          
  fill(16, 0, 8, 8, SKIN_S)       
  fill(0, 8, 8, 8, SKIN)          
  fill(24, 8, 8, 8, SKIN)         
  fill(16, 8, 8, 8, SKIN)         
  fill(8, 8, 8, 8, SKIN)          
  fill(8, 8, 8, 2, HAIR)          
  fill(8, 10, 2, 1, HAIR)         
  fill(14, 10, 2, 1, HAIR)        
  fill(9, 11, 2, 2, EYE)          
  fill(13, 11, 2, 2, EYE)         
  fill(9, 11, 1, 2, PUPIL)        
  fill(13, 11, 1, 2, PUPIL)       
  fill(10, 14, 4, 1, SKIN_S)      

  
  fill(20, 16, 8, 4, SHIRT_S)     
  fill(28, 16, 8, 4, SHIRT_S)     
  fill(16, 20, 4, 12, SHIRT_S)    
  fill(20, 20, 8, 12, SHIRT)      
  fill(28, 20, 4, 12, SHIRT_S)    
  fill(32, 20, 8, 12, SHIRT_S)    

  
  fill(44, 16, 4, 4, SHIRT_S)     
  fill(48, 16, 4, 4, SHIRT_S)     
  fill(40, 16, 4, 4, SKIN)        
  fill(44, 16, 4, 12, SHIRT)      
  fill(48, 16, 4, 12, SKIN)       
  fill(52, 16, 4, 12, SHIRT_S)    
  fill(44, 24, 4, 4, SKIN)        
  fill(52, 24, 4, 4, SKIN)        

  
  fill(36, 48, 4, 4, SHIRT_S)
  fill(40, 48, 4, 4, SHIRT_S)
  fill(32, 48, 4, 4, SKIN)
  fill(36, 48, 4, 12, SHIRT)
  fill(40, 48, 4, 12, SKIN)
  fill(44, 48, 4, 12, SHIRT_S)
  fill(36, 56, 4, 4, SKIN)
  fill(44, 56, 4, 4, SKIN)

  
  fill(4, 16, 4, 4, PANTS_S)
  fill(8, 16, 4, 4, PANTS_S)
  fill(0, 16, 4, 4, PANTS_S)
  fill(4, 16, 4, 12, PANTS)
  fill(8, 16, 4, 12, PANTS_S)
  fill(12, 16, 4, 12, PANTS_S)
  fill(4, 24, 4, 4, SHOES)
  fill(12, 24, 4, 4, SHOES)

  
  fill(4, 48, 4, 4, PANTS_S)
  fill(8, 48, 4, 4, PANTS_S)
  fill(0, 48, 4, 4, PANTS_S)
  fill(4, 48, 4, 12, PANTS)
  fill(8, 48, 4, 12, PANTS_S)
  fill(12, 48, 4, 12, PANTS_S)
  fill(4, 56, 4, 4, SHOES)
  fill(12, 56, 4, 4, SHOES)

  return canvas
}

function dataUrlFromCanvas(canvas: HTMLCanvasElement): string {
  return canvas.toDataURL('image/png')
}

export function SkinViewer3D({
  uuid, username, userType,
  skinDataUrl, customSkinDataUrl, authlibServerUrl,
  capeDataUrl, customCapeDataUrl,
}: SkinViewer3DProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const viewerRef = useRef<any>(null)
  const observerRef = useRef<ResizeObserver | null>(null)
  const mountedRef = useRef(true)
  const resizeTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const skinUrlRef = useRef<string | null>(null)
  const capeUrlRef = useRef<string | null>(null)
  const animRef = useRef<AnimType>('idle')

  const [skinTextureUrl, setSkinTextureUrl] = useState<string | null>(null)
  const [capeTextureUrl, setCapeTextureUrl] = useState<string | null>(null)
  const [defaultSkinUrl, setDefaultSkinUrl] = useState<string | null>(null)
  const [skinLoading, setSkinLoading] = useState(false)
  const [anim, setAnim] = useState<AnimType>('idle')
  const [bg, setBg] = useState<BgType>('blue')
  const [showCape, setShowCape] = useState(true)
  const [viewerReady, setViewerReady] = useState(false)
  const [skinError, setSkinError] = useState<string | null>(null)

  
  
  useEffect(() => {
    let cancelled = false
    invoke<string | null>('get_default_skin', { kind: 'steve' })
      .then((b64) => {
        if (b64 && !cancelled && mountedRef.current) {
          setDefaultSkinUrl(`data:image/png;base64,${b64}`)
        }
      })
      .catch(() => {})
    return () => { cancelled = true }
  }, [])

  
  
  useEffect(() => {
    mountedRef.current = true
    setSkinLoading(true)
    setSkinTextureUrl(null)
    setCapeTextureUrl(null)
    setSkinError(null)

    const run = async () => {
      let skin: string | null = null
      let cape: string | null = null

      if (customSkinDataUrl) {
        skin = customSkinDataUrl
      } else if (skinDataUrl) {
        skin = skinDataUrl
      } else if (userType === 'offline') {
        skin = null
      } else if (userType === 'msa' || userType === 'mojang') {
        try {
          const [s, c] = await invoke<[string | null, string | null]>('get_skin_textures', { uuid })
          if (s) skin = `data:image/png;base64,${s}`
          if (c) cape = `data:image/png;base64,${c}`
        } catch (e) {
          if (mountedRef.current) setSkinError(`获取皮肤失败: ${e}`)
        }
      } else if (userType === 'authlib' && authlibServerUrl) {
        try {
          const [s, c] = await invoke<[string | null, string | null]>('get_authlib_textures', { serverUrl: authlibServerUrl, uuid })
          if (s) skin = `data:image/png;base64,${s}`
          if (c) cape = `data:image/png;base64,${c}`
        } catch {}
      }

      if (customCapeDataUrl) {
        cape = customCapeDataUrl
      } else if (capeDataUrl) {
        cape = capeDataUrl
      }

      if (mountedRef.current) {
        setSkinTextureUrl(skin)
        setCapeTextureUrl(cape)
        setSkinLoading(false)
      }
    }
    run()
    return () => { mountedRef.current = false }
  }, [uuid, userType, skinDataUrl, customSkinDataUrl, capeDataUrl, customCapeDataUrl, authlibServerUrl])

  
  useEffect(() => { skinUrlRef.current = skinTextureUrl ?? defaultSkinUrl }, [skinTextureUrl, defaultSkinUrl])
  useEffect(() => { capeUrlRef.current = capeTextureUrl }, [capeTextureUrl])

  const initViewer = useCallback(async (container: HTMLDivElement) => {
    const mod = await import('skinview3d')
    const { SkinViewer, IdleAnimation } = mod
    const SteveCanvas = createSteveSkin()
    const fallbackUrl = defaultSkinUrl || dataUrlFromCanvas(SteveCanvas)

    const viewer = new SkinViewer({
      width: container.clientWidth || 240,
      height: container.clientHeight || 320,
      skin: skinUrlRef.current ?? fallbackUrl,
      ...(showCape && capeUrlRef.current ? { cape: capeUrlRef.current } : {}),
      enableControls: true,
    })
    viewer.autoRotate = true
    viewer.autoRotateSpeed = 0.5
    viewer.animation = new IdleAnimation()
    viewer.animation.speed = 0.8
    if (username?.trim()) {
      viewer.nameTag = username.trim()
      
      
      try {
        
        ;(viewer as unknown as { nameTagYOffset: number }).nameTagYOffset = 28
        viewer.nameTag?.scale.setScalar(2.1)
        // Override canvas width to remove name length limit
        const nt = viewer.nameTag as any
        if (nt && nt.canvas) {
          const ctx = nt.canvas.getContext('2d')
          if (ctx) {
            const metrics = ctx.measureText(nt.text || username.trim())
            const needed = Math.ceil(metrics.width) + 16
            if (needed > nt.canvas.width) {
              nt.canvas.width = needed
              nt.redraw?.() ?? (() => { ctx.font = '24px sans-serif'; ctx.fillStyle = 'white'; ctx.strokeStyle = 'black'; ctx.lineWidth = 3; ctx.strokeText(nt.text || username.trim(), needed / 2, 18); ctx.fillText(nt.text || username.trim(), needed / 2, 18) })()
              if (nt.material?.map) nt.material.map.needsUpdate = true
            }
          }
        }
      } catch {}
    }
    try {
      const hasBack = showCape && !!capeUrlRef.current
      viewer.playerObject.backEquipment = !hasBack ? null : animRef.current === 'fly' ? 'elytra' : 'cape'
    } catch {}
    viewer.background = BG_COLORS[bg] ?? 0x4c7bd4
    viewer.controls.enableZoom = false
    viewer.controls.enablePan = false
    
    
    
    if (viewer.playerObject?.skin && typeof viewer.playerObject.skin.setOuterLayerVisible === 'function') {
      viewer.playerObject.skin.setOuterLayerVisible(false)
    }
    
    if (viewer.playerObject?.ears) {
      viewer.playerObject.ears.visible = false
    }
    return viewer
  }, [username, bg, showCape, defaultSkinUrl])

  useEffect(() => {
    const v = viewerRef.current
    if (!v || !username) return
    try {
      v.nameTag = username.trim() || '玩家'
      ;(v as unknown as { nameTagYOffset: number }).nameTagYOffset = 28
      v.nameTag?.scale.setScalar(2.1)
      const nt = v.nameTag as any
      if (nt && nt.canvas) {
        const ctx = nt.canvas.getContext('2d')
        if (ctx) {
          const metrics = ctx.measureText(nt.text || username.trim())
          const needed = Math.ceil(metrics.width) + 16
          if (needed > nt.canvas.width) {
            nt.canvas.width = needed
            nt.redraw?.()
            if (nt.material?.map) nt.material.map.needsUpdate = true
          }
        }
      }
    } catch {}
  }, [username])

  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    const runInit = async () => {
      if (!viewerRef.current) {
        container.innerHTML = ''
        const v = await initViewer(container)
        container.innerHTML = ''
        viewerRef.current = v
        container.appendChild(v.canvas)
        const w = container.clientWidth, h = container.clientHeight
        if (w > 0 && h > 0) v.setSize(w, h)
        
        
        
        try {
          if (skinUrlRef.current) {
            v.loadSkin(skinUrlRef.current)
            if (v.playerObject?.skin && typeof v.playerObject.skin.setOuterLayerVisible === 'function') {
              v.playerObject.skin.setOuterLayerVisible(false)
            }
          }
        } catch {}
        try {
          if (showCape && capeUrlRef.current) v.loadCape(capeUrlRef.current)
        } catch {}
        setViewerReady(true)
      }
    }
    runInit()

    observerRef.current = new ResizeObserver(() => {
      if (resizeTimeoutRef.current) clearTimeout(resizeTimeoutRef.current)
      resizeTimeoutRef.current = setTimeout(() => {
        const v = viewerRef.current
        const c = containerRef.current
        if (v && c) {
          const w = c.clientWidth, h = c.clientHeight
          if (w > 0 && h > 0) v.setSize(w, h)
        }
      }, 200)
    })
    observerRef.current.observe(container)

    return () => {
      mountedRef.current = false
      if (resizeTimeoutRef.current) clearTimeout(resizeTimeoutRef.current)
      observerRef.current?.disconnect()
      const v = viewerRef.current
      const c = containerRef.current
      if (v) { try { v.dispose() } catch {} viewerRef.current = null }
      if (c) c.innerHTML = ''
      setViewerReady(false)
    }
  
  }, [])

  
  
  useEffect(() => {
    const v = viewerRef.current
    if (!v || !defaultSkinUrl || skinTextureUrl) return
    try {
      v.loadSkin(defaultSkinUrl)
      if (v.playerObject?.skin && typeof v.playerObject.skin.setOuterLayerVisible === 'function') {
        v.playerObject.skin.setOuterLayerVisible(false)
      }
      if (v.playerObject?.ears) v.playerObject.ears.visible = false
    } catch {}
  }, [defaultSkinUrl])

  
  useEffect(() => {
    const v = viewerRef.current
    const url = skinTextureUrl
    if (!v || !url) return
    try { 
      v.loadSkin(url)
      
      if (v.playerObject?.skin && typeof v.playerObject.skin.setOuterLayerVisible === 'function') {
        v.playerObject.skin.setOuterLayerVisible(false)
      }
      
      if (v.playerObject?.ears) {
        v.playerObject.ears.visible = false
      }
    } catch {}
  }, [skinTextureUrl])

  
  useEffect(() => {
    const v = viewerRef.current
    if (!v) return
    if (showCape && capeTextureUrl) {
      try { v.loadCape(capeTextureUrl) } catch {}
    } else {
      try { v.resetCape() } catch {}
    }
    
    try {
      if (v.playerObject) {
        const hasBack = showCape && !!capeTextureUrl
        v.playerObject.backEquipment = !hasBack ? null : animRef.current === 'fly' ? 'elytra' : 'cape'
      }
    } catch {}
    
    if (v.playerObject?.skin && typeof v.playerObject.skin.setOuterLayerVisible === 'function') {
      v.playerObject.skin.setOuterLayerVisible(false)
    }
    if (v.playerObject?.ears) {
      v.playerObject.ears.visible = false
    }
  }, [capeTextureUrl, showCape, animRef])

  const setAnimation = useCallback((type: AnimType) => {
    setAnim(type)
    animRef.current = type
    const v = viewerRef.current
    if (!v) return
    import('skinview3d').then(mod => {
      const map: Record<AnimType, new () => any> = {
        idle: mod.IdleAnimation, walk: mod.WalkingAnimation, run: mod.RunningAnimation,
        fly: mod.FlyingAnimation, wave: mod.WaveAnimation, crouch: mod.CrouchAnimation,
        hit: mod.HitAnimation, swim: mod.SwimAnimation,
      }
      const Factory = map[type]
      if (!Factory) return
      v.animation = new Factory()
      const speeds: Record<AnimType, number> = { idle: 0.8, walk: 0.8, run: 0.6, fly: 0.8, wave: 0.8, crouch: 0.5, hit: 0.9, swim: 0.7 }
      v.animation.speed = speeds[type] ?? 0.7
      
      try {
        if (v.playerObject) {
          const hasBack = showCape && !!capeUrlRef.current
          v.playerObject.backEquipment = !hasBack ? null : type === 'fly' ? 'elytra' : 'cape'
        }
      } catch {}
    })
  }, [showCape])

  const setBackground = useCallback((color: BgType) => {
    setBg(color)
    const v = viewerRef.current
    if (!v) return
    v.background = BG_COLORS[color] ?? 0x4c7bd4
  }, [])

  return (
    <div className="flex flex-col gap-2 h-full">
      {}
      <div className="relative rounded-xl overflow-hidden border border-white/10 bg-surface-800" style={{ minHeight: 280, flex: 1 }}>
        {skinLoading && !skinTextureUrl && (
          <div className="absolute inset-0 flex items-center justify-center z-10 pointer-events-none">
            <div className="text-center">
              <div className="w-6 h-6 border-2 border-accent-400 border-t-transparent rounded-full animate-spin mx-auto mb-2" />
              <span className="text-xs text-surface-400">加载皮肤中...</span>
            </div>
          </div>
        )}
        {skinError && (
          <div className="absolute inset-0 flex items-center justify-center z-10 pointer-events-none">
            <div className="text-center">
              <p className="text-xs text-red-400">{skinError}</p>
              <p className="text-[10px] text-surface-500 mt-1">将使用默认史蒂夫皮肤</p>
            </div>
          </div>
        )}
        {!viewerReady && !skinLoading && !skinTextureUrl && !skinError && (
          <div className="absolute inset-0 flex items-center justify-center z-10 pointer-events-none">
            <div className="text-center">
              <div className="w-8 h-8 border-2 border-accent-400 border-t-transparent rounded-full animate-spin mx-auto mb-2" />
              <span className="text-xs text-surface-400">初始化中...</span>
            </div>
          </div>
        )}
        <div ref={containerRef} className="w-full h-full" style={{ pointerEvents: 'auto' }} />
      </div>

      {}
      <div className="grid grid-cols-4 gap-1.5">
        {ANIM_OPTIONS.map(a => (
          <button key={a.value} onClick={() => setAnimation(a.value)}
            className={`text-xs py-1.5 rounded-lg transition-colors whitespace-nowrap ${
              anim === a.value ? 'bg-accent-500/30 text-accent-300 border border-accent-500/50'
                : 'bg-surface-800 text-surface-400 border border-white/5 hover:border-white/20 hover:text-surface-200'
            }`}>
            {a.label}
          </button>
        ))}
      </div>

      {}
      <div className="flex items-center gap-3 flex-wrap">
        <span className="text-xs text-surface-500 shrink-0">背景</span>
        <div className="flex gap-1 flex-wrap">
          {BG_OPTIONS.map(b => (
            <button key={b.value} onClick={() => setBackground(b.value)} title={b.label}
              className={`flex items-center gap-1.5 px-2 py-1 rounded-lg text-xs border transition-colors whitespace-nowrap ${
                bg === b.value ? 'border-accent-500/50 bg-accent-500/20 text-accent-200'
                  : 'border-white/10 bg-surface-800 text-surface-400 hover:border-white/20'
              }`}>
              <span className="w-3 h-3 rounded-full border border-white/20 inline-block" style={{ background: b.color }} />
              <span className="hidden sm:inline">{b.label}</span>
            </button>
          ))}
        </div>
        <div className="flex items-center gap-1.5 ml-auto">
          <button onClick={() => setShowCape(v => !v)}
            className={`px-2.5 py-1 rounded-lg text-xs border transition-colors whitespace-nowrap ${
              showCape ? 'border-accent-500/50 bg-accent-500/20 text-accent-200'
                : 'border-white/10 bg-surface-800 text-surface-400 hover:border-white/20'
            }`}>
            {showCape ? '披风: 开' : '披风: 关'}
          </button>
          <span className="text-xs text-surface-600">拖拽旋转</span>
        </div>
      </div>
    </div>
  )
}
