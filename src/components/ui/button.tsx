import * as React from 'react'
import { cn } from '../../lib/utils'

const variants = {
  default: 'bg-accent-600 text-surface-950 dark:text-white hover:bg-accent-700 shadow-xs',
  secondary: 'bg-white text-surface-700 border border-surface-300 hover:bg-surface-50',
  ghost: 'text-surface-500 hover:text-surface-700 hover:bg-surface-100',
  destructive: 'bg-red-600 text-white hover:bg-red-700',
  outline: 'border border-surface-300 text-surface-600 hover:bg-surface-50',
  launch: 'bg-launch-gradient text-white shadow-launch hover:brightness-110 pulse-glow',
  success: 'bg-green-600 text-white hover:bg-green-700',
  warning: 'bg-amber-500 text-white hover:bg-amber-600',
} as const

const sizes = {
  sm: 'h-8 px-3 text-xs',
  default: 'h-10 px-4 text-sm',
  lg: 'h-12 px-6 text-base',
  xl: 'h-14 px-8 text-lg',
  icon: 'h-10 w-10',
  'icon-sm': 'h-8 w-8',
  'icon-lg': 'h-12 w-12',
} as const

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: keyof typeof variants
  size?: keyof typeof sizes
  loading?: boolean
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = 'default', size = 'default', loading, children, disabled, ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(
          'inline-flex items-center justify-center rounded-xl font-semibold transition-all duration-200',
          'active:scale-[0.97]',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-500 focus-visible:ring-offset-2',
          'disabled:opacity-50 disabled:pointer-events-none',
          'cursor-pointer',
          variants[variant],
          sizes[size],
          className,
        )}
        disabled={disabled || loading}
        {...props}
      >
        {loading && (
          <svg className="animate-spin -ml-1 mr-2 h-4 w-4" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
          </svg>
        )}
        {children}
      </button>
    )
  },
)
Button.displayName = 'Button'
