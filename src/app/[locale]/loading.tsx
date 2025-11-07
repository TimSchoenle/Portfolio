'use client'

import { type JSX } from 'react'

import { useTranslations } from 'next-intl'

import type { PageFC } from '@/types/fc'
import type { Translations } from '@/types/i18n'

const Loading: PageFC = (): JSX.Element => {
  const translations: Translations<'loading'> = useTranslations('loading')

  return (
    <div className="bg-background flex min-h-screen items-center justify-center">
      <div className="space-y-4 text-center">
        <div className="relative mx-auto h-16 w-16">
          <div className="border-primary/20 absolute inset-0 rounded-full border-4" />
          <div className="border-primary absolute inset-0 animate-spin rounded-full border-4 border-t-transparent" />
        </div>
        <p className="text-muted-foreground animate-pulse text-sm">
          {translations('title')}
        </p>
      </div>
    </div>
  )
}

export default Loading
