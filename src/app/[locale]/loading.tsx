'use client'

import { type JSX } from 'react'

import { useTranslations } from 'next-intl'

import type { PageFC } from '@/types/fc'
import type { Translations } from '@/types/i18n'

const Loading: PageFC = (): JSX.Element => {
  const translations: Translations<'loading'> = useTranslations('loading')

  return (
    <div className="flex min-h-screen items-center justify-center bg-background">
      <div className="space-y-4 text-center">
        <div className="relative mx-auto h-16 w-16">
          <div className="absolute inset-0 rounded-full border-4 border-primary/20" />
          <div className="absolute inset-0 animate-spin rounded-full border-4 border-primary border-t-transparent" />
        </div>
        <p className="animate-pulse text-sm text-muted-foreground">
          {translations('title')}
        </p>
      </div>
    </div>
  )
}

export default Loading
