
import { createTheme } from '@mui/material/styles'
import type { ThemeOptions } from '@mui/material/styles'

function getCSSVar(name: string): string {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim()
}

export function createMaterialTheme(isDark: boolean = false, accentColor: string = getCSSVar('--accent-color') || '#3b82f6') {
  const accent = accentColor && accentColor.trim() ? accentColor.trim() : getCSSVar('--accent-color') || '#3b82f6'
  const themeOptions: ThemeOptions = {
    palette: {
      mode: isDark ? 'dark' : 'light',
      primary: {
        main: accent,
        light: getCSSVar('--color-accent-400') || '#60a5fa',
        dark: getCSSVar('--color-accent-600') || '#2563eb',
        contrastText: '#ffffff',
      },
      secondary: {
        main: '#8b5cf6',
        light: '#a78bfa',
        dark: '#7c3aed',
      },
      error: {
        main: '#ef4444',
        light: '#f87171',
        dark: '#dc2626',
      },
      warning: {
        main: '#f59e0b',
        light: '#fbbf24',
        dark: '#d97706',
      },
      info: {
        main: '#06b6d4',
        light: '#22d3ee',
        dark: '#0891b2',
      },
      success: {
        main: '#10b981',
        light: '#34d399',
        dark: '#059669',
      },
      background: {
        default: isDark ? '#0a0a0a' : '#f8fafc',
        paper: isDark ? '#181818' : '#ffffff',
      },
      text: {
        primary: isDark ? '#f0f0f0' : '#0f172a',
        secondary: isDark ? '#a0a0a0' : '#64748b',
      },
      divider: isDark ? 'rgba(255, 255, 255, 0.08)' : 'rgba(15, 23, 42, 0.1)',
    },
    typography: {
      fontFamily: "'Inter', 'Segoe UI', system-ui, -apple-system, sans-serif",
      fontSize: 14,
      h1: {
        fontSize: '2rem',
        fontWeight: 700,
      },
      h2: {
        fontSize: '1.5rem',
        fontWeight: 600,
      },
      h3: {
        fontSize: '1.25rem',
        fontWeight: 600,
      },
      h4: {
        fontSize: '1.125rem',
        fontWeight: 600,
      },
      h5: {
        fontSize: '1rem',
        fontWeight: 600,
      },
      h6: {
        fontSize: '0.875rem',
        fontWeight: 600,
      },
      body1: {
        fontSize: '0.875rem',
      },
      body2: {
        fontSize: '0.8125rem',
      },
      button: {
        textTransform: 'none',
        fontWeight: 600,
      },
    },
    shape: {
      borderRadius: 12,
    },
    components: {
      MuiButton: {
        styleOverrides: {
          root: {
            borderRadius: '12px',
            padding: '8px 16px',
            fontSize: '0.875rem',
            fontWeight: 600,
            textTransform: 'none',
            boxShadow: 'none',
            '&:hover': {
              boxShadow: 'none',
            },
          },
          sizeSmall: {
            padding: '6px 12px',
            fontSize: '0.8125rem',
          },
          sizeLarge: {
            padding: '12px 24px',
            fontSize: '1rem',
          },
        },
      },
      MuiIconButton: {
        styleOverrides: {
          root: {
            borderRadius: '12px',
            padding: '8px',
          },
        },
      },
      MuiTextField: {
        styleOverrides: {
          root: {
            '& .MuiOutlinedInput-root': {
              borderRadius: '12px',
            },
          },
        },
      },
      MuiSelect: {
        styleOverrides: {
          root: {
            borderRadius: '12px',
          },
        },
      },
      MuiCard: {
        styleOverrides: {
          root: {
            borderRadius: '16px',
            boxShadow: 'none',
            border: '1px solid',
            borderColor: 'divider',
          },
        },
      },
      MuiChip: {
        styleOverrides: {
          root: {
            borderRadius: '8px',
          },
        },
      },
      MuiDialog: {
        styleOverrides: {
          paper: {
            borderRadius: '20px',
          },
        },
      },
      MuiTab: {
        styleOverrides: {
          root: {
            textTransform: 'none',
            fontWeight: 500,
            minHeight: '40px',
          },
        },
      },
      MuiTabs: {
        styleOverrides: {
          root: {
            minHeight: '40px',
          },
        },
      },
      MuiDrawer: {
        styleOverrides: {
          paper: {
            borderRadius: '20px 0 0 20px',
          },
        },
      },
      MuiAlert: {
        styleOverrides: {
          root: {
            borderRadius: '12px',
          },
        },
      },
      MuiListItemButton: {
        styleOverrides: {
          root: {
            borderRadius: '12px',
          },
        },
      },
    },
  }

  return createTheme(themeOptions)
}

export const darkTheme = createMaterialTheme(true)

export const lightTheme = createMaterialTheme(false)

export default { createMaterialTheme, darkTheme, lightTheme }
