import * as React from 'react'
import { cn } from '../../lib/utils'

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string
}

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, label, id, ...props }, ref) => {
    return (
      <div className="space-y-1.5">
        {label && (
          <label htmlFor={id} className="text-sm text-surface-600 font-medium">
            {label}
          </label>
        )}
        <input
          ref={ref}
          id={id}
          className={cn(
            'w-full h-10 px-3 rounded-lg bg-white border border-surface-300',
            'text-surface-900 text-sm placeholder:text-surface-400',
            'focus:outline-none focus:ring-2 focus:ring-accent-500 focus:border-accent-500',
            'transition-all duration-150',
            className,
          )}
          {...props}
        />
      </div>
    )
  },
)
Input.displayName = 'Input'
