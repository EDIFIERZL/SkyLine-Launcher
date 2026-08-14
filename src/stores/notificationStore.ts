import { create } from 'zustand'

export type NotificationType = 'info' | 'success' | 'warning' | 'error'

export interface Notification {
  id: string
  type: NotificationType
  title: string
  message?: string
  duration?: number
  timestamp: number
}

interface NotificationState {
  notifications: Notification[]
  addNotification: (notification: Omit<Notification, 'id' | 'timestamp'>) => void
  removeNotification: (id: string) => void
  clearAll: () => void
}

let notificationId = 0

export const useNotificationStore = create<NotificationState>((set) => ({
  notifications: [],
  addNotification: (notification) => {
    const id = `notification-${++notificationId}`
    const duration = notification.duration ?? 5000
    const newNotification: Notification = {
      ...notification,
      id,
      timestamp: Date.now(),
      duration,
    }
    set((state) => ({
      notifications: [...state.notifications, newNotification],
    }))

    
    if (duration > 0) {
      setTimeout(() => {
        set((state) => ({
          notifications: state.notifications.filter((n) => n.id !== id),
        }))
      }, duration)
    }
  },
  removeNotification: (id) => {
    set((state) => ({
      notifications: state.notifications.filter((n) => n.id !== id),
    }))
  },
  clearAll: () => {
    set({ notifications: [] })
  },
}))


export const notify = {
  info: (title: string, message?: string) =>
    useNotificationStore.getState().addNotification({ type: 'info', title, message }),
  success: (title: string, message?: string) =>
    useNotificationStore.getState().addNotification({ type: 'success', title, message }),
  warning: (title: string, message?: string) =>
    useNotificationStore.getState().addNotification({ type: 'warning', title, message }),
  error: (title: string, message?: string) =>
    useNotificationStore.getState().addNotification({ type: 'error', title, message, duration: 8000 }),
}
