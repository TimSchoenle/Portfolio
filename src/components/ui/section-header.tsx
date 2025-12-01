import { type JSX } from 'react'

import { cva, type VariantProps } from 'class-variance-authority'

import { Heading } from '@/components/ui/heading'
import { cn } from '@/lib/utilities'
import type { FCStrict } from '@/types/fc'

// eslint-disable-next-line @typescript-eslint/typedef
const sectionHeaderVariants = cva('flex flex-col gap-4 mb-12 md:mb-16', {
  defaultVariants: {
    align: 'center',
  },
  variants: {
    align: {
      center: 'text-center items-center',
      left: 'text-left items-start',
      right: 'text-right items-end',
    },
  },
})

interface SectionHeaderProperties extends VariantProps<
  typeof sectionHeaderVariants
> {
  readonly className?: string
  readonly gradient?: boolean
  readonly subtitle?: string
  readonly title: string
  readonly underline?: boolean
}

export const SectionHeader: FCStrict<SectionHeaderProperties> = ({
  align,
  className,
  gradient,
  subtitle,
  title,
  underline,
}: SectionHeaderProperties): JSX.Element => {
  return (
    <div className={cn(sectionHeaderVariants({ align }), className)}>
      <Heading
        as="h2"
        className={cn(
          'text-4xl font-bold tracking-tight md:text-5xl',
          gradient === true
            ? 'inline-block bg-gradient-to-r from-primary to-primary/60 bg-clip-text py-2 text-transparent'
            : 'text-foreground'
        )}
      >
        {title}
      </Heading>
      {underline === true ? (
        <div className="h-1.5 w-24 rounded-full bg-gradient-to-r from-primary to-primary/40" />
      ) : null}
      {typeof subtitle === 'string' && subtitle.length > 0 ? (
        <p className="max-w-2xl text-lg text-muted-foreground">{subtitle}</p>
      ) : null}
    </div>
  )
}
