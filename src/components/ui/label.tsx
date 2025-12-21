'use client'

import * as React from 'react'
import type { ForwardedRef } from 'react'

import * as LabelPrimitive from '@radix-ui/react-label'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utilities'

const labelVariants: ReturnType<typeof cva> = cva(
  'text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70'
)

const Label: React.ForwardRefExoticComponent<
  React.ComponentPropsWithoutRef<typeof LabelPrimitive.Root> &
    React.RefAttributes<HTMLLabelElement> &
    VariantProps<typeof labelVariants>
> = React.forwardRef<
  React.ComponentRef<typeof LabelPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof LabelPrimitive.Root> &
    VariantProps<typeof labelVariants>
>(
  (
    {
      className,
      ...properties
    }: React.ComponentPropsWithoutRef<typeof LabelPrimitive.Root> &
      VariantProps<typeof labelVariants>,
    reference: ForwardedRef<HTMLLabelElement>
  ): React.JSX.Element => (
    <LabelPrimitive.Root
      className={cn(labelVariants(), className)}
      ref={reference}
      {...properties}
    />
  )
)
Label.displayName = LabelPrimitive.Root.displayName

export { Label }
