'use server'

import { type JSX } from 'react'

import { type Locale } from 'next-intl'

import { Briefcase, Calendar } from 'lucide-react'
import Image from 'next/image'
import { getTranslations } from 'next-intl/server'

import { Card, CardContent } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import type { Translations } from '@/types/i18n'

/* ─────────────────────────── types ─────────────────────────── */

interface ExperienceSectionProperties {
  readonly locale: Locale
}

interface Experience {
  readonly company: string
  readonly dateRange: string
  readonly description: string
  readonly logo?: string | null
  readonly title: string
}

/* ──────────────────────── runtime guards ───────────────────── */

const isRecord: (value: unknown) => value is Record<string, unknown> = (
  value: unknown
): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null

const isExperience: (value: unknown) => value is Experience = (
  value: unknown
): value is Experience => {
  if (!isRecord(value)) {
    return false
  }
  const company: unknown = value['company']
  const logo: unknown = value['logo']
  const title: unknown = value['title']
  const dateRange: unknown = value['dateRange']
  const description: unknown = value['description']
  const logoOk: boolean =
    logo === undefined || logo === null || typeof logo === 'string'
  return (
    typeof company === 'string' &&
    logoOk &&
    typeof title === 'string' &&
    typeof dateRange === 'string' &&
    typeof description === 'string'
  )
}

const parseExperiences: (raw: unknown) => readonly Experience[] = (
  raw: unknown
): readonly Experience[] => {
  if (!Array.isArray(raw)) {
    return []
  }
  const filtered: Experience[] = []
  for (const item of raw) {
    if (isExperience(item)) {
      filtered.push(item)
    }
  }
  return filtered
}

/* ────────────────────────── component ───────────────────────── */

export const ExperienceSection: (
  properties: ExperienceSectionProperties
  // eslint-disable-next-line max-lines-per-function
) => Promise<JSX.Element> = async ({
  locale,
}: ExperienceSectionProperties): Promise<JSX.Element> => {
  const translations: Translations<'experience'> = await getTranslations({
    locale,
    namespace: 'experience',
  })

  // Safely read raw list and validate
  const rawItems: unknown = translations.raw('items')
  const experiences: readonly Experience[] = parseExperiences(rawItems)

  return (
    <section className="px-4 py-20">
      <div className="mx-auto w-full max-w-4xl">
        <div className="mb-12 text-center">
          <Heading as="h2" className="text-foreground mb-3 text-4xl font-bold">
            {translations('title')}
          </Heading>
          <div className="from-primary to-primary/60 mx-auto h-1 w-20 rounded-full bg-gradient-to-r" />
        </div>

        <div className="space-y-6">
          {experiences.map((exp: Experience): JSX.Element => {
            const key: string = `${exp.company}::${exp.title}::${exp.dateRange}`

            return (
              <Card
                className="group hover:border-primary/50 border-2 transition-all duration-300 hover:-translate-y-1 hover:shadow-xl"
                key={key}
              >
                <CardContent className="p-6">
                  <div className="flex gap-6">
                    <div className="flex-shrink-0">
                      <div className="border-border bg-muted relative h-16 w-16 overflow-hidden rounded-xl border-2 shadow-md transition-transform duration-300 group-hover:scale-105 group-hover:shadow-lg">
                        {typeof exp.logo === 'string' && exp.logo.length > 0 ? (
                          <Image
                            alt={`${exp.company} logo`}
                            className="object-cover"
                            fill={true}
                            sizes="64px"
                            src={exp.logo}
                          />
                        ) : (
                          <div className="from-primary/10 to-primary/5 flex h-full w-full items-center justify-center bg-gradient-to-br">
                            <Briefcase className="text-primary h-8 w-8" />
                          </div>
                        )}
                      </div>
                    </div>

                    <div className="min-w-0 flex-1">
                      <Heading
                        as="h3"
                        className="text-foreground group-hover:text-primary mb-1 text-xl font-semibold transition-colors"
                      >
                        {exp.title}
                      </Heading>
                      <p className="text-foreground/80 mb-2 text-base font-medium">
                        {exp.company}
                      </p>
                      <div className="text-muted-foreground mb-4 flex items-center gap-2 text-sm">
                        <Calendar className="h-4 w-4" />
                        <p>{exp.dateRange}</p>
                      </div>

                      <div className="bg-muted/50 hover:bg-muted/70 scrollbar-thin scrollbar-thumb-primary/20 scrollbar-track-transparent max-h-28 overflow-y-auto rounded-lg border-2 p-4 shadow-inner transition-colors">
                        <p className="text-foreground/90 text-sm leading-relaxed whitespace-pre-line">
                          {exp.description}
                        </p>
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            )
          })}
        </div>
      </div>
    </section>
  )
}
