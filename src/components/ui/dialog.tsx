import * as DialogPrimitive from '@radix-ui/react-dialog'
import { XIcon } from 'lucide-react'
import type { ComponentProps, JSX } from 'react'

import { cn } from '@/lib/utils'
import type { FCWithChildren } from '@/types/fc'

/* ───────────── props ───────────── */

type DialogRootProps = ComponentProps<typeof DialogPrimitive.Root>
type DialogTriggerProps = ComponentProps<typeof DialogPrimitive.Trigger>
type DialogPortalProps = ComponentProps<typeof DialogPrimitive.Portal>
type DialogCloseProps = ComponentProps<typeof DialogPrimitive.Close>
type DialogOverlayProps = ComponentProps<typeof DialogPrimitive.Overlay>
interface DialogContentBaseProps
  extends ComponentProps<typeof DialogPrimitive.Content> {
  readonly showCloseButton?: boolean
}
type DialogHeaderProps = ComponentProps<'div'>
type DialogFooterProps = ComponentProps<'div'>
type DialogTitleProps = ComponentProps<typeof DialogPrimitive.Title>
type DialogDescriptionProps = ComponentProps<typeof DialogPrimitive.Description>

/* ───────────── components ───────────── */

const Dialog: FCWithChildren<DialogRootProps> = (
  props: DialogRootProps
): JSX.Element => <DialogPrimitive.Root data-slot="dialog" {...props} />

const DialogTrigger: FCWithChildren<DialogTriggerProps> = (
  props: DialogTriggerProps
): JSX.Element => (
  <DialogPrimitive.Trigger data-slot="dialog-trigger" {...props} />
)

const DialogPortal: FCWithChildren<DialogPortalProps> = (
  props: DialogPortalProps
): JSX.Element => (
  <DialogPrimitive.Portal data-slot="dialog-portal" {...props} />
)

const DialogClose: FCWithChildren<DialogCloseProps> = (
  props: DialogCloseProps
): JSX.Element => <DialogPrimitive.Close data-slot="dialog-close" {...props} />

const DialogOverlay: FCWithChildren<DialogOverlayProps> = ({
  className,
  ...props
}: DialogOverlayProps): JSX.Element => {
  return (
    <DialogPrimitive.Overlay
      className={cn(
        'data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 fixed inset-0 z-50 bg-black/50',
        className
      )}
      data-slot="dialog-overlay"
      {...props}
    />
  )
}

const DialogContent: FCWithChildren<DialogContentBaseProps> = ({
  className,
  children,
  showCloseButton = true,
  ...props
}: DialogContentBaseProps): JSX.Element => {
  return (
    <DialogPortal data-slot="dialog-portal">
      <DialogOverlay />
      <DialogPrimitive.Content
        className={cn(
          'bg-background data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 fixed top-[50%] left-[50%] z-50 grid w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] gap-4 rounded-lg border p-6 shadow-lg duration-200 sm:max-w-lg',
          className
        )}
        data-slot="dialog-content"
        {...props}
      >
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

const DialogHeader: FCWithChildren<DialogHeaderProps> = ({
  className,
  ...props
}: DialogHeaderProps): JSX.Element => {
  return (
    <div
      className={cn('flex flex-col gap-2 text-center sm:text-left', className)}
      data-slot="dialog-header"
      {...props}
    />
  )
}

const DialogFooter: FCWithChildren<DialogFooterProps> = ({
  className,
  ...props
}: DialogFooterProps): JSX.Element => {
  return (
    <div
      className={cn(
        'flex flex-col-reverse gap-2 sm:flex-row sm:justify-end',
        className
      )}
      data-slot="dialog-footer"
      {...props}
    />
  )
}

const DialogTitle: FCWithChildren<DialogTitleProps> = ({
  className,
  ...props
}: DialogTitleProps): JSX.Element => {
  return (
    <DialogPrimitive.Title
      className={cn('text-lg leading-none font-semibold', className)}
      data-slot="dialog-title"
      {...props}
    />
  )
}

const DialogDescription: FCWithChildren<DialogDescriptionProps> = ({
  className,
  ...props
}: DialogDescriptionProps): JSX.Element => {
  return (
    <DialogPrimitive.Description
      className={cn('text-muted-foreground text-sm', className)}
      data-slot="dialog-description"
      {...props}
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
