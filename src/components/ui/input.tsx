import * as React from 'react'

import { cn } from '@/lib/utilities'

export type InputProperties = React.InputHTMLAttributes<HTMLInputElement>

// eslint-disable-next-line @typescript-eslint/typedef
const Input = React.forwardRef<HTMLInputElement, InputProperties>(
  // eslint-disable-next-line @typescript-eslint/typedef
  ({ className, type, ...properties }, reference): React.JSX.Element => {
    return (
      <input
        className={cn(
          'flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50',
          className
        )}
        ref={reference}
        type={type}
        {...properties}
      />
    )
  }
)
Input.displayName = 'Input'

export { Input }
