import type { ComponentProps, JSX } from 'react'

import { cn } from '@/lib/utils'
import type { FCWithChildren } from '@/types/fc'

type CardProps = ComponentProps<'div'>
type CardHeaderProps = ComponentProps<'div'>
type CardTitleProps = ComponentProps<'div'>
type CardDescriptionProps = ComponentProps<'div'>
type CardActionProps = ComponentProps<'div'>
type CardContentProps = ComponentProps<'div'>
type CardFooterProps = ComponentProps<'div'>

const Card: FCWithChildren<CardProps> = ({
  className,
  ...props
}: CardProps): JSX.Element => {
  return (
    <div
      className={cn(
        'bg-card text-card-foreground flex flex-col gap-6 rounded-xl border py-6 shadow-sm',
        className
      )}
      data-slot="card"
      {...props}
    />
  )
}

const CardHeader: FCWithChildren<CardHeaderProps> = ({
  className,
  ...props
}: CardHeaderProps): JSX.Element => {
  return (
    <div
      className={cn(
        '@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-2 px-6 has-data-[slot=card-action]:grid-cols-[1fr_auto] [.border-b]:pb-6',
        className
      )}
      data-slot="card-header"
      {...props}
    />
  )
}

const CardTitle: FCWithChildren<CardTitleProps> = ({
  className,
  ...props
}: CardTitleProps): JSX.Element => {
  return (
    <div
      className={cn('leading-none font-semibold', className)}
      data-slot="card-title"
      {...props}
    />
  )
}

const CardDescription: FCWithChildren<CardDescriptionProps> = ({
  className,
  ...props
}: CardDescriptionProps): JSX.Element => {
  return (
    <div
      className={cn('text-muted-foreground text-sm', className)}
      data-slot="card-description"
      {...props}
    />
  )
}

const CardAction: FCWithChildren<CardActionProps> = ({
  className,
  ...props
}: CardActionProps): JSX.Element => {
  return (
    <div
      className={cn(
        'col-start-2 row-span-2 row-start-1 self-start justify-self-end',
        className
      )}
      data-slot="card-action"
      {...props}
    />
  )
}

const CardContent: FCWithChildren<CardContentProps> = ({
  className,
  ...props
}: CardContentProps): JSX.Element => {
  return (
    <div
      className={cn('px-6', className)}
      data-slot="card-content"
      {...props}
    />
  )
}

const CardFooter: FCWithChildren<CardFooterProps> = ({
  className,
  ...props
}: CardFooterProps): JSX.Element => {
  return (
    <div
      className={cn('flex items-center px-6 [.border-t]:pt-6', className)}
      data-slot="card-footer"
      {...props}
    />
  )
}

export {
  Card,
  CardHeader,
  CardFooter,
  CardTitle,
  CardAction,
  CardDescription,
  CardContent,
}
