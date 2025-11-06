'use server'

import type { Metadata } from 'next'
import { type Locale } from 'next-intl'
import { getTranslations, setRequestLocale } from 'next-intl/server'
import type { JSX } from 'react'

import LegalPageLayout from '@/components/legal-page-layout'
import { ensureLocaleFromParams, maybeLocaleFromParams } from '@/i18n/locale'
import { siteConfig } from '@/lib/config'
import type { FCAsync, FCStrict, NoChildren } from '@/types/fc'
import type { Translations, UnparsedLocalePageProps } from '@/types/i18n'
/* --------------------------------- meta --------------------------------- */

export async function generateMetadata({
  params,
}: {
  params: Promise<UnparsedLocalePageProps>
}): Promise<Metadata> {
  const locale: Locale | null = await maybeLocaleFromParams(params)
  if (locale === null) {
    return {}
  }

  const t: Translations = await getTranslations({
    locale,
    namespace: 'imprint',
  })
  return { title: t('title'), description: t('description') }
}

/* ----------------------------- small components ----------------------------- */

interface AddressProps {
  readonly name: string
  readonly country: string
}

const Address: FCStrict<AddressProps> = ({
  name,
  country,
}: AddressProps): JSX.Element => {
  return (
    <p className="text-muted-foreground">
      {name}
      <br />
      {country}
    </p>
  )
}

interface EmailLineProps {
  readonly label: string
  readonly email: string
}

const EmailLine: FCStrict<EmailLineProps> = ({
  label,
  email,
}: EmailLineProps): JSX.Element => {
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

interface SectionProps {
  readonly title: string
  readonly body: string
}

const Section: FCStrict<SectionProps> = ({
  title,
  body,
}: SectionProps): JSX.Element => {
  return (
    <div>
      <h2 className="mb-2 text-xl font-semibold">{title}</h2>
      <p className="text-muted-foreground text-sm leading-relaxed">{body}</p>
    </div>
  )
}

/* ---------------------------------- page ---------------------------------- */

interface ImprintPageProps extends NoChildren {
  readonly params: Promise<UnparsedLocalePageProps>
}

const ImprintPage: FCAsync<ImprintPageProps> = async ({
  params,
}: ImprintPageProps): Promise<JSX.Element> => {
  const locale: Locale = await ensureLocaleFromParams(params)

  setRequestLocale(locale)

  const t: Translations = await getTranslations({
    locale,
    namespace: 'imprint',
  })
  const tContact: Translations = await getTranslations({
    locale,
    namespace: 'contact',
  })

  const ownerName: string = siteConfig.fullName
  // eslint-disable-next-line @typescript-eslint/no-inferrable-types
  const ownerCountry: string = 'Germany'

  return (
    <LegalPageLayout locale={locale} title={t('title')}>
      <div>
        <h2 className="mb-2 text-xl font-semibold">{t('infoTitle')}</h2>
        <Address country={ownerCountry} name={ownerName} />
      </div>

      <div>
        <h2 className="mb-2 text-xl font-semibold">{t('contactTitle')}</h2>
        <EmailLine email={siteConfig.email} label={tContact('email')} />
      </div>

      <div>
        <h2 className="mb-2 text-xl font-semibold">{t('responsibleTitle')}</h2>
        <Address country={ownerCountry} name={ownerName} />
      </div>

      <Section
        body={t('liabilityContent')}
        title={t('liabilityContentTitle')}
      />
      <Section body={t('liabilityLinks')} title={t('liabilityLinksTitle')} />
      <Section body={t('copyright')} title={t('copyrightTitle')} />
    </LegalPageLayout>
  )
}

export default ImprintPage
