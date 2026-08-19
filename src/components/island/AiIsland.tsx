import { useCallback, useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useNavigate } from 'react-router-dom'
import { Send, Loader2, CheckCircle2, XCircle, Clock, Trash2 } from 'lucide-react'
import { useIslandStore, type IslandTask } from '../../stores/islandStore'
import { useAuthStore } from '../../stores/authStore'
import type { Instance } from '../../types'

const SPRING = 'cubic-bezier(0.34, 1.56, 0.64, 1)'
const CONFIG_KEY = 'skyline-ai-config'

const TOOLS_DESC = `你可以调用以下操作帮助用户管理启动器：
- navigate[页面] 切换页面（可选值：home,download,mods,settings,account,library,music,multiplayer）
- launch_instance[实例名] 启动指定实例（从已安装实例中选择）
- search_mods[关键词] 搜索 Modrinth 模组
- search_modpacks[关键词] 搜索 Modrinth 整合包
- get_status 获取启动器状态（已安装实例数、当前选中实例）
- open_folder[类型] 打开文件夹（可选值：mods,saves,screenshots,versions）
当用户想执行操作时，在回复中包含 {{ACTION:{"name":"操作名","args":{参数}}} 提示。需确认的危险操作先问用户确认。其他正常回复。`

const SYSTEM_PROMPT = `你是 SkyLine Launcher 的专属 AI 助手，擅长 Minecraft 游戏问题分析、崩溃日志诊断、模组推荐和各类启动器操作，你可以直接操作启动器，帮助用户完成各类操作（如启动特定实例，下载各类资源等）。你必须只使用中文回答。回答要简洁直接，不要输出思考过程。
${TOOLS_DESC}`

interface AiConfig {
  provider_id: string
  api_key: string
  custom_endpoint?: string
  custom_model?: string
  custom_api_format?: string
}

function loadConfig(): AiConfig | null {
  try { return JSON.parse(localStorage.getItem(CONFIG_KEY) ?? 'null') } catch { return null }
}

interface ParsedAction {
  name: string
  args: Record<string, string>
}

function parseAction(text: string): ParsedAction | null {
  const m = text.match(/\{\{ACTION:(\{.*?\})\}\}/s)
  if (!m) return null
  try {
    const obj = JSON.parse(m[1])
    return { name: obj.name, args: obj.args || {} }
  } catch {
    return null
  }
}

function stripAction(text: string): string {
  return text.replace(/\{\{ACTION:\{.*?\}\}\}/gs, '').trim()
}

async function speakText(text: string) {
  const clean = text.replace(/[`*_#>\[\]{}|\\]/g, '').replace(/\n{2,}/g, '\n').trim()
  if (!clean) return
  try {
    const result = await invoke<{ ok: boolean; data?: number[]; error?: string }>('tts_speak', { text: clean })
    if (!result.ok || !result.data) return
    const blob = new Blob([new Uint8Array(result.data)], { type: 'audio/mpeg' })
    const url = URL.createObjectURL(blob)
    const audio = new Audio(url)
    audio.onended = () => URL.revokeObjectURL(url)
    audio.onerror = () => URL.revokeObjectURL(url)
    await audio.play()
  } catch {}
}

export function AiIsland() {
  const navigate = useNavigate()
  const [input, setInput] = useState('')
  const [expanded, setExpanded] = useState(false)
  const [message, setMessage] = useState<string | null>(null)
  const [instances, setInstances] = useState<Instance[]>([])
  const inputRef = useRef<HTMLInputElement>(null)
  const tasksRef = useRef<HTMLDivElement>(null)
  const closeTimerRef = useRef<number | null>(null)

  const aiThinking = useIslandStore((s) => s.aiThinking)
  const aiTasks = useIslandStore((s) => s.aiTasks)
  const aiMessage = useIslandStore((s) => s.aiMessage)
  const islandChatHistory = useIslandStore((s) => s.islandChatHistory)
  const { setAiActive, setAiThinking, setAiTasks, updateTask, clearTasks, setAiMessage, setAiOpen, setCompactMode, addIslandChatEntry, clearIslandChatHistory } = useIslandStore.getState()

  const busy = aiThinking || aiTasks.some((t) => t.status === 'running' || t.status === 'pending')

  const cancelClose = useCallback(() => {
    if (closeTimerRef.current) {
      window.clearTimeout(closeTimerRef.current)
      closeTimerRef.current = null
    }
  }, [])

  const scheduleClose = useCallback(() => {
    if (busy) return
    cancelClose()
    closeTimerRef.current = window.setTimeout(() => setExpanded(false), 200)
  }, [busy, cancelClose])

  useEffect(() => () => {
    if (closeTimerRef.current) window.clearTimeout(closeTimerRef.current)
  }, [])

  useEffect(() => {
    setAiOpen(expanded)
  }, [expanded, setAiOpen])

  useEffect(() => {
    invoke<Instance[]>('list_home_instances').then(setInstances).catch(() => {})
  }, [])

  useEffect(() => {
    if (expanded) {
      const t = setTimeout(() => inputRef.current?.focus(), 120)
      return () => clearTimeout(t)
    }
  }, [expanded])

  const runSteps = useCallback(async (titles: string[], stepFns: (() => Promise<void>)[]) => {
    const tasks: IslandTask[] = titles.map((title, i) => ({
      id: `t-${Date.now()}-${i}`,
      title,
      status: i === 0 ? 'running' : 'pending',
    }))
    setAiTasks(tasks)
    setAiActive(true)
    setExpanded(true)
    if (tasks[0]) speakText(`开始${tasks[0].title}`)

    for (let i = 0; i < tasks.length; i++) {
      const t = tasks[i]
      updateTask(t.id, { status: 'running' })
      try {
        await stepFns[i]?.()
        updateTask(t.id, { status: 'done' })
        speakText(`${t.title}完成`)
      } catch (e) {
        updateTask(t.id, { status: 'failed', detail: String(e) })
        speakText(`${t.title}失败`)
        return false
      }
    }
    setAiActive(false)
    setCompactMode(false)
    return true
  }, [setAiTasks, setAiActive, setCompactMode, updateTask])

  const doNavigate = useCallback((page: string) => {
    const routeMap: Record<string, string> = {
      home: '/', download: '/download', mods: '/download', settings: '/settings',
      account: '/account', library: '/library', music: '/music', multiplayer: '/multiplayer', ai: '/ai',
    }
    navigate(routeMap[page] || '/')
  }, [navigate])

  const doLaunchInstance = useCallback(async (nameOrId: string) => {
    const inst = instances.find((i) => i.id === nameOrId || i.name === nameOrId || i.name.includes(nameOrId))
    if (!inst) throw new Error(`未找到实例: ${nameOrId}`)
    try {
      const session = useAuthStore.getState().session
      if (!session) throw new Error('未登录，请先登录账户')
      let auth = session
      if (auth.user_type === 'msa' && auth.refresh_token) {
        try { auth = await invoke('microsoft_auth_refresh', { refreshToken: auth.refresh_token }) } catch {}
      }
      await invoke('launch_game', { instanceId: inst.id, auth, quickWorld: null, quickServer: null })
      setAiMessage(`已启动 ${inst.name}`)
      speakText(`已启动${inst.name}`)
    } catch (e) {
      const msg = String(e)
      if (msg.includes('[launch-crash]')) {
        navigate(`/ai?instance=${inst.id}&auto_analyze=1`)
        throw new Error('游戏启动崩溃，已进入 AI 分析')
      }
      throw e
    }
  }, [instances, navigate, setAiMessage])

  const executeAction = useCallback(async (action: ParsedAction) => {
    switch (action.name) {
      case 'navigate': {
        const page = action.args.page || 'home'
        await runSteps([`切换页面到${page}`], [async () => { doNavigate(page) }])
        break
      }
      case 'launch_instance': {
        const name = action.args.instance || action.args.name || action.args.instance_name || ''
        await runSteps([`启动实例 ${name}`], [async () => { await doLaunchInstance(name) }])
        break
      }
      case 'search_mods': {
        const q = action.args.query || action.args.q || ''
        await runSteps([`搜索模组${q}`], [async () => {
          await new Promise((r) => setTimeout(r, 400))
          navigate(`/download?q=${encodeURIComponent(q)}&tab=mods`)
        }])
        break
      }
      case 'search_modpacks': {
        const q = action.args.query || action.args.q || ''
        await runSteps([`搜索整合包${q}`], [async () => {
          await new Promise((r) => setTimeout(r, 400))
          navigate(`/download?q=${encodeURIComponent(q)}&tab=modpacks`)
        }])
        break
      }
      case 'get_status': {
        try {
          const list = await invoke<Instance[]>('list_home_instances')
          const cfg = await invoke<any>('load_config')
          return `已安装 ${list.length} 个实例，当前选中: ${cfg.last_selected_instance || '无'}，最近游玩: ${list.filter((i) => i.last_played).sort((a, b) => new Date(b.last_played!).getTime() - new Date(a.last_played!).getTime())[0]?.name || '无'}`
        } catch { return '获取状态失败' }
      }
      case 'open_folder': {
        const folderMap: Record<string, string> = { mods: 'mods', saves: 'saves', screenshots: 'screenshots', versions: 'versions' }
        const sub = folderMap[action.args.type || 'mods'] || 'mods'
        await runSteps([`打开${sub}文件夹`], [async () => {
          const cfg = await invoke<any>('load_config')
          const instId = cfg?.last_selected_instance
          if (!instId) throw new Error('请先在首页选择实例')
          try { await invoke('open_instance_folder', { instanceId: instId, subdir: sub }) } catch (e) { throw new Error(`打开失败: ${String(e)}`) }
        }])
        break
      }
      default:
        return null
    }
    return null
  }, [navigate, runSteps, doNavigate, doLaunchInstance])

  const callAI = useCallback(async (text: string) => {
    const cfg = loadConfig()
    if (!cfg || !cfg.api_key) {
      setMessage('请先到 AI 页面配置 API Key')
      navigate('/ai')
      return
    }
    setMessage(text)
    addIslandChatEntry({ role: 'user', content: text, ts: Date.now() })
    setAiThinking(true)
    setAiActive(true)
    setExpanded(true)
    speakText('好的，我来处理')

    try {
      const messages = [
        { role: 'system', content: SYSTEM_PROMPT },
        { role: 'user', content: text },
      ]
       const reply = await invoke<string>('ai_chat_v2', { messages, apiKey: cfg.api_key, model: cfg.custom_model ?? null, reasoningEffort: null })
      const action = parseAction(reply)
      const cleanReply = stripAction(reply)

      if (action) {
        if (cleanReply) { setMessage(cleanReply); addIslandChatEntry({ role: 'assistant', content: cleanReply, ts: Date.now() }); speakText(cleanReply) }
        const result = await executeAction(action)
        if (result) { setMessage(result); addIslandChatEntry({ role: 'assistant', content: result, ts: Date.now() }); speakText(result) }
      } else {
        if (cleanReply) { setMessage(cleanReply); addIslandChatEntry({ role: 'assistant', content: cleanReply, ts: Date.now() }); speakText(cleanReply) }
      }
    } catch (e) {
      const errMsg = `AI 请求失败: ${String(e)}`
      setMessage(errMsg)
      addIslandChatEntry({ role: 'assistant', content: errMsg, ts: Date.now() })
      speakText('处理失败')
    } finally {
      setAiThinking(false)
    }
  }, [executeAction, navigate, addIslandChatEntry])

  const handleSend = () => {
    const t = input.trim()
    if (!t || aiThinking) return
    setInput('')
    void callAI(t)
  }

  const clearAll = () => {
    clearTasks()
    setAiActive(false)
    setAiThinking(false)
    setExpanded(false)
    setAiMessage('')
    setMessage(null)
    setCompactMode(false)
    clearIslandChatHistory()
  }

  const taskIcon = (status: IslandTask['status']) => {
    if (status === 'done') return <CheckCircle2 className="w-3.5 h-3.5 text-emerald-500 shrink-0" />
    if (status === 'failed') return <XCircle className="w-3.5 h-3.5 text-red-500 shrink-0" />
    if (status === 'running') return <Loader2 className="w-3.5 h-3.5 text-[var(--accent-color)] animate-spin shrink-0" />
    return <Clock className="w-3.5 h-3.5 text-surface-400 shrink-0" />
  }

  useEffect(() => {
    if (aiMessage) {
      setMessage(aiMessage)
      setExpanded(true)
    }
  }, [aiMessage])

  return (
    <div
      className="relative z-30"
      data-no-drag
      onMouseEnter={() => { cancelClose(); setExpanded(true) }}
      onMouseLeave={scheduleClose}
    >
      {/* AI 圆卡：常驻 40px 槽位，不改变 flex 布局 */}
      <button
        onClick={() => setExpanded(true)}
        className="island-glass relative overflow-hidden w-10 h-10 rounded-full border bg-white/95 dark:bg-surface-850/95 backdrop-blur-xl border-surface-200 dark:border-surface-700/60 shadow-lg flex items-center justify-center text-[var(--accent-color)] hover:opacity-90 active:scale-95 transition-all cursor-pointer origin-center"
        title="打开 SkyLine AI"
      >
        {aiThinking ? (
          <Loader2 className="w-4 h-4 animate-spin" />
        ) : (
          <span className="text-[10px] font-black tracking-widest">AI</span>
        )}
      </button>

      {/* 展开长条：absolute 居中覆盖层，不改变布局 */}
      {expanded && (
        <div
          onMouseEnter={cancelClose}
          className="island-glass absolute left-1/2 -translate-x-1/2 top-full mt-1.5 flex items-center gap-2 rounded-full border bg-white/95 dark:bg-surface-850/95 backdrop-blur-xl border-surface-200 dark:border-surface-700/60 shadow-lg overflow-hidden origin-top"
          style={{
            width: busy ? 560 : 320,
            height: 40,
            transition: `width 0.45s ${SPRING}, box-shadow 0.35s ease`,
            transform: busy ? 'translateX(-50%) scale(1.02)' : 'translateX(-50%) scale(1)',
            animation: 'ai-pop-in 0.3s cubic-bezier(0.22, 1, 0.36, 1) both',
          }}
        >
          <div className="flex-1 min-w-0 overflow-hidden pl-4">
            {busy ? (
              <div className="text-[11px] text-surface-500 truncate">
                {aiTasks.length > 0 ? `正在执行 ${aiTasks.filter((t) => t.status === 'done').length}/${aiTasks.length} 个任务...` : 'AI 思考中...'}
              </div>
            ) : (
              <input
                ref={inputRef}
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={(e) => { if (e.key === 'Enter') handleSend() }}
                placeholder="请输入问题"
                style={{ outline: 'none' }}
                className="bg-transparent text-xs w-full placeholder:text-surface-400"
              />
            )}
          </div>

          <div className="flex items-center gap-2 pr-2 shrink-0">
            <button
              onClick={clearAll}
              className="w-6 h-6 rounded-full flex items-center justify-center text-surface-400 hover:text-red-500 hover:bg-surface-100 dark:hover:bg-surface-800 transition-colors cursor-pointer"
              title="清空"
            >
              <Trash2 className="w-3 h-3" />
            </button>
            <button
              onClick={handleSend}
              disabled={busy || !input.trim()}
              className="w-7 h-7 rounded-full flex items-center justify-center bg-[var(--accent-color)] text-white hover:opacity-90 active:scale-95 transition-all cursor-pointer disabled:opacity-40 disabled:pointer-events-none"
              title="发送"
            >
              <Send className="w-3 h-3" />
            </button>
          </div>
        </div>
      )}

      {expanded && (
        <div
          className="island-glass absolute top-full left-1/2 -translate-x-1/2 mt-2 w-[560px] max-h-[320px] overflow-y-auto rounded-2xl border bg-white/95 dark:bg-surface-850/95 backdrop-blur-xl border-surface-200 dark:border-surface-700/60 shadow-2xl"
          ref={tasksRef}
        >
          <div className="p-3 space-y-2">
            {islandChatHistory.length > 0 && (
              <div className="space-y-1.5 max-h-[200px] overflow-y-auto">
                {islandChatHistory.slice(-6).map((entry, i) => (
                  <div key={entry.ts + '-' + i} className={`text-xs leading-relaxed whitespace-pre-wrap rounded-lg px-2.5 py-1.5 ${
                    entry.role === 'user'
                      ? 'bg-[var(--accent-color)]/10 text-[var(--accent-color)] ml-8 text-right'
                      : 'bg-surface-100 dark:bg-surface-800 text-surface-700 dark:text-surface-200 mr-8'
                  }`}>
                    {entry.content}
                  </div>
                ))}
              </div>
            )}
            {islandChatHistory.length === 0 && message && (
              <div className="text-xs text-surface-700 dark:text-surface-200 leading-relaxed whitespace-pre-wrap max-h-[140px] overflow-y-auto">
                {message}
              </div>
            )}
            {aiTasks.length > 0 && (
              <div className="space-y-1.5 border-t border-surface-200/60 dark:border-surface-700/40 pt-2">
                {aiTasks.map((t) => (
                  <div key={t.id} className="flex items-center gap-2 text-xs">
                    {taskIcon(t.status)}
                    <span className={`flex-1 ${t.status === 'done' ? 'text-surface-400 line-through' : t.status === 'failed' ? 'text-red-500' : 'text-surface-700 dark:text-surface-200'}`}>
                      {t.title}
                    </span>
                    {t.status === 'running' && <span className="text-[10px] text-[var(--accent-color)] animate-pulse">执行中</span>}
                    {t.status === 'done' && <span className="text-[10px] text-emerald-500">已完成</span>}
                    {t.status === 'failed' && <span className="text-[10px] text-red-500">失败</span>}
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
