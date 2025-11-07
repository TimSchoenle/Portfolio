'use server'

import type { JSX } from 'react'

import type { Metadata } from 'next'
import { type Locale } from 'next-intl'

import { getTranslations, setRequestLocale } from 'next-intl/server'

import { LegalPageLayout } from '@/components/legal-page-layout'
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

/* --------------------------------- metadata -------------------------------- */
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

/* ----------------------------- small components ---------------------------- */

interface ControllerBlockProperties {
  readonly address: string
  readonly addressLabel: string
  readonly email: string
  readonly emailLabel: string
  readonly name: string
  readonly nameLabel: string
  readonly title: string
}

const ControllerBlock: FCStrict<ControllerBlockProperties> = ({
  address,
  addressLabel,
  email,
  emailLabel,
  name,
  nameLabel,
  title,
}: ControllerBlockProperties): JSX.Element => {
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

interface CloudflareBlockProperties {
  readonly policyLink: string
  readonly post: string
  readonly pre: string
  readonly provider: string
  readonly strong: string
  readonly text: string
  readonly title: string
}

const CloudflareBlock: FCStrict<CloudflareBlockProperties> = ({
  policyLink,
  post,
  pre,
  provider,
  strong,
  text,
  title,
}: CloudflareBlockProperties): JSX.Element => {
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
  readonly text: string
  readonly title: string
}

interface SectionsProperties {
  readonly sections: Record<string, SectionData>
}

const Sections: (properties: SectionsProperties) => JSX.Element[] = ({
  sections,
}: SectionsProperties): JSX.Element[] => {
  const nodes: JSX.Element[] = []
  for (const [key, section] of Object.entries(sections)) {
    nodes.push(
      <div key={key}>
        <h2 className="mb-2 text-xl font-semibold">{section.title}</h2>
        <p className="text-muted-foreground text-sm leading-relaxed">
          {section.text}
        </p>
      </div>
    )
  }
  return nodes
}

/* ----------------------------------- page ---------------------------------- */

type PrivacyPageProperties = UnparsedLocalePageProperties

// eslint-disable-next-line max-lines-per-function
const PrivacyPolicyPage: RoutePageFC<PrivacyPageProperties> = async ({
  params,
}: PageParameters<PrivacyPageProperties>): Promise<JSX.Element> => {
  const locale: Locale = await ensureLocaleFromParameters(params)

  setRequestLocale(locale)

  const translations: Translations<'privacy'> = await getTranslations({
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
    address: translations('controller.address'),
    email: translations('controller.email'),
    name: translations('controller.name'),
    title: translations('controller.title'),
  }

  const cloudflare: CloudflareBlockProperties = {
    policyLink: translations('cloudflare.policyLink'),
    post: translations('cloudflare.post'),
    pre: translations('cloudflare.pre'),
    provider: translations('cloudflare.provider'),
    strong: translations('cloudflare.strong'),
    text: translations('cloudflare.text'),
    title: translations('cloudflare.title'),
  }

  const sections: Record<string, SectionData> = {
    changes: {
      text: translations('changes.text'),
      title: translations('changes.title'),
    },
    contact: {
      text: translations('contact.text'),
      title: translations('contact.title'),
    },
    general: {
      text: translations('general.text'),
      title: translations('general.title'),
    },
    logs: {
      text: translations('logs.text'),
      title: translations('logs.title'),
    },
    nocookies: {
      text: translations('nocookies.text'),
      title: translations('nocookies.title'),
    },
    rights: {
      text: translations('rights.text'),
      title: translations('rights.title'),
    },
  }

  return (
    <LegalPageLayout locale={locale} title={translations('title')}>
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
