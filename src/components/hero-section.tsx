'use server'

import { type JSX } from 'react'

import { ArrowDown, GitBranch, Mail } from 'lucide-react'
import { getTranslations } from 'next-intl/server'

import { Button } from '@/components/ui/button'
import { siteConfig } from '@/lib/config'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { LocalePageProperties, Translations } from '@/types/i18n'

/* ── props ─────────────────────────────────────────────────────────────── */

type HeroSectionProperties = LocalePageProperties

interface HeroTitleProperties {
  readonly greeting: string
  readonly name: string
}

interface HeroSubtitleProperties {
  readonly title: string
}

interface HeroLocationTaglineProperties {
  readonly location: string
  readonly tagline: string
}

interface HeroButtonsProperties {
  readonly contactLabel: string
  readonly email: string
  readonly githubLabel: string
  readonly githubUrl: string
}

/* ── subcomponents (small & typed) ─────────────────────────────────────── */

const HeroTitle: FCStrict<HeroTitleProperties> = ({
  greeting,
  name,
}: HeroTitleProperties): JSX.Element => (
  <h1 className="text-foreground animate-in fade-in slide-in-from-bottom-4 mb-6 text-5xl font-bold tracking-tight text-balance duration-1000 md:text-7xl">
    {greeting}{' '}
    <span className="from-primary to-primary/60 bg-gradient-to-r bg-clip-text text-transparent">
      {name}
    </span>
  </h1>
)

const HeroSubtitle: FCStrict<HeroSubtitleProperties> = ({
  title,
}: HeroSubtitleProperties): JSX.Element => (
  <p className="text-muted-foreground animate-in fade-in slide-in-from-bottom-4 mb-3 text-xl text-pretty delay-150 duration-1000 md:text-2xl">
    {title}
  </p>
)

const HeroLocationTagline: FCStrict<HeroLocationTaglineProperties> = ({
  location,
  tagline,
}: HeroLocationTaglineProperties): JSX.Element => {
  const separator: string = ' \u00B7 '
  return (
    <p className="text-muted-foreground animate-in fade-in slide-in-from-bottom-4 mb-10 text-lg delay-300 duration-1000">
      {location}
      {separator}
      {tagline}
    </p>
  )
}

const HeroButtons: FCStrict<HeroButtonsProperties> = ({
  contactLabel,
  email,
  githubLabel,
  githubUrl,
}: HeroButtonsProperties): JSX.Element => (
  <div className="animate-in fade-in slide-in-from-bottom-4 flex flex-wrap items-center justify-center gap-4 delay-500 duration-1000">
    <Button
      asChild={true}
      className="group shadow-lg transition-all hover:shadow-xl"
      size="lg"
    >
      <a href={githubUrl} rel="noopener noreferrer" target="_blank">
        <GitBranch className="mr-2 h-5 w-5 transition-transform group-hover:scale-110" />
        {githubLabel}
      </a>
    </Button>

    <Button
      asChild={true}
      className="group bg-transparent shadow-md transition-all hover:shadow-lg"
      size="lg"
      variant="outline"
    >
      <a href={`mailto:${email}`}>
        <Mail className="mr-2 h-5 w-5 transition-transform group-hover:scale-110" />
        {contactLabel}
      </a>
    </Button>
  </div>
)

const HeroScrollHint: FCStrict = (): JSX.Element => (
  <div className="animate-in fade-in mt-16 delay-700 duration-1000">
    <ArrowDown className="text-muted-foreground mx-auto h-6 w-6 animate-bounce" />
  </div>
)

/* ── main ──────────────────────────────────────────────────── */
export const HeroSection: AsyncPageFC<HeroSectionProperties> = async ({
  locale,
}: HeroSectionProperties): Promise<JSX.Element> => {
  const translations: Translations<'hero'> = await getTranslations({
    locale,
    namespace: 'hero',
  })

  return (
    <section className="relative flex h-screen min-h-screen snap-start items-center justify-center px-4 py-20">
      <div className="from-primary/5 absolute inset-0 -z-10 bg-gradient-to-b via-transparent to-transparent" />

      <div className="max-w-4xl text-center">
        <HeroTitle
          greeting={translations('greeting')}
          name={translations('name')}
        />
        <HeroSubtitle title={translations('title')} />
        <HeroLocationTagline
          location={translations('location')}
          tagline={translations('tagline')}
        />
        <HeroButtons
          contactLabel={translations('contact')}
          email={siteConfig.email}
          githubLabel={translations('github')}
          githubUrl={siteConfig.github}
        />
        <HeroScrollHint />
      </div>
    </section>
  )
}
