import { readFile } from 'node:fs/promises'
import path from 'node:path'

import { type JSX } from 'react'

// eslint-disable-next-line sonarjs/deprecation
import { Eye, Github, Linkedin, Mail, ShieldCheck } from 'lucide-react'
import { getTranslations } from 'next-intl/server'

import { BlueprintCard } from '@/components/blueprint/blueprint-card'
import { BlueprintContainer } from '@/components/blueprint/blueprint-container'
import { BlueprintSectionTitle } from '@/components/blueprint/blueprint-section-title'
import { ResumeVerificationDialog } from '@/components/resume/resume-verification-dialog'
import { siteConfig } from '@/lib/config'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { LocalePageProperties, Translations } from '@/types/i18n'

/* ── types ─────────────────────────────────────────────────────────────── */

const TRANSMISSION_END: string = ':: END_OF_TRANSMISSION ::'

interface ContactItemProperties {
  readonly href: string
  readonly icon: JSX.Element
  readonly label: string
  readonly subLabel?: string
}

/* ── subcomponents ─────────────────────────────────────────────────────── */

const ContactItem: FCStrict<ContactItemProperties> = ({
  href,
  icon,
  label,
  subLabel,
}: ContactItemProperties): JSX.Element => (
  <a
    className="group hover:shadow-[0_0_10px_color-mix(in srgb, var(--brand), transparent 90%)] relative flex items-center gap-4 border border-brand/30 bg-brand/5 p-4 transition-all hover:bg-brand/10"
    href={href}
    rel="noreferrer"
    target="_blank"
  >
    <div className="flex h-10 w-10 items-center justify-center rounded-none border border-brand bg-blueprint-bg text-brand shadow-[0_0_5px_#60A5FA]">
      {icon}
    </div>
    <div className="flex flex-col">
      <span className="font-mono text-sm font-bold tracking-wide text-blueprint-text transition-colors group-hover:text-brand">
        {label}
      </span>
      {Boolean(subLabel) && (
        <span className="font-mono text-xs tracking-wider text-blueprint-muted uppercase">
          {subLabel}
        </span>
      )}
    </div>

    {/* Corner Accents */}
    <div className="absolute top-0 right-0 h-1.5 w-1.5 border-t border-r border-brand" />
    <div className="absolute bottom-0 left-0 h-1.5 w-1.5 border-b border-l border-brand" />
  </a>
)

/* ── sub-components: cards ─────────────────────────────────────────────── */

interface CardProperties {
  readonly fingerprint?: string | undefined
  readonly languageName: string
  readonly locale: string
  readonly translations: Translations<'contact'>
}

const DirectUplinkCard: FCStrict<CardProperties> = ({
  fingerprint,
  languageName,
  locale,
  translations,
}: CardProperties): JSX.Element => (
  <BlueprintCard label="DIRECT_UPLINK" noPadding={true}>
    <div className="flex flex-col gap-4 p-6">
      <ContactItem
        href={`mailto:${siteConfig.email}`}
        icon={<Mail className="h-5 w-5" />}
        label={translations('email')}
        subLabel={siteConfig.email}
      />
      <div className="relative">
        <ContactItem
          href={`/resume/${locale}.pdf`}
          icon={<Eye className="h-5 w-5" />}
          label={translations('downloadResume')}
          subLabel={translations('pdfVersion', {
            language: languageName.toUpperCase(),
          })}
        />
        {Boolean(fingerprint) && (
          <ResumeVerificationDialog fingerprint={fingerprint ?? ''}>
            <button
              className="absolute top-2 right-2 p-2 text-blueprint-muted transition-colors hover:text-brand"
              title={translations('verification.title')}
              type="button"
            >
              <ShieldCheck className="h-5 w-5" />
              <span className="sr-only">
                {translations('verification.title')}
              </span>
            </button>
          </ResumeVerificationDialog>
        )}
      </div>
    </div>
  </BlueprintCard>
)

const NetworkNodesCard: FCStrict<CardProperties> = ({
  translations,
}: CardProperties): JSX.Element => (
  <BlueprintCard label="NETWORK_NODES" noPadding={true}>
    <div className="flex flex-col gap-4 p-6">
      <ContactItem
        href={siteConfig.socials.github}
        // eslint-disable-next-line @typescript-eslint/no-deprecated, sonarjs/deprecation
        icon={<Github className="h-5 w-5" />}
        label={translations('github')}
        subLabel={translations('sourceControl')}
      />
      {Boolean(siteConfig.socials.linkedin) && (
        <ContactItem
          href={siteConfig.socials.linkedin ?? ''}
          // eslint-disable-next-line @typescript-eslint/no-deprecated, sonarjs/deprecation
          icon={<Linkedin className="h-5 w-5" />}
          label={translations('linkedin')}
          subLabel={translations('professionalNetwork')}
        />
      )}
    </div>
  </BlueprintCard>
)

const ContactColumns: FCStrict<CardProperties> = ({
  fingerprint,
  languageName,
  locale,
  translations,
}: CardProperties): JSX.Element => (
  <div className="mt-12 grid w-full grid-cols-1 gap-8 md:grid-cols-2">
    <DirectUplinkCard
      fingerprint={fingerprint}
      languageName={languageName}
      locale={locale}
      translations={translations}
    />
    <NetworkNodesCard
      languageName={languageName}
      locale={locale}
      translations={translations}
    />
  </div>
)

const TransmissionEnd: FCStrict = (): JSX.Element => (
  <div className="mt-16 text-center opacity-60 select-none">
    <svg
      aria-hidden="true"
      className="inline-block h-10 w-64 overflow-visible"
      role="img"
    >
      <rect
        className="fill-blueprint-bg stroke-brand/30"
        height="100%"
        rx="2"
        strokeWidth="1"
        width="100%"
        x="0"
        y="0"
      />
      <text
        className="fill-brand font-mono text-[10px] tracking-[0.2em] uppercase"
        dominantBaseline="middle"
        textAnchor="middle"
        x="50%"
        y="50%"
      >
        {TRANSMISSION_END}
      </text>
    </svg>
  </div>
)

/* ── main ──────────────────────────────────────────────────── */

type ContactSectionProperties = LocalePageProperties

export const ContactSection: AsyncPageFC<ContactSectionProperties> = async ({
  locale,
}: ContactSectionProperties): Promise<JSX.Element> => {
  const translations: Translations<'contact'> = await getTranslations({
    locale,
    namespace: 'contact',
  })
  const commonTranslations: Translations<'common'> = await getTranslations({
    locale,
    namespace: 'common',
  })

  let fingerprint: string | undefined
  try {
    const fingerprintPath: string = path.join(
      process.cwd(),
      'public/resume-fingerprint.json'
    )
    const fileContent: string = await readFile(fingerprintPath, 'utf8')
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    const data: { fingerprint: string } = JSON.parse(fileContent)
    fingerprint = data.fingerprint
  } catch {
    // Fingerprint file missing or invalid - verification disabled
  }

  return (
    <BlueprintContainer id="contact" isLazy={true}>
      <div className="mx-auto flex w-full max-w-4xl flex-col items-center">
        <BlueprintSectionTitle
          sectionLabel="// COMMUNICATION_CHANNELS"
          title={translations('title')}
        />

        <ContactColumns
          fingerprint={fingerprint}
          languageName={commonTranslations('languageName')}
          locale={locale}
          translations={translations}
        />

        <TransmissionEnd />
      </div>
    </BlueprintContainer>
  )
}

export default ContactSection
