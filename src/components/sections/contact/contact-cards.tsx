import { type JSX } from 'react'

import { Code2, Eye, Mail, Network, ShieldCheck } from 'lucide-react'

import { BlueprintCard } from '@/components/blueprint/blueprint-card'
import { BlueprintItem } from '@/components/blueprint/blueprint-item'
import { ResumeVerificationDialog } from '@/components/resume/resume-verification-dialog'
import { siteConfig } from '@/data/config'
import type { FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'

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
      <BlueprintItem
        href={`mailto:${siteConfig.email}`}
        icon={<Mail className="h-5 w-5" />}
        label={translations('email')}
        subLabel={siteConfig.email}
      />
      <div className="relative">
        <BlueprintItem
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
      <BlueprintItem
        href={siteConfig.socials.github}
        icon={<Code2 className="h-5 w-5" />}
        label={translations('github')}
        subLabel={translations('sourceControl')}
      />

      {Boolean(siteConfig.socials.linkedin) && (
        <BlueprintItem
          href={siteConfig.socials.linkedin ?? ''}
          icon={<Network className="h-5 w-5" />}
          label={translations('linkedin')}
          subLabel={translations('professionalNetwork')}
        />
      )}
    </div>
  </BlueprintCard>
)

export const ContactColumns: FCStrict<CardProperties> = ({
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
