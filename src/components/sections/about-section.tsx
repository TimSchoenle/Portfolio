import { type JSX, type ReactNode } from 'react'

import { type Locale } from 'next-intl'

import { getTranslations } from 'next-intl/server'

import {
  Card,
  CARD_HOVERS,
  CARD_VARIANTS,
  CardContent,
} from '@/components/ui/card'
import { Section } from '@/components/ui/section'
import { SectionContainer } from '@/components/ui/section-container'
import { SectionHeader } from '@/components/ui/section-header'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { LocalePageProperties, Translations } from '@/types/i18n'

type AboutSectionProperties = LocalePageProperties & {
  readonly performance?: boolean
}

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
  <div
    className="rounded-lg border border-primary/20 bg-primary/5 px-4 py-2 text-sm font-medium text-foreground transition-all hover:border-primary/30 hover:bg-primary/10"
    itemProp="knowsAbout"
  >
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
      <span className="font-semibold">{chunks}</span>
    ),
  })

  const competencies: readonly string[] = translations.raw(
    'competencies'
  ) as string[]

  return { competencies, summary, translations }
}

const AboutSection: AsyncPageFC<AboutSectionProperties> = async ({
  locale,
  performance,
}: AboutSectionProperties): Promise<JSX.Element> => {
  const aboutTranslations: AboutTranslations =
    await getAboutTranslations(locale)
  const { competencies, summary, translations }: AboutTranslations =
    aboutTranslations

  return (
    <Section className="py-24" id="about" performance={performance ?? false}>
      {/* Background gradient */}
      <div className="absolute inset-0 -z-10 bg-gradient-to-b from-muted/30 via-background to-background" />

      <SectionContainer size="sm">
        {/* Section Title */}
        <SectionHeader gradient={true} title={translations('title')} />

        {/* Main Content Card */}
        <Card
          className="p-6"
          hover={CARD_HOVERS.MODERATE}
          variant={CARD_VARIANTS.INTERACTIVE}
        >
          <CardContent className="space-y-8 p-8 md:p-12">
            {/* Summary Text */}
            <div
              className="text-center text-lg leading-relaxed text-foreground/85 md:text-xl"
              itemProp="description"
            >
              {summary}
            </div>

            {/* Key Competencies */}
            <div className="mx-auto max-w-lg">
              <h3 className="mb-4 text-center text-sm font-semibold tracking-wider text-foreground uppercase">
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
      </SectionContainer>
    </Section>
  )
}

export default AboutSection
