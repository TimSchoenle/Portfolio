'use client'

import * as React from 'react'

import * as LabelPrimitive from '@radix-ui/react-label'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utilities'

// eslint-disable-next-line @typescript-eslint/typedef
const labelVariants = cva(
  'text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70'
)

// eslint-disable-next-line @typescript-eslint/typedef
const Label = React.forwardRef<
  React.ComponentRef<typeof LabelPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof LabelPrimitive.Root> &
    VariantProps<typeof labelVariants>
  // eslint-disable-next-line @typescript-eslint/typedef
>(
  ({ className, ...properties }, reference): React.JSX.Element => (
    <LabelPrimitive.Root
      className={cn(labelVariants(), className)}
      ref={reference}
      {...properties}
    />
  )
)
Label.displayName = LabelPrimitive.Root.displayName

export { Label }
