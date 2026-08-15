import { useEffect, useRef, useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft, Map as MapIcon, Hash, RefreshCw, ZoomIn, ZoomOut, Maximize2, Loader2, Copy, Check } from 'lucide-react'

interface WorldInfo {
  name: string
  path: string
  game_mode: string
  seed: number | null
  size_kb: number
  icon?: string
  play_time: number
  spawn_x?: number | null
  spawn_z?: number | null
}

interface RegionTile {
  region_x: number
  region_z: number
  pixels: number[] | Uint8Array | ArrayBuffer
}

const REGION = 512
const MIN_ZOOM = 0.125
const MAX_ZOOM = 16

const GAME_MODE_CN: Record<string, string> = {
  survival: '生存',
  creative: '创造',
  adventure: '冒险',
  spectator: '旁观',
  unknown: '未知',
  '0': '生存',
  '1': '创造',
  '2': '冒险',
  '3': '旁观',
}
function gameModeCn(mode: string | undefined | null): string {
  if (!mode) return '未知'
  const lower = mode.toLowerCase()
  return GAME_MODE_CN[lower] ?? lower
}

function formatPlayTime(s: number): string {
  if (!s) return ''
  const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60)
  return h > 0 ? `${h}h ${m}m` : `${m}m`
}

function normalizePixels(raw: unknown): Uint8ClampedArray {
  if (raw instanceof Uint8Array) return new Uint8ClampedArray(raw)
  if (raw instanceof ArrayBuffer) return new Uint8ClampedArray(raw)
  if (Array.isArray(raw)) return new Uint8ClampedArray(raw)
  if (raw && typeof raw === 'object' && 'buffer' in raw) {
    const b = (raw as { buffer: ArrayBufferLike }).buffer
    return new Uint8ClampedArray(b)
  }
  return new Uint8ClampedArray()
}

export default function WorldMapPreview() {
  const { instanceId: instanceIdParam } = useParams<{ instanceId: string }>()
  const navigate = useNavigate()
  const instanceId = instanceIdParam || ''

  const [worlds, setWorlds] = useState<WorldInfo[]>([])
  const [selectedWorld, setSelectedWorld] = useState<WorldInfo | null>(null)
  const [loading, setLoading] = useState(false)
  const [mapLoading, setMapLoading] = useState(false)
  const [mapError, setMapError] = useState<string | null>(null)
  const [copiedSeed, setCopiedSeed] = useState(false)

  const [view, setView] = useState({ cx: 0, cz: 0, zoom: 2 })
  const [canvasSize, setCanvasSize] = useState({ w: 0, h: 0 })
  const [tilesVersion, setTilesVersion] = useState(0)
  const [, setLoadCount] = useState(0)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const tilesRef = useRef<Map<string, HTMLCanvasElement>>(new Map())
  const inFlightRef = useRef<Set<string>>(new Set())
  const dragRef = useRef<{ startX: number; startY: number; cx: number; cz: number; zoom: number } | null>(null)
  const selectedWorldRef = useRef<WorldInfo | null>(null)
  selectedWorldRef.current = selectedWorld

  
  useEffect(() => {
    if (!instanceId) return
    setLoading(true)
    invoke<WorldInfo[]>('list_instance_worlds', { instanceId })
      .then(data => {
        setWorlds(data)
        if (data.length > 0) selectWorld(data[0])
      })
      .catch((e) => { console.error('list_instance_worlds error:', e); setWorlds([]) })
      .finally(() => setLoading(false))
  }, [instanceId])

  const resetTiles = useCallback(() => {
    tilesRef.current.clear()
    inFlightRef.current.clear()
    setLoadCount(0)
    setTilesVersion(v => v + 1)
    setMapLoading(false)
    setMapError(null)
  }, [])

  const selectWorld = useCallback((world: WorldInfo) => {
    setSelectedWorld(world)
    resetTiles()
    setView({ cx: world.spawn_x ?? 0, cz: world.spawn_z ?? 0, zoom: 2 })
  }, [resetTiles])

  const scheduleLoad = useCallback((v: { cx: number; cz: number; zoom: number }, size: { w: number; h: number }, world: WorldInfo | null) => {
    if (!world || size.w <= 0 || size.h <= 0) return
    const left = v.cx - (size.w / 2) / v.zoom
    const right = v.cx + (size.w / 2) / v.zoom
    const top = v.cz - (size.h / 2) / v.zoom
    const bottom = v.cz + (size.h / 2) / v.zoom
    const rx0 = Math.floor(left / REGION)
    const rx1 = Math.floor((right - 1) / REGION)
    const rz0 = Math.floor(top / REGION)
    const rz1 = Math.floor((bottom - 1) / REGION)
    let any = false
    for (let rz = rz0; rz <= rz1; rz++) {
      for (let rx = rx0; rx <= rx1; rx++) {
        const key = `${rx},${rz}`
        if (tilesRef.current.has(key)) continue
        if (inFlightRef.current.has(key)) { any = true; continue }
        inFlightRef.current.add(key)
        any = true
        setLoadCount(c => c + 1)
        invoke<RegionTile | null>('world_map_region', { worldPath: world.path, regionX: rx, regionZ: rz })
          .then(res => {
            if (res) {
              const canvas = document.createElement('canvas')
              canvas.width = REGION
              canvas.height = REGION
              const ctx = canvas.getContext('2d')
              if (ctx) {
                const img = ctx.createImageData(REGION, REGION)
                img.data.set(normalizePixels(res.pixels).slice(0, REGION * REGION * 4))
                ctx.putImageData(img, 0, 0)
                tilesRef.current.set(key, canvas)
              }
            }
          })
          .catch(e => { console.error('world_map_region error:', key, e); setMapError(String(e)) })
          .finally(() => {
            inFlightRef.current.delete(key)
            setLoadCount(c => c - 1)
            setTilesVersion(v => v + 1)
          })
      }
    }
    setMapLoading(any)
  }, [])

  useEffect(() => {
    scheduleLoad(view, canvasSize, selectedWorldRef.current)
  }, [view, canvasSize, selectedWorld, scheduleLoad])

  useEffect(() => {
    const el = containerRef.current
    if (!el) return
    const ro = new ResizeObserver(() => {
      setCanvasSize({ w: el.clientWidth, h: el.clientHeight })
    })
    ro.observe(el)
    setCanvasSize({ w: el.clientWidth, h: el.clientHeight })
    return () => ro.disconnect()
  }, [])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const ctx = canvas.getContext('2d')
    if (!ctx) return
    const { w, h } = canvasSize
    if (w <= 0 || h <= 0) return
    canvas.width = w
    canvas.height = h
    ctx.imageSmoothingEnabled = false
    ctx.fillStyle = '#0b0e14'
    ctx.fillRect(0, 0, w, h)
    const { cx, cz, zoom } = view
    tilesRef.current.forEach((tile, key) => {
      const [rx, rz] = key.split(',').map(Number)
      const sx = (rx * REGION - cx) * zoom + w / 2
      const sz = (rz * REGION - cz) * zoom + h / 2
      const side = REGION * zoom
      if (sx + side < 0 || sz + side < 0 || sx > w || sz > h) return
      ctx.drawImage(tile, sx, sz, side, side)
    })
    
    ctx.strokeStyle = 'rgba(255,255,255,0.04)'
    ctx.lineWidth = 1
    for (let i = -20; i <= 20; i++) {
      const gx = (i * REGION - cx) * zoom + w / 2
      const gz = (i * REGION - cz) * zoom + h / 2
      if (gx >= 0 && gx <= w) { ctx.beginPath(); ctx.moveTo(gx, 0); ctx.lineTo(gx, h); ctx.stroke() }
      if (gz >= 0 && gz <= h) { ctx.beginPath(); ctx.moveTo(0, gz); ctx.lineTo(w, gz); ctx.stroke() }
    }
    
    ctx.strokeStyle = 'rgba(255,80,80,0.6)'
    ctx.beginPath()
    ctx.moveTo(w / 2 - 6, h / 2); ctx.lineTo(w / 2 + 6, h / 2)
    ctx.moveTo(w / 2, h / 2 - 6); ctx.lineTo(w / 2, h / 2 + 6)
    ctx.stroke()
    
    if (selectedWorld && selectedWorld.spawn_x != null && selectedWorld.spawn_z != null) {
      const spx = (selectedWorld.spawn_x - cx) * zoom + w / 2
      const spz = (selectedWorld.spawn_z - cz) * zoom + h / 2
      if (spx >= 0 && spx <= w && spz >= 0 && spz <= h) {
        ctx.fillStyle = '#fbbf24'
        ctx.beginPath()
        ctx.arc(spx, spz, 5, 0, Math.PI * 2)
        ctx.fill()
        ctx.strokeStyle = '#92400e'
        ctx.lineWidth = 1.5
        ctx.stroke()
      }
    }
  }, [view, canvasSize, tilesVersion, selectedWorld])

  const handleWheel = useCallback((e: React.WheelEvent) => {
    const canvas = canvasRef.current
    if (!canvas) return
    const { w, h } = canvasSize
    if (w <= 0 || h <= 0) return
    const factor = e.deltaY < 0 ? 1.25 : 1 / 1.25
    const newZoom = Math.min(Math.max(view.zoom * factor, MIN_ZOOM), MAX_ZOOM)
    const rect = canvas.getBoundingClientRect()
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top
    const worldX = view.cx + (mx - w / 2) / view.zoom
    const worldZ = view.cz + (my - h / 2) / view.zoom
    setView({
      cx: worldX - (mx - w / 2) / newZoom,
      cz: worldZ - (my - h / 2) / newZoom,
      zoom: newZoom,
    })
  }, [view, canvasSize])

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return
    dragRef.current = { startX: e.clientX, startY: e.clientY, cx: view.cx, cz: view.cz, zoom: view.zoom }
  }, [view])

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    const d = dragRef.current
    if (!d) return
    setView({
      cx: d.cx - (e.clientX - d.startX) / d.zoom,
      cz: d.cz - (e.clientY - d.startY) / d.zoom,
      zoom: d.zoom,
    })
  }, [])

  const handleMouseUp = useCallback(() => { dragRef.current = null }, [])

  const handleZoomIn = () => setView(v => ({ ...v, zoom: Math.min(v.zoom * 1.5, MAX_ZOOM) }))
  const handleZoomOut = () => setView(v => ({ ...v, zoom: Math.max(v.zoom / 1.5, MIN_ZOOM) }))
  const handleReset = useCallback(() => {
    if (!selectedWorldRef.current) return
    setView({ cx: selectedWorldRef.current.spawn_x ?? 0, cz: selectedWorldRef.current.spawn_z ?? 0, zoom: 2 })
  }, [])

  const copySeed = async () => {
    if (selectedWorld?.seed == null) return
    try {
      await navigator.clipboard.writeText(String(selectedWorld.seed))
      setCopiedSeed(true)
      setTimeout(() => setCopiedSeed(false), 2000)
    } catch {}
  }

  return (
    <div className="flex h-full bg-surface-950">
      {}
      <div className="w-64 shrink-0 border-r border-white/5 flex flex-col">
        <div className="p-3 border-b border-white/5 flex items-center gap-2">
          <button onClick={() => navigate(-1)} className="p-1.5 rounded-lg hover:bg-surface-800 transition-colors">
            <ArrowLeft className="w-4 h-4 text-surface-400" />
          </button>
          <span className="text-sm font-medium text-surface-200">世界地图</span>
        </div>

        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          {loading && worlds.length === 0 ? (
            <div className="text-xs text-surface-500 text-center py-4">加载中...</div>
          ) : worlds.length === 0 ? (
            <div className="text-xs text-surface-500 text-center py-4">暂无世界</div>
          ) : (
            worlds.map(w => (
              <button
                key={w.path}
                onClick={() => selectWorld(w)}
                className={`w-full text-left px-3 py-2.5 rounded-lg transition-colors ${
                  selectedWorld?.path === w.path
                    ? 'bg-blue-500/20 text-blue-300'
                    : 'hover:bg-surface-800 text-surface-300'
                }`}
              >
                <div className="flex items-center gap-2">
                  {w.icon ? (
                    <img src={w.icon} alt="" className="w-5 h-5 rounded shrink-0" />
                  ) : (
                    <MapIcon className="w-4 h-4 shrink-0" />
                  )}
                  <div className="min-w-0 flex-1">
                    <div className="text-xs font-medium truncate">{w.name}</div>
                    <div className="text-[10px] text-surface-500">
                      {w.seed !== null ? `种子: ${w.seed}` : '种子: 未知'}
                      {w.game_mode && ` · ${gameModeCn(w.game_mode)}`}
                    </div>
                  </div>
                </div>
              </button>
            ))
          )}
        </div>
      </div>

      {}
      <div className="flex-1 flex flex-col min-w-0">
        {}
        <div className="shrink-0 px-4 py-3 border-b border-white/5 flex items-center gap-4">
          {selectedWorld && (
            <>
              <div className="flex-1 min-w-0">
                <h2 className="text-sm font-medium text-surface-200 truncate">{selectedWorld.name}</h2>
                <div className="flex items-center gap-3 mt-0.5 flex-wrap">
                  <span className="text-[11px] text-surface-500 flex items-center gap-1">
                    <Hash className="w-3 h-3" />
                    种子: {selectedWorld.seed ?? '未知'}
                    <button
                      onClick={copySeed}
                      className="text-surface-600 hover:text-surface-300 transition-colors ml-0.5"
                      title="复制种子"
                    >
                      {copiedSeed ? <Check className="w-3 h-3 text-green-400" /> : <Copy className="w-3 h-3" />}
                    </button>
                  </span>
                  <span className="text-[11px] text-surface-500">
                    模式: {gameModeCn(selectedWorld.game_mode)}
                  </span>
                  {selectedWorld.play_time > 0 && (
                    <span className="text-[11px] text-surface-500">
                      游玩: {formatPlayTime(selectedWorld.play_time)}
                    </span>
                  )}
                  <span className="text-[11px] text-surface-500">
                    大小: {(selectedWorld.size_kb / 1024).toFixed(1)} MB
                  </span>
                </div>
              </div>

              {}
              <div className="flex items-center gap-1">
                <button onClick={handleZoomOut} className="p-1.5 rounded-lg hover:bg-surface-800 text-surface-400 transition-colors" title="缩小">
                  <ZoomOut className="w-4 h-4" />
                </button>
                <button onClick={handleReset} className="p-1.5 rounded-lg hover:bg-surface-800 text-surface-400 transition-colors" title="回到出生点">
                  <Maximize2 className="w-4 h-4" />
                </button>
                <button onClick={handleZoomIn} className="p-1.5 rounded-lg hover:bg-surface-800 text-surface-400 transition-colors" title="放大">
                  <ZoomIn className="w-4 h-4" />
                </button>
                <button
                  onClick={() => { if (selectedWorld) selectWorld(selectedWorld) }}
                  className="p-1.5 rounded-lg hover:bg-surface-800 text-surface-400 transition-colors"
                  title="刷新"
                >
                  <RefreshCw className={`w-4 h-4 ${mapLoading ? 'animate-spin' : ''}`} />
                </button>
              </div>
            </>
          )}
        </div>

        {}
        <div ref={containerRef} className="flex-1 overflow-hidden relative bg-surface-900">
          {!selectedWorld ? (
            <div className="absolute inset-0 flex items-center justify-center">
              <div className="text-center">
                <MapIcon className="w-12 h-12 text-surface-700 mx-auto mb-3" />
                <p className="text-sm text-surface-500">选择一个世界开始预览</p>
              </div>
            </div>
          ) : (
            <>
              <div
                className="flex-1 relative cursor-move"
                onMouseDown={handleMouseDown}
                onMouseMove={handleMouseMove}
                onMouseUp={handleMouseUp}
                onMouseLeave={handleMouseUp}
                onWheel={handleWheel}
              >
                <canvas ref={canvasRef} className="w-full h-full" style={{ imageRendering: 'pixelated' }} />
                {mapLoading && (
                  <div className="absolute top-3 left-1/2 -translate-x-1/2 flex items-center gap-2 px-3 py-1.5 rounded-full bg-black/60 text-[11px] text-surface-300 pointer-events-none">
                    <Loader2 className="w-3.5 h-3.5 animate-spin text-sky-400" />
                    加载地形...
                  </div>
                )}
                {mapError && (
                  <div className="absolute bottom-3 left-3 right-3 px-3 py-2 rounded-lg bg-red-950/80 text-[10px] text-red-300 pointer-events-none">
                    {mapError}
                  </div>
                )}
              </div>
            </>
          )}
        </div>

        {}
        {selectedWorld && (
          <div className="shrink-0 px-4 py-2 border-t border-white/5 flex items-center gap-4 text-[10px] text-surface-500">
            <span>缩放: {view.zoom.toFixed(2)}x</span>
            <span>中心: {Math.round(view.cx)}, {Math.round(view.cz)}</span>
            <span className="ml-auto">滚轮缩放 · 拖拽移动 · 无限加载</span>
          </div>
        )}
      </div>
    </div>
  )
}
