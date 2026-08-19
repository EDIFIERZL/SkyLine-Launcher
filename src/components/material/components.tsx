
import React from 'react'
import {
  Button as MuiButton,
  IconButton as MuiIconButton,
  TextField as MuiTextField,
  Select as MuiSelect,
  MenuItem,
  FormControl,
  InputLabel,
  Card as MuiCard,
  CardContent,
  Chip as MuiChip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Tooltip as MuiTooltip,
  Badge as MuiBadge,
  CircularProgress,
  LinearProgress,
  Box,
  Typography,
  Tabs as MuiTabs,
  Tab as MuiTab,
  Drawer,
  AppBar,
  Toolbar,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Divider,
  Avatar,
  Alert,
  Snackbar,
  Switch as MuiSwitch,
  Slider as MuiSlider,
} from '@mui/material'
import type { SelectChangeEvent } from '@mui/material'


interface ButtonProps {
  children: React.ReactNode
  variant?: 'contained' | 'outlined' | 'text' | 'ghost'
  size?: 'small' | 'medium' | 'large'
  color?: 'primary' | 'secondary' | 'error' | 'warning' | 'info' | 'success'
  startIcon?: React.ReactNode
  endIcon?: React.ReactNode
  loading?: boolean
  disabled?: boolean
  fullWidth?: boolean
  type?: 'button' | 'submit' | 'reset'
  onClick?: () => void
  className?: string
}

export function Button({
  children,
  variant = 'contained',
  size = 'medium',
  color = 'primary',
  startIcon,
  endIcon,
  loading,
  disabled,
  fullWidth,
  type = 'button',
  onClick,
  className,
}: ButtonProps) {
  const muiVariant = variant === 'ghost' ? 'text' : variant
  
  return (
    <MuiButton
      type={type}
      variant={muiVariant}
      size={size}
      color={color}
      startIcon={loading ? <CircularProgress size={16} color="inherit" /> : startIcon}
      endIcon={endIcon}
      disabled={disabled || loading}
      fullWidth={fullWidth}
      onClick={onClick}
      className={className}
      sx={{
        borderRadius: '12px',
        textTransform: 'none',
        fontWeight: 600,
        flexShrink: 0,
        columnGap: '6px',
        padding: size === 'small' ? '6px 14px' : size === 'large' ? '12px 26px' : '9px 18px',
        ...(variant === 'ghost' && {
          '&:hover': {
            backgroundColor: 'action.hover',
          },
        }),
      }}
    >
      {children}
    </MuiButton>
  )
}


interface IconButtonProps {
  children: React.ReactNode
  size?: 'small' | 'medium' | 'large'
  color?: 'primary' | 'secondary' | 'error' | 'warning' | 'info' | 'success' | 'default'
  disabled?: boolean
  onClick?: () => void
  title?: string
  className?: string
}

export function IconButton({
  children,
  size = 'medium',
  color = 'default',
  disabled,
  onClick,
  title,
  className,
}: IconButtonProps) {
  return (
    <MuiTooltip title={title || ''}>
      <MuiIconButton
        size={size}
        color={color}
        disabled={disabled}
        onClick={onClick}
        className={className}
        sx={{ borderRadius: '12px' }}
      >
        {children}
      </MuiIconButton>
    </MuiTooltip>
  )
}


interface InputProps {
  label?: string
  placeholder?: string
  value?: string
  onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void
  onKeyDown?: (e: React.KeyboardEvent<HTMLInputElement>) => void
  type?: string
  disabled?: boolean
  error?: boolean
  helperText?: string
  fullWidth?: boolean
  size?: 'small' | 'medium'
  variant?: 'outlined' | 'filled' | 'standard'
  className?: string
  multiline?: boolean
  rows?: number
}

export function Input({
  label,
  placeholder,
  value,
  onChange,
  onKeyDown,
  type = 'text',
  disabled,
  error,
  helperText,
  fullWidth = true,
  size = 'medium',
  variant = 'outlined',
  className,
  multiline,
  rows,
}: InputProps) {
  return (
    <MuiTextField
      label={label}
      placeholder={placeholder}
      value={value}
      onChange={onChange}
      onKeyDown={onKeyDown as React.KeyboardEventHandler<HTMLDivElement>}
      type={type}
      disabled={disabled}
      error={error}
      helperText={helperText}
      fullWidth={fullWidth}
      size={size}
      variant={variant}
      className={className}
      multiline={multiline}
      rows={rows}
      sx={{
        '& .MuiOutlinedInput-root': {
          borderRadius: '12px',
        },
      }}
    />
  )
}


interface SelectOption {
  value: string
  label: string
  disabled?: boolean
}

interface SelectProps {
  label?: string
  value: string
  onChange: (value: string) => void
  options: SelectOption[]
  disabled?: boolean
  fullWidth?: boolean
  size?: 'small' | 'medium'
  className?: string
  renderValue?: (value: string) => React.ReactNode
}

export function Select({
  label,
  value,
  onChange,
  options,
  disabled,
  fullWidth = true,
  size = 'medium',
  className,
  renderValue,
}: SelectProps) {
  const handleChange = (e: SelectChangeEvent) => {
    onChange(e.target.value)
  }

  return (
    <FormControl fullWidth={fullWidth} size={size} className={className}>
      {label && <InputLabel>{label}</InputLabel>}
      <MuiSelect
        value={value}
        onChange={handleChange}
        label={label}
        disabled={disabled}
        renderValue={renderValue}
        sx={{ borderRadius: '12px' }}
      >
        {options.map((opt) => (
          <MenuItem key={opt.value} value={opt.value} disabled={opt.disabled}>
            {opt.label}
          </MenuItem>
        ))}
      </MuiSelect>
    </FormControl>
  )
}


interface CardProps {
  children: React.ReactNode
  className?: string
  style?: React.CSSProperties
  onClick?: () => void
  hoverable?: boolean
  padding?: 'none' | 'small' | 'medium' | 'large'
}

export function Card({
  children,
  className,
  style,
  onClick,
  hoverable,
  padding = 'medium',
}: CardProps) {
  const paddingMap = {
    none: 0,
    small: 1.5,
    medium: 2,
    large: 3,
  }

  return (
    <MuiCard
      className={className}
      style={style}
      onClick={onClick}
      sx={{
        borderRadius: '16px',
        border: '1px solid',
        borderColor: 'divider',
        boxShadow: 'none',
        cursor: onClick || hoverable ? 'pointer' : 'default',
        transition: 'all 0.2s ease-in-out',
        '&:hover': onClick || hoverable ? {
          borderColor: 'primary.main',
          boxShadow: '0 4px 12px rgba(0,0,0,0.08)',
          transform: 'translateY(-2px)',
        } : {},
      }}
    >
      <CardContent sx={{ padding: paddingMap[padding], '&:last-child': { paddingBottom: paddingMap[padding] } }}>
        {children}
      </CardContent>
    </MuiCard>
  )
}


interface ChipProps {
  label: string
  variant?: 'filled' | 'outlined'
  color?: 'default' | 'primary' | 'secondary' | 'error' | 'info' | 'success' | 'warning'
  size?: 'small' | 'medium'
  icon?: React.ReactNode
  onDelete?: () => void
  className?: string
}

export function Chip({
  label,
  variant = 'filled',
  color = 'default',
  size = 'small',
  icon,
  onDelete,
  className,
}: ChipProps) {
  return (
    <MuiChip
      label={label}
      variant={variant}
      color={color}
      size={size}
      icon={icon as React.ReactElement}
      onDelete={onDelete}
      className={className}
      sx={{ borderRadius: '8px' }}
    />
  )
}


interface DialogProps {
  open: boolean
  onClose: () => void
  title?: string
  children: React.ReactNode
  actions?: React.ReactNode
  maxWidth?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
  fullWidth?: boolean
}

export function DialogBox({
  open,
  onClose,
  title,
  children,
  actions,
  maxWidth = 'sm',
  fullWidth = true,
}: DialogProps) {
  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth={maxWidth}
      fullWidth={fullWidth}
      slotProps={{
        paper: {
          sx: {
            borderRadius: '20px',
            padding: 1,
          },
        },
      }}
    >
      {title && (
        <DialogTitle sx={{ fontWeight: 600 }}>{title}</DialogTitle>
      )}
      <DialogContent>{children}</DialogContent>
      {actions && <DialogActions>{actions}</DialogActions>}
    </Dialog>
  )
}


interface ProgressProps {
  variant?: 'determinate' | 'indeterminate'
  value?: number
  color?: 'primary' | 'secondary' | 'error' | 'info' | 'success' | 'warning'
  size?: number
  className?: string
}

export function Progress({
  variant = 'indeterminate',
  value,
  color = 'primary',
  size = 24,
  className,
}: ProgressProps) {
  if (variant === 'determinate') {
    return (
      <LinearProgress
        variant="determinate"
        value={value}
        color={color}
        className={className}
        sx={{ borderRadius: '4px', height: '6px' }}
      />
    )
  }
  return <CircularProgress size={size} color={color} className={className} />
}


interface TabItem {
  value: string
  label: string
  icon?: React.ReactNode
}

interface TabsProps {
  items: TabItem[]
  value: string
  onChange: (value: string) => void
  className?: string
}

export function Tabs({ items, value, onChange, className }: TabsProps) {
  return (
    <MuiTabs
      value={value}
      onChange={(_, v) => onChange(v)}
      className={className}
      sx={{
        minHeight: '40px',
        '& .MuiTab-root': {
          minHeight: '40px',
          textTransform: 'none',
          fontWeight: 500,
          borderRadius: '8px',
        },
        '& .Mui-selected': {
          color: 'var(--accent-color)',
        },
        '& .MuiTabs-indicator': {
          backgroundColor: 'var(--accent-color)',
        },
      }}
    >
      {items.map((item) => (
        <MuiTab
          key={item.value}
          value={item.value}
          label={item.label}
          icon={item.icon as React.ReactElement}
          iconPosition="start"
        />
      ))}
    </MuiTabs>
  )
}


interface DrawerProps {
  open: boolean
  onClose: () => void
  anchor?: 'left' | 'right' | 'top' | 'bottom'
  children: React.ReactNode
  width?: number | string
}

export function DrawerBox({
  open,
  onClose,
  anchor = 'right',
  children,
  width = 320,
}: DrawerProps) {
  return (
    <Drawer
      open={open}
      onClose={onClose}
      anchor={anchor}
      slotProps={{
        paper: {
          sx: {
            width,
            borderRadius: anchor === 'right' ? '20px 0 0 20px' : '0 20px 20px 0',
          },
        },
      }}
    >
      {children}
    </Drawer>
  )
}


interface AlertProps {
  severity: 'error' | 'warning' | 'info' | 'success'
  children: React.ReactNode
  onClose?: () => void
  className?: string
}

export function AlertBox({ severity, children, onClose, className }: AlertProps) {
  return (
    <Alert
      severity={severity}
      onClose={onClose}
      className={className}
      sx={{ borderRadius: '12px' }}
    >
      {children}
    </Alert>
  )
}


interface SnackbarProps {
  open: boolean
  onClose: () => void
  message: string
  severity?: 'error' | 'warning' | 'info' | 'success'
  autoHideDuration?: number
}

export function SnackbarAlert({
  open,
  onClose,
  message,
  severity = 'info',
  autoHideDuration = 3000,
}: SnackbarProps) {
  return (
    <Snackbar
      open={open}
      autoHideDuration={autoHideDuration}
      onClose={onClose}
      anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
    >
      <Alert severity={severity} onClose={onClose} sx={{ borderRadius: '12px' }}>
        {message}
      </Alert>
    </Snackbar>
  )
}


interface SwitchProps {
  checked: boolean
  onChange: (checked: boolean) => void
  label?: string
  disabled?: boolean
  color?: 'primary' | 'secondary' | 'error' | 'info' | 'success' | 'warning'
}

export function Switch({
  checked,
  onChange,
  label,
  disabled,
  color = 'primary',
}: SwitchProps) {
  return (
    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
      <MuiSwitch
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        disabled={disabled}
        color={color}
      />
      {label && (
        <Typography variant="body2">{label}</Typography>
      )}
    </Box>
  )
}


interface SliderProps {
  value: number
  onChange: (value: number) => void
  min?: number
  max?: number
  step?: number
  label?: string
  disabled?: boolean
  color?: 'primary' | 'secondary' | 'error' | 'info' | 'success' | 'warning'
  className?: string
}

export function Slider({
  value,
  onChange,
  min = 0,
  max = 100,
  step = 1,
  label,
  disabled,
  color = 'primary',
  className,
}: SliderProps) {
  return (
    <Box sx={{ width: '100%' }} className={className}>
      {label && (
        <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
          <Typography variant="body2">{label}</Typography>
          <Typography variant="body2" color="text.secondary">{value}</Typography>
        </Box>
      )}
      <MuiSlider
        value={value}
        onChange={(_, v) => onChange(v as number)}
        min={min}
        max={max}
        step={step}
        disabled={disabled}
        color={color}
        valueLabelDisplay="auto"
        sx={{ touchAction: 'none' }}
      />
    </Box>
  )
}


interface AvatarProps {
  src?: string
  alt?: string
  size?: number
  className?: string
}

export function AvatarIcon({ src, alt, size = 40, className }: AvatarProps) {
  return (
    <Avatar
      src={src}
      alt={alt}
      className={className}
      sx={{ width: size, height: size }}
    />
  )
}


interface BadgeProps {
  children: React.ReactNode
  badgeContent?: number | string
  color?: 'primary' | 'secondary' | 'error' | 'info' | 'success' | 'warning'
  max?: number
  showZero?: boolean
}

export function Badge({
  children,
  badgeContent,
  color = 'primary',
  max = 99,
  showZero,
}: BadgeProps) {
  return (
    <MuiBadge
      badgeContent={badgeContent}
      color={color}
      max={max}
      showZero={showZero}
    >
      {children}
    </MuiBadge>
  )
}


interface TooltipProps {
  children: React.ReactNode
  title: string
  placement?: 'top' | 'bottom' | 'left' | 'right'
}

export function Tooltip({ children, title, placement = 'top' }: TooltipProps) {
  return (
    <MuiTooltip title={title} placement={placement} arrow>
      <span>{children}</span>
    </MuiTooltip>
  )
}


export function DividerLine({ className }: { className?: string }) {
  return <Divider className={className} sx={{ borderColor: 'divider' }} />
}


interface ListItemData {
  id: string
  label: string
  icon?: React.ReactNode
  onClick?: () => void
  disabled?: boolean
  selected?: boolean
}

interface ListProps {
  items: ListItemData[]
  className?: string
}

export function ListMenu({ items, className }: ListProps) {
  return (
    <List className={className} sx={{ padding: 0 }}>
      {items.map((item, index) => (
        <React.Fragment key={item.id}>
          <ListItem disablePadding>
            <ListItemButton
              onClick={item.onClick}
              disabled={item.disabled}
              selected={item.selected}
              sx={{
                borderRadius: '12px',
                '&.Mui-selected': {
                  backgroundColor: 'var(--accent-color)',
                  color: 'white',
                  '&:hover': {
                    backgroundColor: 'var(--accent-color)',
                  },
                },
              }}
            >
              {item.icon && <ListItemIcon sx={{ minWidth: 40 }}>{item.icon}</ListItemIcon>}
              <ListItemText primary={item.label} />
            </ListItemButton>
          </ListItem>
          {index < items.length - 1 && <Divider variant="inset" component="li" />}
        </React.Fragment>
      ))}
    </List>
  )
}


export function Loading({ size = 24 }: { size?: number }) {
  return (
    <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', padding: 3 }}>
      <CircularProgress size={size} />
    </Box>
  )
}


interface EmptyStateProps {
  icon?: React.ReactNode
  title: string
  description?: string
  action?: React.ReactNode
}

export function EmptyState({ icon, title, description, action }: EmptyStateProps) {
  return (
    <Box
      sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 6,
        color: 'text.secondary',
      }}
    >
      {icon && <Box sx={{ marginBottom: 2, opacity: 0.5 }}>{icon}</Box>}
      <Typography variant="h6" sx={{ marginBottom: 1 }}>{title}</Typography>
      {description && (
        <Typography variant="body2" sx={{ marginBottom: 2 }}>{description}</Typography>
      )}
      {action}
    </Box>
  )
}


export {
  Box,
  Typography,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Divider,
  AppBar,
  Toolbar,
}
