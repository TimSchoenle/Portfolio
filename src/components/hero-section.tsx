'use server'

import { Mail, GitBranch, ArrowDown } from 'lucide-react'
import { getTranslations } from 'next-intl/server'
import { type JSX } from 'react'

import { Button } from '@/components/ui/button'
import { siteConfig } from '@/lib/config'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { LocalePageProps, Translations } from '@/types/i18n'

/* ── props ─────────────────────────────────────────────────────────────── */

type HeroSectionProps = LocalePageProps

interface HeroTitleProps {
  readonly greeting: string
  readonly name: string
}

interface HeroSubtitleProps {
  readonly title: string
}

interface HeroLocationTaglineProps {
  readonly location: string
  readonly tagline: string
}

interface HeroButtonsProps {
  readonly githubUrl: string
  readonly email: string
  readonly githubLabel: string
  readonly contactLabel: string
}

/* ── subcomponents (small & typed) ─────────────────────────────────────── */

const HeroTitle: FCStrict<HeroTitleProps> = ({
  greeting,
  name,
}: HeroTitleProps): JSX.Element => (
  <h1 className="text-foreground animate-in fade-in slide-in-from-bottom-4 mb-6 text-5xl font-bold tracking-tight text-balance duration-1000 md:text-7xl">
    {greeting}{' '}
    <span
      className="gradient-text-protected from-primary to-primary/60 bg-gradient-to-r bg-clip-text text-transparent"
      data-darkreader-inline-bgcolor=""
      data-darkreader-inline-color=""
    >
      {name}
    </span>
  </h1>
)

const HeroSubtitle: FCStrict<HeroSubtitleProps> = ({
  title,
}: HeroSubtitleProps): JSX.Element => (
  <p className="text-muted-foreground animate-in fade-in slide-in-from-bottom-4 mb-3 text-xl text-pretty delay-150 duration-1000 md:text-2xl">
    {title}
  </p>
)

const HeroLocationTagline: FCStrict<HeroLocationTaglineProps> = ({
  location,
  tagline,
}: HeroLocationTaglineProps): JSX.Element => {
  const sep: string = ' \u00B7 '
  return (
    <p className="text-muted-foreground animate-in fade-in slide-in-from-bottom-4 mb-10 text-lg delay-300 duration-1000">
      {location}
      {sep}
      {tagline}
    </p>
  )
}

const HeroButtons: FCStrict<HeroButtonsProps> = ({
  githubUrl,
  email,
  githubLabel,
  contactLabel,
}: HeroButtonsProps): JSX.Element => (
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
export const HeroSection: AsyncPageFC<HeroSectionProps> = async ({
  locale,
}: HeroSectionProps): Promise<JSX.Element> => {
  const t: Translations<'hero'> = await getTranslations({
    locale,
    namespace: 'hero',
  })

  return (
    <section className="relative flex h-screen min-h-screen snap-start items-center justify-center px-4 py-20">
      <div className="from-primary/5 absolute inset-0 -z-10 bg-gradient-to-b via-transparent to-transparent" />

      <div className="max-w-4xl text-center">
        <HeroTitle greeting={t('greeting')} name={t('name')} />
        <HeroSubtitle title={t('title')} />
        <HeroLocationTagline location={t('location')} tagline={t('tagline')} />
        <HeroButtons
          contactLabel={t('contact')}
          email={siteConfig.email}
          githubLabel={t('github')}
          githubUrl={siteConfig.github}
        />
        <HeroScrollHint />
      </div>
    </section>
  )
}
