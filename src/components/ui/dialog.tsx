'use client'

import type { ComponentProps, JSX, ReactNode } from 'react'

import * as DialogPrimitive from '@radix-ui/react-dialog'
import { XIcon } from 'lucide-react'

import { cn } from '@/lib/utilities'
import type { FCWithChildren } from '@/types/fc'

/* ───────────── props ───────────── */

type DialogRootProperties = ComponentProps<typeof DialogPrimitive.Root>
type DialogTriggerProperties = ComponentProps<typeof DialogPrimitive.Trigger>
type DialogPortalProperties = ComponentProps<typeof DialogPrimitive.Portal>
type DialogCloseProperties = ComponentProps<typeof DialogPrimitive.Close>
type DialogOverlayProperties = ComponentProps<typeof DialogPrimitive.Overlay>
interface DialogContentBaseProperties
  extends ComponentProps<typeof DialogPrimitive.Content> {
  readonly showCloseButton?: boolean
}

interface DialogContentProperties extends DialogContentBaseProperties {
  readonly description?: ReactNode
  readonly title?: string
}

type DialogHeaderProperties = ComponentProps<'div'>
type DialogFooterProperties = ComponentProps<'div'>
type DialogTitleProperties = ComponentProps<typeof DialogPrimitive.Title>
type DialogDescriptionProperties = ComponentProps<
  typeof DialogPrimitive.Description
>

/* ───────────── components ───────────── */
const Dialog: FCWithChildren<DialogRootProperties> = (
  properties: DialogRootProperties
): JSX.Element => <DialogPrimitive.Root data-slot="dialog" {...properties} />

const DialogTrigger: FCWithChildren<DialogTriggerProperties> = (
  properties: DialogTriggerProperties
): JSX.Element => (
  <DialogPrimitive.Trigger data-slot="dialog-trigger" {...properties} />
)

const DialogPortal: FCWithChildren<DialogPortalProperties> = (
  properties: DialogPortalProperties
): JSX.Element => (
  <DialogPrimitive.Portal data-slot="dialog-portal" {...properties} />
)

const DialogClose: FCWithChildren<DialogCloseProperties> = (
  properties: DialogCloseProperties
): JSX.Element => (
  <DialogPrimitive.Close data-slot="dialog-close" {...properties} />
)

const DialogOverlay: FCWithChildren<DialogOverlayProperties> = ({
  className,
  ...properties
}: DialogOverlayProperties): JSX.Element => {
  return (
    <DialogPrimitive.Overlay
      className={cn(
        'data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 fixed inset-0 z-50 bg-black/50',
        className
      )}
      data-slot="dialog-overlay"
      {...properties}
    />
  )
}

// eslint-disable-next-line max-lines-per-function
const DialogContent: FCWithChildren<DialogContentProperties> = ({
  children,
  className,
  description,
  showCloseButton = true,
  title,
  ...properties
}: DialogContentProperties): JSX.Element => {
  return (
    <DialogPortal data-slot="dialog-portal">
      <DialogOverlay />
      <DialogPrimitive.Content
        className={cn(
          'bg-background data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 fixed top-[50%] left-[50%] z-50 grid w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] gap-4 rounded-lg border p-6 shadow-lg duration-200 sm:max-w-lg',
          className
        )}
        data-slot="dialog-content"
        {...properties}
      >
        {title !== undefined || description !== undefined ? (
          <div
            className="flex flex-col gap-2 text-center sm:text-left"
            data-slot="dialog-header"
          >
            {title !== undefined ? (
              <DialogPrimitive.Title
                /** Force a single, semantic heading; avoid asChild & nested headings */
                asChild={false}
                className="text-lg leading-none font-semibold"
                data-slot="dialog-title"
                suppressHydrationWarning={true}
              >
                {title}
              </DialogPrimitive.Title>
            ) : null}
            {description !== undefined ? (
              <DialogPrimitive.Description
                className="text-muted-foreground text-sm"
                data-slot="dialog-description"
              >
                {description}
              </DialogPrimitive.Description>
            ) : null}
          </div>
        ) : null}

        {children}

        {showCloseButton ? (
          <DialogPrimitive.Close
            aria-label="Close"
            className="ring-offset-background focus:ring-ring data-[state=open]:bg-accent data-[state=open]:text-muted-foreground absolute top-4 right-4 rounded-xs opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4"
            data-slot="dialog-close"
          >
            <XIcon aria-hidden="true" />
          </DialogPrimitive.Close>
        ) : null}
      </DialogPrimitive.Content>
    </DialogPortal>
  )
}

const DialogHeader: FCWithChildren<DialogHeaderProperties> = ({
  className,
  ...properties
}: DialogHeaderProperties): JSX.Element => {
  return (
    <div
      className={cn('flex flex-col gap-2 text-center sm:text-left', className)}
      data-slot="dialog-header"
      {...properties}
    />
  )
}

const DialogFooter: FCWithChildren<DialogFooterProperties> = ({
  className,
  ...properties
}: DialogFooterProperties): JSX.Element => {
  return (
    <div
      className={cn(
        'flex flex-col-reverse gap-2 sm:flex-row sm:justify-end',
        className
      )}
      data-slot="dialog-footer"
      {...properties}
    />
  )
}

const DialogTitle: FCWithChildren<DialogTitleProperties> = ({
  className,
  ...properties
}: DialogTitleProperties): JSX.Element => {
  return (
    <DialogPrimitive.Title
      asChild={false}
      className={cn('text-lg leading-none font-semibold', className)}
      data-slot="dialog-title"
      suppressHydrationWarning={true}
      {...properties}
    />
  )
}

const DialogDescription: FCWithChildren<DialogDescriptionProperties> = ({
  className,
  ...properties
}: DialogDescriptionProperties): JSX.Element => {
  return (
    <DialogPrimitive.Description
      className={cn('text-muted-foreground text-sm', className)}
      data-slot="dialog-description"
      {...properties}
    />
  )
}

export {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogOverlay,
  DialogPortal,
  DialogTitle,
  DialogTrigger,
}
