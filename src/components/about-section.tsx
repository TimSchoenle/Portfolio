'use server'

import { BookOpen, Code2 } from 'lucide-react'
import { type Locale } from 'next-intl'
import { getTranslations } from 'next-intl/server'
import { type JSX, type ReactNode } from 'react'

import { Card, CardContent } from '@/components/ui/card'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { LocalePageProps, Translations } from '@/types/i18n'

type AboutSectionProps = LocalePageProps

interface InfoCardProps {
  readonly icon: ReactNode
  readonly title: string
  readonly description: ReactNode
}

interface AboutTranslations {
  readonly t: Translations
  readonly learningDescription: ReactNode
  readonly expertiseDescription: ReactNode
}

const InfoCard: FCStrict<InfoCardProps> = ({
  icon,
  title,
  description,
}: InfoCardProps): JSX.Element => (
  <Card className="group hover:border-primary/50 border-2 transition-all duration-300 hover:shadow-xl">
    <CardContent className="flex items-start gap-4 p-6">
      <div className="from-primary/10 to-primary/5 rounded-lg bg-gradient-to-br p-3 transition-transform duration-300 group-hover:scale-110">
        {icon}
      </div>
      <div>
        <h3 className="text-foreground mb-2 text-xl font-semibold">{title}</h3>
        <p className="text-muted-foreground leading-relaxed">{description}</p>
      </div>
    </CardContent>
  </Card>
)

async function getAboutTranslations(
  locale: Locale
): Promise<AboutTranslations> {
  const t: Translations = await getTranslations({
    locale,
    namespace: 'about',
  })

  const learningDescription: ReactNode = t.rich('learning.description', {
    highlight: (chunks: ReactNode): JSX.Element => (
      <span className="text-foreground font-medium">{chunks}</span>
    ),
  })

  const expertiseDescription: ReactNode = t.rich('expertise.description', {
    highlight: (chunks: ReactNode): JSX.Element => (
      <span className="text-foreground font-medium">{chunks}</span>
    ),
  })

  return { t, learningDescription, expertiseDescription }
}

interface CreateDescriptionParams {
  readonly description: ReactNode
}

const createDescription: FCStrict<CreateDescriptionParams> = ({
  description,
}: CreateDescriptionParams): JSX.Element => {
  return (
    <div className="text-muted-foreground leading-relaxed">{description}</div>
  )
}

const AboutSection: AsyncPageFC<AboutSectionProps> = async ({
  locale,
}: AboutSectionProps): Promise<JSX.Element> => {
  const aboutTranslations: AboutTranslations =
    await getAboutTranslations(locale)
  const { t, learningDescription, expertiseDescription }: AboutTranslations =
    aboutTranslations

  return (
    <section className="px-4 py-20" id="about">
      <div className="mx-auto w-full max-w-6xl">
        <div className="mb-12 text-center">
          <h2 className="text-foreground mb-3 text-4xl font-bold">
            {t('title')}
          </h2>
          <div className="from-primary to-primary/60 mx-auto h-1 w-20 rounded-full bg-gradient-to-r" />
        </div>

        <div className="grid gap-6 md:grid-cols-2">
          <InfoCard
            description={createDescription({
              description: learningDescription,
            })}
            icon={<BookOpen className="text-primary h-6 w-6" />}
            title={t('learning.title')}
          />
          <InfoCard
            description={createDescription({
              description: expertiseDescription,
            })}
            icon={<Code2 className="text-primary h-6 w-6" />}
            title={t('expertise.title')}
          />
        </div>
      </div>
    </section>
  )
}

export default AboutSection
