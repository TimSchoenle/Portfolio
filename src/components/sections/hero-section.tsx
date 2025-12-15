import { type JSX } from 'react'

import { ArrowDown, GitBranch, Mail } from 'lucide-react'
import { getTranslations } from 'next-intl/server'

import { Button } from '@/components/ui/button'
import { Heading } from '@/components/ui/heading'
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
  <Heading
    as="h1"
    className="mb-6 text-5xl font-bold tracking-tight text-balance text-foreground md:text-7xl"
  >
    {greeting}
    &nbsp;
    <span className="bg-gradient-to-r from-primary to-primary/60 bg-clip-text text-transparent">
      {name}
    </span>
  </Heading>
)

const HeroSubtitle: FCStrict<HeroSubtitleProperties> = ({
  title,
}: HeroSubtitleProperties): JSX.Element => (
  <p className="mb-3 text-xl text-pretty text-muted-foreground md:text-2xl">
    {title}
  </p>
)

const HeroLocationTagline: FCStrict<HeroLocationTaglineProperties> = ({
  location,
  tagline,
}: HeroLocationTaglineProperties): JSX.Element => {
  const separator: string = ' \u00B7 '
  return (
    <p className="mb-10 text-lg text-muted-foreground">
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
  <div className="flex flex-wrap items-center justify-center gap-4">
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
  <div className="mt-16">
    <ArrowDown className="mx-auto h-6 w-6 animate-bounce text-muted-foreground" />
  </div>
)

/* ── main ──────────────────────────────────────────────────── */
export const HeroSection: AsyncPageFC<HeroSectionProperties> = async ({
  locale,
}: HeroSectionProperties): Promise<JSX.Element> => {
  const translation: Translations<''> = await getTranslations({ locale })
  const heroTranslation: Translations<'hero'> = await getTranslations({
    locale,
    namespace: 'hero',
  })

  return (
    <section className="relative flex h-screen min-h-screen snap-start items-center justify-center px-4 py-20">
      <div className="absolute inset-0 -z-10 bg-gradient-to-b from-primary/5 via-transparent to-transparent" />

      <div className="max-w-4xl text-center">
        <HeroTitle
          greeting={heroTranslation('greeting')}
          name={siteConfig.name}
        />
        <HeroSubtitle title={translation('personalInfo.jobTitle')} />
        <HeroLocationTagline
          location={heroTranslation('location', {
            country: translation('personalInfo.country'),
          })}
          tagline={heroTranslation('tagline')}
        />
        <HeroButtons
          contactLabel={heroTranslation('contact')}
          email={siteConfig.email}
          githubLabel={translation('common.socials.github')}
          githubUrl={siteConfig.socials.github}
        />
        <HeroScrollHint />
      </div>
    </section>
  )
}
