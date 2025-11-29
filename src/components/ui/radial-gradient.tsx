import { type JSX } from 'react'

import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utilities'
import type { FCStrict } from '@/types/fc'

// eslint-disable-next-line @typescript-eslint/typedef
const radialGradientVariants = cva('absolute -z-10', {
  defaultVariants: {
    position: 'top-right',
  },
  variants: {
    position: {
      'bottom-left': 'bottom-0 left-0',
      'bottom-right': 'bottom-0 right-0',
      'top-left': 'top-0 left-0',
      'top-right': 'top-0 right-0',
    },
  },
})

interface RadialGradientProperties
  extends VariantProps<typeof radialGradientVariants> {
  readonly className?: string
  readonly size?: number
}

export const RadialGradient: FCStrict<RadialGradientProperties> = ({
  className,
  position,
  size = 600,
}: RadialGradientProperties): JSX.Element => {
  return (
    <div
      className={cn(radialGradientVariants({ position }), className)}
      style={{
        backgroundImage: `radial-gradient(circle ${String(size)}px at 50% 50%, rgba(var(--primary-rgb, 99, 102, 241), 0.08), transparent)`,
        height: `${String(size)}px`,
        width: `${String(size)}px`,
      }}
    />
  )
}
