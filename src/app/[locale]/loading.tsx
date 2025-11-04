'use client'

import { useTranslations } from 'next-intl'

export default function Loading() {
  const t = useTranslations('loading')

  return (
    <div className="bg-background flex min-h-screen items-center justify-center">
      <div className="space-y-4 text-center">
        <div className="relative mx-auto h-16 w-16">
          <div className="border-primary/20 absolute inset-0 rounded-full border-4" />
          <div className="border-primary absolute inset-0 animate-spin rounded-full border-4 border-t-transparent" />
        </div>
        <p className="text-muted-foreground animate-pulse text-sm">
          {t('title')}
        </p>
      </div>
    </div>
  )
}
