import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open } from '@tauri-apps/plugin-shell'
import { useCallback, useRef, useState, useEffect } from 'react'
import { Send, Paperclip, FileText, Image, Film, X, User, Bot, Copy, Trash2, History, Plus, StopCircle, Settings, Sparkles, ChevronRight, ChevronDown, ExternalLink, Volume2, VolumeX } from 'lucide-react'
import { useSearchParams, useNavigate } from 'react-router-dom'
import { useIslandStore, type IslandTask } from '../stores/islandStore'

interface ChatMsg {
  role: 'user' | 'assistant'
  content: string
  file?: { name: string; type: string; dataUrl: string } | null
  id: string
  action?: { name: string; args: Record<string, string> }
}

interface ChatSession {
  id: string
  name: string
  messages: ChatMsg[]
  createdAt: number
  instanceId?: string
}

interface AiProviderDef {
  id: string
  name: string
  endpoint: string
  api_format: string
  default_model: string
  models?: string[]
  model_names?: Record<string, string>
  icon?: string
  color?: string
}

interface ProviderConfig {
  provider_id: string
  api_key: string
  custom_endpoint?: string
  custom_model?: string
  custom_models?: string[]
  custom_api_format?: string
}

const SESSIONS_KEY = 'skyline-ai-chats'
const CONFIG_KEY = 'skyline-ai-config'
const KEYS_KEY = 'skyline-ai-keys'
const TTS_KEY = 'skyline-ai-tts'

const FAVICON = (domain: string) => `https://www.google.com/s2/favicons?domain=${domain}&sz=64`

const PROVIDERS: AiProviderDef[] = [
  { id: 'agnes', name: 'Agnes AI', endpoint: 'https://apihub.agnes-ai.com/v1/chat/completions', api_format: 'openai', default_model: 'agnes-2.5-flash', models: ['agnes-2.5-flash', 'agnes-2.5-pro', 'agnes-mini'], model_names: { 'agnes-2.5-flash': 'Agnes 2.5 Flash', 'agnes-2.5-pro': 'Agnes 2.5 Pro', 'agnes-mini': 'Agnes Mini' }, icon: FAVICON('agnes-ai.com'), color: '#3b82f6' },
  { id: 'deepseek', name: 'DeepSeek', endpoint: 'https://api.deepseek.com/v1/chat/completions', api_format: 'openai', default_model: 'deepseek-chat', models: ['deepseek-chat', 'deepseek-reasoner'], model_names: { 'deepseek-chat': 'DeepSeek V3', 'deepseek-reasoner': 'DeepSeek R1' }, icon: FAVICON('deepseek.com'), color: '#4f46e5' },
  { id: 'openai', name: 'OpenAI', endpoint: 'https://api.openai.com/v1/chat/completions', api_format: 'openai', default_model: 'gpt-4o-mini', models: ['gpt-4o-mini', 'gpt-4o', 'gpt-4.1', 'gpt-4.1-mini', 'gpt-4.1-nano', 'o3-mini', 'o4-mini'], model_names: { 'gpt-4o-mini': 'GPT-4o Mini', 'gpt-4o': 'GPT-4o', 'gpt-4.1': 'GPT-4.1', 'gpt-4.1-mini': 'GPT-4.1 Mini', 'gpt-4.1-nano': 'GPT-4.1 Nano', 'o3-mini': 'o3 Mini', 'o4-mini': 'o4 Mini' }, icon: FAVICON('openai.com'), color: '#10a37f' },
  { id: 'anthropic', name: 'Anthropic', endpoint: 'https://api.anthropic.com/v1/messages', api_format: 'anthropic', default_model: 'claude-sonnet-4-20250514', models: ['claude-sonnet-4-20250514', 'claude-haiku-4-5-20251001', 'claude-opus-4-20250514'], model_names: { 'claude-sonnet-4-20250514': 'Claude Sonnet 4', 'claude-haiku-4-5-20251001': 'Claude Haiku 4.5', 'claude-opus-4-20250514': 'Claude Opus 4' }, icon: FAVICON('claude.ai'), color: '#d97706' },
  { id: 'google', name: 'Google Gemini', endpoint: 'https://generativelanguage.googleapis.com/v1beta', api_format: 'google', default_model: 'gemini-2.0-flash', models: ['gemini-2.0-flash', 'gemini-2.5-flash', 'gemini-2.5-pro'], model_names: { 'gemini-2.0-flash': 'Gemini 2.0 Flash', 'gemini-2.5-flash': 'Gemini 2.5 Flash', 'gemini-2.5-pro': 'Gemini 2.5 Pro' }, icon: 'https://www.gstatic.com/images/branding/product/2x/googleg_48dp.png', color: '#4285f4' },
  { id: 'moonshot', name: 'Moonshot Kimi', endpoint: 'https://api.moonshot.cn/v1/chat/completions', api_format: 'openai', default_model: 'moonshot-v1-8k', models: ['moonshot-v1-8k', 'moonshot-v1-32k', 'kimi-k2-0711-preview'], model_names: { 'moonshot-v1-8k': 'Kimi V1 8K', 'moonshot-v1-32k': 'Kimi V1 32K', 'kimi-k2-0711-preview': 'Kimi K2' }, icon: FAVICON('moonshot.cn'), color: '#6366f1' },
  { id: 'zhipu', name: '智谱 GLM', endpoint: 'https://open.bigmodel.cn/api/paas/v4/chat/completions', api_format: 'openai', default_model: 'glm-4-flash', models: ['glm-4-flash', 'glm-4-plus', 'glm-4-air', 'glm-z1-flash'], model_names: { 'glm-4-flash': 'GLM-4 Flash', 'glm-4-plus': 'GLM-4 Plus', 'glm-4-air': 'GLM-4 Air', 'glm-z1-flash': 'GLM-Z1 Flash' }, icon: FAVICON('bigmodel.cn'), color: '#8b5cf6' },
  { id: 'siliconflow', name: 'SiliconFlow', endpoint: 'https://api.siliconflow.cn/v1/chat/completions', api_format: 'openai', default_model: 'deepseek-ai/DeepSeek-V3', models: ['deepseek-ai/DeepSeek-V3', 'deepseek-ai/DeepSeek-R1', 'Qwen/Qwen2.5-72B-Instruct', 'THUDM/glm-4-9b-chat'], model_names: { 'deepseek-ai/DeepSeek-V3': 'DeepSeek V3', 'deepseek-ai/DeepSeek-R1': 'DeepSeek R1', 'Qwen/Qwen2.5-72B-Instruct': 'Qwen 2.5 72B', 'THUDM/glm-4-9b-chat': 'GLM-4 9B' }, icon: FAVICON('siliconflow.cn'), color: '#06b6d4' },
  { id: 'openrouter', name: 'OpenRouter', endpoint: 'https://openrouter.ai/api/v1/chat/completions', api_format: 'openai', default_model: 'openai/gpt-4o-mini', models: ['openai/gpt-4o-mini', 'anthropic/claude-3.5-sonnet', 'deepseek/deepseek-chat'], model_names: { 'openai/gpt-4o-mini': 'GPT-4o Mini', 'anthropic/claude-3.5-sonnet': 'Claude 3.5 Sonnet', 'deepseek/deepseek-chat': 'DeepSeek V3' }, icon: FAVICON('openrouter.ai'), color: '#f97316' },
  { id: 'groq', name: 'Groq', endpoint: 'https://api.groq.com/openai/v1/chat/completions', api_format: 'openai', default_model: 'llama-3.3-70b-versatile', models: ['llama-3.3-70b-versatile', 'llama-3.1-8b-instant', 'mixtral-8x7b-32768', 'gemma2-9b-it'], model_names: { 'llama-3.3-70b-versatile': 'Llama 3.3 70B', 'llama-3.1-8b-instant': 'Llama 3.1 8B', 'mixtral-8x7b-32768': 'Mixtral 8x7B', 'gemma2-9b-it': 'Gemma 2 9B' }, icon: FAVICON('groq.com'), color: '#3b82f6' },
  { id: 'mistral', name: 'Mistral AI', endpoint: 'https://api.mistral.ai/v1/chat/completions', api_format: 'openai', default_model: 'mistral-small-latest', models: ['mistral-small-latest', 'mistral-large-latest', 'open-mistral-nemo', 'codestral-latest'], model_names: { 'mistral-small-latest': 'Mistral Small', 'mistral-large-latest': 'Mistral Large', 'open-mistral-nemo': 'Mistral Nemo', 'codestral-latest': 'Codestral' }, icon: FAVICON('mistral.ai'), color: '#f97316' },
  { id: 'xai', name: 'xAI Grok', endpoint: 'https://api.x.ai/v1/chat/completions', api_format: 'openai', default_model: 'grok-3', models: ['grok-3', 'grok-3-mini', 'grok-2-1212'], model_names: { 'grok-3': 'Grok 3', 'grok-3-mini': 'Grok 3 Mini', 'grok-2-1212': 'Grok 2' }, icon: FAVICON('x.ai'), color: '#111827' },
  { id: 'qwen', name: '通义千问 Qwen', endpoint: 'https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions', api_format: 'openai', default_model: 'qwen-turbo', models: ['qwen-turbo', 'qwen-plus', 'qwen-max', 'qwen-long'], model_names: { 'qwen-turbo': 'Qwen Turbo', 'qwen-plus': 'Qwen Plus', 'qwen-max': 'Qwen Max', 'qwen-long': 'Qwen Long' }, icon: FAVICON('tongyi.aliyun.com'), color: '#6366f1' },
  { id: 'custom', name: '自定义', endpoint: '', api_format: 'openai', default_model: '', models: [], icon: '', color: '#6b7280' },
]

const TOOLS_DESC = `你可以调用以下操作帮助用户管理启动器：
- navigate[页面] 切换页面（可选值：home,download,mods,settings,account,library,music,multiplayer）
- launch_instance[实例名] 启动指定实例（从已安装实例中选择，支持实例名或ID模糊匹配）
- launch_game 启动当前选中的实例（不需要指定名称）
- search_mods[关键词] 搜索 Modrinth 模组
- search_modpacks[关键词] 搜索 Modrinth 整合包
- get_status 获取启动器状态（已安装实例数、当前选中实例）
- open_folder[类型] 打开文件夹（可选值：mods,saves,screenshots,versions）
当用户想执行操作时，只输出一个 JSON 操作标记，不要向用户解释标记或原始命令：{{ACTION:{"name":"操作名","args":{参数}}}}。执行完成后由启动器生成结果。危险操作先等待确认。`

const SYSTEM_PROMPT = `你是 SkyLine Launcher 的专属 AI 助手，擅长 Minecraft 游戏问题分析、崩溃日志诊断、模组推荐和启动器操作。需要执行操作时必须使用 ACTION 标记，不要把 ACTION 标记或命令文本展示给用户。你必须只使用中文回答，普通问答简洁直接。
${TOOLS_DESC}`

function genId() {
  return Math.random().toString(36).slice(2, 10)
}

function loadSessions(): ChatSession[] {
  try { return JSON.parse(localStorage.getItem(SESSIONS_KEY) ?? '[]') } catch { return [] }
}

function saveSessions(s: ChatSession[]) {
  localStorage.setItem(SESSIONS_KEY, JSON.stringify(s))
}

function loadConfig(): ProviderConfig | null {
  try { return JSON.parse(localStorage.getItem(CONFIG_KEY) ?? 'null') } catch { return null }
}

function loadKeys(): Record<string, string> {
  try { return JSON.parse(localStorage.getItem(KEYS_KEY) ?? '{}') } catch { return {} }
}

function saveKeys(keys: Record<string, string>) {
  localStorage.setItem(KEYS_KEY, JSON.stringify(keys))
}

function saveConfig(c: ProviderConfig) {
  localStorage.setItem(CONFIG_KEY, JSON.stringify(c))
  const p = getProvider(c.provider_id)
  const ep = c.custom_endpoint || p.endpoint
  const mdl = c.custom_model || p.default_model
  const fmt = c.custom_api_format || p.api_format
  if (c.api_key) {
    const keys = loadKeys()
    keys[c.provider_id] = c.api_key
    saveKeys(keys)
  }
  invoke('save_ai_provider_config', {
    provider: {
      id: c.provider_id,
      name: p.name,
      endpoint: ep,
      api_format: fmt,
      default_model: mdl,
    }
  }).catch(() => {})
}

function getProvider(id: string): AiProviderDef {
  return PROVIDERS.find(p => p.id === id) ?? PROVIDERS[0]
}

function getModelList(provider: AiProviderDef, cfg: ProviderConfig | null): string[] {
  if (provider.id === 'custom') return cfg?.custom_models?.length ? cfg.custom_models : []
  return provider.models ?? [provider.default_model]
}

function getModelDisplayName(provider: AiProviderDef, modelId: string): string {
  return provider.model_names?.[modelId] || modelId
}

async function testConnection(provider: AiProviderDef, apiKey: string, model: string): Promise<{ ok: boolean; message: string }> {
  try {
    const messages = [{ role: 'user', content: 'Hi' }]
    const result = await invoke<string>('ai_chat_v2', { messages, apiKey, model, reasoningEffort: null })
    return { ok: true, message: `${provider.name} / ${getModelDisplayName(provider, model)} 连接成功！模型回复: ${result.slice(0, 50)}...` }
  } catch (e: any) {
    return { ok: false, message: `连接失败: ${String(e)}` }
  }
}

function ProviderIcon({ provider, size = 40 }: { provider: AiProviderDef; size?: number }) {
  const labels: Record<string, string> = { agnes: 'A', deepseek: 'DS', openai: 'OA', anthropic: 'C', google: 'G', moonshot: 'K', zhipu: 'GLM', siliconflow: 'SF', openrouter: 'OR', groq: 'GQ', mistral: 'M', xai: 'xAI', qwen: 'Q', custom: '+' }
  const label = labels[provider.id] ?? provider.name.charAt(0).toUpperCase()
  const bgColor = provider.color || '#6b7280'
  return (
    <div className="shrink-0 rounded-lg flex items-center justify-center text-white font-black shadow-sm ring-1 ring-white/20"
      style={{ width: size, height: size, background: `linear-gradient(145deg, ${bgColor}, color-mix(in srgb, ${bgColor} 65%, #111827))`, fontSize: Math.max(7, size * (label.length > 2 ? 0.25 : 0.34)) }}>
      {label}
    </div>
  )
}

function extractAction(text: string): { action: { name: string; args: Record<string, string> }; clean: string } | null {
  const marker = '{{ACTION:'
  const markerStart = text.indexOf(marker)
  if (markerStart < 0) return null
  const jsonStart = markerStart + marker.length
  let depth = 0
  let jsonEnd = -1
  for (let i = jsonStart; i < text.length; i++) {
    if (text[i] === '{') depth++
    else if (text[i] === '}') {
      depth--
      if (depth === 0) { jsonEnd = i; break }
    }
  }
  if (jsonEnd < 0) return null
  try {
    const obj = JSON.parse(text.slice(jsonStart, jsonEnd + 1))
    if (!obj.name) return null
    let markerEnd = jsonEnd + 1
    while (markerEnd < text.length && text[markerEnd] === '}') markerEnd++
    return {
      action: { name: obj.name, args: obj.args || {} },
      clean: `${text.slice(0, markerStart)} ${text.slice(markerEnd)}`.trim(),
    }
  } catch {}
  return null
}

let ttsAudio: HTMLAudioElement | null = null

async function speakText(text: string) {
  const clean = text.replace(/[`*_#>\[\]{}|\\]/g, '').replace(/\n{2,}/g, '\n').trim()
  if (!clean) return
  try {
    const result = await invoke<{ ok: boolean; data?: number[]; error?: string }>('tts_speak', { text: clean })
    if (!result.ok || !result.data) return
    const blob = new Blob([new Uint8Array(result.data)], { type: 'audio/mpeg' })
    const url = URL.createObjectURL(blob)
    ttsAudio = new Audio(url)
    ttsAudio.onended = () => { URL.revokeObjectURL(url); ttsAudio = null }
    ttsAudio.onerror = () => { URL.revokeObjectURL(url); ttsAudio = null }
    await ttsAudio.play()
  } catch {}
}

function stopTts() {
  if (ttsAudio) { ttsAudio.pause(); ttsAudio = null }
}

export default function AiCrash() {
  const [searchParams] = useSearchParams()
  const navigate = useNavigate()
  const instanceId = searchParams.get('instance')

  const [sessions, setSessions] = useState<ChatSession[]>([])
  const [activeId, setActiveId] = useState<string | null>(null)
  const [config, setConfig] = useState<ProviderConfig | null>(null)
  const [showSetup, setShowSetup] = useState(false)
  const [setupStep, setSetupStep] = useState<'select' | 'apikey'>('select')
  const [selectedProviderId, setSelectedProviderId] = useState('agnes')
  const [setupApiKey, setSetupApiKey] = useState('')
  const [setupCustomEndpoint, setSetupCustomEndpoint] = useState('')
  const [setupCustomModel, setSetupCustomModel] = useState('')
  const [setupCustomModels, setSetupCustomModels] = useState<string[]>([])
  const [setupCustomApiFormat, setSetupCustomApiFormat] = useState('openai')
  const [model, setModel] = useState<string>('')

  const [messages, setMessages] = useState<ChatMsg[]>([])
  const [input, setInput] = useState('')
  const [loading, setLoading] = useState(false)
  const [attachedFile, setAttachedFile] = useState<{ name: string; size: number; type: string } | null>(null)
  const [logContent, setLogContent] = useState<string | null>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const messagesRef = useRef<HTMLDivElement>(null)
  const [showHistory, setShowHistory] = useState(false)
  const [showNewSessionDialog, setShowNewSessionDialog] = useState(false)
  const [newSessionName, setNewSessionName] = useState('')
  const abortRef = useRef<AbortController | null>(null)
  const isMountedRef = useRef(true)
  const [dragOver, setDragOver] = useState(false)
  const [showSettings, setShowSettings] = useState(false)
  const [showModelPicker, setShowModelPicker] = useState(false)
  const [ttsEnabled, setTtsEnabled] = useState(() => {
    try { return localStorage.getItem(TTS_KEY) !== 'false' } catch { return true }
  })
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<string | null>(null)

  const selectProvider = useCallback((id: string, source: 'setup' | 'settings' | 'picker') => {
    const keys = loadKeys()
    setSelectedProviderId(id)
    setSetupApiKey(keys[id] ?? '')
    const p = getProvider(id)
    if (id === 'custom' && config?.provider_id === 'custom') {
      setSetupCustomEndpoint(config.custom_endpoint ?? '')
      setSetupCustomModels(config.custom_models ?? [])
      setSetupCustomModel(config.custom_model ?? '')
      setSetupCustomApiFormat(config.custom_api_format ?? 'openai')
    } else if (id === 'custom') {
      setSetupCustomEndpoint('')
      setSetupCustomModels([])
      setSetupCustomModel('')
      setSetupCustomApiFormat('openai')
    } else {
      setSetupCustomEndpoint('')
      setSetupCustomModels([])
    }
    if (source === 'picker') {
      const first = getModelList(p, config)[0] ?? p.default_model
      setModel(first)
      if (config && p.id !== 'custom') {
        const newCfg = { ...config, provider_id: p.id, api_key: keys[p.id] ?? '', custom_model: first, custom_models: undefined }
        saveConfig(newCfg); setConfig(newCfg)
      }
    }
  }, [config])

  useEffect(() => {
    isMountedRef.current = true
    return () => {
      isMountedRef.current = false
      abortRef.current?.abort()
      stopTts()
    }
  }, [])

  useEffect(() => {
    const stored = loadSessions()
    setSessions(stored)
    const cfg = loadConfig()
    setConfig(cfg)
    // 迁移旧版 key 到按厂商存储
    if (cfg?.api_key) {
      const keys = loadKeys()
      if (!keys[cfg.provider_id]) {
        keys[cfg.provider_id] = cfg.api_key
        saveKeys(keys)
      }
    }
    if (cfg) setModel(cfg.custom_model || getProvider(cfg.provider_id).default_model)
    if (!cfg || !cfg.api_key) setShowSetup(true)

    if (instanceId) {
      const hasExisting = stored.find(s => s.instanceId === instanceId)
      const autoAnalyze = searchParams.get('auto_analyze') === '1'
      if (autoAnalyze && cfg?.api_key) {
        const base = hasExisting ? hasExisting.messages : []
        const pending: ChatMsg[] = [...base, { id: genId(), role: 'assistant', content: '检测到游戏启动崩溃，正在自动分析日志...' }]
        if (!hasExisting) {
          const ns = createSession(instanceId)
          const updated = [ns, ...stored]
          setSessions(updated); saveSessions(updated); setActiveId(ns.id)
        } else { setActiveId(hasExisting.id) }
        setMessages(pending)
        invoke<string>('analyze_crash_auto', { instanceId, apiKey: cfg.api_key })
          .then(a => { if (isMountedRef.current) setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: a }]) })
          .catch(e => { if (isMountedRef.current) setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: `分析失败: ${String(e)}` }]) })
        return
      }
      if (hasExisting) { setActiveId(hasExisting.id); setMessages(hasExisting.messages); return }
      if (cfg?.api_key) {
        const ns = createSession(instanceId)
        const updated = [ns, ...stored]
        setSessions(updated); saveSessions(updated); setActiveId(ns.id)
        invoke<string>('read_latest_log', { instanceId }).then(content => {
          setLogContent(content.split('\n').slice(-300).join('\n'))
          setMessages([{ id: genId(), role: 'assistant', content: `检测到实例 \`${instanceId}\` 最近崩溃，已加载 latest.log 最后 300 行。正在自动分析中...` }])
        }).catch(() => {
          setMessages([{ id: genId(), role: 'assistant', content: `检测到实例 \`${instanceId}\` 最近崩溃。请描述你遇到的问题，我会帮你分析。` }])
        })
      }
      return
    }

    if (stored.length > 0) { setActiveId(stored[0].id); setMessages(stored[0].messages) }
    else {
      const greeting = cfg?.api_key
        ? '你好！我是 SkyLine AI 助手。你可以问我 Minecraft 相关问题、上传崩溃日志让我分析，或者让我帮你搜索模组、管理启动器。'
        : '你好！请先配置 API Key 以开始使用 AI 功能。'
      setMessages([{ id: genId(), role: 'assistant', content: greeting }])
      try { const tts = localStorage.getItem(TTS_KEY); if (tts !== 'false') speakText(greeting) } catch {}
    }
  }, [])

  useEffect(() => {
    if (!activeId || sessions.length === 0) return
    const updated = sessions.map(s => s.id === activeId ? { ...s, messages } : s)
    setSessions(updated)
    saveSessions(updated)
  }, [messages, activeId])

  useEffect(() => {
    const el = messagesRef.current
    if (el) el.scrollTo({ top: el.scrollHeight, behavior: 'smooth' })
  }, [messages, loading])

  useEffect(() => {
    const store = useIslandStore.getState()
    const history = store.islandChatHistory
    if (history.length === 0) return
    const lastEntry = history[history.length - 1]
    const alreadyInMessages = messages.some(m => m.role === lastEntry.role && m.content === lastEntry.content)
    if (alreadyInMessages) return
    if (lastEntry.role === 'user') {
      setMessages(prev => [...prev, { id: genId(), role: 'user', content: lastEntry.content, file: null }])
    } else if (messages.length > 0) {
      setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: lastEntry.content }])
    }
  }, [useIslandStore((s) => s.islandChatHistory)])

  function createSession(instanceId?: string): ChatSession {
    return { id: genId(), name: instanceId ? `实例: ${instanceId}` : `新对话 ${Date.now()}`, messages: [], createdAt: Date.now(), instanceId }
  }

  const switchSession = (id: string) => { setActiveId(id); const s = sessions.find(x => x.id === id); if (s) setMessages(s.messages); setShowHistory(false) }

  const confirmNewSession = () => {
    const name = newSessionName.trim() || `对话 ${sessions.length + 1}`
    const s: ChatSession = { id: genId(), name, messages: [], createdAt: Date.now() }
    const updated = [s, ...sessions]
    setSessions(updated); saveSessions(updated); setActiveId(s.id)
    setMessages([{ id: genId(), role: 'assistant', content: '你好！请上传崩溃日志、截图或录屏，我会帮你分析报错问题。' }])
    setLogContent(null); setAttachedFile(null); setShowHistory(false); setShowNewSessionDialog(false); setNewSessionName('')
  }

  const deleteSession = (id: string, e: React.MouseEvent) => {
    e.preventDefault(); e.stopPropagation()
    const updated = sessions.filter(s => s.id !== id)
    setSessions(updated)
    if (activeId === id) { setActiveId(updated[0]?.id ?? null); setMessages(updated[0]?.messages ?? []) }
  }

  const handleFileSelect = useCallback(async (file: File) => {
    setAttachedFile({ name: file.name, size: file.size, type: file.type || 'text/plain' })
    const reader = new FileReader()
    reader.onload = () => {
      const base64 = (reader.result as string).split(',')[1]
      if (file.type.startsWith('image/')) setLogContent(`data:${file.type};base64,${base64}`)
      else if (file.type.startsWith('video/')) setLogContent(`data:${file.type};base64,${base64}`)
      else setLogContent(new TextDecoder().decode(Uint8Array.from(atob(base64), c => c.charCodeAt(0))))
    }
    reader.readAsDataURL(file)
  }, [])

  const handlePathDrop = useCallback(async (path: string) => {
    const name = path.split(/[\\/]/).pop() || path
    const ext = (path.split('.').pop() || '').toLowerCase()
    const isImage = ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp', 'ico'].includes(ext)
    const isVideo = ['mp4', 'webm', 'mov', 'avi', 'mkv', 'flv'].includes(ext)
    const type = isImage ? `image/${ext === 'jpg' ? 'jpeg' : ext}` : isVideo ? `video/${ext}` : 'text/plain'
    setAttachedFile({ name, size: 0, type })
    try {
      if (isImage || isVideo) {
        const b64 = await invoke<string>('read_file_as_base64', { path })
        setLogContent(`data:${type};base64,${b64}`)
        setAttachedFile({ name, size: Math.round(b64.length * 0.75), type })
      } else {
        const text = await invoke<string>('read_crash_file', { path })
        setLogContent(text)
        setAttachedFile({ name, size: new TextEncoder().encode(text).length, type: 'text/plain' })
      }
    } catch (err) {
      setLogContent(null); setAttachedFile(null)
      setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: `读取文件 ${name} 失败: ${String(err)}` }])
    }
  }, [])

  useEffect(() => {
    let unlisten: (() => void) | undefined
    let disposed = false
    getCurrentWebview().onDragDropEvent((event) => {
      if (disposed) return
      if (event.payload.type === 'enter' || event.payload.type === 'over') setDragOver(true)
      else if (event.payload.type === 'leave') setDragOver(false)
      else if (event.payload.type === 'drop') { setDragOver(false); const p = event.payload.paths[0]; if (p) void handlePathDrop(p) }
    }).then((fn) => { if (disposed) fn(); else unlisten = fn }).catch(() => {})
    return () => { disposed = true; unlisten?.() }
  }, [handlePathDrop])

  const handlePaste = useCallback((e: React.ClipboardEvent) => {
    const items = e.clipboardData?.items
    if (items?.[0]?.type.startsWith('image/')) { const file = items[0].getAsFile(); if (file) handleFileSelect(file) }
  }, [handleFileSelect])

  const copyMessage = (content: string) => navigator.clipboard.writeText(content)

  const deleteMessage = (id: string) => {
    if (loading) { abortRef.current?.abort(); setLoading(false) }
    setMessages(prev => {
      const idx = prev.findIndex(m => m.id === id)
      if (idx === -1) return prev
      const next = prev[idx + 1]
      if (next && next.role === 'assistant') return prev.filter(m => m.id !== id && m.id !== next.id)
      return prev.filter(m => m.id !== id)
    })
  }

  const executeAction = useCallback(async (action: { name: string; args: Record<string, string> }) => {
    switch (action.name) {
      case 'navigate': {
        const page = action.args.page || 'home'
        const routeMap: Record<string, string> = { home: '/', download: '/download', mods: '/download', settings: '/settings', account: '/account', library: '/library', music: '/music', multiplayer: '/multiplayer' }
        navigate(routeMap[page] || '/')
        return `已切换到 ${page} 页面`
      }
      case 'search_mods': {
        const q = action.args.query || action.args.q || ''
        if (q) { navigate(`/download?q=${encodeURIComponent(q)}&tab=mods`); return `正在搜索模组: ${q}` }
        return '请提供搜索关键词'
      }
      case 'search_modpacks': {
        const q = action.args.query || action.args.q || ''
        if (q) { navigate(`/download?q=${encodeURIComponent(q)}&tab=modpacks`); return `正在搜索整合包: ${q}` }
        return '请提供搜索关键词'
      }
      case 'launch_instance': {
        const nameOrId = action.args.instance || action.args.name || action.args.instance_name || ''
        try {
          const instances = await invoke<any[]>('list_home_instances')
          const inst = instances.find((i: any) => i.id === nameOrId || i.name === nameOrId || i.name.includes(nameOrId))
          if (!inst) return `未找到实例: ${nameOrId}`
          const { useAuthStore } = await import('../stores/authStore')
          const session = useAuthStore.getState().session
          if (!session) { navigate('/account'); return '未登录，请先登录账户' }
          let auth = session
          if (auth.user_type === 'msa' && auth.refresh_token) {
            try { auth = await invoke('microsoft_auth_refresh', { refreshToken: auth.refresh_token }) } catch {}
          }
          await invoke('launch_game', { instanceId: inst.id, auth, quickWorld: null, quickServer: null })
          return `已启动实例: ${inst.name}`
        } catch (e: any) {
          const msg = String(e)
          if (msg.includes('[launch-crash]')) { navigate(`/ai?instance=${encodeURIComponent(nameOrId)}&auto_analyze=1`); return '游戏启动崩溃，已进入 AI 分析' }
          return `启动失败: ${msg}`
        }
      }
      case 'launch_game': {
        try {
          const instances = await invoke<any[]>('list_home_instances')
          const cfg = await invoke<any>('load_config')
          const inst = cfg.last_selected_instance ? instances.find((i: any) => i.id === cfg.last_selected_instance) : instances[0]
          if (!inst) return '没有可用的实例，请先创建或导入一个实例'
          const { useAuthStore } = await import('../stores/authStore')
          const session = useAuthStore.getState().session
          if (!session) { navigate('/account'); return '未登录，请先登录账户' }
          let auth = session
          if (auth.user_type === 'msa' && auth.refresh_token) {
            try { auth = await invoke('microsoft_auth_refresh', { refreshToken: auth.refresh_token }) } catch {}
          }
          await invoke('launch_game', { instanceId: inst.id, auth, quickWorld: null, quickServer: null })
          return `已启动实例: ${inst.name}`
        } catch (e: any) {
          const msg = String(e)
          if (msg.includes('[launch-crash]')) return '游戏启动崩溃，请查看日志分析原因'
          return `启动失败: ${msg}`
        }
      }
      case 'open_folder': {
        const folderMap: Record<string, string> = { mods: 'mods', saves: 'saves', screenshots: 'screenshots', versions: 'versions' }
        const sub = folderMap[action.args.type || 'mods'] || 'mods'
        try {
          const cfg = await invoke<any>('load_config')
          const instId = cfg?.last_selected_instance
          if (!instId) return '请先在首页选择一个实例'
          await invoke('open_instance_folder', { instanceId: instId, subdir: sub })
          return `已打开 ${sub} 文件夹`
        } catch (e: any) { return `打开文件夹失败: ${String(e)}` }
      }
      case 'get_status': {
        try {
          const instances = await invoke<any[]>('list_home_instances')
          const cfg = await invoke<any>('load_config')
          const selected = cfg.last_selected_instance ? instances.find((i: any) => i.id === cfg.last_selected_instance) : null
          return `已安装 ${instances.length} 个实例，当前选中: ${selected?.name || '无'}`
        } catch { return '获取状态失败' }
      }
      default:
        return `未知操作: ${action.name}`
    }
  }, [navigate])

  const runActionWithPlan = useCallback(async (action: { name: string; args: Record<string, string> }) => {
    const titleMap: Record<string, string> = {
      navigate: `切换到 ${action.args.page || '首页'}`,
      search_mods: `搜索模组 ${action.args.query || action.args.q || ''}`,
      search_modpacks: `搜索整合包 ${action.args.query || action.args.q || ''}`,
      launch_instance: `启动实例 ${action.args.instance || action.args.name || ''}`,
      launch_game: '启动当前实例',
      open_folder: `打开 ${action.args.type || 'mods'} 文件夹`,
      get_status: '读取启动器状态',
    }
    const task: IslandTask = { id: `agent-${Date.now()}`, title: titleMap[action.name] || `执行 ${action.name}`, status: 'running' }
    const island = useIslandStore.getState()
    island.setAiActive(true)
    island.setAiThinking(false)
    island.setAiOpen(true)
    island.setAiMessage('正在执行启动器操作')
    island.setAiTasks([task])
    try {
      const result = await executeAction(action)
      island.updateTask(task.id, { status: result?.includes('失败') || result?.includes('未知') ? 'failed' : 'done', detail: result || undefined })
      island.setAiMessage(result || '操作完成')
      return result
    } catch (e) {
      const result = `执行失败: ${String(e)}`
      island.updateTask(task.id, { status: 'failed', detail: result })
      island.setAiMessage(result)
      return result
    } finally {
      island.setAiThinking(false)
      // 保持 aiActive 和任务列表可见 5 秒，让用户看到执行结果
      setTimeout(() => {
        const s = useIslandStore.getState()
        if (s.aiTasks.length > 0 && s.aiTasks.every(t => t.status === 'done' || t.status === 'failed')) {
          s.setAiActive(false)
          s.clearTasks()
          s.setAiMessage('')
        }
      }, 5000)
    }
  }, [executeAction])

  const callAI = useCallback(async (userMsg: string) => {
    if (!config?.api_key) { setShowSetup(true); return }
    if (loading) return

    setLoading(true)
    const controller = new AbortController()
    abortRef.current = controller

    const userContent: any[] = []
    userContent.push({ type: 'text', text: userMsg || '请分析这个文件' })
    if (logContent) {
      if (logContent.startsWith('data:image/')) userContent.push({ type: 'image_url', image_url: { url: logContent } })
      else if (logContent.startsWith('data:video/')) userContent.push({ type: 'video_url', video_url: { url: logContent } })
      else userContent[0].text = `${userMsg || '请分析以下崩溃日志'}\n\n--- 崩溃日志内容 ---\n${logContent}`
    }

    const newUserMsg: ChatMsg = {
      id: genId(), role: 'user',
      content: attachedFile ? `[文件: ${attachedFile.name}] ${userMsg || '请分析这个文件'}` : userMsg,
      file: attachedFile ? { name: attachedFile.name, type: attachedFile.type, dataUrl: logContent ?? '' } : null,
    }
    const userMsgId = newUserMsg.id
    setMessages(prev => [...prev, newUserMsg])
    useIslandStore.getState().addIslandChatEntry({ role: 'user', content: userMsg, ts: Date.now() })
    setAttachedFile(null); setInput(''); setLogContent(null)

    try {
      const historyMsgs = messages
        .filter(m => m.id !== userMsgId)
        .map(m => {
          if (!m.file) return { role: m.role, content: m.content }
          const parts: any[] = [{ type: 'text', text: m.content }]
          if (m.file.type.startsWith('image/')) parts.push({ type: 'image_url', image_url: { url: m.file.dataUrl } })
          else if (m.file.type.startsWith('video/')) parts.push({ type: 'video_url', video_url: { url: m.file.dataUrl } })
          return { role: m.role, content: parts }
        })

      const allMessages = [
        { role: 'system', content: SYSTEM_PROMPT },
        ...historyMsgs,
        { role: 'user', content: userContent },
      ]

       const reply = await invoke<string>('ai_chat_v2', { messages: allMessages, apiKey: config.api_key, model, reasoningEffort: null })
      if (abortRef.current !== controller) return

      const parsed = extractAction(reply)
      const cleanReply = parsed?.clean || reply.trim()

      if (parsed) {
        const result = await runActionWithPlan(parsed.action)
        const finalText = result || cleanReply || '操作完成'
        setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: finalText }])
        useIslandStore.getState().addIslandChatEntry({ role: 'assistant', content: finalText, ts: Date.now() })
        if (ttsEnabled) speakText(finalText)
      } else {
        const text = cleanReply || reply.trimStart()
        if (isMountedRef.current) setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: text }])
        useIslandStore.getState().addIslandChatEntry({ role: 'assistant', content: text, ts: Date.now() })
        if (ttsEnabled && cleanReply) speakText(cleanReply)
      }
    } catch (e: any) {
      if (abortRef.current !== controller) return
      const errText = `请求失败: ${String(e)}`
      if (isMountedRef.current) setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: errText }])
      useIslandStore.getState().addIslandChatEntry({ role: 'assistant', content: errText, ts: Date.now() })
    } finally {
      if (abortRef.current === controller) abortRef.current = null
      if (isMountedRef.current) setLoading(false)
    }
  }, [messages, attachedFile, logContent, config, loading, ttsEnabled, runActionWithPlan])

  const handleSend = useCallback(() => {
    const text = input.trim() || (attachedFile ? '请分析这个文件' : '') || (logContent ? '请分析崩溃日志' : '')
    if (!text || loading) return
    callAI(text)
  }, [input, attachedFile, logContent, callAI, loading])

  const handleStop = () => {
    const c = abortRef.current
    abortRef.current = null
    c?.abort()
    setLoading(false)
    stopTts()
  }

  const finishSetup = () => {
    const provider = getProvider(selectedProviderId)
    const isCustom = selectedProviderId === 'custom'
    const ep = isCustom ? setupCustomEndpoint : provider.endpoint
    const models = isCustom ? (setupCustomModels.filter(m => m.trim()).map(m => m.trim())) : (provider.models ?? [provider.default_model])
    const mdl = isCustom ? (setupCustomModel || models[0] || '') : (model || provider.default_model)
    const fmt = isCustom ? setupCustomApiFormat : provider.api_format
    const newConfig: ProviderConfig = {
      provider_id: selectedProviderId,
      api_key: setupApiKey.trim(),
      custom_endpoint: ep,
      custom_model: mdl,
      custom_models: isCustom ? models : undefined,
      custom_api_format: fmt,
    }
    saveConfig(newConfig); setConfig(newConfig); setModel(mdl); setShowSetup(false)
    if (messages.length === 0 || (messages.length === 1 && messages[0].content.includes('请先配置'))) {
      setMessages([{ id: genId(), role: 'assistant', content: `已连接 ${provider.name}！我是 SkyLine AI 助手，可以帮你分析崩溃日志、搜索模组、管理启动器。请问有什么需要帮助的？` }])
    }
  }

  const toggleTts = () => {
    const next = !ttsEnabled
    setTtsEnabled(next)
    localStorage.setItem(TTS_KEY, String(next))
    if (!next) stopTts()
  }

  const iconForFile = (f: { type: string }) => {
    if (f.type.startsWith('image/')) return <Image className="w-4 h-4" />
    if (f.type.startsWith('video/')) return <Film className="w-4 h-4" />
    return <FileText className="w-4 h-4" />
  }

  const currentProvider = config ? getProvider(config.provider_id) : null

  if (showSetup) {
    return (
      <div className="flex h-full bg-white dark:bg-[#0a0a0a] items-center justify-center p-6">
        <div className="w-full max-w-lg">
          <div className="text-center mb-8">
            <div className="w-16 h-16 rounded-2xl bg-blue-50 dark:bg-blue-500/15 flex items-center justify-center mx-auto mb-4">
              <Sparkles className="w-8 h-8 text-blue-500 dark:text-blue-400" />
            </div>
            <h1 className="text-xl font-bold text-surface-800 dark:text-surface-100">配置 AI 模型</h1>
            <p className="text-sm text-surface-400 dark:text-surface-500 mt-1">选择服务商并填写 API Key 即可开始使用</p>
          </div>

          {setupStep === 'select' ? (
            <div className="space-y-3 max-h-[60vh] overflow-y-auto pr-1">
              <div onClick={() => { selectProvider('agnes', 'setup'); setSetupStep('apikey') }}
                className="relative p-4 rounded-2xl border-2 border-blue-400 dark:border-blue-500/50 bg-blue-50/50 dark:bg-blue-500/5 cursor-pointer transition-all hover:shadow-lg hover:shadow-blue-500/10">
                <div className="absolute -top-2.5 left-4 px-2 py-0.5 bg-blue-500 text-white text-[10px] font-bold rounded-full">推荐</div>
                <div className="flex items-center gap-3">
                  <ProviderIcon provider={getProvider('agnes')} />
                  <div className="flex-1">
                    <div className="text-sm font-semibold text-surface-800 dark:text-surface-100">Agnes AI</div>
                    <div className="text-xs text-surface-400 dark:text-surface-500">专为国内优化，使用简单，适合普通用户使用</div>
                  </div>
                  <ChevronRight className="w-4 h-4 text-surface-300 dark:text-surface-600" />
                </div>
              </div>
              {PROVIDERS.filter(p => p.id !== 'agnes').map(p => (
                <div key={p.id} onClick={() => { selectProvider(p.id, 'setup'); setSetupStep('apikey') }}
                  className="p-4 rounded-2xl border border-surface-200 dark:border-surface-700/50 bg-white dark:bg-surface-800/50 cursor-pointer transition-all hover:border-surface-300 dark:hover:border-surface-600 hover:shadow-md">
                  <div className="flex items-center gap-3">
                    <ProviderIcon provider={p} size={40} />
                    <div className="flex-1">
                      <div className="text-sm font-semibold text-surface-800 dark:text-surface-100">{p.name}</div>
                      <div className="text-xs text-surface-400 dark:text-surface-500">{p.default_model}</div>
                    </div>
                    <ChevronRight className="w-4 h-4 text-surface-300 dark:text-surface-600" />
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="space-y-4">
              <button onClick={() => setSetupStep('select')} className="text-xs text-surface-400 hover:text-surface-600 dark:hover:text-surface-300 transition-colors">← 返回选择</button>
              <div className="p-3 rounded-xl bg-surface-50 dark:bg-surface-800/60 flex items-center gap-3">
                <ProviderIcon provider={getProvider(selectedProviderId)} size={32} />
                <div>
                  <div className="text-sm font-medium text-surface-800 dark:text-surface-100">{getProvider(selectedProviderId).name}</div>
                  <div className="text-[11px] text-surface-400">{getProvider(selectedProviderId).default_model}</div>
                </div>
              </div>
              <div>
                <label className="block text-xs font-medium text-surface-500 dark:text-surface-400 mb-1.5">API Key</label>
                <input type="password" value={setupApiKey} onChange={(e) => setSetupApiKey(e.target.value)} placeholder="sk-xxxxxxxxxxxxxxxx"
                  className="w-full bg-surface-50 dark:bg-surface-900 rounded-xl px-4 py-3 text-sm text-surface-800 dark:text-surface-200 placeholder-surface-400 dark:placeholder-surface-600 outline-none border border-surface-200 dark:border-white/5 focus:border-blue-400 dark:focus:border-blue-500/30 font-mono transition-colors" autoFocus />
                {selectedProviderId === 'agnes' && (
                  <p className="text-[11px] text-surface-400 dark:text-surface-500 mt-1.5">
                    前往 <button onClick={() => open('https://platform.agnes-ai.com/settings/apiKeys')} className="text-blue-500 hover:underline inline-flex items-center gap-0.5">platform.agnes-ai.com <ExternalLink className="w-3 h-3" /></button> 获取
                  </p>
                )}
              </div>
              {selectedProviderId === 'custom' && (
                <>
                  <div>
                    <label className="block text-xs font-medium text-surface-500 dark:text-surface-400 mb-1.5">接口地址</label>
                    <input value={setupCustomEndpoint} onChange={(e) => setSetupCustomEndpoint(e.target.value)} placeholder="https://your-api.com/v1/chat/completions"
                      className="w-full bg-surface-50 dark:bg-surface-900 rounded-xl px-4 py-3 text-sm text-surface-800 dark:text-surface-200 placeholder-surface-400 dark:placeholder-surface-600 outline-none border border-surface-200 dark:border-white/5 focus:border-blue-400 dark:focus:border-blue-500/30 font-mono transition-colors" />
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-surface-500 dark:text-surface-400 mb-1.5">模型名称</label>
                    <input value={setupCustomModel} onChange={(e) => setSetupCustomModel(e.target.value)} placeholder="gpt-4o"
                      className="w-full bg-surface-50 dark:bg-surface-900 rounded-xl px-4 py-3 text-sm text-surface-800 dark:text-surface-200 placeholder-surface-400 dark:placeholder-surface-600 outline-none border border-surface-200 dark:border-white/5 focus:border-blue-400 dark:focus:border-blue-500/30 font-mono transition-colors" />
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-surface-500 dark:text-surface-400 mb-1.5">已保存的模型列表</label>
                    <div className="space-y-1.5">
                      {setupCustomModels.length === 0 && <p className="text-[11px] text-surface-400">暂无模型，请在下方输入并添加</p>}
                      {setupCustomModels.map((m, i) => (
                        <div key={`${m}-${i}`} className="flex items-center gap-2">
                          <span className="flex-1 px-3 py-1.5 rounded-lg bg-surface-50 dark:bg-surface-900 text-xs text-surface-700 dark:text-surface-200 border border-surface-200 dark:border-white/5 font-mono truncate">{m}</span>
                          <button onClick={() => setSetupCustomModels(prev => prev.filter((_, j) => j !== i))} className="p-1 rounded hover:bg-red-100 dark:hover:bg-red-500/20 text-red-400" title="删除">
                            <X className="w-3.5 h-3.5" />
                          </button>
                        </div>
                      ))}
                    </div>
                    <div className="flex gap-1.5 mt-1.5">
                      <input value={setupCustomModel} onChange={(e) => setSetupCustomModel(e.target.value)} placeholder="gpt-4o"
                        className="flex-1 bg-surface-50 dark:bg-surface-900 rounded-lg px-3 py-2 text-xs text-surface-800 dark:text-surface-200 placeholder-surface-400 dark:placeholder-surface-600 outline-none border border-surface-200 dark:border-white/5 focus:border-blue-400 dark:focus:border-blue-500/30 font-mono" />
                      <button
                        onClick={() => {
                          const m = setupCustomModel.trim()
                          if (!m) return
                          if (!setupCustomModels.includes(m)) {
                            setSetupCustomModels(prev => [...prev, m])
                            setSetupCustomModel('')
                          }
                        }}
                        className="px-3 py-1.5 rounded-lg bg-blue-50 dark:bg-blue-500/10 hover:bg-blue-100 dark:hover:bg-blue-500/20 text-blue-600 dark:text-blue-300 text-xs font-medium transition-colors shrink-0">添加</button>
                    </div>
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-surface-500 dark:text-surface-400 mb-1.5">API 格式</label>
                    <select value={setupCustomApiFormat} onChange={(e) => setSetupCustomApiFormat(e.target.value)}
                      className="w-full bg-surface-50 dark:bg-surface-900 rounded-xl px-4 py-3 text-sm text-surface-800 dark:text-surface-200 outline-none border border-surface-200 dark:border-white/5 focus:border-blue-400 dark:focus:border-blue-500/30 transition-colors">
                      <option value="openai">OpenAI 兼容</option>
                      <option value="anthropic">Anthropic</option>
                      <option value="google">Google Gemini</option>
                    </select>
                  </div>
                </>
              )}
              <button onClick={finishSetup} disabled={!setupApiKey.trim()}
                className="w-full py-3 rounded-xl bg-blue-500 hover:bg-blue-600 disabled:bg-surface-200 dark:disabled:bg-surface-700 text-white disabled:text-surface-400 font-medium text-sm transition-colors">开始使用</button>
            </div>
          )}
          <p className="text-[10px] text-surface-300 dark:text-surface-700 mt-6 text-center">AI 生成内容仅供参考，请以实际情况为准</p>
        </div>
      </div>
    )
  }

  return (
    <div className="flex h-full bg-white dark:bg-[#0a0a0a] relative ai-page" onClick={(e) => {
      const sidebar = (e.target as HTMLElement).closest('[data-sidebar]')
      if (!sidebar && showHistory) setShowHistory(false)
    }}>
      {showHistory && <div className="fixed inset-0 z-10 bg-transparent" />}

      <div data-sidebar className={`shrink-0 border-r border-surface-200 dark:border-white/5 flex flex-col absolute left-0 top-0 bottom-0 z-20 bg-surface-50 dark:bg-surface-950 transition-all duration-200 ease-out ${showHistory ? 'w-64 opacity-100 translate-x-0' : 'w-0 opacity-0 -translate-x-4 pointer-events-none'}`}>
        <div className="p-3 border-b border-surface-200 dark:border-white/5 flex items-center justify-between">
          <span className="text-xs text-surface-400 font-medium">对话历史</span>
          <button onClick={() => { setShowNewSessionDialog(true); setNewSessionName('') }} className="p-1 rounded hover:bg-surface-200 dark:hover:bg-surface-800 transition-colors" title="新对话">
            <Plus className="w-4 h-4 text-surface-400" />
          </button>
        </div>
        <div className="flex-1 overflow-y-auto">
          {sessions.map(s => (
            <div key={s.id} onClick={() => switchSession(s.id)}
              className={`px-3 py-2.5 cursor-pointer border-b border-surface-200 dark:border-white/5 flex items-center gap-2 group transition-colors ${s.id === activeId ? 'bg-blue-50 dark:bg-blue-500/10' : 'hover:bg-surface-100 dark:hover:bg-surface-800'}`}>
              <div className="flex-1 min-w-0">
                <div className="text-xs text-surface-700 dark:text-surface-200 truncate">{s.name}</div>
                <div className="text-[10px] text-surface-400 dark:text-surface-500">{s.messages.length} 条消息</div>
              </div>
              <button onClick={(e) => deleteSession(s.id, e)} className="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-red-100 dark:hover:bg-red-500/20 transition-all">
                <Trash2 className="w-3 h-3 text-red-400" />
              </button>
            </div>
          ))}
        </div>
      </div>

      <div className="flex-1 flex flex-col min-w-0">
        <div className="shrink-0 px-4 py-3 border-b border-surface-200 dark:border-white/5 flex items-center gap-3">
          <button onClick={() => setShowHistory(v => !v)} className="p-2 rounded-lg bg-surface-100 dark:bg-surface-800 hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors">
            <History className="w-4 h-4 text-surface-400" />
          </button>
          <div className="flex-1 min-w-0">
            <h2 className="text-sm font-medium text-surface-800 dark:text-surface-200 truncate">{sessions.find(s => s.id === activeId)?.name ?? 'AI 助手'}</h2>
            <p className="text-[11px] text-surface-400 dark:text-surface-500">{currentProvider?.name ?? 'AI'} · {currentProvider ? getModelDisplayName(currentProvider, model || config?.custom_model || currentProvider.default_model) : ''}</p>
          </div>
          <button onClick={toggleTts}
            className={`p-2 rounded-lg transition-colors ${ttsEnabled ? 'bg-blue-50 dark:bg-blue-500/10 text-blue-500 dark:text-blue-400' : 'bg-surface-100 dark:bg-surface-800 text-surface-400'}`}
            title={ttsEnabled ? '关闭语音播报' : '开启语音播报'}>
            {ttsEnabled ? <Volume2 className="w-4 h-4" /> : <VolumeX className="w-4 h-4" />}
          </button>
          <button onClick={() => {
            setShowSettings(v => !v)
            setSelectedProviderId(config?.provider_id ?? 'agnes')
            const keys = loadKeys()
            setSetupApiKey(keys[config?.provider_id ?? 'agnes'] ?? '')
            if (config?.provider_id === 'custom') {
              setSetupCustomEndpoint(config.custom_endpoint ?? '')
              setSetupCustomModels(config.custom_models ?? [])
              setSetupCustomApiFormat(config.custom_api_format ?? 'openai')
            }
          }}
            className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-surface-100 dark:bg-surface-800 hover:bg-surface-200 dark:hover:bg-surface-700 text-surface-600 dark:text-surface-300 text-xs transition-colors">
            <Settings className="w-3.5 h-3.5" /><span>设置</span>
          </button>
        </div>

        {showSettings && (
          <div className="shrink-0 border-b border-surface-200 dark:border-white/5 bg-surface-50 dark:bg-surface-900 p-4 space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium text-surface-500 dark:text-surface-400">模型设置</span>
              <button onClick={() => setShowSettings(false)} className="p-1 rounded hover:bg-surface-200 dark:hover:bg-surface-700"><X className="w-3.5 h-3.5 text-surface-400" /></button>
            </div>
            <div className="flex flex-wrap gap-2">
              {PROVIDERS.map(p => (
                <button key={p.id} onClick={() => selectProvider(p.id, 'settings')}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs transition-colors ${
                    selectedProviderId === p.id ? 'bg-blue-500 text-white' : 'bg-surface-100 dark:bg-surface-800 text-surface-600 dark:text-surface-300 hover:bg-surface-200 dark:hover:bg-surface-700'
                  }`}>
                  <ProviderIcon provider={p} size={16} />
                  {p.name}
                </button>
              ))}
            </div>
            <input type="password" value={setupApiKey} onChange={(e) => setSetupApiKey(e.target.value)} placeholder={setupApiKey ? '已保存的 API Key（可修改）' : '输入 API Key...'}
              className="w-full bg-white dark:bg-surface-800 rounded-lg px-3 py-2 text-xs text-surface-800 dark:text-surface-200 placeholder-surface-400 dark:placeholder-surface-600 outline-none border border-surface-200 dark:border-white/5 focus:border-blue-400 dark:focus:border-blue-500/30 font-mono" />
            {selectedProviderId === 'custom' && (
              <>
                <div>
                  <label className="block text-xs font-medium text-surface-500 dark:text-surface-400 mb-1">接口地址</label>
                  <input value={setupCustomEndpoint} onChange={(e) => setSetupCustomEndpoint(e.target.value)} placeholder="https://your-api.com/v1/chat/completions"
                    className="w-full bg-white dark:bg-surface-800 rounded-lg px-3 py-2 text-xs text-surface-800 dark:text-surface-200 placeholder-surface-400 dark:placeholder-surface-600 outline-none border border-surface-200 dark:border-white/5 focus:border-blue-400 dark:focus:border-blue-500/30 font-mono" />
                </div>
                <div>
                  <label className="block text-xs font-medium text-surface-500 dark:text-surface-400 mb-1">模型列表</label>
                  <div className="space-y-1">
                    {setupCustomModels.map((m, i) => (
                      <div key={`${m}-${i}`} className="flex items-center gap-2">
                        <span className="flex-1 px-3 py-1.5 rounded-lg bg-white dark:bg-surface-800 text-xs text-surface-700 dark:text-surface-200 border border-surface-200 dark:border-white/5 font-mono truncate">{m}</span>
                        <button onClick={() => setSetupCustomModels(prev => prev.filter((_, j) => j !== i))} className="p-1 rounded hover:bg-red-100 dark:hover:bg-red-500/20 text-red-400" title="删除">
                          <X className="w-3 h-3" />
                        </button>
                      </div>
                    ))}
                  </div>
                  <div className="flex gap-1.5 mt-1">
                    <input value={setupCustomModel} onChange={(e) => setSetupCustomModel(e.target.value)} placeholder="添加模型名称"
                      className="flex-1 bg-white dark:bg-surface-800 rounded-lg px-3 py-2 text-xs text-surface-800 dark:text-surface-200 placeholder-surface-400 dark:placeholder-surface-600 outline-none border border-surface-200 dark:border-white/5 focus:border-blue-400 dark:focus:border-blue-500/30 font-mono" />
                    <button
                      onClick={() => {
                        const m = setupCustomModel.trim()
                        if (m && !setupCustomModels.includes(m)) { setSetupCustomModels(prev => [...prev, m]); setSetupCustomModel('') }
                      }}
                      className="px-3 py-1.5 rounded-lg bg-blue-50 dark:bg-blue-500/10 hover:bg-blue-100 dark:hover:bg-blue-500/20 text-blue-600 dark:text-blue-300 text-xs font-medium transition-colors shrink-0">添加</button>
                  </div>
                </div>
                <div>
                  <label className="block text-xs font-medium text-surface-500 dark:text-surface-400 mb-1">API 格式</label>
                  <select value={setupCustomApiFormat} onChange={(e) => setSetupCustomApiFormat(e.target.value)}
                    className="w-full bg-white dark:bg-surface-800 rounded-lg px-3 py-2 text-xs text-surface-800 dark:text-surface-200 outline-none border border-surface-200 dark:border-white/5 focus:border-blue-400 dark:focus:border-blue-500/30 transition-colors">
                    <option value="openai">OpenAI 兼容</option>
                    <option value="anthropic">Anthropic</option>
                    <option value="google">Google Gemini</option>
                  </select>
                </div>
              </>
            )}
            <div className="flex gap-2">
              <button onClick={async () => {
                const provider = getProvider(selectedProviderId)
                const mdl = selectedProviderId === 'custom' ? (setupCustomModel || setupCustomModels[0] || '') : (model || provider.default_model)
                if (!setupApiKey.trim()) { alert('请先填写 API Key'); return }
                setTesting(true)
                const result = await testConnection(provider, setupApiKey.trim(), mdl)
                setTesting(false)
                setTestResult(result.message)
                setTimeout(() => setTestResult(null), 5000)
              }} disabled={testing || !setupApiKey.trim()}
                className="flex-1 py-1.5 rounded-lg bg-surface-100 dark:bg-surface-700 hover:bg-surface-200 dark:hover:bg-surface-600 disabled:opacity-50 text-surface-600 dark:text-surface-300 text-xs transition-colors">
                {testing ? '测试中...' : '测试连接'}
              </button>
              <button onClick={() => setShowSettings(false)} className="flex-1 py-1.5 rounded-lg bg-surface-100 dark:bg-surface-700 hover:bg-surface-200 dark:hover:bg-surface-600 text-surface-600 dark:text-surface-300 text-xs transition-colors">取消</button>
              <button onClick={() => {
                const provider = getProvider(selectedProviderId)
                const isCustom = selectedProviderId === 'custom'
                const ep = isCustom ? setupCustomEndpoint : provider.endpoint
                const models = isCustom ? (setupCustomModels.filter(m => m.trim()).map(m => m.trim())) : (provider.models ?? [provider.default_model])
                const mdl = isCustom ? (models[0] ?? '') : (model || provider.default_model)
                const newCfg: ProviderConfig = {
                  provider_id: selectedProviderId,
                  api_key: setupApiKey.trim(),
                  custom_endpoint: ep,
                  custom_model: mdl,
                  custom_models: isCustom ? models : undefined,
                  custom_api_format: isCustom ? setupCustomApiFormat : provider.api_format,
                }
                saveConfig(newCfg); setConfig(newCfg); setModel(mdl); setShowSettings(false)
              }}
                className="flex-1 py-1.5 rounded-lg bg-blue-500 hover:bg-blue-600 text-white text-xs transition-colors">保存</button>
            </div>
            {selectedProviderId === 'agnes' && (
              <p className="text-[10px] text-surface-400">获取 Key: <button onClick={() => open('https://platform.agnes-ai.com/settings/apiKeys')} className="text-blue-500 underline">platform.agnes-ai.com</button></p>
            )}
            {testResult && (
              <div className={`text-xs px-3 py-2 rounded-lg ${testResult.includes('成功') ? 'bg-green-50 dark:bg-green-500/10 text-green-600 dark:text-green-400' : 'bg-red-50 dark:bg-red-500/10 text-red-600 dark:text-red-400'}`}>
                {testResult}
              </div>
            )}
          </div>
        )}

        <div ref={messagesRef} className={`flex-1 overflow-y-auto px-4 py-3 space-y-3 ${dragOver ? 'ai-drag-over' : ''}`}
          onPaste={handlePaste}>
          {messages.map((msg) => (
            <div key={msg.id} className={`ai-msg flex gap-2.5 ${msg.role === 'user' ? 'justify-end' : ''}`}>
              {msg.role === 'assistant' && (
                <div className="w-7 h-7 rounded-lg bg-blue-50 dark:bg-blue-500/15 flex items-center justify-center shrink-0 mt-0.5">
                  <Bot className="w-4 h-4 text-blue-500 dark:text-blue-400" />
                </div>
              )}
              <div className={`max-w-[85%] ${msg.role === 'user' ? '' : ''}`}>
                <div className={`rounded-2xl px-3.5 pt-2.5 pb-3 text-sm leading-relaxed whitespace-pre-wrap break-words select-text ${
                  msg.role === 'user' ? 'bg-blue-50 dark:bg-blue-500/15 text-blue-800 dark:text-blue-100 rounded-br-md' : 'bg-surface-100 dark:bg-surface-850 text-surface-800 dark:text-surface-200 rounded-bl-md'
                }`}>
                  {msg.file && msg.file.dataUrl && (
                    <div className="mb-2">
                      {msg.file.type.startsWith('image/') ? <img src={msg.file.dataUrl} alt={msg.file.name} className="max-w-full rounded-lg max-h-48 object-contain" /> :
                       msg.file.type.startsWith('video/') ? <video src={msg.file.dataUrl} controls className="max-w-full rounded-lg max-h-48" /> :
                        <div className="text-xs text-surface-500 dark:text-surface-400 font-mono bg-surface-50 dark:bg-surface-900 rounded px-2 py-1 max-h-24 overflow-y-auto">{msg.file.name}</div>}
                    </div>
                  )}
                  {msg.action && (
                    <div className="mb-2 px-3 py-2 rounded-lg bg-blue-50 dark:bg-blue-500/10 border border-blue-200 dark:border-blue-500/20 text-xs text-blue-600 dark:text-blue-300 flex items-center gap-2">
                      <span className="animate-spin">⟳</span> 正在执行: {msg.action.name}
                    </div>
                  )}
                  <div className="whitespace-pre-wrap select-text">{msg.content}</div>
                </div>
                <div className="flex gap-1 mt-1 px-1 h-5">
                  <button onClick={() => copyMessage(msg.content)} className="p-1 rounded hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors" title="复制"><Copy className="w-3 h-3 text-surface-400" /></button>
                  <button onClick={() => deleteMessage(msg.id)} className="p-1 rounded hover:bg-red-500/20 transition-colors" title="删除"><Trash2 className="w-3 h-3 text-red-400" /></button>
                </div>
              </div>
              {msg.role === 'user' && (
                <div className="w-7 h-7 rounded-lg bg-surface-200 dark:bg-surface-750 flex items-center justify-center shrink-0 mt-0.5">
                  <User className="w-4 h-4 text-surface-400" />
                </div>
              )}
            </div>
          ))}
          {loading && (
            <div className="ai-msg flex gap-2.5">
              <div className="w-7 h-7 rounded-lg bg-blue-50 dark:bg-blue-500/15 flex items-center justify-center shrink-0 mt-0.5">
                <Bot className="w-4 h-4 text-blue-500 dark:text-blue-400" />
              </div>
              <div className="bg-surface-100 dark:bg-surface-850 rounded-2xl rounded-bl-md px-3.5 py-3 flex items-center gap-1.5">
                <span className="ai-typing-dot" /><span className="ai-typing-dot" /><span className="ai-typing-dot" />
              </div>
            </div>
          )}
        </div>

        {attachedFile && (
          <div className="shrink-0 mx-4 mb-1 bg-surface-100 dark:bg-surface-800 rounded-xl px-3 py-2 flex items-center gap-2">
            {iconForFile(attachedFile)}
            <span className="text-xs text-surface-600 dark:text-surface-300 truncate flex-1">{attachedFile.name}</span>
            <span className="text-[10px] text-surface-400 dark:text-surface-500">{(attachedFile.size / 1024).toFixed(0)} KB</span>
            <button onClick={() => setAttachedFile(null)} className="p-0.5 hover:bg-surface-200 dark:hover:bg-surface-700 rounded"><X className="w-3.5 h-3.5 text-surface-400" /></button>
          </div>
        )}

        <div className="shrink-0 px-4 pt-2.5 pb-3 border-t border-surface-200 dark:border-white/5 relative">
          <div className="flex items-center gap-2 mb-2">
            <button
              onClick={() => setShowModelPicker(v => !v)}
              className="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg bg-surface-100 dark:bg-surface-800 hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors cursor-pointer max-w-[240px]"
            >
              {currentProvider && <ProviderIcon provider={currentProvider} size={20} />}
              <span className="text-xs font-medium text-surface-700 dark:text-surface-200 truncate">
                {currentProvider ? getModelDisplayName(currentProvider, model || config?.custom_model || currentProvider.default_model) : '选择模型'}
              </span>
              <ChevronDown className="w-3 h-3 text-surface-400 shrink-0" />
            </button>
            <div className="flex-1" />
            <button onClick={() => { setShowSettings(v => !v); setShowModelPicker(false) }}
              className="p-1.5 rounded-lg bg-surface-100 dark:bg-surface-800 hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors cursor-pointer"
              title="模型设置">
              <Settings className="w-3.5 h-3.5 text-surface-400" />
            </button>
          </div>

          {/* 模型选择面板（仿 opencode） */}
          {showModelPicker && (
            <div className="absolute left-4 right-4 bottom-full mb-1 rounded-xl border border-surface-200 dark:border-surface-700/50 bg-white dark:bg-surface-850 shadow-2xl overflow-hidden z-20">
              <div className="max-h-72 overflow-y-auto p-1.5 space-y-0.5">
                {(() => {
                  const keys = loadKeys()
                  const configuredProviders = PROVIDERS.filter(p => keys[p.id] || (p.id === 'custom' && config?.provider_id === 'custom'))
                  return (
                    <>
                      {configuredProviders.map(p => {
                        const models = getModelList(p, config)
                        const isActive = p.id === currentProvider?.id
                        return (
                          <div key={p.id}>
                            <button
                              onClick={() => {
                                selectProvider(p.id, 'picker')
                                if (p.id === 'custom') { setShowModelPicker(false); setShowSettings(true) }
                                else setModel(getModelList(p, config)[0] ?? p.default_model)
                              }}
                              className={`w-full flex items-center gap-2 px-2 py-1.5 rounded-lg transition-colors cursor-pointer ${isActive ? 'bg-blue-50 dark:bg-blue-500/10' : 'hover:bg-surface-100 dark:hover:bg-surface-800'}`}
                            >
                              <ProviderIcon provider={p} size={18} />
                              <span className={`text-xs ${isActive ? 'font-semibold text-blue-600 dark:text-blue-400' : 'text-surface-700 dark:text-surface-200'}`}>{p.name}</span>
                              {isActive && <ChevronDown className="w-3 h-3 text-blue-500 rotate-180 ml-auto" />}
                            </button>
                            {isActive && p.id !== 'custom' && models.length > 0 && (
                              <div className="pl-8 pb-1 flex flex-wrap gap-1">
                                {models.map(m => (
                                  <button key={m}
                                    onClick={() => {
                                      setModel(m)
                                      if (config) { const newCfg = { ...config, custom_model: m }; saveConfig(newCfg); setConfig(newCfg) }
                                    }}
                                    className={`px-2 py-0.5 rounded-md text-[11px] transition-colors cursor-pointer ${
                                      (model || config?.custom_model || currentProvider?.default_model) === m
                                        ? 'bg-blue-500 text-white'
                                        : 'bg-surface-100 dark:bg-surface-800 text-surface-600 dark:text-surface-300 hover:bg-surface-200 dark:hover:bg-surface-700'
                                    }`}
                                    title={m}
                                  >
                                    {getModelDisplayName(p, m)}
                                  </button>
                                ))}
                              </div>
                            )}
                            {isActive && p.id === 'custom' && (
                              <div className="pl-8 pb-1">
                                {getModelList(p, config).length === 0 ? (
                                  <button onClick={() => { setShowModelPicker(false); setShowSettings(true) }}
                                    className="text-[11px] text-blue-500 hover:underline cursor-pointer">去配置自定义模型 →</button>
                                ) : (
                                  <div className="flex flex-wrap gap-1">
                                    {getModelList(p, config).map(m => (
                                      <button key={m}
                                        onClick={() => { setModel(m); if (config) { const newCfg = { ...config, custom_model: m }; saveConfig(newCfg); setConfig(newCfg) } }}
                                        className={`px-2 py-0.5 rounded-md text-[11px] font-mono transition-colors cursor-pointer ${
                                          (model || config?.custom_model) === m ? 'bg-blue-500 text-white' : 'bg-surface-100 dark:bg-surface-800 text-surface-600 dark:text-surface-300 hover:bg-surface-200 dark:hover:bg-surface-700'
                                        }`}
                                      >{m}</button>
                                    ))}
                                  </div>
                                )}
                              </div>
                            )}
                          </div>
                        )
                      })}
                      {configuredProviders.length === 0 && (
                        <div className="px-2 py-3 text-xs text-surface-400">暂无已连接模型，请先在设置中测试并保存连接。</div>
                      )}
                    </>
                  )
                })()}
              </div>
            </div>
          )}

          <div className="flex gap-2">
            <input type="file" ref={fileInputRef} className="hidden" accept=".log,.txt,.png,.jpg,.jpeg,.mp4,.webm"
              onChange={(e) => { const f = e.target.files?.[0]; if (f) handleFileSelect(f) }} />
            <button onClick={() => fileInputRef.current?.click()} className="p-2.5 rounded-xl bg-surface-100 dark:bg-surface-800 hover:bg-surface-200 dark:hover:bg-surface-700 text-surface-500 dark:text-surface-400 transition-colors" title="上传文件" disabled={loading}>
              <Paperclip className="w-5 h-5" />
            </button>
            <input value={input} onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && !e.shiftKey && !loading && handleSend()}
              placeholder={loading ? '正在分析中...' : '描述你遇到的问题，或让我帮你操作启动器...'}
              disabled={loading}
              className="flex-1 bg-surface-50 dark:bg-surface-850 rounded-xl px-3.5 py-2.5 text-sm text-surface-800 dark:text-surface-200 placeholder-surface-400 dark:placeholder-surface-600 outline-none border border-surface-200 dark:border-white/5 focus:border-blue-400 dark:focus:border-blue-500/30 transition-colors disabled:opacity-50 select-text" />
            {loading ? (
              <button onClick={handleStop} className="p-2.5 rounded-xl bg-red-50 dark:bg-red-500/10 hover:bg-red-100 dark:hover:bg-red-500/20 text-red-500 dark:text-red-400 transition-colors" title="停止">
                <StopCircle className="w-5 h-5" />
              </button>
            ) : (
              <button onClick={handleSend} disabled={loading || (!input.trim() && !attachedFile && !logContent)}
                className="p-2.5 rounded-xl bg-blue-50 dark:bg-blue-500/10 hover:bg-blue-100 dark:hover:bg-blue-500/20 text-blue-500 dark:text-blue-400 disabled:opacity-30 transition-colors">
                <Send className="w-5 h-5" />
              </button>
            )}
          </div>
          <p className="text-[10px] text-surface-400 dark:text-surface-600 mt-2 text-center">
            支持上传崩溃日志 (.log/.txt)、截图、录屏 · 拖拽文件到聊天区域 · 可让我帮你搜索模组、启动游戏
          </p>
          <p className="text-[10px] text-surface-300 dark:text-surface-700 mt-1 text-center">
            AI 生成内容仅供参考，请以实际情况为准
          </p>
        </div>
      </div>

      {showNewSessionDialog && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" onClick={() => { setShowNewSessionDialog(false); setNewSessionName('') }}>
          <div className="ai-modal bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-600 rounded-2xl shadow-2xl w-full max-w-sm mx-4 overflow-hidden" onClick={(e) => e.stopPropagation()}>
            <div className="px-5 py-4 border-b border-surface-200 dark:border-white/5">
              <p className="text-sm font-medium text-surface-800 dark:text-surface-200">新建对话</p>
              <p className="text-xs text-surface-400 dark:text-surface-500 mt-0.5">为本次对话取一个名字（可选）</p>
            </div>
            <div className="px-5 py-4 space-y-3">
              <input value={newSessionName} onChange={(e) => setNewSessionName(e.target.value)} placeholder="输入对话名称..."
                className="w-full bg-surface-50 dark:bg-surface-900 rounded-xl px-3.5 py-2.5 text-sm text-surface-800 dark:text-surface-200 placeholder-surface-400 dark:placeholder-surface-600 outline-none border border-surface-200 dark:border-white/5 focus:border-blue-400 dark:focus:border-blue-500/30 transition-colors"
                onKeyDown={(e) => { if (e.key === 'Enter') confirmNewSession() }} autoFocus />
              <div className="flex gap-2">
                <button onClick={() => { setShowNewSessionDialog(false); setNewSessionName('') }} className="flex-1 py-2.5 rounded-xl bg-surface-100 dark:bg-surface-700 hover:bg-surface-200 dark:hover:bg-surface-600 text-surface-600 dark:text-surface-300 transition-colors text-sm">取消</button>
                <button onClick={confirmNewSession} className="flex-1 py-2.5 rounded-xl bg-blue-50 dark:bg-blue-500/10 hover:bg-blue-100 dark:hover:bg-blue-500/20 text-blue-600 dark:text-blue-300 transition-colors text-sm font-medium">创建</button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
