import { useEffect, useRef, useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useParams, useNavigate, useSearchParams } from 'react-router-dom'
import { ArrowLeft, Map as MapIcon, Hash, RefreshCw, ZoomIn, ZoomOut, Loader2, Copy, Check, Gamepad2, Clock, Skull, Search, Compass, Layers, LocateFixed } from 'lucide-react'

interface WorldInfo {
  name: string
  path: string
  game_mode: string
  seed: number | null
  size_kb: number
  icon?: string
  play_time: number
  is_hardcore?: boolean
  spawn_x?: number | null
  spawn_z?: number | null
}

interface RegionTile {
  region_x: number
  region_z: number
  pixels: number[] | Uint8Array | ArrayBuffer
}

interface SeedStructure {
  name: string
  x: number
  z: number
  distance: number
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
  const [searchParams] = useSearchParams()
  const instanceId = instanceIdParam || ''

  const [worlds, setWorlds] = useState<WorldInfo[]>([])
  const [selectedWorld, setSelectedWorld] = useState<WorldInfo | null>(null)
  const [loading, setLoading] = useState(false)
  const [mapLoading, setMapLoading] = useState(false)
  const [mapError, setMapError] = useState<string | null>(null)
  const [copiedSeed, setCopiedSeed] = useState(false)
  const [worldSearch, setWorldSearch] = useState('')
  const [mapMode, setMapMode] = useState<'world' | 'seed' | 'biomes'>('world')
  const [structures, setStructures] = useState<SeedStructure[]>([])
  const [seedSpawn, setSeedSpawn] = useState({ x: 0, z: 0 })

  const [view, setView] = useState({ cx: 0, cz: 0, zoom: 2 })
  const [canvasSize, setCanvasSize] = useState({ w: 0, h: 0 })
  const [tilesVersion, setTilesVersion] = useState(0)
  const [, setLoadCount] = useState(0)
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const tilesRef = useRef<Map<string, HTMLCanvasElement>>(new Map())
  const inFlightRef = useRef<Set<string>>(new Set())
  const requestEpochRef = useRef(0)
  const dragRef = useRef<{ startX: number; startY: number; cx: number; cz: number; zoom: number } | null>(null)
  const selectedWorldRef = useRef<WorldInfo | null>(null)
  selectedWorldRef.current = selectedWorld

  
  useEffect(() => {
    if (!instanceId) return
    setLoading(true)
    invoke<WorldInfo[]>('list_instance_worlds', { instanceId })
      .then(data => {
        setWorlds(data)
        if (data.length > 0) {
          const requestedPath = searchParams.get('world')
          selectWorld(data.find((world) => world.path === requestedPath) ?? data[0])
        }
      })
      .catch((e) => { console.error('list_instance_worlds error:', e); setWorlds([]) })
      .finally(() => setLoading(false))
  }, [instanceId, searchParams])

  const resetTiles = useCallback(() => {
    tilesRef.current.clear()
    inFlightRef.current.clear()
    requestEpochRef.current += 1
    setLoadCount(0)
    setTilesVersion(v => v + 1)
    setMapLoading(false)
    setMapError(null)
  }, [])

  const loadSeedData = useCallback(async (world: WorldInfo) => {
    if (world.seed == null) return
    try {
      const result = await invoke<{ spawn_x: number; spawn_z: number; structures: SeedStructure[] }>('seed_results', {
        seed: world.seed,
        worldType: 'normal',
      })
      setSeedSpawn({ x: result.spawn_x, z: result.spawn_z })
      setStructures(result.structures ?? [])
    } catch (error) {
      console.error('seed_results error:', error)
      setStructures([])
    }
  }, [])

  const selectWorld = useCallback((world: WorldInfo) => {
    setSelectedWorld(world)
    setMapMode('world')
    void loadSeedData(world)
    resetTiles()
    setView({ cx: world.spawn_x ?? 0, cz: world.spawn_z ?? 0, zoom: 2 })
  }, [loadSeedData, resetTiles])

  const scheduleLoad = useCallback((v: { cx: number; cz: number; zoom: number }, size: { w: number; h: number }, world: WorldInfo | null) => {
    if (!world || size.w <= 0 || size.h <= 0) return
    const requestEpoch = requestEpochRef.current
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
        const request = mapMode === 'world'
          ? invoke<RegionTile | null>('world_map_region', { worldPath: world.path, regionX: rx, regionZ: rz })
          : invoke<RegionTile>(mapMode === 'biomes' ? 'seed_biome_region' : 'seed_map_region', {
            seed: world.seed ?? 0,
            worldType: 'normal',
            regionX: rx,
            regionZ: rz,
          })
        request
          .then(res => {
            if (requestEpoch !== requestEpochRef.current) return
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
  }, [mapMode])

  useEffect(() => {
    scheduleLoad(view, canvasSize, selectedWorldRef.current)
  }, [view, canvasSize, selectedWorld, mapMode, scheduleLoad])

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
    
    const spawn = mapMode === 'world'
      ? (selectedWorld && selectedWorld.spawn_x != null && selectedWorld.spawn_z != null ? { x: selectedWorld.spawn_x, z: selectedWorld.spawn_z } : null)
      : { x: seedSpawn.x, z: seedSpawn.z }
    if (spawn) {
      const spx = (spawn.x - cx) * zoom + w / 2
      const spz = (spawn.z - cz) * zoom + h / 2
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
    if (mapMode !== 'world') {
      ctx.font = '11px sans-serif'
      structures.slice(0, 40).forEach((structure) => {
        const sx = (structure.x - cx) * zoom + w / 2
        const sz = (structure.z - cz) * zoom + h / 2
        if (sx < -20 || sz < -20 || sx > w + 20 || sz > h + 20) return
        ctx.fillStyle = '#f59e0b'
        ctx.beginPath()
        ctx.arc(sx, sz, 4, 0, Math.PI * 2)
        ctx.fill()
        ctx.fillStyle = 'rgba(255,255,255,0.8)'
        ctx.fillText(structure.name, sx + 7, sz + 3)
      })
    }
  }, [view, canvasSize, tilesVersion, selectedWorld, mapMode, seedSpawn, structures])

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
    const world = selectedWorldRef.current
    const spawn = mapMode === 'world' ? { x: world.spawn_x ?? 0, z: world.spawn_z ?? 0 } : seedSpawn
    setView({ cx: spawn.x, cz: spawn.z, zoom: 2 })
  }, [mapMode, seedSpawn])

  const selectMapMode = (mode: 'world' | 'seed' | 'biomes') => {
    if (!selectedWorld) return
    setMapMode(mode)
    resetTiles()
    const spawn = mode === 'world'
      ? { x: selectedWorld.spawn_x ?? 0, z: selectedWorld.spawn_z ?? 0 }
      : seedSpawn
    setView({ cx: spawn.x, cz: spawn.z, zoom: 2 })
  }

  const visibleWorlds = worlds.filter((world) => {
    const query = worldSearch.trim().toLowerCase()
    return !query || [world.name, world.game_mode, world.seed == null ? '' : String(world.seed)]
      .some((value) => value.toLowerCase().includes(query))
  })

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
      <div className="w-72 shrink-0 border-r border-white/5 flex flex-col">
        <div className="p-3 border-b border-white/5 flex items-center gap-2">
          <button onClick={() => navigate(-1)} className="p-1.5 rounded-lg hover:bg-surface-800 transition-colors">
            <ArrowLeft className="w-4 h-4 text-surface-400" />
          </button>
          <span className="text-sm font-medium text-surface-200">世界地图</span>
          {instanceId && (
            <button
              onClick={() => navigate(`/instances/${instanceId}/manage?type=worlds`)}
              className="ml-auto text-[10px] text-surface-500 hover:text-surface-300 transition-colors"
            >
              管理
            </button>
          )}
        </div>

        <div className="px-2 pt-2">
          <div className="relative">
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-surface-500" />
            <input
              value={worldSearch}
              onChange={(event) => setWorldSearch(event.target.value)}
              placeholder="搜索世界..."
              className="w-full h-8 pl-8 pr-2 rounded-lg bg-surface-800 border border-white/5 text-xs text-surface-200 placeholder:text-surface-500 outline-none focus:border-blue-500/50"
            />
          </div>
        </div>
        <div className="flex-1 overflow-y-auto p-2 space-y-1">
          {loading && worlds.length === 0 ? (
            <div className="text-xs text-surface-500 text-center py-4">加载中...</div>
          ) : worlds.length === 0 ? (
            <div className="text-xs text-surface-500 text-center py-4">暂无世界</div>
          ) : visibleWorlds.length === 0 ? (
            <div className="text-xs text-surface-500 text-center py-4">没有匹配的世界</div>
          ) : (
            visibleWorlds.map(w => (
              <button
                key={w.path}
                onClick={() => selectWorld(w)}
                className={`w-full text-left px-3 py-2.5 rounded-lg transition-colors ${
                  selectedWorld?.path === w.path
                    ? 'bg-blue-500/20 text-blue-300 border border-blue-500/30'
                    : 'hover:bg-surface-800 text-surface-300 border border-transparent'
                }`}
              >
                <div className="flex items-center gap-2.5">
                  {w.icon ? (
                    <img src={w.icon} alt="" className="w-8 h-8 rounded-lg object-cover shrink-0 border border-white/10" />
                  ) : (
                    <div className="w-8 h-8 rounded-lg bg-surface-800 flex items-center justify-center shrink-0">
                      <MapIcon className="w-4 h-4 text-surface-500" />
                    </div>
                  )}
                  <div className="min-w-0 flex-1">
                    <div className="text-xs font-medium truncate flex items-center gap-1.5">
                      {w.name}
                      {w.is_hardcore && <Skull className="w-3 h-3 text-red-400 shrink-0" />}
                    </div>
                    <div className="text-[10px] text-surface-500 flex items-center gap-2 mt-0.5">
                      <span className="flex items-center gap-0.5">
                        <Gamepad2 className="w-2.5 h-2.5" />
                        {GAME_MODE_CN[w.game_mode?.toLowerCase()] ?? w.game_mode ?? '未知'}
                      </span>
                      {w.seed != null && (
                        <span className="flex items-center gap-0.5">
                          <Hash className="w-2.5 h-2.5" />
                          {w.seed}
                        </span>
                      )}
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
         <div className="shrink-0 px-4 py-3 border-b border-white/5 flex items-center gap-4 flex-wrap">
          {selectedWorld && (
            <>
              <div className="flex-1 min-w-0">
                <h2 className="text-sm font-medium text-surface-200 truncate flex items-center gap-2">
                  {selectedWorld.name}
                  {selectedWorld.is_hardcore && <Skull className="w-3.5 h-3.5 text-red-400 shrink-0" />}
                </h2>
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
                  <span className="text-[11px] text-surface-500 flex items-center gap-1">
                    <Gamepad2 className="w-3 h-3" />
                    {gameModeCn(selectedWorld.game_mode)}
                  </span>
                  {selectedWorld.play_time > 0 && (
                    <span className="text-[11px] text-surface-500 flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      {formatPlayTime(selectedWorld.play_time)}
                    </span>
                  )}
                  <span className="text-[11px] text-surface-500">
                    {(selectedWorld.size_kb / 1024).toFixed(1)} MB
                  </span>
                </div>
              </div>

              {}
               <div className="flex items-center gap-2 ml-auto">
                 <div className="flex items-center gap-1 rounded-lg bg-surface-800 p-1">
                   <button onClick={() => selectMapMode('world')} className={`px-2 py-1 rounded-md text-[10px] transition-colors ${mapMode === 'world' ? 'bg-blue-500/20 text-blue-300' : 'text-surface-500 hover:text-surface-300'}`} title="查看实际存档地形">
                     <MapIcon className="w-3 h-3 inline mr-1" />实景
                   </button>
                   <button onClick={() => selectMapMode('seed')} disabled={selectedWorld.seed == null} className={`px-2 py-1 rounded-md text-[10px] transition-colors disabled:opacity-30 ${mapMode === 'seed' ? 'bg-blue-500/20 text-blue-300' : 'text-surface-500 hover:text-surface-300'}`} title="根据种子生成地形">
                     <Layers className="w-3 h-3 inline mr-1" />种子
                   </button>
                   <button onClick={() => selectMapMode('biomes')} disabled={selectedWorld.seed == null} className={`px-2 py-1 rounded-md text-[10px] transition-colors disabled:opacity-30 ${mapMode === 'biomes' ? 'bg-blue-500/20 text-blue-300' : 'text-surface-500 hover:text-surface-300'}`} title="查看生物群系">
                     <Compass className="w-3 h-3 inline mr-1" />群系
                   </button>
                 </div>
                 <button onClick={handleZoomOut} className="p-1.5 rounded-lg hover:bg-surface-800 text-surface-400 transition-colors" title="缩小">
                  <ZoomOut className="w-4 h-4" />
                </button>
                 <button onClick={handleReset} className="p-1.5 rounded-lg hover:bg-surface-800 text-surface-400 transition-colors" title="回到出生点">
                   <LocateFixed className="w-4 h-4" />
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
