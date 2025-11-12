import type { ComponentProps, JSX } from 'react'

import { cn } from '@/shared/utils'
import type { FCWithChildren } from '@/types/fc'

type CardProperties = ComponentProps<'div'>
type CardHeaderProperties = ComponentProps<'div'>
type CardTitleProperties = ComponentProps<'div'>
type CardDescriptionProperties = ComponentProps<'div'>
type CardActionProperties = ComponentProps<'div'>
type CardContentProperties = ComponentProps<'div'>
type CardFooterProperties = ComponentProps<'div'>

const Card: FCWithChildren<CardProperties> = ({
  className,
  ...properties
}: CardProperties): JSX.Element => {
  return (
    <div
      className={cn(
        'bg-card text-card-foreground flex flex-col gap-6 rounded-xl border py-6 shadow-sm',
        className
      )}
      data-slot="card"
      {...properties}
    />
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
      className={cn('text-muted-foreground text-sm', className)}
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
