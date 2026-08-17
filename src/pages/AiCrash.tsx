import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open } from '@tauri-apps/plugin-shell'
import { useCallback, useRef, useState, useEffect } from 'react'
import { Send, Paperclip, FileText, Image, Film, X, User, Bot, Copy, Trash2, History, Key, Plus, StopCircle, Check } from 'lucide-react'
import { useSearchParams } from 'react-router-dom'

interface ChatMsg {
  role: 'user' | 'assistant'
  content: string
  file?: { name: string; type: string; dataUrl: string } | null
  id: string
}

interface ChatSession {
  id: string
  name: string
  messages: ChatMsg[]
  createdAt: number
  instanceId?: string
}

const SESSIONS_KEY = 'skyline-ai-chats'
const API_KEY_KEY = 'skyline-agnes-api-key'
const MODEL = 'agnes-2.5-flash'

function genId() {
  return Math.random().toString(36).slice(2, 10)
}

function loadSessions(): ChatSession[] {
  try {
    return JSON.parse(localStorage.getItem(SESSIONS_KEY) ?? '[]')
  } catch { return [] }
}

function saveSessions(s: ChatSession[]) {
  localStorage.setItem(SESSIONS_KEY, JSON.stringify(s))
}

function loadApiKey(): string {
  return localStorage.getItem(API_KEY_KEY) ?? ''
}

export default function AiCrash() {
  const [searchParams] = useSearchParams()
  const instanceId = searchParams.get('instance')

  const [sessions, setSessions] = useState<ChatSession[]>([])
  const [activeId, setActiveId] = useState<string | null>(null)
  const [apiKey, setApiKey] = useState('')
  const [apiKeySaved, setApiKeySaved] = useState(false)
  const [showKeyEditor, setShowKeyEditor] = useState(false)
  const [editKey, setEditKey] = useState('')

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

  
  useEffect(() => {
    isMountedRef.current = true
    return () => { isMountedRef.current = false }
  }, [])

  
  useEffect(() => {
    const stored = loadSessions()
    setSessions(stored)
    const key = loadApiKey()
    setApiKey(key)
    setApiKeySaved(!!key)

    if (instanceId) {
      const hasExisting = stored.find(s => s.instanceId === instanceId)
      const autoAnalyze = searchParams.get('auto_analyze') === '1'
      if (autoAnalyze) {
        const key = apiKey
        if (!key) {
          const base = hasExisting ? hasExisting.messages : []
          setActiveId(hasExisting?.id ?? null)
          setMessages(base.length ? base : [{ id: genId(), role: 'assistant' as const, content: '请先配置 Agnes API Key' }])
          setShowKeyEditor(true)
          return
        }
        const base = hasExisting ? hasExisting.messages : []
        const pending: ChatMsg[] = [...base, { id: genId(), role: 'assistant' as const, content: '检测到游戏启动崩溃，正在自动分析日志...' }]
        if (!hasExisting) {
          const newSession = createSession(instanceId)
          const updated = [newSession, ...stored]
          setSessions(updated)
          saveSessions(updated)
          setActiveId(newSession.id)
        } else {
          setActiveId(hasExisting.id)
        }
        setMessages(pending)
        invoke<string>('analyze_crash_auto', { instanceId, apiKey: key })
          .then((analysis) => {
            if (isMountedRef.current) {
              setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: analysis }])
            }
          })
          .catch((e) => {
            if (isMountedRef.current) {
              setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: `分析失败: ${String(e)}` }])
            }
          })
        return
      }
      if (hasExisting) {
        setActiveId(hasExisting.id)
        setMessages(hasExisting.messages)
        return
      }
      const newSession = createSession(instanceId)
      const updated = [newSession, ...stored]
      setSessions(updated)
      saveSessions(updated)
      setActiveId(newSession.id)
      
      invoke<string>('read_latest_log', { instanceId }).then((content) => {
        const tail = content.split('\n').slice(-300).join('\n')
        setLogContent(tail)
        setMessages([{ id: genId(), role: 'assistant', content: `检测到实例 \`${instanceId}\` 最近崩溃，已加载 latest.log 最后 300 行。正在自动分析中...` }])
      }).catch(() => {
        setMessages([{ id: genId(), role: 'assistant', content: `检测到实例 \`${instanceId}\` 最近崩溃。请描述你遇到的问题，上传截图、视频或崩溃日志文件，我会帮你分析问题。` }])
      })
      return
    }

    if (stored.length > 0) {
      setActiveId(stored[0].id)
      setMessages(stored[0].messages)
    } else {
      setMessages([{ id: genId(), role: 'assistant', content: '你好！我是 Agnes 报错分析助手。请上传崩溃日志（.log/.txt）、截图或录屏，我会帮你分析 Minecraft 报错问题并提供解决方案。' }])
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

  function createSession(instanceId?: string): ChatSession {
    return { id: genId(), name: instanceId ? `实例: ${instanceId}` : `新对话 ${Date.now()}`, messages: [], createdAt: Date.now(), instanceId }
  }

  const switchSession = (id: string) => {
    setActiveId(id)
    const s = sessions.find(x => x.id === id)
    if (s) setMessages(s.messages)
    setShowHistory(false)
  }

  const newSession = () => {
    setShowNewSessionDialog(true)
    setNewSessionName('')
  }

  const confirmNewSession = () => {
    const name = newSessionName.trim() || `对话 ${sessions.length + 1}`
    const s: ChatSession = { id: genId(), name, messages: [], createdAt: Date.now() }
    const updated = [s, ...sessions]
    setSessions(updated)
    saveSessions(updated)
    setActiveId(s.id)
    setMessages([{ id: genId(), role: 'assistant', content: '你好！请上传崩溃日志、截图或录屏，我会帮你分析报错问题。' }])
    setLogContent(null)
    setAttachedFile(null)
    setShowHistory(false)
    setShowNewSessionDialog(false)
    setNewSessionName('')
  }

  const deleteSession = (id: string, e: React.MouseEvent) => {
    e.preventDefault()
    e.stopPropagation()
    const updated = sessions.filter(s => s.id !== id)
    setSessions(updated)
    if (activeId === id) {
      setActiveId(updated[0]?.id ?? null)
      setMessages(updated[0]?.messages ?? [])
    }
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
        const dataUrl = `data:${type};base64,${b64}`
        setLogContent(dataUrl)
        setAttachedFile({ name, size: Math.round(b64.length * 0.75), type })
      } else {
        const text = await invoke<string>('read_crash_file', { path })
        setLogContent(text)
        setAttachedFile({ name, size: new TextEncoder().encode(text).length, type: 'text/plain' })
      }
    } catch (err) {
      setLogContent(null)
      setAttachedFile(null)
      setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: `读取文件 ${name} 失败: ${String(err)}` }])
    }
  }, [])

  
  useEffect(() => {
    let unlisten: (() => void) | undefined
    let disposed = false
    getCurrentWebview().onDragDropEvent((event) => {
      if (disposed) return
      if (event.payload.type === 'enter' || event.payload.type === 'over') {
        setDragOver(true)
      } else if (event.payload.type === 'leave') {
        setDragOver(false)
      } else if (event.payload.type === 'drop') {
        setDragOver(false)
        const p = event.payload.paths[0]
        if (p) void handlePathDrop(p)
      }
    }).then((fn) => {
      if (disposed) fn()
      else unlisten = fn
    }).catch((err) => console.error('onDragDropEvent setup failed', err))
    return () => { disposed = true; unlisten?.() }
  }, [handlePathDrop])

  const handlePaste = useCallback((e: React.ClipboardEvent) => {
    const items = e.clipboardData?.items
    if (items?.[0]?.type.startsWith('image/')) {
      const file = items[0].getAsFile()
      if (file) handleFileSelect(file)
    }
  }, [handleFileSelect])

  const copyMessage = (content: string) => navigator.clipboard.writeText(content)

  const deleteMessage = (id: string) => {
    if (loading) {
      abortRef.current?.abort()
      abortRef.current = null
      setLoading(false)
      setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: '对话已停止' }])
    }
    setMessages(prev => {
      const idx = prev.findIndex(m => m.id === id)
      if (idx === -1) return prev
      
      const next = prev[idx + 1]
      if (next && next.role === 'assistant') {
        return prev.filter(m => m.id !== id && m.id !== next.id)
      }
      return prev.filter(m => m.id !== id)
    })
  }

  const callAI = useCallback(async (userMsg: string) => {
    const key = apiKey
    if (!key) { setShowKeyEditor(true); return }
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
      id: genId(),
      role: 'user',
      content: attachedFile ? `[文件: ${attachedFile.name}] ${userMsg || '请分析这个文件'}` : userMsg,
      file: attachedFile ? { name: attachedFile.name, type: attachedFile.type, dataUrl: logContent ?? '' } : null,
    }
    const userMsgId = newUserMsg.id
    setMessages(prev => [...prev, newUserMsg])
    setAttachedFile(null)
    setInput('')
    setLogContent(null)

    try {
      const body = JSON.stringify({
        model: MODEL,
        messages: [
          { role: 'system', content: '你是Minecraft游戏报错分析专家Agnes。你必须只使用中文回答。分析用户提供的崩溃日志、截图或视频，找出崩溃原因并给出详细可用、分步骤的解决方案。回答要简洁直接，不要输出思考过程。' },
          ...messages.filter(m => m.id !== userMsgId).map(m => {
            if (!m.file) return { role: m.role, content: m.content }
            const parts: any[] = [{ type: 'text', text: m.content }]
            if (m.file.type.startsWith('image/')) {
              parts.push({ type: 'image_url', image_url: { url: m.file.dataUrl } })
            } else if (m.file.type.startsWith('video/')) {
              parts.push({ type: 'video_url', video_url: { url: m.file.dataUrl } })
            }
            return { role: m.role, content: parts }
          }),
          { role: 'user', content: userContent }
        ],
        temperature: 0.3,
      })
      const reply = await invoke<string>('ai_chat', { body, apiKey: key })
      if (isMountedRef.current) {
        setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: reply.trimStart() }])
      }
    } catch (e: any) {
      if (isMountedRef.current) {
        if (e.name === 'AbortError' || String(e).includes('aborted')) {
          
          setMessages(prev => prev.filter(m => m.id !== userMsgId))
        } else {
          setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: `请求失败: ${String(e)}` }])
        }
      }
    } finally {
      if (abortRef.current === controller) abortRef.current = null
      if (isMountedRef.current) setLoading(false)
    }
  }, [messages, attachedFile, logContent, apiKey, loading])

  const handleSend = useCallback(() => {
    const text = input.trim() || (attachedFile ? '请分析这个文件' : '') || (logContent ? '请分析崩溃日志' : '')
    if (!text || loading) return
    callAI(text)
  }, [input, attachedFile, logContent, callAI, loading])

  const handleStop = () => {
    abortRef.current?.abort()
    abortRef.current = null
    setLoading(false)
    
    setMessages(prev => [...prev, { id: genId(), role: 'assistant', content: '对话已停止' }])
  }

  const applyKeyEdit = () => {
    const key = editKey.trim()
    if (!key) return
    localStorage.setItem(API_KEY_KEY, key)
    setApiKey(key)
    setApiKeySaved(true)
    invoke('save_agnes_api_key', { apiKey: key }).catch(console.error)
    setShowKeyEditor(false)
  }

  const iconForFile = (f: { type: string }) => {
    if (f.type.startsWith('image/')) return <Image className="w-4 h-4" />
    if (f.type.startsWith('video/')) return <Film className="w-4 h-4" />
    return <FileText className="w-4 h-4" />
  }

  return (
    <div className="flex h-full bg-white dark:bg-[#0a0a0a] relative ai-page" onClick={(e) => {
      const sidebar = (e.target as HTMLElement).closest('[data-sidebar]')
      if (!sidebar && showHistory) setShowHistory(false)
    }}>
      {showHistory && <div className="fixed inset-0 z-10 bg-transparent" />}
      
      {}
      <div data-sidebar className={`shrink-0 border-r border-surface-200 dark:border-white/5 flex flex-col absolute left-0 top-0 bottom-0 z-20 bg-surface-50 dark:bg-surface-950 transition-all duration-200 ease-out ${showHistory ? 'w-64 opacity-100 translate-x-0' : 'w-0 opacity-0 -translate-x-4 pointer-events-none'}`}>
        <div className="p-3 border-b border-surface-200 dark:border-white/5 flex items-center justify-between">
          <span className="text-xs text-surface-400 font-medium">对话历史</span>
          <button onClick={newSession} className="p-1 rounded hover:bg-surface-200 dark:hover:bg-surface-800 transition-colors" title="新对话">
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
        <div className="p-3 border-t border-white/5">
          <button onClick={() => setShowKeyEditor(true)} className="w-full flex items-center gap-2 px-3 py-2 rounded-lg bg-surface-100 dark:bg-surface-800 hover:bg-surface-200 dark:hover:bg-surface-700 text-surface-600 dark:text-surface-300 text-xs transition-colors">
            <Key className="w-3.5 h-3.5" />
            {apiKeySaved ? 'API Key 已配置' : '配置 API Key'}
          </button>
        </div>
      </div>

      {}
      <div className="flex-1 flex flex-col min-w-0">
        {}
        <div className="shrink-0 px-4 py-3 border-b border-surface-200 dark:border-white/5 flex items-center gap-3">
          <button onClick={() => setShowHistory(v => !v)} className="p-2 rounded-lg bg-surface-100 dark:bg-surface-800 hover:bg-surface-200 dark:hover:bg-surface-700 transition-colors">
            <History className="w-4 h-4 text-surface-400" />
          </button>
          <div className="flex-1 min-w-0">
            <h2 className="text-sm font-medium text-surface-800 dark:text-surface-200 truncate">{sessions.find(s => s.id === activeId)?.name ?? 'AI 报错分析'}</h2>
            <p className="text-[11px] text-surface-400 dark:text-surface-500">Agnes · {MODEL}</p>
          </div>
          {}
          <div className="relative">
            <button onClick={() => { setShowKeyEditor(v => !v); setEditKey(apiKey) }}
              className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs transition-colors ${apiKeySaved ? 'bg-surface-100 dark:bg-surface-800 hover:bg-surface-200 dark:hover:bg-surface-700 text-surface-600 dark:text-surface-300' : 'bg-blue-50 dark:bg-blue-500/10 hover:bg-blue-100 dark:hover:bg-blue-500/20 text-blue-600 dark:text-blue-300'}`}>
              {apiKeySaved ? (
                <>
                  <Check className="w-3.5 h-3.5 text-emerald-500 dark:text-emerald-400" />
                  <span>API Key 已配置</span>
                </>
              ) : (
                <>
                  <Key className="w-3.5 h-3.5" />
                  <span>配置 Key</span>
                </>
              )}
            </button>
            {showKeyEditor && (
              <div className="ai-modal absolute right-0 top-full mt-2 w-72 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-xl shadow-2xl z-30 p-3 space-y-2">
                <p className="text-xs text-surface-500 dark:text-surface-400">更换 API Key</p>
                <input value={editKey} onChange={(e) => setEditKey(e.target.value)}
                  placeholder="sk-xxxxxxxxxxxxxxxx"
                  className="w-full bg-surface-50 dark:bg-surface-900 rounded-lg px-3 py-2 text-xs text-surface-800 dark:text-surface-200 placeholder-surface-400 dark:placeholder-surface-600 outline-none border border-surface-200 dark:border-white/5 focus:border-blue-400 dark:focus:border-blue-500/30 font-mono" />
                <div className="flex gap-2">
                  <button onClick={() => setShowKeyEditor(false)} className="flex-1 py-1.5 rounded-lg bg-surface-100 dark:bg-surface-700 hover:bg-surface-200 dark:hover:bg-surface-600 text-surface-600 dark:text-surface-300 text-xs transition-colors">取消</button>
                  <button onClick={applyKeyEdit} className="flex-1 py-1.5 rounded-lg bg-blue-50 dark:bg-blue-500/10 hover:bg-blue-100 dark:hover:bg-blue-500/20 text-blue-600 dark:text-blue-300 text-xs transition-colors">保存</button>
                </div>
                <p className="text-[10px] text-surface-400 dark:text-surface-500">
                  获取 Key: <button onClick={() => open('https://platform.agnes-ai.com/settings/apiKeys')} className="text-blue-500 dark:text-blue-400 underline">platform.agnes-ai.com</button>
                </p>
              </div>
            )}
          </div>
        </div>

        {}
        <div ref={messagesRef} className={`flex-1 overflow-y-auto px-4 py-3 space-y-3 ${dragOver ? 'ai-drag-over' : ''}`}
          onPaste={handlePaste}>
          {messages.map((msg) => (
            <div key={msg.id} className={`ai-msg flex gap-2.5 ${msg.role === 'user' ? 'justify-end' : ''}`}>
              {msg.role === 'assistant' && (
                <div className="w-7 h-7 rounded-lg bg-blue-50 dark:bg-blue-500/15 flex items-center justify-center shrink-0 mt-0.5">
                  <Bot className="w-4 h-4 text-blue-500 dark:text-blue-400" />
                </div>
              )}
              <div className={`group max-w-[85%] rounded-2xl px-3.5 py-2.5 text-sm leading-relaxed whitespace-pre-wrap break-words relative ${
                msg.role === 'user' ? 'bg-blue-50 dark:bg-blue-500/15 text-blue-800 dark:text-blue-100 rounded-br-md' : 'bg-surface-100 dark:bg-surface-850 text-surface-800 dark:text-surface-200 rounded-bl-md'
              }`}>
                {msg.file && msg.file.dataUrl && (
                  <div className="mb-2">
                    {msg.file.type.startsWith('image/') ? <img src={msg.file.dataUrl} alt={msg.file.name} className="max-w-full rounded-lg max-h-48 object-contain" /> :
                     msg.file.type.startsWith('video/') ? <video src={msg.file.dataUrl} controls className="max-w-full rounded-lg max-h-48" /> :
                      <div className="text-xs text-surface-500 dark:text-surface-400 font-mono bg-surface-50 dark:bg-surface-900 rounded px-2 py-1 max-h-24 overflow-y-auto">{msg.file.name}</div>}
                  </div>
                )}
                <div className="whitespace-pre-wrap">{msg.content}</div>
                <div className="absolute -bottom-5 right-0 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
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
                <span className="ai-typing-dot" />
                <span className="ai-typing-dot" />
                <span className="ai-typing-dot" />
              </div>
            </div>
          )}
        </div>

        {}
        {attachedFile && (
          <div className="shrink-0 mx-4 mb-1 bg-surface-100 dark:bg-surface-800 rounded-xl px-3 py-2 flex items-center gap-2">
            {iconForFile(attachedFile)}
            <span className="text-xs text-surface-600 dark:text-surface-300 truncate flex-1">{attachedFile.name}</span>
            <span className="text-[10px] text-surface-400 dark:text-surface-500">{(attachedFile.size / 1024).toFixed(0)} KB</span>
            <button onClick={() => setAttachedFile(null)} className="p-0.5 hover:bg-surface-200 dark:hover:bg-surface-700 rounded"><X className="w-3.5 h-3.5 text-surface-400" /></button>
          </div>
        )}

        {}
        <div className="shrink-0 px-4 py-3 border-t border-surface-200 dark:border-white/5">
          <div className="flex gap-2">
            <input type="file" ref={fileInputRef} className="hidden" accept=".log,.txt,.png,.jpg,.jpeg,.mp4,.webm"
              onChange={(e) => { const f = e.target.files?.[0]; if (f) handleFileSelect(f) }} />
            <button onClick={() => fileInputRef.current?.click()} className="p-2.5 rounded-xl bg-surface-100 dark:bg-surface-800 hover:bg-surface-200 dark:hover:bg-surface-700 text-surface-500 dark:text-surface-400 transition-colors" title="上传文件" disabled={loading}>
              <Paperclip className="w-5 h-5" />
            </button>
            <input value={input} onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && !e.shiftKey && !loading && handleSend()}
              placeholder={loading ? '正在分析中...' : '描述你遇到的问题...'}
              disabled={loading}
              className="flex-1 bg-surface-50 dark:bg-surface-850 rounded-xl px-3.5 py-2.5 text-sm text-surface-800 dark:text-surface-200 placeholder-surface-400 dark:placeholder-surface-600 outline-none border border-surface-200 dark:border-white/5 focus:border-blue-400 dark:focus:border-blue-500/30 transition-colors disabled:opacity-50" />
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
            支持上传崩溃日志 (.log/.txt)、截图、录屏 · 拖拽文件到聊天区域
          </p>
          <p className="text-[10px] text-surface-300 dark:text-surface-700 mt-1 text-center">
            AI 生成内容仅供参考，请以实际情况为准
          </p>
        </div>
      </div>

      {}

      {}
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
