import { useCallback, useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Box, Typography, Card, Button, Input, AlertBox, Chip, DividerLine } from '../components/material'
import { useAuthStore } from '../stores/authStore'
import {
  Network,
  Play,
  Crown,
  Users,
  ArrowLeft,
  Loader2,
  Copy,
  Check,
  Power,
  RefreshCw,
  Server,
  LogOut,
  Link2,
  UserRound,
  Monitor,
} from 'lucide-react'

type MemberState =
  | 'waiting'
  | 'host-scanning'
  | 'host-starting'
  | 'host-ok'
  | 'guest-connecting'
  | 'guest-input'
  | 'guest-starting'
  | 'guest-ok'
  | 'exception'

interface StateData {
  state: string
  index?: number
  room?: string
  url?: string
  type?: number
  player?: string
  profiles?: TerracottaProfile[]
  public_nodes?: string[]
}

interface TerracottaProfile {
  type?: string
  machine_id?: string
  vendor?: string
  name?: string
  player?: string
}

function profileDisplayName(p: TerracottaProfile): string {
  return p.name || p.player || p.machine_id || '未知玩家'
}

const PROFILE_TYPE_LABEL: Record<string, { label: string; cls: string }> = {
  HOST: { label: '房主', cls: 'bg-amber-50 text-amber-700 dark:bg-amber-500/10 dark:text-amber-400' },
  LOCAL: { label: '本地', cls: 'bg-accent-50 text-accent-700 dark:bg-accent-500/10 dark:text-accent-400' },
  GUEST: { label: '房客', cls: 'bg-sky-50 text-sky-700 dark:bg-sky-500/10 dark:text-sky-400' },
}

function buildMembers(stateData: StateData | null): TerracottaProfile[] {
  if (!stateData) return []
  if (Array.isArray(stateData.profiles) && stateData.profiles.length > 0) {
    return stateData.profiles
  }
  if (stateData.player) {
    return [{ type: 'LOCAL', player: stateData.player, machine_id: stateData.player }]
  }
  return []
}

const EXCEPTION_DESC: Record<number, { title: string; desc: string }> = {
  0: { title: '加入房间失败', desc: '房间已关闭或网络不稳定' },
  1: { title: '房间连接断开', desc: '房间已关闭或网络不稳定' },
  2: { title: '加入房间失败', desc: 'EasyTier 已崩溃，请向开发者反馈该问题' },
  3: { title: '创建房间失败', desc: 'EasyTier 已崩溃，请向开发者反馈该问题' },
  4: { title: '房间已关闭', desc: '您已退出游戏存档，房间已自动关闭' },
  5: { title: '协议错误', desc: '房主发送了错误的响应数据，请向开发者反馈该问题' },
}

export function Multiplayer() {
  const [port, setPort] = useState<number | null>(null)
  const [starting, setStarting] = useState(false)
  const [stopping, setStopping] = useState(false)
  const [meta, setMeta] = useState<string>('')
  const [stateData, setStateData] = useState<StateData | null>(null)
  const [memberState, setMemberState] = useState<MemberState>('waiting')
  const [inviteCode, setInviteCode] = useState('')
  const [inviteHint, setInviteHint] = useState('没有邀请码？请创建房间。')
  const [copied, setCopied] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const timerRef = useRef<number | null>(null)
  const [mode, setMode] = useState<'terracotta'>('terracotta')

  const applyState = useCallback((data: StateData) => {
    setStateData(data)
    switch (data.state) {
      case 'waiting':
        setMemberState((prev) => (prev === 'guest-input' ? 'guest-input' : 'waiting'))
        break
      case 'host-scanning':
        setMemberState('host-scanning')
        break
      case 'host-starting':
        setMemberState('host-starting')
        break
      case 'host-ok':
        setMemberState('host-ok')
        break
      case 'guest-connecting':
        setMemberState('guest-connecting')
        break
      case 'guest-starting':
        setMemberState('guest-starting')
        break
      case 'guest-ok':
        setMemberState('guest-ok')
        break
      case 'exception':
        setMemberState('exception')
        break
    }
  }, [])

  const poll = useCallback(async () => {
    try {
      const data = await invoke<StateData>('terracotta_state')
      applyState(data)
    } catch {
    }
  }, [applyState])

  const handleEnsure = useCallback(async () => {
    setStarting(true)
    setError(null)
    try {
      const p = await invoke<number>('ensure_terracotta_running')
      setPort(p)
      const m = await invoke<Record<string, unknown>>('terracotta_meta')
      setMeta(`${m.version ?? '未知版本'} · ${m.target_tuple ?? ''}`)
      await poll()
    } catch (e) {
      setError(String(e))
    } finally {
      setStarting(false)
    }
  }, [poll])

  const handleStop = useCallback(async () => {
    setStopping(true)
    try {
      await invoke('terracotta_stop')
      setPort(null)
      setStateData(null)
      setMemberState('waiting')
    } catch (e) {
      setError(String(e))
    } finally {
      setStopping(false)
    }
  }, [])

  useEffect(() => {
    handleEnsure()
    return () => {
      if (timerRef.current) {
        window.clearInterval(timerRef.current)
        timerRef.current = null
      }
    }
  }, [handleEnsure])

  useEffect(() => {
    if (!port) return
    timerRef.current = window.setInterval(poll, 1000)
    return () => {
      if (timerRef.current) {
        window.clearInterval(timerRef.current)
        timerRef.current = null
      }
    }
  }, [port, poll])

  const handleHost = useCallback(async () => {
    setError(null)
    try {
      const player = useAuthStore.getState().session?.username
      await invoke('terracotta_scanning', { player: player || null })
      setMemberState('host-scanning')
    } catch (e) {
      setError(String(e))
    }
  }, [])

  const handleGuest = useCallback(async () => {
    setError(null)
    setInviteHint('没有邀请码？请创建房间。')
    setInviteCode('')
    setMemberState('guest-input')
  }, [])

  const handleJoin = useCallback(async () => {
    if (!inviteCode.trim()) {
      setInviteHint('请输入邀请码')
      return
    }
    setError(null)
    try {
      const player = useAuthStore.getState().session?.username
      const ok = await invoke<boolean>('terracotta_guesting', { room: inviteCode.trim(), player: player || null })
      if (ok) {
        setMemberState('guest-connecting')
      } else {
        setInviteHint('邀请码格式错误')
      }
    } catch (e) {
      setError(String(e))
    }
  }, [inviteCode])

  const handleIdle = useCallback(async () => {
    setError(null)
    try {
      await invoke('terracotta_ide')
      setMemberState('waiting')
    } catch (e) {
      setError(String(e))
    }
  }, [])

  const handleBack = useCallback(() => {
    setMemberState('waiting')
    invoke('terracotta_ide').catch(() => {})
  }, [])

  const copyCode = useCallback(
    (code: string) => {
      navigator.clipboard.writeText(code).then(() => {
        setCopied(true)
        setTimeout(() => setCopied(false), 1500)
      })
    },
    []
  )

  const isHostFlow =
    memberState === 'host-scanning' ||
    memberState === 'host-starting' ||
    memberState === 'host-ok'

  return (
    <Box className="h-full flex flex-col overflow-hidden">
      {}
      <Box className="shrink-0 mb-3">
        <Typography variant="h6" className="font-bold">联机</Typography>
        <Box className="flex gap-2 mt-2">
          <button
            onClick={() => setMode('terracotta')}
            className={`px-4 py-2 rounded-xl text-sm font-medium transition-all ${
              mode === 'terracotta'
                ? 'bg-accent-500/20 text-accent-300 border border-accent-500/40'
                : 'bg-surface-800 text-surface-400 border border-white/5 hover:border-white/20'
            }`}
          >
            <span className="flex items-center gap-2">
              <Server className="w-4 h-4" />陶瓦联机
            </span>
          </button>
        </Box>
      </Box>

      {mode === 'terracotta' && (
      <Card className="shrink-0 !px-4 !py-2.5 mb-4">
        <Box className="flex items-center gap-3">
          <Box className="w-9 h-9 rounded-xl bg-accent-50 dark:bg-accent-500/10 flex items-center justify-center shrink-0">
            <Server className="w-4.5 h-4.5 text-accent-600 dark:text-accent-400" />
          </Box>
          <Box className="min-w-0 flex-1">
            <Box className="flex items-center gap-2">
              <Typography variant="body1" className="font-semibold leading-tight">陶瓦联机</Typography>
              {port ? <Chip label="运行中" color="success" size="small" /> : <Chip label="离线" color="default" size="small" />}
            </Box>
            <Typography variant="body2" color="text.secondary" className="text-xs truncate">
              {port ? `127.0.0.1:${port}` : '未运行'}
              {port && meta ? ` · ${meta}` : ''}
            </Typography>
          </Box>
          <Box className="flex items-center gap-2 shrink-0">
            {!port ? (
              <Button
                variant="contained"
                size="small"
                loading={starting}
                startIcon={starting ? <Loader2 className="w-4 h-4 animate-spin" /> : <Play className="w-4 h-4" />}
                onClick={handleEnsure}
              >
                启动联机服务
              </Button>
            ) : (
              <>
                <Button variant="ghost" size="small" startIcon={<RefreshCw className="w-4 h-4" />} onClick={poll}>
                  刷新状态
                </Button>
                <Button
                  variant="outlined"
                  size="small"
                  color="error"
                  loading={stopping}
                  startIcon={stopping ? <Loader2 className="w-4 h-4 animate-spin" /> : <Power className="w-4 h-4" />}
                  onClick={handleStop}
                >
                  停止服务
                </Button>
              </>
            )}
          </Box>
        </Box>
      </Card>
      )}

      {mode === 'terracotta' && (
      <Box className="flex-1 flex min-h-0">
        <main className="flex-1 min-w-0 overflow-y-auto flex flex-col">
          <Box className="flex-1 flex flex-col items-center justify-center py-4 min-h-full">
            {!port ? (
              <OfflineView starting={starting || !meta} onStart={handleEnsure} error={error} onClose={() => setError(null)} />
            ) : !stateData ? (
              <Box className="flex items-center justify-center h-48">
                <Loader2 className="w-8 h-8 text-accent-400 animate-spin" />
              </Box>
            ) : (
              <Box className="w-full max-w-2xl flex flex-col items-center gap-4">
                {memberState === 'waiting' && (
                  <RoleSelect onHost={handleHost} onGuest={handleGuest} isHostFlow={isHostFlow} />
                )}
                {memberState === 'guest-input' && (
                  <GuestInputCard
                    value={inviteCode}
                    onChange={setInviteCode}
                    hint={inviteHint}
                    onJoin={handleJoin}
                    onBack={handleBack}
                  />
                )}
                {(memberState === 'host-scanning' ||
                  memberState === 'host-starting' ||
                  memberState === 'guest-connecting' ||
                  memberState === 'guest-starting') && (
                  <LoadingCard
                    title={
                      memberState === 'host-scanning'
                        ? '正在等待局域网开放…'
                        : memberState === 'host-starting'
                          ? '正在创建房间…'
                          : '正在加入房间…'
                    }
                    desc={
                      memberState === 'host-scanning'
                        ? '进入单人存档后按 ESC，选择「对局域网开放」并创建局域网世界'
                        : memberState === 'host-starting'
                          ? '正在连接 EasyTier 网络并生成邀请码'
                          : '正在连接 EasyTier 网络，请稍候'
                    }
                    onBack={handleBack}
                  />
                )}
                {memberState === 'host-ok' && (
                  <HostResultCard
                    room={stateData.room ?? ''}
                    copied={copied}
                    onCopy={copyCode}
                    onClose={handleIdle}
                    members={buildMembers(stateData)}
                    selfName={useAuthStore.getState().session?.username ?? ''}
                  />
                )}
                {memberState === 'guest-ok' && (
                  <GuestResultCard
                    url={stateData.url ?? ''}
                    onExit={handleIdle}
                    members={buildMembers(stateData)}
                    selfName={useAuthStore.getState().session?.username ?? ''}
                  />
                )}
                {memberState === 'exception' && (
                  <ErrorCard info={EXCEPTION_DESC[stateData.type ?? 0] ?? EXCEPTION_DESC[0]} onBack={handleBack} />
                )}
                {error && <AlertBox severity="error" onClose={() => setError(null)} className="w-full">{error}</AlertBox>}
              </Box>
            )}
          </Box>
        </main>
      </Box>
      )}
    </Box>
  )
}


function OfflineView({
  starting,
  onStart,
  error,
  onClose,
}: {
  starting: boolean
  onStart: () => void
  error: string | null
  onClose: () => void
}) {
  return (
    <Card className="flex flex-col items-center justify-center text-center px-8 py-14 w-full max-w-2xl">
      <Box className="w-20 h-20 rounded-2xl bg-accent-50 dark:bg-accent-500/10 flex items-center justify-center mx-auto mb-5">
        <Network className="w-9 h-9 text-accent-600 dark:text-accent-400" />
      </Box>
      <Typography variant="h5" className="font-bold mb-2">陶瓦联机</Typography>
      <Typography variant="body2" color="text.secondary" className="mb-7 max-w-sm leading-relaxed">
        打开后即可与好友跨网络联机，无需公网 IP、无需端口映射。
        <br />
        服务在后台运行，不会弹出浏览器窗口。
      </Typography>
      <Button
        size="large"
        variant="contained"
        loading={starting}
        startIcon={starting ? <Loader2 className="w-4 h-4 animate-spin" /> : <Play className="w-4 h-4" />}
        onClick={onStart}
      >
        开始使用
      </Button>
      {error && <AlertBox severity="error" onClose={onClose} className="mt-5 w-full">{error}</AlertBox>}
    </Card>
  )
}

function RoleSelect({ onHost, onGuest, isHostFlow }: { onHost: () => void; onGuest: () => void; isHostFlow: boolean }) {
  return (
    <Box className="w-full">
      <Box className="text-center mb-5">
        <Typography variant="h6" className="font-bold">你想扮演什么角色？</Typography>
        <Typography variant="body2" color="text.secondary" className="mt-1 text-sm">
          房主创建房间生成邀请码，房客凭邀请码加入
        </Typography>
      </Box>
      <Box className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <Card hoverable className="!p-6 text-center cursor-pointer" onClick={onHost}>
          <Box className="w-16 h-16 rounded-2xl bg-amber-50 dark:bg-amber-500/10 flex items-center justify-center mx-auto mb-4">
            <Crown className="w-8 h-8 text-amber-500" />
          </Box>
          <Box className="flex items-center justify-center gap-1.5 mb-1.5">
            <Typography variant="subtitle1" className="font-semibold">我想当房主</Typography>
            {isHostFlow && <Chip label="当前" color="warning" size="small" />}
          </Box>
          <Typography variant="body2" color="text.secondary" className="text-[13px] leading-relaxed">
            创建房间并生成邀请码
            <br />
            与好友一起畅玩
          </Typography>
          <Button variant="contained" color="warning" size="medium" className="!mt-4" startIcon={<Crown className="w-4 h-4" />} onClick={onHost}>
            创建房间
          </Button>
        </Card>
        <Card hoverable className="!p-6 text-center cursor-pointer" onClick={onGuest}>
          <Box className="w-16 h-16 rounded-2xl bg-sky-50 dark:bg-sky-500/10 flex items-center justify-center mx-auto mb-4">
            <Users className="w-8 h-8 text-sky-500" />
          </Box>
          <Box className="flex items-center justify-center gap-1.5 mb-1.5">
            <Typography variant="subtitle1" className="font-semibold">我想当房客</Typography>
          </Box>
          <Typography variant="body2" color="text.secondary" className="text-[13px] leading-relaxed">
            输入房主提供的邀请码
            <br />
            加入游戏世界
          </Typography>
          <Button variant="contained" color="info" size="medium" className="!mt-4" startIcon={<Users className="w-4 h-4" />} onClick={onGuest}>
            加入房间
          </Button>
        </Card>
      </Box>
    </Box>
  )
}

function GuestInputCard({
  value,
  onChange,
  hint,
  onJoin,
  onBack,
}: {
  value: string
  onChange: (v: string) => void
  hint: string
  onJoin: () => void
  onBack: () => void
}) {
  const isError = hint === '邀请码格式错误' || hint === '请输入邀请码'
  return (
    <Card className="w-full !p-8 text-center">
      <Box className="w-14 h-14 rounded-2xl bg-sky-50 dark:bg-sky-500/10 flex items-center justify-center mx-auto mb-4">
        <Link2 className="w-6 h-6 text-sky-500" />
      </Box>
      <Typography variant="h6" className="font-bold mb-1">输入邀请码</Typography>
      <Typography variant="body2" color="text.secondary" className="mb-6 text-sm">
        向房主索取邀请码，格式如 U/XXXX-XXXX-XXXX-XXXX
      </Typography>
      <Box className="max-w-sm mx-auto mb-3">
        <Input
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') onJoin()
          }}
          placeholder="U/XXXX-XXXX-XXXX-XXXX"
          size="medium"
          className="text-center tracking-[0.12em]"
          error={isError}
          helperText={hint}
        />
      </Box>
      <Box className="flex items-center justify-center gap-2">
        <Button variant="contained" color="info" startIcon={<Users className="w-4 h-4" />} onClick={onJoin}>
          加入房间
        </Button>
        <Button variant="outlined" startIcon={<ArrowLeft className="w-4 h-4" />} onClick={onBack}>
          返回
        </Button>
      </Box>
    </Card>
  )
}

function LoadingCard({ title, desc, onBack }: { title: string; desc: string; onBack: () => void }) {
  return (
    <Card className="w-full !p-10 text-center">
      <Box className="relative w-20 h-20 mx-auto mb-6">
        <Box className="absolute inset-0 rounded-2xl bg-accent-50 dark:bg-accent-500/10 animate-pulse" />
        <Box className="absolute inset-0 flex items-center justify-center">
          <Loader2 className="w-8 h-8 text-accent-500 animate-spin" />
        </Box>
      </Box>
      <Typography variant="h6" className="font-bold mb-2">{title}</Typography>
      <Typography variant="body2" color="text.secondary" className="mb-7 max-w-sm mx-auto leading-relaxed">{desc}</Typography>
      <Button variant="outlined" startIcon={<ArrowLeft className="w-4 h-4" />} onClick={onBack}>
        返回
      </Button>
    </Card>
  )
}

function HostResultCard({
  room,
  copied,
  onCopy,
  onClose,
  members,
  selfName,
}: {
  room: string
  copied: boolean
  onCopy: (code: string) => void
  onClose: () => void
  members: TerracottaProfile[]
  selfName: string
}) {
  return (
    <Card className="w-full !p-8 text-center">
      <Box className="w-14 h-14 rounded-2xl bg-green-50 dark:bg-green-500/10 flex items-center justify-center mx-auto mb-4">
        <Crown className="w-7 h-7 text-green-600 dark:text-green-400" />
      </Box>
      <Typography variant="h6" className="font-bold mb-1">成功创建房间</Typography>
      <Typography variant="body2" color="text.secondary" className="mb-6 text-sm">
        把邀请码发给好友，对方选择「我想当房客」即可加入
      </Typography>

      <Box
        className="flex items-center justify-between gap-3 bg-accent-50 dark:bg-accent-500/10 border-2 border-dashed border-accent-300 dark:border-accent-500/40 rounded-2xl px-5 py-4 mb-6 cursor-pointer select-all hover:bg-accent-100 dark:hover:bg-accent-500/20 transition-colors"
        onClick={() => onCopy(room)}
        title="点击复制"
      >
        <Typography className="font-mono font-bold tracking-[0.15em] text-accent-700 dark:text-accent-300 text-sm break-all">
          {room}
        </Typography>
        <Box className="shrink-0">
          {copied ? <Check className="w-5 h-5 text-green-500" /> : <Copy className="w-5 h-5 text-accent-500" />}
        </Box>
      </Box>

      <MembersCard members={members} selfName={selfName} emptyHint="等待好友加入…" />

      <DividerLine className="mb-5" />
      <Typography variant="body2" color="text.secondary" className="text-[13px] leading-relaxed mb-6">
        请保持陶瓦联机运行，等待好友加入。
        <br />
        好友进入世界后即可一起联机，关闭房间将断开所有连接。
      </Typography>
      <Button variant="outlined" color="error" startIcon={<LogOut className="w-4 h-4" />} onClick={onClose}>
        关闭房间
      </Button>
    </Card>
  )
}

function GuestResultCard({ url, onExit, members, selfName }: { url: string; onExit: () => void; members: TerracottaProfile[]; selfName: string }) {
  return (
    <Card className="w-full !p-8 text-center">
      <Box className="w-14 h-14 rounded-2xl bg-green-50 dark:bg-green-500/10 flex items-center justify-center mx-auto mb-4">
        <Check className="w-7 h-7 text-green-600 dark:text-green-400" />
      </Box>
      <Typography variant="h6" className="font-bold mb-1">成功加入房间</Typography>
      <Typography variant="body2" color="text.secondary" className="mb-6 text-sm">
        启动 Minecraft，选择多人游戏，双击进入陶瓦联机大厅
      </Typography>

      <Box className="bg-surface-100 dark:bg-surface-800 rounded-2xl px-5 py-4 mb-2 select-all">
        <Typography className="font-mono font-semibold text-surface-700 dark:text-surface-200 text-sm break-all">
          {url || '127.0.0.1'}
        </Typography>
      </Box>
      <Typography variant="caption" color="text.secondary" className="block text-[11px] mb-6">
        备用联机地址 · 双击进入的服务器地址
      </Typography>

      <MembersCard members={members} selfName={selfName} emptyHint="暂无其他用户加入" />

      <Button variant="outlined" startIcon={<LogOut className="w-4 h-4" />} onClick={onExit}>
        退出房间
      </Button>
    </Card>
  )
}

function MembersCard({
  members,
  selfName,
  emptyHint,
}: {
  members: TerracottaProfile[]
  selfName: string
  emptyHint: string
}) {
  return (
    <Box className="bg-surface-100 dark:bg-surface-800 rounded-2xl px-5 py-4 mb-6 text-left">
      <Box className="flex items-center gap-2 mb-3">
        <Users className="w-4 h-4 text-accent-600 dark:text-accent-400" />
        <Typography variant="subtitle2" className="font-semibold">当前加入的用户</Typography>
        {members.length > 0 && (
          <Chip label={`${members.length} 人`} size="small" color="primary" variant="outlined" />
        )}
      </Box>
      {members.length === 0 ? (
        <Typography variant="body2" color="text.secondary" className="text-xs">
          {emptyHint}
        </Typography>
      ) : (
        <Box className="space-y-2">
          {members.map((m, i) => {
            const name = profileDisplayName(m)
            const typeMeta = PROFILE_TYPE_LABEL[m.type ?? ''] ?? null
            const isSelf = selfName && name === selfName
            return (
              <Box key={`${name}-${i}`} className="flex items-center gap-3 bg-white dark:bg-surface-850 rounded-xl px-3 py-2">
                <Box
                  className={`w-8 h-8 rounded-lg flex items-center justify-center shrink-0 ${
                    m.type === 'HOST' ? 'bg-amber-50 dark:bg-amber-500/10' : 'bg-accent-50 dark:bg-accent-500/10'
                  }`}
                >
                  {m.type === 'HOST' ? (
                    <Crown className="w-4 h-4 text-amber-500" />
                  ) : m.type === 'LOCAL' ? (
                    <Monitor className="w-4 h-4 text-accent-500" />
                  ) : (
                    <UserRound className="w-4 h-4 text-sky-500" />
                  )}
                </Box>
                <Box className="min-w-0 flex-1">
                  <Box className="flex items-center gap-1.5">
                    <Typography variant="body2" className="font-medium truncate">{name}</Typography>
                    {isSelf && <Chip label="你" size="small" color="success" />}
                  </Box>
                  {(m.machine_id || m.vendor) && (
                    <Typography variant="caption" color="text.secondary" className="text-[11px]">
                      {[m.vendor, m.machine_id && m.machine_id !== name ? m.machine_id : ''].filter(Boolean).join(' · ')}
                    </Typography>
                  )}
                </Box>
                {typeMeta && (
                  <span className={`px-1.5 py-0.5 rounded text-[10px] font-medium shrink-0 ${typeMeta.cls}`}>
                    {typeMeta.label}
                  </span>
                )}
              </Box>
            )
          })}
        </Box>
      )}
    </Box>
  )
}

function ErrorCard({
  info,
  onBack,
}: {
  info: { title: string; desc: string }
  onBack: () => void
}) {
  return (
    <Card className="w-full !p-8 text-center">
      <Box className="w-14 h-14 rounded-2xl bg-red-50 dark:bg-red-500/10 flex items-center justify-center mx-auto mb-4">
        <Network className="w-7 h-7 text-red-500" />
      </Box>
      <Typography variant="h6" className="font-bold mb-1">{info.title}</Typography>
      <Typography variant="body2" color="text.secondary" className="mb-7 text-sm">{info.desc}</Typography>
      <Button variant="outlined" startIcon={<ArrowLeft className="w-4 h-4" />} onClick={onBack}>
        返回主页
      </Button>
    </Card>
  )
}
