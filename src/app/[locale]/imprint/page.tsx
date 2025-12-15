'use server'

import type { JSX } from 'react'

import type { Metadata } from 'next'
import { type Locale } from 'next-intl'

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

  const translations: Translations<'imprint'> = await getTranslations({
    locale,
    namespace: 'imprint',
  })
  return {
    description: translations('description'),
    title: translations('title'),
  }
}

/* ---------------------------------- page ---------------------------------- */

type ImprintPageProperties = UnparsedLocalePageProperties

// eslint-disable-next-line max-lines-per-function
const ImprintPage: RoutePageFC<ImprintPageProperties> = async ({
  params,
}: PageParameters<ImprintPageProperties>): Promise<JSX.Element> => {
  const locale: Locale = await ensureLocaleFromParameters(params)

  setRequestLocale(locale)

  const translations: Translations<'imprint'> = await getTranslations({
    locale,
    namespace: 'imprint',
  })

  const lastUpdated: Date = new Date(siteConfig.legals.imprintLastChange)

  const imprintKeys: string[] = [
    'tmg',
    'contact',
    'vat',
    'mstv',
    'dispute',
    'socialMedia',
    'liabilityContent',
    'liabilityLinks',
    'copyright',
  ] as const

  // Render the content using rich text with component mappings
  const content: JSX.Element[] = imprintKeys.map(
    (key: string): JSX.Element => (
      <div key={key}>
        {translations.rich(key, {
          // Variables for placeholder replacement
          address: siteConfig.legals.address,
          country: translations('serverLocation'),
          email: siteConfig.email,
          name: siteConfig.fullName,
          profiles: [
            siteConfig.github ? `GitHub: ${siteConfig.github}` : null,
            typeof siteConfig.linkedin === 'string'
              ? `LinkedIn: ${siteConfig.linkedin}`
              : null,
            siteConfig.twitter ? `Twitter: ${siteConfig.twitter}` : null,
          ]
            .filter(Boolean)
            .join('\n'),
          secondContact: siteConfig.legals.secondContact,
          vatId: siteConfig.legals.vatId,

          // Use shared component mappings
          ...legalPageComponentMappings,
        })}
      </div>
    )
  )

  return (
    <LegalPageLayout locale={locale} title={translations('title')}>
      <div className="space-y-6">
        {content}
        <LastUpdateNotice lastUpdate={lastUpdated} locale={locale} />
      </div>
    </LegalPageLayout>
  )
}

export default ImprintPage
