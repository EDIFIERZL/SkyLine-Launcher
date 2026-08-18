import { useRef, useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import { useAuthStore } from '../stores/authStore'
import { Box, Typography, Card, Button, Input, Chip, Tabs, DialogBox } from '@/components/material'
import type { AuthSession, MicrosoftDeviceCode, LittleSkinDeviceCode } from '../types'
import { LogOut, Gamepad2, Shirt, UserRound, RotateCw, Trash, Globe, ExternalLink, Users, Copy, Check, QrCode } from 'lucide-react'
import { SkinAvatar } from '../components/SkinAvatar'
import { SkinViewer3D } from '../components/SkinViewer3D'
import type { SavedAccount } from '../stores/authStore'

function readFileAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = () => reject(new Error('读取文件失败'))
    reader.readAsDataURL(file)
  })
}

type LoginMode = 'offline' | 'microsoft' | 'third'

const LITTLE_SKIN_YGGDRASIL = 'https://littleskin.cn/api/yggdrasil'

const LOGIN_TABS = [
  { value: 'microsoft', label: 'Microsoft账号', icon: <Globe className="w-4 h-4" /> },
  { value: 'offline', label: '离线模式', icon: <Gamepad2 className="w-4 h-4" /> },
  { value: 'third', label: 'Little Skin', icon: <QrCode className="w-4 h-4" /> },
]

export function Account() {
  const { session, cosmetics, setSession, clearSession, setSkin, setCape, accounts, saveAccount, removeAccount } = useAuthStore()
  const [username, setUsername] = useState('')
  const [loading, setLoading] = useState(false)
  const [saving, setSaving] = useState(false)
  const [loginMode, setLoginMode] = useState<LoginMode>('microsoft')
  const skinInputRef = useRef<HTMLInputElement>(null)
  const capeInputRef = useRef<HTMLInputElement>(null)
  const [skinPreview, setSkinPreview] = useState<string | null>(null)
  const [capePreview, setCapePreview] = useState<string | null>(null)
  const [deviceCode, setDeviceCode] = useState<MicrosoftDeviceCode | null>(null)
  const [msWaiting, setMsWaiting] = useState(false)
  const [countdown, setCountdown] = useState(0)
  const [copied, setCopied] = useState(false)
  const msPollRef = useRef<AbortController | null>(null)
  const lsPollRef = useRef<AbortController | null>(null)
  const [lsDeviceInfo, setLsDeviceInfo] = useState<LittleSkinDeviceCode | null>(null)
  const [lsPolling, setLsPolling] = useState(false)

  const isOffline = session?.user_type === 'offline'
  const isAuthlib = session?.user_type === 'authlib'
  const myCosmetics = session ? cosmetics[session.uuid] : undefined
  const currentSkin = skinPreview ?? myCosmetics?.skin ?? null
  const currentCape = capePreview ?? myCosmetics?.cape ?? null

  useEffect(() => {
    if (!session?.refresh_token) return
    if (session.user_type === 'msa') {
      const timer = setInterval(() => {
        invoke<AuthSession>('microsoft_auth_refresh', { refreshToken: session.refresh_token })
          .then((auth) => { setSession(auth); saveAccount(auth) })
          .catch(console.error)
      }, 2 * 60 * 1000)
      return () => clearInterval(timer)
    }
    if (session.user_type === 'authlib') {
      const timer = setInterval(() => {
        invoke<AuthSession>('littleskin_auth_refresh', { refreshToken: session.refresh_token })
          .then((auth) => { setSession(auth); saveAccount(auth) })
          .catch(console.error)
      }, 10 * 60 * 1000)
      return () => clearInterval(timer)
    }
  }, [session?.refresh_token, session?.user_type])

  const handleOfflineLogin = async () => {
    if (!username.trim()) return
    setLoading(true)
    try {
      const auth = await invoke<AuthSession>('login_offline', { username })
      setSession(auth)
      saveAccount(auth)
    } catch (e) { console.error(e) }
    setLoading(false)
  }

  const handleMicrosoftLogin = async () => {
    setLoading(true)
    try {
      const code = await invoke<MicrosoftDeviceCode>('microsoft_auth_start')
      setDeviceCode(code)
      setMsWaiting(false)
      setCountdown(code.expires_in)
      setCopied(false)
      
      const verifyUrl = code.verification_uri_complete ?? (code.verification_uri ? `${code.verification_uri}?code=${code.user_code}` : null) ?? 'https://login.microsoftonline.com/common/oauth2/v2.0/authorize'
      open(verifyUrl).catch(() => {
        
        const urlToCopy = code.verification_uri_complete ?? verifyUrl
        navigator.clipboard.writeText(urlToCopy).catch(() => navigator.clipboard.writeText(code.user_code).catch(() => {}))
      })
    } catch (e) {
      console.error(e)
      const msg = String(e)
      if (msg.includes('first party') || msg.includes('invalid_request') || msg.includes('does not have consent')) {
        alert('微软授权出错：Azure AD 应用需要先授予 XboxLive 权限的管理员同意。\n\n请前往 Azure Portal → 应用注册 → API 权限 → 点击"授予管理员同意"按钮后重试。')
      } else {
        alert(`获取设备码失败: ${msg}`)
      }
    }
    setLoading(false)
  }

  const handleMicrosoftCancel = () => {
    msPollRef.current?.abort()
    msPollRef.current = null
    setDeviceCode(null)
    setMsWaiting(false)
    setCountdown(0)
  }

  const handleCopyCode = async () => {
    if (!deviceCode) return
    try {
      await navigator.clipboard.writeText(deviceCode.user_code)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch {
      
    }
  }

  const startPolling = useCallback((code: MicrosoftDeviceCode) => {
    msPollRef.current?.abort()
    const controller = new AbortController()
    msPollRef.current = controller
    setMsWaiting(true)

    const poll = async () => {
      try {
        const auth = await invoke<AuthSession>('microsoft_auth_poll', { info: code }, { signal: controller.signal } as any)
        if (controller.signal.aborted) return
        setSession(auth)
        saveAccount(auth)
        setDeviceCode(null)
        setMsWaiting(false)
        setCountdown(0)
      } catch (e: any) {
        if (controller.signal.aborted) return
        const msg = String(e)
        if (msg.includes('authorization_pending') || msg.includes('slow_down')) {
          const delay = code.interval * 1000
          setTimeout(poll, delay)
          return
        }
        setDeviceCode(null)
        setMsWaiting(false)
        setCountdown(0)
        if (msg.includes('first party') || msg.includes('does not have consent')) {
          alert('微软授权页面提示"first party application"错误。\n\n这是因为 Azure AD 应用未授予 XboxLive.signin 权限的管理员同意。\n\n解决方法：请前往 Azure Portal → 应用注册 → 选择你的应用 → API 权限 → 点击"为 [租户] 授予管理员同意"按钮。')
        } else if (!msg.includes('denied') && !msg.includes('expired') && !msg.includes('cancelled')) {
          alert(`登录失败: ${msg}`)
        }
      }
    }
    setTimeout(poll, code.interval * 1000)
  }, [setSession, saveAccount])

  useEffect(() => {
    if (deviceCode && !msWaiting) {
      startPolling(deviceCode)
    }
    return () => { msPollRef.current?.abort() }
  }, [deviceCode])

  
  
  useEffect(() => {
    const onVisibilityChange = () => {
      if (document.visibilityState === 'hidden' && deviceCode && msWaiting) {
        handleMicrosoftCancel()
      }
    }
    document.addEventListener('visibilitychange', onVisibilityChange)
    return () => document.removeEventListener('visibilitychange', onVisibilityChange)
  }, [deviceCode, msWaiting])

  
  useEffect(() => {
    if (!deviceCode || msWaiting) return
    const timer = setTimeout(() => {
      if (deviceCode && msWaiting) handleMicrosoftCancel()
    }, 90_000)
    return () => clearTimeout(timer)
  }, [deviceCode, msWaiting])

  useEffect(() => {
    if (countdown <= 0) return
    const timer = setInterval(() => {
      setCountdown((prev) => {
        if (prev <= 1) {
          clearInterval(timer)
          msPollRef.current?.abort()
          setDeviceCode(null)
          setMsWaiting(false)
          return 0
        }
        return prev - 1
      })
    }, 1000)
    return () => clearInterval(timer)
  }, [countdown > 0])

  const formatCountdown = (s: number) => {
    const m = Math.floor(s / 60)
    const sec = s % 60
    return `${m}:${sec.toString().padStart(2, '0')}`
  }

  const handleMicrosoftRefresh = async () => {
    if (!session?.refresh_token) return
    setLoading(true)
    try {
      const auth = await invoke<AuthSession>('microsoft_auth_refresh', { refreshToken: session.refresh_token })
      setSession(auth)
      saveAccount(auth)
    } catch (e) {
      console.error(e)
      alert(`刷新登录失败: ${e}`)
    }
    setLoading(false)
  }

  const handleLittleSkinStart = async () => {
    setLoading(true)
    try {
      const code = await invoke<LittleSkinDeviceCode>('littleskin_auth_status')
      setLsDeviceInfo(code)
      navigator.clipboard.writeText(code.user_code).catch(() => {})
      setLsPolling(true)
      startLsPolling(code)

      const verifyUrl = code.verification_uri_complete ?? (code.verification_uri ? `${code.verification_uri}?code=${code.user_code}` : null)
      if (verifyUrl) {
        open(verifyUrl).catch(() => {})
      }
    } catch (e) {
      console.error(e)
      const msg = String(e)
      if (msg.includes('invalid_scope')) {
        alert('LittleSkin 授权失败：当前申请的白名单缺少 Yggdrasil 相关权限。\n\n请前往 LittleSkin 管理页，将回调 URL 设为 https://open.littleskin.cn/oauth/callback，并发送邮件工单申请补充权限：\nYggdrasil.PlayerProfiles.Select\nYggdrasil.MinecraftToken.Create\nYggdrasil.Server.Join\n\n需要这些权限才能为启动器签发可用于外置登录的 Minecraft 令牌。')
      } else if (msg.includes('invalid_client')) {
        alert('LittleSkin 授权失败：应用未通过设备代码流白名单校验。\n\n请确认已在 LittleSkin「OAuth 2 应用」页面将回调 URL 设为 https://open.littleskin.cn/oauth/callback，并发送邮件工单申请设备代码流白名单。')
      } else {
        alert(`登录失败: ${msg}`)
      }
    }
    setLoading(false)
  }

  const startLsPolling = (code: LittleSkinDeviceCode) => {
    lsPollRef.current?.abort()
    const controller = new AbortController()
    lsPollRef.current = controller

    const poll = async () => {
      if (controller.signal.aborted) return
      try {
        const auth = await invoke<AuthSession>('littleskin_auth_poll', { info: code }, { signal: controller.signal } as any)
        if (controller.signal.aborted) return
        setLsPolling(false)
        setLsDeviceInfo(null)
        setSession(auth)
        saveAccount(auth)
      } catch (e: any) {
        if (controller.signal.aborted) return
        const msg = String(e)
        if (msg.includes('authorization_pending') || msg.includes('slow_down')) {
          setTimeout(poll, (code.interval * 1000) || 5000)
          return
        }
        setLsPolling(false)
        setLsDeviceInfo(null)
        if (msg.includes('invalid_scope')) {
          alert('LittleSkin 授权失败：当前申请的白名单缺少 Yggdrasil 相关权限。\n\n请发送邮件工单申请补充：\nYggdrasil.PlayerProfiles.Select\nYggdrasil.MinecraftToken.Create\nYggdrasil.Server.Join')
        } else if (!msg.includes('expired') && !msg.includes('拒绝') && !msg.includes('cancelled')) {
          alert(`登录失败: ${msg}`)
        }
      }
    }
    setTimeout(poll, (code.interval * 1000) || 5000)
  }

  const handleLittleSkinCancel = () => {
    lsPollRef.current?.abort()
    lsPollRef.current = null
    setLsPolling(false)
    setLsDeviceInfo(null)
  }

  const handleLogout = () => { clearSession(); setLoginMode('microsoft') }
  const [switchOpen, setSwitchOpen] = useState(false)
  const handleSwitch = () => setSwitchOpen(true)
  const handleSwitchAccount = (account: SavedAccount) => {
    
    msPollRef.current?.abort()
    lsPollRef.current?.abort()
    msPollRef.current = null
    lsPollRef.current = null
    setDeviceCode(null)
    setMsWaiting(false)
    setCountdown(0)
    setSession(account.session)
    setSwitchOpen(false)
  }
  const handleSwitchNew = () => {
    setSwitchOpen(false)
    setSession(null)
    setLoginMode('microsoft')
  }
  const handleDeleteAccount = (account: SavedAccount) => {
    if (!confirm(`确定删除账户 ${account.username} 吗？`)) return
    removeAccount(account.id)
    if (session && (session.uuid === account.id || (account.id === account.username && session.username === account.username))) {
      clearSession()
    }
  }

  const accountTypeLabel = (t: string) =>
    t === 'msa' ? 'Microsoft账号' :
    t === 'mojang' ? 'Mojang' :
    t === 'authlib' ? 'Little Skin' : '离线'

  const pickSkin = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file || !session) return
    const dataUrl = await readFileAsDataUrl(file)
    setSkinPreview(dataUrl)
    e.target.value = ''
  }

  const pickCape = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file || !session) return
    const dataUrl = await readFileAsDataUrl(file)
    setCapePreview(dataUrl)
    e.target.value = ''
  }

  const handleSaveCosmetics = async () => {
    if (!session) return
    setSaving(true)
    try {
      const stripPrefix = (b64: string | null | undefined) =>
        b64 ? b64.replace(/^data:[^;]+;base64,/, '') : null
      const skinB64 = stripPrefix(skinPreview ?? myCosmetics?.skin ?? null)
      const capeB64 = stripPrefix(capePreview ?? myCosmetics?.cape ?? null)
      await invoke('save_custom_skin', {
        accountUuid: session.uuid,
        skinB64,
        capeB64,
      })
      setSkin(session.uuid, skinPreview ?? myCosmetics?.skin ?? null)
      setCape(session.uuid, capePreview ?? myCosmetics?.cape ?? null)
      setSkinPreview(null)
      setCapePreview(null)
    } catch (e) { console.error(e) }
    setSaving(false)
  }

  const clearSkin = () => {
    setSkinPreview('')
    setSkin(session!.uuid, null)
  }
  const clearCape = () => {
    setCapePreview('')
    setCape(session!.uuid, null)
  }

  if (session) {
    return (
      <div className="flex gap-4 items-stretch h-full min-h-0">
        {}
        <div className="w-96 shrink-0 flex flex-col gap-4 pr-1 min-h-0 pb-2">
          <div className="overflow-y-auto overflow-x-hidden flex-1 min-h-0 pr-1 space-y-4">
            <div>
              <Typography variant="h5">账户管理</Typography>
            </div>

          <Card>
            <Box className="space-y-4">
              <Box className="flex items-center gap-4">
                <SkinAvatar size={64} />
                <Box>
                   <Typography variant="h6" className="font-bold tracking-wide">{session.username?.trim() || `玩家-${session.uuid.slice(0, 6)}`}</Typography>
                  <Box className="flex items-center gap-2 mt-1">
                    <Chip label={accountTypeLabel(session.user_type)} size="small" variant="outlined" />
                  </Box>
                </Box>
              </Box>
              <Box className="space-y-1 text-xs text-surface-500">
                <Typography variant="caption" className="break-all block">UUID: {session.uuid}</Typography>
                <Typography variant="body2" color="text.secondary" className="text-xs">当前登录状态已保存，切换账户后仍会保留。</Typography>
              </Box>
               <Box className="flex flex-wrap gap-2 pt-3 border-t border-surface-200 dark:border-surface-700">
                 {session.user_type === 'msa' && session.refresh_token && (
                   <Button variant="outlined" startIcon={<RotateCw className="w-4 h-4" />} onClick={handleMicrosoftRefresh} loading={loading} className="flex-1 min-w-0 basis-[110px]">
                     <span className="whitespace-nowrap">刷新登录</span>
                   </Button>
                 )}
                 <Button variant="outlined" startIcon={<RotateCw className="w-4 h-4" />} onClick={handleSwitch} className="flex-1 min-w-0 basis-[110px]">
                   <span className="whitespace-nowrap">切换账户</span>
                 </Button>
                 <Button variant="outlined" color="error" startIcon={<LogOut className="w-4 h-4" />} onClick={handleLogout} className="flex-1 min-w-0 basis-[110px]">
                   <span className="whitespace-nowrap">退出登录</span>
                 </Button>
               </Box>
            </Box>
          </Card>

          {(isOffline || isAuthlib) && (
            <Card>
              <Box className="space-y-4">
                <Typography variant="subtitle1" className="flex items-center gap-2">
                  <Shirt className="w-4 h-4 text-[var(--accent-color)]" /> 皮肤与披风
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {isOffline
                    ? '离线模式下的皮肤与披风仅在本启动器内显示，联机时需服务器安装皮肤类模组才能被他人看到。'
                    : '第三方皮肤站的皮肤将自动同步展示。'}
                </Typography>
                <Box className="flex gap-4">
                  <Box className="flex-1 flex flex-col items-center gap-2">
                    <Box className="w-20 h-20 rounded-xl bg-accent-50 dark:bg-accent-500/10 border border-surface-200 dark:border-surface-800 overflow-hidden flex items-center justify-center">
                      {currentSkin ? (
                        <img src={currentSkin} className="w-full h-full object-contain" alt="皮肤预览" />
                      ) : (
                        <UserRound className="w-8 h-8 text-surface-300 dark:text-surface-600" />
                      )}
                    </Box>
                    <Box className="flex gap-1.5">
                      <Button size="small" variant="outlined" onClick={() => skinInputRef.current?.click()}>
                        {currentSkin ? '更换' : '选择'}
                      </Button>
                      {currentSkin && (
                        <Button size="small" variant="outlined" color="error" onClick={clearSkin} startIcon={<Trash className="w-3.5 h-3.5" />}>
                          清除
                        </Button>
                      )}
                    </Box>
                    <Typography variant="caption" color="text.secondary">皮肤 (64x32)</Typography>
                  </Box>
                  <Box className="flex-1 flex flex-col items-center gap-2">
                    <Box className="w-20 h-20 rounded-xl bg-accent-50 dark:bg-accent-500/10 border border-surface-200 dark:border-surface-800 overflow-hidden flex items-center justify-center">
                      {currentCape ? (
                        <img src={currentCape} className="w-full h-full object-contain" alt="披风预览" />
                      ) : (
                        <Shirt className="w-8 h-8 text-surface-300 dark:text-surface-600" />
                      )}
                    </Box>
                    <Box className="flex gap-1.5">
                      <Button size="small" variant="outlined" onClick={() => capeInputRef.current?.click()}>
                        {currentCape ? '更换' : '选择'}
                      </Button>
                      {currentCape && (
                        <Button size="small" variant="outlined" color="error" onClick={clearCape} startIcon={<Trash className="w-3.5 h-3.5" />}>
                          清除
                        </Button>
                      )}
                    </Box>
                    <Typography variant="caption" color="text.secondary">披风 (64x32)</Typography>
                  </Box>
                </Box>
                {(skinPreview || capePreview) && (
                  <Button onClick={handleSaveCosmetics} loading={saving} fullWidth>
                    保存并应用
                  </Button>
                )}
                <input ref={skinInputRef} type="file" accept="image/png,image/jpeg" className="hidden" onChange={pickSkin} />
                <input ref={capeInputRef} type="file" accept="image/png,image/jpeg" className="hidden" onChange={pickCape} />
              </Box>
            </Card>
          )}

          {accounts.length > 0 && (
            <Card>
              <Box className="space-y-3">
                <Typography variant="subtitle1" className="flex items-center gap-2">
                  <Users className="w-4 h-4 text-[var(--accent-color)]" /> 已保存的账户 ({accounts.length})
                </Typography>
                <Box className="space-y-2 max-h-60 overflow-y-auto pr-1">
                  {accounts.map((account) => {
                    const isCurrent = session.uuid === account.id || session.username === account.username
                    return (
                      <Box
                        key={account.id}
                        className={`flex items-center justify-between px-4 py-2.5 rounded-lg border transition-colors ${
                          isCurrent
                            ? 'bg-accent-50 dark:bg-accent-500/10 border-accent-200 dark:border-accent-500/30'
                            : 'bg-surface-50 dark:bg-surface-800 border-surface-200 dark:border-surface-700'
                        }`}
                      >
                        <button
                          className="flex items-center gap-3 min-w-0 text-left flex-1 cursor-pointer"
                          onClick={() => handleSwitchAccount(account)}
                        >
                          <SkinAvatar size={32} uuid={account.session.uuid} username={account.username} userType={account.user_type} />
                          <Box className="min-w-0">
                            <Typography variant="subtitle2" className="truncate">{account.username}</Typography>
                            <Typography variant="caption" color="text.secondary">{accountTypeLabel(account.user_type)}</Typography>
                          </Box>
                          {isCurrent && <Chip label="当前" size="small" color="primary" className="ml-2" />}
                        </button>
                        <Button size="small" variant="text" onClick={() => handleDeleteAccount(account)}>
                          <Trash className="w-4 h-4 text-red-400" />
                        </Button>
                      </Box>
                    )
                  })}
                </Box>
              </Box>
            </Card>
          )}
          </div>
        </div>

        {}
        <div className="flex-1 min-w-0 self-start">
          <Card>
            <Box className="flex flex-col gap-3 p-3">
              <Typography variant="subtitle1" className="flex items-center gap-2 shrink-0">
                <Shirt className="w-4 h-4 text-[var(--accent-color)]" /> 3D 皮肤预览
              </Typography>
              <div className="min-h-[280px]" style={{ pointerEvents: 'auto' }}>
                <SkinViewer3D
                  key={session.uuid}
                  uuid={session.uuid}
                  username={session.username}
                  userType={session.user_type}
                  skinDataUrl={myCosmetics?.skin ?? null}
                  customSkinDataUrl={skinPreview}
                  authlibServerUrl={session.user_type === 'authlib' ? LITTLE_SKIN_YGGDRASIL : null}
                  capeDataUrl={myCosmetics?.cape ?? null}
                  customCapeDataUrl={capePreview}
                />
              </div>
            </Box>
          </Card>
        </div>

        <DialogBox open={switchOpen} onClose={() => setSwitchOpen(false)} title="切换账户" maxWidth="sm">
          {accounts.length > 0 ? (
            <Box className="space-y-2 pb-1">
              {accounts.map((account) => {
                const isCurrent = session.uuid === account.id || session.username === account.username
                return (
                  <button
                    key={account.id}
                    onClick={() => handleSwitchAccount(account)}
                    className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl border transition-colors cursor-pointer text-left ${
                      isCurrent
                        ? 'bg-accent-50 dark:bg-accent-500/10 border-accent-200 dark:border-accent-500/30'
                        : 'bg-surface-50 dark:bg-surface-800 border-surface-200 dark:border-surface-700 hover:bg-surface-100'
                    }`}
                  >
                    <SkinAvatar size={36} uuid={account.session.uuid} username={account.username} userType={account.user_type} />
                    <Box className="min-w-0 flex-1">
                      <Typography variant="subtitle2" className="truncate">{account.username}</Typography>
                      <Typography variant="caption" color="text.secondary">{accountTypeLabel(account.user_type)}</Typography>
                    </Box>
                    {isCurrent && <Chip label="当前" size="small" color="primary" />}
                  </button>
                )
              })}
            </Box>
          ) : (
            <Typography variant="body2" color="text.secondary" className="py-4 text-center">
              暂无已保存的账户
            </Typography>
          )}
          <Button variant="outlined" fullWidth className="mt-3" onClick={handleSwitchNew}>
            添加账户
          </Button>
        </DialogBox>
      </div>
    )
  }

  return (
    <Box className="max-w-xl space-y-6">
      <Box>
        <Typography variant="h5">账户管理</Typography>
        <Typography variant="body2" color="text.secondary">请登录账户以启动游戏。支持 Microsoft 正版、离线与 Little Skin 皮肤站。</Typography>
      </Box>

      <Tabs
        items={LOGIN_TABS}
        value={loginMode}
        onChange={(v) => setLoginMode(v as LoginMode)}
      />

      {accounts.length > 0 && (
        <Card>
          <Box className="space-y-3">
            <Typography variant="subtitle1" className="flex items-center gap-2">
              <Users className="w-4 h-4 text-[var(--accent-color)]" /> 已保存的账户 ({accounts.length})
            </Typography>
            <Typography variant="body2" color="text.secondary" className="text-xs">
              点击即可切换，所有已登录账户都会保留登录状态。
            </Typography>
            <Box className="space-y-2">
              {accounts.map((account) => (
                <Box
                  key={account.id}
                  className="flex items-center justify-between px-4 py-2.5 rounded-lg bg-surface-50 dark:bg-surface-800 border border-surface-200 dark:border-surface-700"
                >
                  <button
                    className="flex items-center gap-3 min-w-0 text-left flex-1 cursor-pointer hover:opacity-80"
                    onClick={() => handleSwitchAccount(account)}
                  >
                    <SkinAvatar size={32} uuid={account.session.uuid} username={account.username} userType={account.user_type} />
                    <Box className="min-w-0">
                      <Typography variant="subtitle2" className="truncate">{account.username}</Typography>
                      <Typography variant="caption" color="text.secondary">{accountTypeLabel(account.user_type)}</Typography>
                    </Box>
                  </button>
                  <Button size="small" variant="text" onClick={() => handleDeleteAccount(account)}>
                    <Trash className="w-4 h-4 text-red-400" />
                  </Button>
                </Box>
              ))}
            </Box>
          </Box>
        </Card>
      )}

      {loginMode === 'microsoft' && (
        <Card>
          <Box className="space-y-4">
            <Typography variant="subtitle1" className="flex items-center gap-2">
              <Globe className="w-4 h-4 text-[var(--accent-color)]" /> Microsoft 账号登录
            </Typography>
            {!deviceCode ? (
              <>
                <Typography variant="body2" color="text.secondary">
                  登录你的 Microsoft 账号。点击开始后，启动器会生成一个验证码，请在浏览器中打开微软授权页面并输入该验证码。
                </Typography>
                <Button onClick={handleMicrosoftLogin} loading={loading} fullWidth size="large">
                  登录 Microsoft 账号
                </Button>
              </>
            ) : (
              <Box className="space-y-4">
                <Box className="rounded-xl bg-surface-50 dark:bg-surface-800 border border-surface-200 dark:border-surface-700 p-5">
                  <Box className="flex items-center justify-between mb-3">
                    <Typography variant="caption" color="text.secondary">请访问以下链接并输入验证码</Typography>
                    <Typography variant="caption" color={countdown < 60 ? 'error' : 'text.secondary'} className="font-mono font-bold">
                      {formatCountdown(countdown)}
                    </Typography>
                  </Box>
                  <Box className="flex items-center gap-3 mb-4">
                    <Typography variant="h2" className="font-mono tracking-[0.3em] font-bold flex-1 text-center select-all">
                      {deviceCode.user_code}
                    </Typography>
                    <Button
                      variant="outlined"
                      size="small"
                      startIcon={copied ? <Check className="w-3.5 h-3.5" /> : <Copy className="w-3.5 h-3.5" />}
                      onClick={handleCopyCode}
                    >
                      {copied ? '已复制' : '复制'}
                    </Button>
                  </Box>
                  <Button
                    variant="outlined"
                    fullWidth
                    startIcon={<ExternalLink className="w-4 h-4" />}
                    onClick={() => {
                      const url = deviceCode.verification_uri_complete ?? (deviceCode.verification_uri ? `${deviceCode.verification_uri}?code=${deviceCode.user_code}` : 'https://login.microsoftonline.com/common/oauth2/v2.0/authorize')
                      open(url).catch(console.error)
                    }}
                  >
                    打开授权页面
                  </Button>
                </Box>
                <Box className="flex items-center gap-2 text-xs text-surface-400">
                  <RotateCw className={`w-3 h-3 ${msWaiting ? 'animate-spin' : ''}`} />
                  <span>{msWaiting ? '正在等待授权完成...' : '正在准备...'}</span>
                </Box>
                <Button
                  variant="text"
                  fullWidth
                  color="error"
                  onClick={handleMicrosoftCancel}
                  disabled={!deviceCode}
                >
                  取消登录
                </Button>
              </Box>
            )}
          </Box>
        </Card>
      )}

      {loginMode === 'offline' && (
        <Card>
          <Box className="space-y-4">
            <Typography variant="subtitle1" className="flex items-center gap-2">
              <Gamepad2 className="w-4 h-4 text-[var(--accent-color)]" /> 离线模式
            </Typography>
            <Typography variant="body2" color="text.secondary">
              无需账号密码，使用任意名称进入游戏。适用于单人游戏或局域网联机。
            </Typography>
            <Input label="玩家名称" value={username} onChange={(e) => setUsername(e.target.value)} placeholder="Steve" />
            <Button onClick={handleOfflineLogin} loading={loading} fullWidth>进入离线模式</Button>
          </Box>
        </Card>
      )}

      {loginMode === 'third' && (
        <Card>
          <Box className="space-y-4">
            <Typography variant="subtitle1" className="flex items-center gap-2">
              <QrCode className="w-4 h-4 text-[var(--accent-color)]" /> Little Skin 登录
            </Typography>
            <Typography variant="body2" color="text.secondary">
              通过 Little Skin OAuth 设备码授权登录，无需输入密码。
            </Typography>

            {!lsDeviceInfo && (
              <Button onClick={handleLittleSkinStart} loading={loading} fullWidth>
                开始授权登录
              </Button>
            )}

            {lsDeviceInfo && (
              <Box className="space-y-3">
                <Box className="bg-surface-50 dark:bg-surface-800 p-3 rounded-lg space-y-2">
                  <Typography variant="subtitle2">请在浏览器中打开以下链接完成授权：</Typography>
                  <Box className="flex items-center gap-2">
                    <input
                      value={lsDeviceInfo.verification_uri_complete ?? lsDeviceInfo.verification_uri}
                      readOnly
                      className="flex-1 text-xs font-mono bg-surface-100 dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg px-3 py-1.5 text-surface-700 dark:text-surface-200 outline-none"
                    />
                    <Button size="small" variant="outlined" onClick={() => {
                      const url = lsDeviceInfo.verification_uri_complete ?? lsDeviceInfo.verification_uri
                      if (url) navigator.clipboard.writeText(url)
                      setCopied(true)
                      setTimeout(() => setCopied(false), 2000)
                    }}>
                      {copied ? '已复制' : '复制'}
                    </Button>
                  </Box>
                  <Typography variant="caption" color="text.secondary">
                    设备码：<span className="font-mono font-bold text-surface-200">{lsDeviceInfo.user_code}</span>
                    &nbsp;（已自动复制）
                  </Typography>
                  <Button
                    size="small"
                    variant="outlined"
                    startIcon={<ExternalLink className="w-3 h-3" />}
                    onClick={() => {
                      const url = lsDeviceInfo.verification_uri_complete ?? lsDeviceInfo.verification_uri
                      if (url) open(url).catch(console.error)
                    }}
                    fullWidth
                  >
                    打开授权页面
                  </Button>
                </Box>
                <Box className="flex items-center gap-2 text-xs text-surface-400">
                  <RotateCw className={`w-3 h-3 ${lsPolling ? 'animate-spin' : ''}`} />
                  <span>{lsPolling ? '正在等待授权完成...' : '点击上方按钮打开链接'}</span>
                </Box>
                <Button variant="text" fullWidth color="error" onClick={handleLittleSkinCancel} disabled={lsPolling}>
                  取消登录
                </Button>
              </Box>
            )}
          </Box>
        </Card>
      )}
    </Box>
  )
}
