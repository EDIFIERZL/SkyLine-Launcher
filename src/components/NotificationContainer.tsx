import { useNotificationStore, type NotificationType } from '../stores/notificationStore'
import { X, CheckCircle2, AlertCircle, AlertTriangle, Info } from 'lucide-react'

const ICONS: Record<NotificationType, typeof Info> = {
  info: Info,
  success: CheckCircle2,
  warning: AlertTriangle,
  error: AlertCircle,
}

const COLORS: Record<NotificationType, string> = {
  info: 'bg-blue-50 dark:bg-blue-500/10 border-blue-200 dark:border-blue-500/30 text-blue-800 dark:text-blue-300',
  success: 'bg-green-50 dark:bg-green-500/10 border-green-200 dark:border-green-500/30 text-green-800 dark:text-green-300',
  warning: 'bg-amber-50 dark:bg-amber-500/10 border-amber-200 dark:border-amber-500/30 text-amber-800 dark:text-amber-300',
  error: 'bg-red-50 dark:bg-red-500/10 border-red-200 dark:border-red-500/30 text-red-800 dark:text-red-300',
}

const ICON_COLORS: Record<NotificationType, string> = {
  info: 'text-blue-500',
  success: 'text-green-500',
  warning: 'text-amber-500',
  error: 'text-red-500',
}

export function NotificationContainer() {
  const { notifications, removeNotification } = useNotificationStore()

  if (notifications.length === 0) return null

  return (
    <div className="fixed top-12 right-4 z-50 flex flex-col gap-2 max-w-sm w-full pointer-events-none">
      {notifications.map((notification) => {
        const Icon = ICONS[notification.type]
        return (
          <div
            key={notification.id}
            className={`pointer-events-auto flex items-start gap-3 p-4 rounded-xl border shadow-lg backdrop-blur-sm animate-slide-in-right ${COLORS[notification.type]}`}
          >
            <Icon className={`w-5 h-5 shrink-0 mt-0.5 ${ICON_COLORS[notification.type]}`} />
            <div className="flex-1 min-w-0">
              <p className="font-medium text-sm">{notification.title}</p>
              {notification.message && (
                <p className="text-xs opacity-80 mt-1">{notification.message}</p>
              )}
            </div>
            <button
              onClick={() => removeNotification(notification.id)}
              className="shrink-0 opacity-60 hover:opacity-100 transition-opacity"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        )
      })}
    </div>
  )
}
