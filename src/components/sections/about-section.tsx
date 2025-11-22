'use server'

import { type JSX, type ReactNode } from 'react'

import { type Locale } from 'next-intl'

import { BookOpen, Code2 } from 'lucide-react'
import { getTranslations } from 'next-intl/server'

import { Card, CardContent } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { LocalePageProperties, Translations } from '@/types/i18n'

type AboutSectionProperties = LocalePageProperties

interface InfoCardProperties {
  readonly description: ReactNode
  readonly icon: ReactNode
  readonly title: string
}

interface AboutTranslations {
  readonly expertiseDescription: ReactNode
  readonly learningDescription: ReactNode
  readonly t: Translations<'about'>
}

const InfoCard: FCStrict<InfoCardProperties> = ({
  description,
  icon,
  title,
}: InfoCardProperties): JSX.Element => (
  <Card className="group border-2 transition-all duration-300 hover:border-primary/50 hover:shadow-xl">
    <CardContent className="flex items-start gap-4 p-6">
      <div className="rounded-lg bg-gradient-to-br from-primary/10 to-primary/5 p-3 transition-transform duration-300 group-hover:scale-110">
        {icon}
      </div>
      <div>
        <Heading as="h3" className="mb-2 text-xl font-semibold text-foreground">
          {title}
        </Heading>
        <div className="leading-relaxed text-muted-foreground">
          {description}
        </div>
      </div>
    </CardContent>
  </Card>
)

async function getAboutTranslations(
  locale: Locale
): Promise<AboutTranslations> {
  const translations: Translations<'about'> = await getTranslations({
    locale,
    namespace: 'about',
  })

  const learningDescription: ReactNode = translations.rich(
    'learning.description',
    {
      highlight: (chunks: ReactNode): JSX.Element => (
        <span className="font-medium text-foreground">{chunks}</span>
      ),
    }
  )

  const expertiseDescription: ReactNode = translations.rich(
    'expertise.description',
    {
      highlight: (chunks: ReactNode): JSX.Element => (
        <span className="font-medium text-foreground">{chunks}</span>
      ),
    }
  )

  return { expertiseDescription, learningDescription, t: translations }
}

const AboutSection: AsyncPageFC<AboutSectionProperties> = async ({
  locale,
}: AboutSectionProperties): Promise<JSX.Element> => {
  const aboutTranslations: AboutTranslations =
    await getAboutTranslations(locale)
  const { expertiseDescription, learningDescription, t }: AboutTranslations =
    aboutTranslations

  return (
    <section className="px-4 py-20" id="about">
      <div className="mx-auto w-full max-w-6xl">
        <div className="mb-12 text-center">
          <Heading as="h2" className="mb-3 text-4xl font-bold text-foreground">
            {t('title')}
          </Heading>
          <div className="mx-auto h-1 w-20 rounded-full bg-gradient-to-r from-primary to-primary/60" />
        </div>

        <div className="grid gap-6 md:grid-cols-2">
          <InfoCard
            description={learningDescription}
            icon={<BookOpen className="h-6 w-6 text-primary" />}
            title={t('learning.title')}
          />
          <InfoCard
            description={expertiseDescription}
            icon={<Code2 className="h-6 w-6 text-primary" />}
            title={t('expertise.title')}
          />
        </div>
      </div>
    </section>
  )
}

export default AboutSection
