'use server'

import { Briefcase, Calendar } from 'lucide-react'
import Image from 'next/image'
import { type Locale } from 'next-intl'
import { getTranslations } from 'next-intl/server'
import { type JSX } from 'react'

import { Card, CardContent } from '@/components/ui/card'
import type { Translations } from '@/types/i18n'

/* ─────────────────────────── types ─────────────────────────── */

interface ExperienceSectionProps {
  readonly locale: Locale
}

interface Experience {
  readonly company: string
  readonly logo?: string | null
  readonly title: string
  readonly dateRange: string
  readonly description: string
}

/* ──────────────────────── runtime guards ───────────────────── */

const isRecord: (v: unknown) => v is Record<string, unknown> = (
  v: unknown
): v is Record<string, unknown> => typeof v === 'object' && v !== null

const isExperience: (v: unknown) => v is Experience = (
  v: unknown
): v is Experience => {
  if (!isRecord(v)) {
    return false
  }
  const company: unknown = v['company']
  const logo: unknown = v['logo']
  const title: unknown = v['title']
  const dateRange: unknown = v['dateRange']
  const description: unknown = v['description']
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
  props: ExperienceSectionProps
  // eslint-disable-next-line max-lines-per-function
) => Promise<JSX.Element> = async ({
  locale,
}: ExperienceSectionProps): Promise<JSX.Element> => {
  const t: Translations<'experience'> = await getTranslations({
    locale,
    namespace: 'experience',
  })

  // Safely read raw list and validate
  const rawItems: unknown = t.raw('items')
  const experiences: readonly Experience[] = parseExperiences(rawItems)

  return (
    <section className="px-4 py-20">
      <div className="mx-auto w-full max-w-4xl">
        <div className="mb-12 text-center">
          <h2 className="text-foreground mb-3 text-4xl font-bold">
            {t('title')}
          </h2>
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
                      <h3 className="text-foreground group-hover:text-primary mb-1 text-xl font-semibold transition-colors">
                        {exp.title}
                      </h3>
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
