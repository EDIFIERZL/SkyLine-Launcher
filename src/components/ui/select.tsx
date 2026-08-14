import * as React from 'react'
import { cn } from '../../lib/utils'

interface SelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
  label?: string
  options: { value: string; label: string }[]
}

export const Select = React.forwardRef<HTMLSelectElement, SelectProps>(
  ({ className, label, id, options, ...props }, ref) => {
    return (
      <div className="space-y-1.5">
        {label && (
          <label htmlFor={id} className="text-sm text-surface-600 font-medium">
            {label}
          </label>
        )}
        <select
          ref={ref}
          id={id}
          className={cn(
            'w-full h-10 px-3 rounded-lg bg-white border border-surface-300',
            'text-surface-900 text-sm',
            'focus:outline-none focus:ring-2 focus:ring-accent-500 focus:border-accent-500',
            'transition-all duration-150',
            className,
          )}
          {...props}
        >
          {options.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>
    )
  },
)
Select.displayName = 'Select'
