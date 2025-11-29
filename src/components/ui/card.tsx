import type { ComponentProps, JSX } from 'react'

import { cva } from 'class-variance-authority'

import { cn } from '@/lib/utilities'
import type { FCWithChildren } from '@/types/fc'

/* -------------------------------- Variants -------------------------------- */

// eslint-disable-next-line @typescript-eslint/typedef
const cardVariants = cva(
  // Base styles applied to all cards
  'flex flex-col gap-6 rounded-xl border bg-card py-6 text-card-foreground shadow-sm',
  {
    defaultVariants: {
      decorative: 'none',
      hover: 'none',
      variant: 'default',
    },
    variants: {
      // Decorative elements that can be added
      decorative: {
        none: '',
        overlay:
          'group relative overflow-hidden before:absolute before:inset-0 before:-z-10 before:bg-gradient-to-br before:from-primary/10 before:via-transparent before:to-primary/5 before:opacity-0 before:transition-opacity before:duration-500 hover:before:opacity-100',
        pattern:
          'relative overflow-hidden before:absolute before:inset-0 before:-z-10 before:bg-[linear-gradient(to_right,#ffffff08_1px,transparent_1px),linear-gradient(to_bottom,#ffffff08_1px,transparent_1px)] before:bg-[size:16px_16px] before:opacity-50',
        premium:
          'group relative overflow-hidden before:absolute before:inset-0 before:-z-10 before:bg-gradient-to-br before:from-primary/10 before:via-transparent before:to-primary/5 before:opacity-0 before:transition-opacity before:duration-500 hover:before:opacity-100 after:absolute after:top-0 after:left-0 after:h-1 after:w-0 after:bg-gradient-to-r after:from-primary after:to-primary/40 after:transition-all after:duration-500 group-hover:after:w-full',
        topBar:
          'group relative overflow-hidden after:absolute after:top-0 after:left-0 after:h-1 after:w-0 after:bg-gradient-to-r after:from-primary after:to-primary/40 after:transition-all after:duration-500 group-hover:after:w-full',
      },
      // Hover effects with varying intensities
      hover: {
        intense:
          'border-2 transition-all duration-300 hover:scale-105 hover:border-primary/50 hover:shadow-2xl',
        moderate:
          'border-2 transition-all duration-300 hover:border-primary/50 hover:shadow-xl',
        none: '',
        subtle:
          'border-2 transition-all duration-300 hover:border-primary/50 hover:shadow-lg',
      },
      // Card style variants
      variant: {
        default: '',
        interactive: 'group relative overflow-hidden',
        premium:
          'group relative overflow-hidden bg-gradient-to-br from-background to-muted/30 backdrop-blur-sm',
        stats:
          'group relative overflow-hidden bg-gradient-to-br from-background to-muted/20 backdrop-blur-sm',
      },
    },
  }
)

/* ------------------------------- Components ------------------------------- */
export const CARD_DECORATIONS: {
  readonly NONE: 'none'
  readonly OVERLAY: 'overlay'
  readonly PATTERN: 'pattern'
  readonly PREMIUM: 'premium'
  readonly TOP_BAR: 'topBar'
} = {
  NONE: 'none',
  OVERLAY: 'overlay',
  PATTERN: 'pattern',
  PREMIUM: 'premium',
  TOP_BAR: 'topBar',
} as const

export const CARD_HOVERS: {
  readonly INTENSE: 'intense'
  readonly MODERATE: 'moderate'
  readonly NONE: 'none'
  readonly SUBTLE: 'subtle'
} = {
  INTENSE: 'intense',
  MODERATE: 'moderate',
  NONE: 'none',
  SUBTLE: 'subtle',
} as const

export const CARD_VARIANTS: {
  readonly DEFAULT: 'default'
  readonly INTERACTIVE: 'interactive'
  readonly PREMIUM: 'premium'
  readonly STATS: 'stats'
} = {
  DEFAULT: 'default',
  INTERACTIVE: 'interactive',
  PREMIUM: 'premium',
  STATS: 'stats',
} as const

interface CardOwnProperties {
  readonly decorative?: (typeof CARD_DECORATIONS)[keyof typeof CARD_DECORATIONS]
  readonly hover?: (typeof CARD_HOVERS)[keyof typeof CARD_HOVERS]
  readonly variant?: (typeof CARD_VARIANTS)[keyof typeof CARD_VARIANTS]
}

type CardProperties = CardOwnProperties & ComponentProps<'div'>

type CardHeaderProperties = ComponentProps<'div'>
type CardTitleProperties = ComponentProps<'div'>
type CardDescriptionProperties = ComponentProps<'div'>
type CardActionProperties = ComponentProps<'div'>
type CardContentProperties = ComponentProps<'div'>
type CardFooterProperties = ComponentProps<'div'>

const Card: FCWithChildren<CardProperties> = ({
  children,
  className,
  decorative,
  hover,
  variant,
  ...properties
}: CardProperties): JSX.Element => {
  return (
    <div
      className={cn(cardVariants({ decorative, hover, variant }), className)}
      data-slot="card"
      {...properties}
    >
      {children}
    </div>
  )
}

const CardHeader: FCWithChildren<CardHeaderProperties> = ({
  className,
  ...properties
}: CardHeaderProperties): JSX.Element => {
  return (
    <div
      className={cn(
        '@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-2 px-6 has-data-[slot=card-action]:grid-cols-[1fr_auto] [.border-b]:pb-6',
        className
      )}
      data-slot="card-header"
      {...properties}
    />
  )
}

const CardTitle: FCWithChildren<CardTitleProperties> = ({
  className,
  ...properties
}: CardTitleProperties): JSX.Element => {
  return (
    <div
      className={cn('leading-none font-semibold', className)}
      data-slot="card-title"
      {...properties}
    />
  )
}

const CardDescription: FCWithChildren<CardDescriptionProperties> = ({
  className,
  ...properties
}: CardDescriptionProperties): JSX.Element => {
  return (
    <div
      className={cn('text-sm text-muted-foreground', className)}
      data-slot="card-description"
      {...properties}
    />
  )
}

const CardAction: FCWithChildren<CardActionProperties> = ({
  className,
  ...properties
}: CardActionProperties): JSX.Element => {
  return (
    <div
      className={cn(
        'col-start-2 row-span-2 row-start-1 self-start justify-self-end',
        className
      )}
      data-slot="card-action"
      {...properties}
    />
  )
}

const CardContent: FCWithChildren<CardContentProperties> = ({
  className,
  ...properties
}: CardContentProperties): JSX.Element => {
  return (
    <div
      className={cn('px-6', className)}
      data-slot="card-content"
      {...properties}
    />
  )
}

const CardFooter: FCWithChildren<CardFooterProperties> = ({
  className,
  ...properties
}: CardFooterProperties): JSX.Element => {
  return (
    <div
      className={cn('flex items-center px-6 [.border-t]:pt-6', className)}
      data-slot="card-footer"
      {...properties}
    />
  )
}

export {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
}
