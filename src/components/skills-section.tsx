'use server'

import { type JSX } from 'react'

import { type Locale } from 'next-intl'

import { getTranslations } from 'next-intl/server'

import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import { siteConfig } from '@/config'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'

/* ────────────────────────── types ────────────────────────── */

interface SkillsSectionProperties {
  readonly locale: Locale
}

type BadgeVariant = 'default' | 'outline' | 'secondary'

interface SkillCardProperties {
  readonly items: readonly string[]
  readonly title: string
  readonly variant: BadgeVariant
}

/* ────────────────────── subcomponent ─────────────────────── */

const SkillCard: FCStrict<SkillCardProperties> = ({
  items,
  title,
  variant,
}: SkillCardProperties): JSX.Element => {
  return (
    <Card className="group hover:border-primary/50 border-2 transition-all duration-300 hover:-translate-y-1 hover:shadow-xl">
      <CardHeader>
        <CardTitle className="text-foreground flex items-center gap-2">
          <div className="bg-primary h-2 w-2 rounded-full" />
          {title}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="flex flex-wrap gap-2">
          {items.map((skill: string, index: number): JSX.Element => {
            const delay: string = String(index * 50) + 'ms'
            return (
              <Badge
                className="transition-all hover:scale-105 hover:shadow-md"
                key={skill}
                style={{ animationDelay: delay }}
                variant={variant}
              >
                {skill}
              </Badge>
            )
          })}
        </div>
      </CardContent>
    </Card>
  )
}

/* ─────────────────────── main section ────────────────────── */

export const SkillsSection: AsyncPageFC<SkillsSectionProperties> = async ({
  locale,
}: SkillsSectionProperties): Promise<JSX.Element> => {
  const translations: Translations<'skills'> = await getTranslations({
    locale,
    namespace: 'skills',
  })

  const expertise: readonly string[] = siteConfig.skills.expertise
  const learning: readonly string[] = siteConfig.skills.learning
  const tools: readonly string[] = siteConfig.skills.tools

  return (
    <section className="bg-muted/30 relative px-4 py-20" id="skills">
      <div className="absolute inset-0 -z-10 bg-[linear-gradient(to_right,#8080800a_1px,transparent_1px),linear-gradient(to_bottom,#8080800a_1px,transparent_1px)] bg-[size:14px_24px]" />

      <div className="mx-auto w-full max-w-6xl">
        <div className="mb-12 text-center">
          <Heading as="h2" className="text-foreground mb-3 text-4xl font-bold">
            {translations('title')}
          </Heading>
          <div className="from-primary to-primary/60 mx-auto h-1 w-20 rounded-full bg-gradient-to-r" />
        </div>

        <div className="grid gap-6 md:grid-cols-3">
          <SkillCard
            items={expertise}
            title={translations('expertise')}
            variant="default"
          />
          <SkillCard
            items={learning}
            title={translations('learning')}
            variant="secondary"
          />
          <SkillCard
            items={tools}
            title={translations('tools')}
            variant="outline"
          />
        </div>
      </div>
    </section>
  )
}
