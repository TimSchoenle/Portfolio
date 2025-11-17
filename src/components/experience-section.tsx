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
          <Heading as="h2" className="mb-3 text-4xl font-bold text-foreground">
            {translations('title')}
          </Heading>
          <div className="mx-auto h-1 w-20 rounded-full bg-gradient-to-r from-primary to-primary/60" />
        </div>

        <div className="space-y-6">
          {experiences.map((exp: Experience): JSX.Element => {
            const key: string = `${exp.company}::${exp.title}::${exp.dateRange}`

            return (
              <Card
                className="group border-2 transition-all duration-300 hover:-translate-y-1 hover:border-primary/50 hover:shadow-xl"
                key={key}
              >
                <CardContent className="p-6">
                  <div className="flex gap-6">
                    <div className="flex-shrink-0">
                      <div className="relative h-16 w-16 overflow-hidden rounded-xl border-2 border-border bg-muted shadow-md transition-transform duration-300 group-hover:scale-105 group-hover:shadow-lg">
                        {typeof exp.logo === 'string' && exp.logo.length > 0 ? (
                          <Image
                            alt={`${exp.company} logo`}
                            className="object-cover"
                            fill={true}
                            sizes="64px"
                            src={exp.logo}
                          />
                        ) : (
                          <div className="flex h-full w-full items-center justify-center bg-gradient-to-br from-primary/10 to-primary/5">
                            <Briefcase className="h-8 w-8 text-primary" />
                          </div>
                        )}
                      </div>
                    </div>

                    <div className="min-w-0 flex-1">
                      <Heading
                        as="h3"
                        className="mb-1 text-xl font-semibold text-foreground transition-colors group-hover:text-primary"
                      >
                        {exp.title}
                      </Heading>
                      <p className="mb-2 text-base font-medium text-foreground/80">
                        {exp.company}
                      </p>
                      <div className="mb-4 flex items-center gap-2 text-sm text-muted-foreground">
                        <Calendar className="h-4 w-4" />
                        <p>{exp.dateRange}</p>
                      </div>

                      <div className="scrollbar-thin scrollbar-thumb-primary/20 scrollbar-track-transparent max-h-28 overflow-y-auto rounded-lg border-2 bg-muted/50 p-4 shadow-inner transition-colors hover:bg-muted/70">
                        <p className="text-sm leading-relaxed whitespace-pre-line text-foreground/90">
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
