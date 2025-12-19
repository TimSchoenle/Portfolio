/* eslint-disable @typescript-eslint/no-deprecated */
/* eslint-disable sonarjs/deprecation */
import { type JSX } from 'react'

import { ArrowDown, Github, Mail } from 'lucide-react'
import { getTranslations } from 'next-intl/server'

import { BlueprintContainer } from '@/components/blueprint/blueprint-container'
import { BlueprintSectionTitle } from '@/components/blueprint/blueprint-section-title'
import { ScrollSnapPairController } from '@/components/features/scroll-snap/scroll-snap-pair-controller'
import { siteConfig } from '@/lib/config'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { LocalePageProperties, Translations } from '@/types/i18n'

/* ── props ─────────────────────────────────────────────────────────────── */

type HeroSectionProperties = LocalePageProperties

/* ── subcomponents ─────────────────────────────────────────────────────── */

const LOCATION_LABEL: string = 'LOCATION_VECTOR'
const MISSION_LABEL: string = 'CURRENT_MISSION'

const BlueprintStatus: FCStrict<{
  readonly location: string
  readonly tagline: string
}> = ({
  location,
  tagline,
}: {
  readonly location: string
  readonly tagline: string
}): JSX.Element => (
  <div className="mt-16 grid w-full max-w-5xl grid-cols-1 items-center gap-6 font-mono text-xs tracking-widest text-brand/80 uppercase sm:grid-cols-[1fr_auto_1fr] sm:gap-8">
    {/* Detail Block A - Aligned to End (Right) of Left Column */}
    <div className="shadow-[0_0_10px_color-mix(in srgb, var(--brand), transparent 90%)] flex w-full flex-col gap-2 justify-self-center border-l-2 border-brand bg-brand/5 py-2 pl-4 text-left sm:max-w-[280px] sm:justify-self-end">
      <span aria-hidden="true" className="text-[10px] font-bold text-brand">
        {LOCATION_LABEL}
      </span>
      <span className="text-sm font-bold text-blueprint-text">{location}</span>
    </div>

    {/* Decorative Hex - Centered in Middle Column */}
    <div className="mx-auto hidden h-8 w-8 shrink-0 rotate-45 items-center justify-center border border-brand bg-blueprint-bg sm:flex">
      <div className="h-3 w-3 animate-pulse bg-brand shadow-[0_0_8px_#60A5FA]" />
    </div>

    {/* Detail Block B - Aligned to Start (Left) of Right Column */}
    <div className="shadow-[0_0_10px_color-mix(in srgb, var(--brand), transparent 90%)] flex w-full flex-col gap-2 justify-self-center border-r-2 border-brand bg-brand/5 py-2 pr-4 text-right sm:max-w-[280px] sm:justify-self-start">
      <span aria-hidden="true" className="text-[10px] font-bold text-brand">
        {MISSION_LABEL}
      </span>
      <span className="flex items-center justify-end gap-3 text-sm font-bold text-blueprint-text">
        {tagline}
      </span>
    </div>
  </div>
)

const HeroActionButtons: FCStrict<{
  readonly contactText: string
  readonly githubLabel: string
}> = ({
  contactText,
  githubLabel,
}: {
  readonly contactText: string
  readonly githubLabel: string
}): JSX.Element => (
  <div className="mt-16 flex flex-wrap gap-8">
    {/* Custom Technical Button */}
    <a
      className="group relative px-8 py-3 font-mono text-sm tracking-widest text-brand uppercase transition-colors hover:text-blueprint-text"
      href={siteConfig.socials.github}
      rel="noreferrer"
      target="_blank"
    >
      <span className="relative z-10 flex items-center gap-2">
        <Github className="h-4 w-4" />
        {githubLabel}
      </span>
      {/* Button Frame */}
      <div className="group-hover:shadow-[0_0_15px_color-mix(in srgb, var(--brand), transparent 70%)] absolute inset-0 skew-x-[-12deg] border border-brand bg-brand/5 transition-all group-hover:bg-brand/20" />
      <div className="absolute right-0 bottom-0 h-1 w-1 bg-brand" />
      <div className="absolute top-0 left-0 h-1 w-1 bg-brand" />
    </a>

    <a
      className="group relative px-8 py-3 font-mono text-sm tracking-widest text-blueprint-bg uppercase transition-colors hover:text-blueprint-text"
      href={`mailto:${siteConfig.email}`}
    >
      <span className="relative z-10 flex items-center gap-2 font-bold">
        <Mail className="h-4 w-4" />
        {contactText}
      </span>
      {/* Button Frame Filled */}
      <div className="shadow-[0_0_15px_color-mix(in srgb, var(--brand), transparent 60%)] absolute inset-0 skew-x-[-12deg] border border-brand bg-brand transition-all group-hover:bg-transparent group-hover:text-brand" />
    </a>
  </div>
)

/* ── main ──────────────────────────────────────────────────── */

const GRID_REF: string = 'GRID_REF_X-99'

export const HeroSection: AsyncPageFC<HeroSectionProperties> = async ({
  locale,
}: HeroSectionProperties): Promise<JSX.Element> => {
  const translations: Translations<string> = await getTranslations({ locale })
  const heroTranslations: Translations<'hero'> = await getTranslations({
    locale,
    namespace: 'hero',
  })

  return (
    <BlueprintContainer
      id="hero"
      overlay={
        <div className="absolute bottom-[calc(var(--app-padding)+1rem)] left-1/2 z-20 -translate-x-1/2 animate-bounce cursor-pointer text-brand/50 transition-colors hover:text-brand">
          <a aria-label="Scroll to About Section" href="#about">
            <ArrowDown className="h-6 w-6" />
          </a>
        </div>
      }
    >
      <div className="relative z-10 container flex min-h-[60vh] flex-col items-center justify-center">
        {/* Floating tech markings */}
        <div
          aria-hidden="true"
          className="writing-vertical-rl absolute top-20 left-20 font-mono text-[10px] text-brand/40 select-none"
        >
          {GRID_REF}
        </div>

        <BlueprintSectionTitle
          greeting={heroTranslations('greeting')}
          sectionLabel="// MAIN_ENTRY_POINT"
          subtitle={translations('personalInfo.jobTitle')}
          title={siteConfig.fullName}
        />

        <BlueprintStatus
          location={heroTranslations('location', {
            country: translations('personalInfo.country'),
          })}
          tagline={heroTranslations('tagline')}
        />

        <HeroActionButtons
          contactText={heroTranslations('contact')}
          githubLabel={translations('common.socials.github')}
        />
      </div>

      <ScrollSnapPairController bottomSectionId="about" topSectionId="hero" />
    </BlueprintContainer>
  )
}
