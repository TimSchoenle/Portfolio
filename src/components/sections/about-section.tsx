'use server'

import { type JSX, type ReactNode } from 'react'

import { type Locale } from 'next-intl'

import { getTranslations } from 'next-intl/server'

import { Card, CardContent } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { LocalePageProperties, Translations } from '@/types/i18n'

type AboutSectionProperties = LocalePageProperties

interface CompetencyBadgeProperties {
  readonly label: string
}

interface AboutTranslations {
  readonly competencies: readonly string[]
  readonly summary: ReactNode
  readonly translations: Translations<'about'>
}

const CompetencyBadge: FCStrict<CompetencyBadgeProperties> = ({
  label,
}: CompetencyBadgeProperties): JSX.Element => (
  <div className="rounded-lg border border-primary/20 bg-primary/5 px-4 py-2 text-sm font-medium text-foreground transition-all hover:border-primary/30 hover:bg-primary/10">
    {label}
  </div>
)

async function getAboutTranslations(
  locale: Locale
): Promise<AboutTranslations> {
  const translations: Translations<'about'> = await getTranslations({
    locale,
    namespace: 'about',
  })

  const summary: ReactNode = translations.rich('summary', {
    highlight: (chunks: ReactNode): JSX.Element => (
      <span className="font-semibold text-primary">{chunks}</span>
    ),
  })

  const competencies: readonly string[] = translations.raw(
    'competencies'
  ) as string[]

  return { competencies, summary, translations }
}

const AboutSection: AsyncPageFC<AboutSectionProperties> = async ({
  locale,
}: AboutSectionProperties): Promise<JSX.Element> => {
  const aboutTranslations: AboutTranslations =
    await getAboutTranslations(locale)
  const { competencies, summary, translations }: AboutTranslations =
    aboutTranslations

  return (
    <section className="relative overflow-hidden px-4 py-24" id="about">
      {/* Background gradient */}
      <div className="absolute inset-0 -z-10 bg-gradient-to-b from-muted/30 via-background to-background" />

      <div className="mx-auto w-full max-w-4xl">
        {/* Section Title */}
        <div className="mb-12 text-center">
          <Heading
            as="h2"
            className="mb-4 inline-block bg-gradient-to-r from-primary to-primary/60 bg-clip-text py-1 text-4xl leading-tight font-bold text-transparent md:text-5xl"
          >
            {translations('title')}
          </Heading>
        </div>

        {/* Main Content Card */}
        <Card className="border-border/50 bg-card/50 shadow-lg backdrop-blur-sm">
          <CardContent className="space-y-8 p-8 md:p-12">
            {/* Summary Text */}
            <div className="text-center text-lg leading-relaxed md:text-xl">
              {summary}
            </div>

            {/* Key Competencies */}
            <div className="mx-auto max-w-lg">
              <h3 className="mb-4 text-center text-sm font-semibold tracking-wider uppercase">
                {translations('competenciesLabel')}
              </h3>
              <div className="flex flex-wrap justify-center gap-x-2 gap-y-1.5 md:gap-x-2.5 md:gap-y-2">
                {competencies.map(
                  (competency: string): JSX.Element => (
                    <CompetencyBadge key={competency} label={competency} />
                  )
                )}
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </section>
  )
}

export default AboutSection
