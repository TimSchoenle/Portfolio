import { type ComponentProps, type JSX } from 'react'

import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utilities'
import type { FCWithChildren } from '@/types/fc'

// eslint-disable-next-line @typescript-eslint/typedef
const sectionContainerVariants = cva('mx-auto w-full', {
  defaultVariants: {
    size: 'sm',
  },
  variants: {
    size: {
      lg: 'max-w-6xl',
      md: 'max-w-5xl',
      sm: 'max-w-4xl',
      xl: 'max-w-7xl',
    },
  },
})

type SectionContainerProperties = ComponentProps<'div'> &
  VariantProps<typeof sectionContainerVariants>

export const SectionContainer: FCWithChildren<SectionContainerProperties> = ({
  children,
  className,
  size,
  ...properties
}: SectionContainerProperties): JSX.Element => {
  return (
    <div
      className={cn(sectionContainerVariants({ size }), className)}
      {...properties}
    >
      {children}
    </div>
  )
}
