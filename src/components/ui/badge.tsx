import { Slot } from '@radix-ui/react-slot'
import { cva } from 'class-variance-authority'
import * as React from 'react'

import { cn } from '@/lib/utils'
import type { FCWithChildren } from '@/types/fc'

/* ── types ───────────────────────────────────────────────────────────── */

type BadgeVariant = 'default' | 'secondary' | 'destructive' | 'outline'

interface BadgeVariantProps {
  readonly variant?: BadgeVariant
}

type BadgeClassGenerator = (options?: BadgeVariantProps) => string

export interface BadgeProps
  extends React.ComponentProps<'span'>,
    BadgeVariantProps {
  readonly asChild?: boolean
}

/* ── implementations ─────────────────────────────────────────────────── */
export const badgeVariants: BadgeClassGenerator = cva(
  'inline-flex items-center justify-center rounded-md border px-2 py-0.5 text-xs font-medium w-fit whitespace-nowrap shrink-0 [&>svg]:size-3 gap-1 [&>svg]:pointer-events-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive transition-[color,box-shadow] overflow-hidden',
  {
    variants: {
      variant: {
        default:
          'border-transparent bg-primary text-primary-foreground [a&]:hover:bg-primary/90',
        secondary:
          'border-transparent bg-secondary text-secondary-foreground [a&]:hover:bg-secondary/90',
        destructive:
          'border-transparent bg-destructive text-white [a&]:hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/60',
        outline:
          'text-foreground [a&]:hover:bg-accent [a&]:hover:text-accent-foreground',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  }
)

export const Badge: FCWithChildren<BadgeProps> = ({
  className,
  variant,
  asChild = false,
  ...props
}: BadgeProps): React.JSX.Element => {
  const Comp: React.ElementType = asChild ? Slot : 'span'

  // With exactOptionalPropertyTypes: omit the key when undefined.
  const variantArg: BadgeVariantProps | undefined =
    variant !== undefined ? { variant } : undefined

  const classes: string = cn(badgeVariants(variantArg), className)

  return <Comp className={classes} data-slot="badge" {...props} />
}
