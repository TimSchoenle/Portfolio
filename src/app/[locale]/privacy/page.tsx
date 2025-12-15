'use server'

import type { JSX } from 'react'

import type { Metadata } from 'next'
import { type Locale, type RichTagsFunction } from 'next-intl'

import { getTranslations, setRequestLocale } from 'next-intl/server'

import { LegalPageLayout } from '@/components/layout/legal-page-layout'
import LastUpdateNotice from '@/components/ui/last-update-notice'
import {
  ensureLocaleFromParameters,
  maybeLocaleFromParameters,
} from '@/i18n/locale'
import { siteConfig } from '@/lib/config'
import { legalPageComponentMappings } from '@/lib/i18n-legal-components'
import type { Translations, UnparsedLocalePageProperties } from '@/types/i18n'
import type {
  GenerateMetadataFC,
  PageParameters,
  RoutePageFC,
} from '@/types/page'

/* --------------------------------- meta --------------------------------- */

export const generateMetadata: GenerateMetadataFC<
  UnparsedLocalePageProperties
> = async ({
  params,
}: PageParameters<UnparsedLocalePageProperties>): Promise<Metadata> => {
  const locale: Locale | null = await maybeLocaleFromParameters(params)
  if (locale === null) {
    return {}
  }

  const translations: Translations<'privacy'> = await getTranslations({
    locale,
    namespace: 'privacy',
  })
  return {
    description: translations('description'),
    title: translations('title'),
  }
}

/* ---------------------------------- page ---------------------------------- */

type PrivacyPageProperties = UnparsedLocalePageProperties

const PrivacyPolicyPage: RoutePageFC<PrivacyPageProperties> = async ({
  params,
}: PageParameters<PrivacyPageProperties>): Promise<JSX.Element> => {
  const locale: Locale = await ensureLocaleFromParameters(params)

  setRequestLocale(locale)

  const translations: Translations<'privacy'> = await getTranslations({
    locale,
    namespace: 'privacy',
  })

  const lastUpdated: Date = new Date(siteConfig.legals.privacyPolicyLastChange)

  // Prepare variables for rich text rendering
  const variables: Record<string, Date | RichTagsFunction | number | string> = {
    // Variables for placeholder replacement
    cloudflarePolicyUrl: siteConfig.legals.cloudflare.policyUrl,
    cloudflareProvider: siteConfig.legals.cloudflare.address,
    controllerAddress: siteConfig.legals.address,
    controllerEmail: siteConfig.email,
    controllerName: siteConfig.fullName,
    hostingAddress: siteConfig.legals.hosting.address,
    hostingName: siteConfig.legals.hosting.name,
    hostingPolicyUrl: siteConfig.legals.hosting.policyUrl,
    logRetentionDays: siteConfig.legals.logRetentionDays,
    secondContact: siteConfig.legals.secondContact,
    serverLocation: translations('serverLocation'),

    // Use shared component mappings
    ...legalPageComponentMappings,
  }

  return (
    <LegalPageLayout locale={locale} title={translations('title')}>
      <div className="space-y-6">
        <div>{translations.rich('controller', variables)}</div>
        <div>{translations.rich('general', variables)}</div>
        <div>{translations.rich('hosting', variables)}</div>
        <div>{translations.rich('encryption', variables)}</div>
        <div>{translations.rich('logs', variables)}</div>
        <div>{translations.rich('cloudflare', variables)}</div>
        <div>{translations.rich('contact', variables)}</div>
        <div>{translations.rich('rights', variables)}</div>
        <div>{translations.rich('cookies', variables)}</div>
        <div>{translations.rich('noTracking', variables)}</div>
        <div>{translations.rich('changes', variables)}</div>
        <LastUpdateNotice lastUpdate={lastUpdated} locale={locale} />
      </div>
    </LegalPageLayout>
  )
}

export default PrivacyPolicyPage
