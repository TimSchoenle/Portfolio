'use server'

import type { Metadata } from 'next'
import { type Locale } from 'next-intl'
import { getTranslations, setRequestLocale } from 'next-intl/server'
import type { JSX } from 'react'

import LegalPageLayout from '@/components/legal-page-layout'
import { ensureLocaleFromParams, maybeLocaleFromParams } from '@/i18n/locale'
import type { FCAsync, FCStrict, NoChildren } from '@/types/fc'
import type { Translations, UnparsedLocalePageProps } from '@/types/i18n'

/* --------------------------------- metadata -------------------------------- */

interface PrivacyMetaParams {
  readonly params: Promise<UnparsedLocalePageProps>
}

type GenerateMeta = (a: PrivacyMetaParams) => Promise<Metadata>

export const generateMetadata: GenerateMeta = async (
  props: PrivacyMetaParams
): Promise<Metadata> => {
  const locale: Locale | null = await maybeLocaleFromParams(props.params)
  if (locale === null) {
    return {}
  }

  const t: Translations = await getTranslations({
    locale,
    namespace: 'privacy',
  })
  return { title: t('title'), description: t('description') }
}

/* ----------------------------- small components ---------------------------- */

interface ControllerBlockProps {
  readonly title: string
  readonly nameLabel: string
  readonly addressLabel: string
  readonly emailLabel: string
  readonly name: string
  readonly address: string
  readonly email: string
}

const ControllerBlock: FCStrict<ControllerBlockProps> = ({
  title,
  nameLabel,
  addressLabel,
  emailLabel,
  name,
  address,
  email,
}: ControllerBlockProps): JSX.Element => {
  return (
    <div>
      <h2 className="mb-2 text-xl font-semibold">{title}</h2>
      <p className="text-muted-foreground">
        <strong>{nameLabel}</strong> {name}
        <br />
        <strong>{addressLabel}</strong> {address}
        <br />
        <strong>{emailLabel}</strong>{' '}
        <a className="text-primary hover:underline" href={`mailto:${email}`}>
          {email}
        </a>
      </p>
    </div>
  )
}

interface CloudflareBlockProps {
  readonly title: string
  readonly pre: string
  readonly strong: string
  readonly post: string
  readonly provider: string
  readonly policyLink: string
  readonly text: string
}

const CloudflareBlock: FCStrict<CloudflareBlockProps> = ({
  title,
  pre,
  strong,
  post,
  provider,
  policyLink,
  text,
}: CloudflareBlockProps): JSX.Element => {
  return (
    <div>
      <h2 className="mb-2 text-xl font-semibold">{title}</h2>
      <p className="text-muted-foreground text-sm leading-relaxed">
        {pre} <strong>{strong}</strong> {post}
      </p>
      <p className="text-muted-foreground mt-1 text-sm leading-relaxed">
        {provider}
      </p>
      <p className="text-muted-foreground mt-1 text-sm leading-relaxed">
        <a
          className="text-primary hover:underline"
          href={policyLink}
          rel="noopener noreferrer"
          target="_blank"
        >
          {policyLink}
        </a>
      </p>
      <p className="text-muted-foreground mt-2 text-sm leading-relaxed">
        {text}
      </p>
    </div>
  )
}

interface SectionData {
  readonly title: string
  readonly text: string
}

interface SectionsProps {
  readonly sections: Record<string, SectionData>
}

const Sections: (props: SectionsProps) => JSX.Element[] = ({
  sections,
}: SectionsProps): JSX.Element[] => {
  const nodes: JSX.Element[] = []
  for (const [key, s] of Object.entries(sections)) {
    nodes.push(
      <div key={key}>
        <h2 className="mb-2 text-xl font-semibold">{s.title}</h2>
        <p className="text-muted-foreground text-sm leading-relaxed">
          {s.text}
        </p>
      </div>
    )
  }
  return nodes
}

/* ----------------------------------- page ---------------------------------- */

interface PrivacyPageProps extends NoChildren {
  readonly params: Promise<UnparsedLocalePageProps>
}

// eslint-disable-next-line max-lines-per-function
const PrivacyPolicyPage: FCAsync<PrivacyPageProps> = async ({
  params,
}: PrivacyPageProps): Promise<JSX.Element> => {
  const locale: Locale = await ensureLocaleFromParams(params)

  setRequestLocale(locale)

  const t: Translations = await getTranslations({
    locale,
    namespace: 'privacy',
  })

  // eslint-disable-next-line @typescript-eslint/typedef
  const nameLabel = 'Name:'
  // eslint-disable-next-line @typescript-eslint/typedef
  const addressLabel = 'Address:'
  // eslint-disable-next-line @typescript-eslint/typedef
  const emailLabel = 'Email:'

  const controller: Readonly<{
    title: string
    name: string
    address: string
    email: string
  }> = {
    title: t('controller.title'),
    name: t('controller.name'),
    address: t('controller.address'),
    email: t('controller.email'),
  }

  const cloudflare: CloudflareBlockProps = {
    title: t('cloudflare.title'),
    pre: t('cloudflare.pre'),
    strong: t('cloudflare.strong'),
    post: t('cloudflare.post'),
    provider: t('cloudflare.provider'),
    policyLink: t('cloudflare.policyLink'),
    text: t('cloudflare.text'),
  }

  const sections: Record<string, SectionData> = {
    general: { title: t('general.title'), text: t('general.text') },
    logs: { title: t('logs.title'), text: t('logs.text') },
    contact: { title: t('contact.title'), text: t('contact.text') },
    rights: { title: t('rights.title'), text: t('rights.text') },
    nocookies: { title: t('nocookies.title'), text: t('nocookies.text') },
    changes: { title: t('changes.title'), text: t('changes.text') },
  }

  return (
    <LegalPageLayout locale={locale} title={t('title')}>
      <ControllerBlock
        address={controller.address}
        addressLabel={addressLabel}
        email={controller.email}
        emailLabel={emailLabel}
        name={controller.name}
        nameLabel={nameLabel}
        title={controller.title}
      />
      <CloudflareBlock {...cloudflare} />
      <Sections sections={sections} />
    </LegalPageLayout>
  )
}

export default PrivacyPolicyPage
