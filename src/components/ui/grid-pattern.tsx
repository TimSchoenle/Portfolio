import { type JSX } from 'react'

import { cn } from '@/lib/utilities'
import type { FCStrict } from '@/types/fc'

interface GridPatternProperties {
  readonly className?: string
  readonly offsetX?: number
  readonly offsetY?: number
  readonly size?: number
}

export const GridPattern: FCStrict<GridPatternProperties> = ({
  className,
  offsetX = 0,
  offsetY = 0,
  size = 32,
}: GridPatternProperties): JSX.Element => {
  return (
    <div
      className={cn(
        'absolute inset-0 -z-10 bg-[linear-gradient(to_right,#8080800a_1px,transparent_1px),linear-gradient(to_bottom,#8080800a_1px,transparent_1px)]',
        className
      )}
      style={{
        backgroundPosition: `${String(offsetX)}px ${String(offsetY)}px`,
        backgroundSize: `${String(size)}px ${String(size)}px`,
      }}
    />
  )
}
