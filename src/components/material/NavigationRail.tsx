
import React from 'react'
import Box from '@mui/material/Box'
import Typography from '@mui/material/Typography'
import ButtonBase from '@mui/material/ButtonBase'

export interface NavigationRailItem {
  id: string
  label: string
  icon?: React.ReactNode
  activeIcon?: React.ReactNode
}

interface NavigationRailProps {
  topItems: NavigationRailItem[]
  bottomItems?: NavigationRailItem[]
  accountItems?: NavigationRailItem[]
  activeId: string
  onNavigate: (id: string) => void
  width?: number
  showLabels?: boolean
}

export function NavigationRail({
  topItems,
  bottomItems = [],
  accountItems = [],
  activeId,
  onNavigate,
  width = 88,
  showLabels = true,
}: NavigationRailProps) {
  const renderItem = (item: NavigationRailItem) => {
    const active = activeId === item.id
    const icon = active && item.activeIcon ? item.activeIcon : item.icon

    return (
      <ButtonBase
        key={item.id}
        aria-label={item.label}
        aria-pressed={active}
        onClick={() => onNavigate(item.id)}
        focusRipple
        sx={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          gap: '4px',
          width: '100%',
          minHeight: 56,
          padding: '6px 8px',
          borderRadius: '8px',
          transition: 'background-color 0.2s ease-in-out',
          backgroundColor: active ? 'action.selected' : 'transparent',
          '&:hover': {
            backgroundColor: active ? 'action.selected' : 'action.hover',
          },
        }}
      >
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            width: 32,
            height: 32,
            borderRadius: '8px',
            color: active ? 'primary.main' : 'text.secondary',
            transition: 'color 0.2s ease-in-out',
            ...(active && {
              backgroundColor: 'primary.main',
              color: 'primary.contrastText',
            }),
          }}
        >
          {icon}
        </Box>
        {showLabels && (
          <Typography
            variant="caption"
            sx={{
              fontSize: '11px',
              lineHeight: 1,
              fontWeight: active ? 600 : 500,
              color: active ? 'primary.main' : 'text.secondary',
            }}
          >
            {item.label}
          </Typography>
        )}
      </ButtonBase>
    )
  }

  const renderSection = (items: NavigationRailItem[]) => (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'stretch',
        gap: '4px',
        width: '100%',
        px: 1,
      }}
    >
      {items.map(renderItem)}
    </Box>
  )

  return (
    <Box
      component="nav"
      role="navigation"
      aria-label="主导航"
      className="nav-rail"
      sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        width,
        flexShrink: 0,
        height: '100%',
        paddingY: 1,
        bgcolor: 'background.paper',
        borderRight: '1px solid',
        borderColor: 'divider',
      }}
    >
      <Box sx={{ flex: 1, width: '100%', overflowY: 'auto' }}>{renderSection(topItems)}</Box>
      {(bottomItems.length > 0 || accountItems.length > 0) && (
        <Box sx={{ width: '100%', display: 'flex', flexDirection: 'column' }}>
          {accountItems.length > 0 && renderSection(accountItems)}
          {bottomItems.length > 0 && renderSection(bottomItems)}
        </Box>
      )}
    </Box>
  )
}

export default NavigationRail