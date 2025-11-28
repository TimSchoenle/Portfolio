import { type JSX } from 'react'

import { type Locale } from 'next-intl'

import { Briefcase, Calendar, MapPin } from 'lucide-react'
import { getTranslations } from 'next-intl/server'

import { Card, CardContent } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import type { FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'
import type { ResumeExperience } from '@/types/resume'

/* ─────────────────────────── types ─────────────────────────── */

interface ExperienceSectionProperties {
  readonly locale: Locale
}

interface ExperienceCardProperties {
  readonly experience: ResumeExperience
  readonly index: number
}

/* ──────────────────────── subcomponent ─────────────────────── */

// eslint-disable-next-line max-lines-per-function
const ExperienceCard: FCStrict<ExperienceCardProperties> = ({
  experience,
  index,
}: ExperienceCardProperties): JSX.Element => {
  const key: string = `${experience.company}::${experience.title}::${index.toString()}`
  const dateRange: string = `${experience.startDate} - ${experience.endDate}`

  return (
    <Card
      className="group border-2 transition-all duration-300 hover:-translate-y-1 hover:border-primary/50 hover:shadow-xl"
      key={key}
    >
      <CardContent className="p-6">
        <div className="flex gap-6">
          <div className="flex-shrink-0">
            <div className="relative h-16 w-16 overflow-hidden rounded-xl border-2 border-border bg-muted shadow-md transition-transform duration-300 group-hover:scale-105 group-hover:shadow-lg">
              <div className="flex h-full w-full items-center justify-center bg-gradient-to-br from-primary/10 to-primary/5">
                <Briefcase className="h-8 w-8 text-primary" />
              </div>
            </div>
          </div>

          <div className="min-w-0 flex-1">
            <Heading
              as="h3"
              className="mb-1 text-xl font-semibold text-foreground transition-colors group-hover:text-primary"
            >
              {experience.title}
            </Heading>
            <p className="mb-2 text-base font-medium text-foreground/80">
              {experience.company}
            </p>

            <div className="mb-4 flex flex-wrap gap-4 text-sm text-muted-foreground">
              <div className="flex items-center gap-2">
                <Calendar className="h-4 w-4" />
                <p>{dateRange}</p>
              </div>
              <div className="flex items-center gap-2">
                <MapPin className="h-4 w-4" />
                <p>{experience.location}</p>
              </div>
            </div>

            <div className="scrollbar-thin scrollbar-thumb-primary/20 scrollbar-track-transparent max-h-60 overflow-y-auto rounded-lg border-2 bg-muted/50 p-4 shadow-inner transition-colors hover:bg-muted/70">
              <ul className="list-disc space-y-2 pl-4">
                {experience.achievements.map(
                  (achievement: string): JSX.Element => (
                    <li
                      className="text-sm leading-relaxed text-foreground/90"
                      key={achievement}
                    >
                      {achievement}
                    </li>
                  )
                )}
              </ul>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

/* ────────────────────────── component ───────────────────────── */

export const ExperienceSection: (
  properties: ExperienceSectionProperties
) => Promise<JSX.Element> = async ({
  locale,
}: ExperienceSectionProperties): Promise<JSX.Element> => {
  const translations: Translations<'resume'> = await getTranslations({
    locale,
    namespace: 'resume',
  })

  // Safely read raw list and validate
  const experiences: ResumeExperience[] = translations.raw(
    'experience'
  ) as ResumeExperience[]
  const sectionTitle: string = translations('sectionTitles.experience')

  return (
    <section className="px-4 py-20">
      <div className="mx-auto w-full max-w-4xl">
        <div className="mb-12 text-center">
          <Heading as="h2" className="mb-3 text-4xl font-bold text-foreground">
            {sectionTitle}
          </Heading>
          <div className="mx-auto h-1 w-20 rounded-full bg-gradient-to-r from-primary to-primary/60" />
        </div>

        <ul className="space-y-6">
          {experiences.map(
            (exp: ResumeExperience, index: number): JSX.Element => (
              <li key={`${exp.company}-${index.toString()}`}>
                <ExperienceCard experience={exp} index={index} />
              </li>
            )
          )}
        </ul>
      </div>
    </section>
  )
}
