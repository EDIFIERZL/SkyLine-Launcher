import { useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'

interface ShortcutMap {
  [key: string]: () => void
}

export function useKeyboardShortcuts(shortcuts: ShortcutMap = {}) {
  const navigate = useNavigate()

  const defaultShortcuts: ShortcutMap = {
    'ctrl+1': () => navigate('/'),
    'ctrl+2': () => navigate('/download'),
    'ctrl+3': () => navigate('/settings'),
    'ctrl+comma': () => navigate('/settings'),
  }

  const allShortcuts = { ...defaultShortcuts, ...shortcuts }

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      
      const target = event.target as HTMLElement
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
        return
      }

      const parts: string[] = []
      if (event.ctrlKey || event.metaKey) parts.push('ctrl')
      if (event.shiftKey) parts.push('shift')
      if (event.altKey) parts.push('alt')

      
      let key = event.key.toLowerCase()
      if (key === ',') key = 'comma'
      if (key === '.') key = 'period'
      if (key === '/') key = 'slash'
      if (key === 'escape') key = 'esc'

      parts.push(key)
      const combo = parts.join('+')

      if (allShortcuts[combo]) {
        event.preventDefault()
        allShortcuts[combo]()
      }
    },
    [allShortcuts]
  )

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleKeyDown])
}


export const SHORTCUTS = {
  HOME: 'Ctrl+1',
  DOWNLOAD: 'Ctrl+2',
  SETTINGS: 'Ctrl+3',
  SEARCH: 'Ctrl+K',
  REFRESH: 'F5',
  ESCAPE: 'Esc',
}
