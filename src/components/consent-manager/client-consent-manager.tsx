'use client'

import {
  ConsentManagerDialog,
  type ConsentManagerOptions,
  ConsentManagerProvider,
  CookieBanner,
} from '@c15t/nextjs'
import React, { useMemo } from 'react'
import HideConsentBrandingFooter from '@/components/consent-manager/hide-consent-branding-footer'
import type { Locale } from 'next-intl'

interface Props {
  locale: Locale
  translations: object // the object you already built on the server
  children: React.ReactNode
}

export default function ClientConsentManager({
  locale,
  translations,
  children,
}: Props) {
  // Memoize provider options to avoid unnecessary re-renders
  const providerOptions: ConsentManagerOptions = useMemo(
    () => ({
      mode: 'offline',
      consentCategories: ['necessary'],
      translations: {
        defaultLanguage: locale,
        translations: translations,
        disableAutoLanguageSwitch: false,
      },
    }),
    [locale, translations]
  )

  console.log(providerOptions)

  return (
    <ConsentManagerProvider options={providerOptions}>
      <CookieBanner />
      <ConsentManagerDialog />
      <HideConsentBrandingFooter />
      {children}
    </ConsentManagerProvider>
  )
}
