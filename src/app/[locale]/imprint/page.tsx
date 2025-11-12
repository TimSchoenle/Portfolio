'use server'

import type { JSX } from 'react'

import type { Metadata } from 'next'
import { type Locale } from 'next-intl'

import { getTranslations, setRequestLocale } from 'next-intl/server'

import { LegalPageLayout } from '@/components/legal-page-layout'
import { Heading } from '@/components/ui/heading'
import { siteConfig } from '@/config'
import {
  ensureLocaleFromParameters,
  maybeLocaleFromParameters,
} from '@/i18n/locale'
import type { FCStrict } from '@/types/fc'
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

/* ----------------------------- small components ----------------------------- */

interface AddressProperties {
  readonly country: string
  readonly name: string
}

const Address: FCStrict<AddressProperties> = ({
  country,
  name,
}: AddressProperties): JSX.Element => {
  return (
    <p className="text-muted-foreground">
      {name}
      <br />
      {country}
    </p>
  )
}

interface EmailLineProperties {
  readonly email: string
  readonly label: string
}

const EmailLine: FCStrict<EmailLineProperties> = ({
  email,
  label,
}: EmailLineProperties): JSX.Element => {
  return (
    <p className="text-muted-foreground">
      {label}
      {': '}
      <a className="text-primary hover:underline" href={`mailto:${email}`}>
        {email}
      </a>
    </p>
  )
}

interface SectionProperties {
  readonly body: string
  readonly title: string
}

const Section: FCStrict<SectionProperties> = ({
  body,
  title,
}: SectionProperties): JSX.Element => {
  return (
    <div>
      <Heading as="h2" className="mb-2 text-xl font-semibold">
        {title}
      </Heading>
      <p className="text-muted-foreground text-sm leading-relaxed">{body}</p>
    </div>
  )
}

/* ---------------------------------- page ---------------------------------- */

type ImprintPageProperties = UnparsedLocalePageProperties

const ImprintPage: RoutePageFC<ImprintPageProperties> = async ({
  params,
}: PageParameters<ImprintPageProperties>): Promise<JSX.Element> => {
  const locale: Locale = await ensureLocaleFromParameters(params)

  setRequestLocale(locale)

  const translations: Translations<'imprint'> = await getTranslations({
    locale,
    namespace: 'imprint',
  })
  const translationsContact: Translations<'contact'> = await getTranslations({
    locale,
    namespace: 'contact',
  })

  const ownerName: string = siteConfig.fullName
  const ownerCountry: string = 'Germany'

  return (
    <LegalPageLayout locale={locale} title={translations('title')}>
      <div>
        <Heading as="h2" className="mb-2 text-xl font-semibold">
          {translations('infoTitle')}
        </Heading>
        <Address country={ownerCountry} name={ownerName} />
      </div>

      <div>
        <Heading as="h2" className="mb-2 text-xl font-semibold">
          {translations('contactTitle')}
        </Heading>
        <EmailLine
          email={siteConfig.email}
          label={translationsContact('email')}
        />
      </div>

      <div>
        <Heading as="h2" className="mb-2 text-xl font-semibold">
          {translations('responsibleTitle')}
        </Heading>
        <Address country={ownerCountry} name={ownerName} />
      </div>

      <Section
        body={translations('liabilityContent')}
        title={translations('liabilityContentTitle')}
      />
      <Section
        body={translations('liabilityLinks')}
        title={translations('liabilityLinksTitle')}
      />
      <Section
        body={translations('copyright')}
        title={translations('copyrightTitle')}
      />
    </LegalPageLayout>
  )
}

export default ImprintPage
