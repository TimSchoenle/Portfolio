import { type ComponentProps, type JSX } from 'react'

import { cn } from '@/lib/utilities'
import type { FCWithChildren } from '@/types/fc'

export const SECTION_BACKGROUNDS: {
  readonly DEFAULT: 'default'
  readonly GRADIENT: 'gradient'
  readonly MUTED: 'muted'
} = {
  DEFAULT: 'default',
  GRADIENT: 'gradient',
  MUTED: 'muted',
} as const

type SectionProperties = ComponentProps<'section'> & {
  readonly background?: (typeof SECTION_BACKGROUNDS)[keyof typeof SECTION_BACKGROUNDS]
  readonly performance?: boolean
}

export const Section: FCWithChildren<SectionProperties> = ({
  background = SECTION_BACKGROUNDS.DEFAULT,
  children,
  className,
  performance = false,
  ...properties
}: SectionProperties): JSX.Element => {
  return (
    <section
      className={cn(
        'relative overflow-hidden px-4 py-20 md:py-24',
        {
          'bg-background': background === 'default',
          'bg-gradient-to-b from-background via-muted/20 to-background':
            background === 'gradient',
          'bg-muted/30': background === 'muted',
          'content-auto': performance,
        },
        className
      )}
      {...properties}
    >
      {children}
    </section>
  )
}
