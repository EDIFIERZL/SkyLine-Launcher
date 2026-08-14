
import React, { createContext, useContext, useEffect, useMemo } from 'react'
import { ThemeProvider } from '@mui/material/styles'
import CssBaseline from '@mui/material/CssBaseline'
import { createMaterialTheme } from './theme'
import { useSettingsStore } from '../../stores/settingsStore'

interface MaterialContextType {
  isDark: boolean
  theme: ReturnType<typeof createMaterialTheme>
}

const MaterialContext = createContext<MaterialContextType>({
  isDark: false,
  theme: createMaterialTheme(false),
})

export function useMaterialTheme() {
  return useContext(MaterialContext)
}

interface MaterialProviderProps {
  children: React.ReactNode
}

export function MaterialProvider({ children }: MaterialProviderProps) {
  const { config } = useSettingsStore()

  const isDark = useMemo(() => {
    if (config.theme_mode === 'dark') return true
    if (config.theme_mode === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return false
  }, [config.theme_mode])

  const theme = useMemo(
    () => createMaterialTheme(isDark, config.accent_color),
    [isDark, config.accent_color]
  )

  useEffect(() => {
    if (config.theme_mode === 'system') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      const handler = () => {
      }
      mediaQuery.addEventListener('change', handler)
      return () => mediaQuery.removeEventListener('change', handler)
    }
  }, [config.theme_mode])

  const contextValue = useMemo(
    () => ({ isDark, theme }),
    [isDark, theme]
  )

  return (
    <MaterialContext.Provider value={contextValue}>
      <ThemeProvider theme={theme}>
        <CssBaseline />
        {children}
      </ThemeProvider>
    </MaterialContext.Provider>
  )
}

export default MaterialProvider
