'use server'

import type { ReactNode } from 'react'
import type { Locale } from 'next-intl'
import { getTranslations } from 'next-intl/server'
import ClientConsentManager from '@/components/consent-manager/client-consent-manager'

export default async function ConsentManager({
  children,
  locale,
}: {
  children: ReactNode
  locale: Locale
}) {
  const t = await getTranslations({ locale: locale, namespace: 'cookies' })
  const translations = [
    locale,
    {
      common: {
        acceptAll: t('common.acceptAll'),
        rejectAll: t('common.rejectAll'),
        customize: t('common.customize'),
        save: t('common.save'),
      },
      cookieBanner: {
        title: t('banner.title'),
        description: t('banner.description'),
      },
      consentManagerDialog: {
        title: t('consentManagerDialog.title'),
        description: t('consentManagerDialog.description'),
      },
      consentTypes: {
        necessary: {
          title: t('consentTypes.necessary.title'),
          description: t('consentTypes.necessary.description'),
        },
        experience: {
          title: t('consentTypes.experience.title'),
          description: t('consentTypes.experience.description'),
        },
        functionality: {
          title: t('consentTypes.functionality.title'),
          description: t('consentTypes.functionality.description'),
        },
        marketing: {
          title: t('consentTypes.marketing.title'),
          description: t('consentTypes.marketing.description'),
        },
        measurement: {
          title: t('consentTypes.measurement.title'),
          description: t('consentTypes.measurement.description'),
        },
      },
      frame: {
        // This needs to be a raw string for the builtin {category} placeholder to work
        title: t.raw('frame.title'),
        // This needs to be a raw string for the builtin {category} placeholder to work
        actionButton: t.raw('frame.actionButton'),
      },
    },
  ]

  return (
    <ClientConsentManager locale={locale} translations={translations}>
      {children}
    </ClientConsentManager>
  )
}
