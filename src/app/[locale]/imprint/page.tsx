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
import {
  legalPageComponentMappings,
  type LegalRichTagsFunctionMappers,
} from '@/lib/i18n-legal-components'
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

  // Prepare variables for rich text rendering
  const variables: Record<string, Date | RichTagsFunction | number | string> = {
    // Variables for placeholder replacement
    address: siteConfig.legals.address,
    country: translations('serverLocation'),
    email: siteConfig.email,
    name: siteConfig.fullName,
    profiles: (): JSX.Element => {
      const { link }: LegalRichTagsFunctionMappers = legalPageComponentMappings

      return (
        <>
          {siteConfig.socials.github ? (
            <div>
              {'GitHub: '}
              {link(siteConfig.socials.github)}
            </div>
          ) : null}
          {typeof siteConfig.socials.linkedin === 'string' ? (
            <div>
              {'LinkedIn: '}
              {link(siteConfig.socials.linkedin)}
            </div>
          ) : null}
        </>
      )
    },
    secondContact: siteConfig.legals.secondContact,
    vatId: siteConfig.legals.vatId,

    // Use shared component mappings
    ...legalPageComponentMappings,
  }

  return (
    <LegalPageLayout locale={locale} title={translations('title')}>
      <div className="space-y-6">
        <div>{translations.rich('tmg', variables)}</div>
        <div>{translations.rich('contact', variables)}</div>
        <div>{translations.rich('vat', variables)}</div>
        <div>{translations.rich('mstv', variables)}</div>
        <div>{translations.rich('dispute', variables)}</div>
        <div>{translations.rich('socialMedia', variables)}</div>
        <div>{translations.rich('liabilityContent', variables)}</div>
        <div>{translations.rich('liabilityLinks', variables)}</div>
        <div>{translations.rich('copyright', variables)}</div>
        <LastUpdateNotice lastUpdate={lastUpdated} locale={locale} />
      </div>
    </LegalPageLayout>
  )
}

export default ImprintPage
