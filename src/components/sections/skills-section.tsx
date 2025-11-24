/* eslint-disable @typescript-eslint/typedef */
'use server'

import { type JSX } from 'react'

import { type Locale } from 'next-intl'

import { type LucideIcon } from 'lucide-react'
import { getTranslations } from 'next-intl/server'

import { getSkillIcon } from '@/components/sections/skill-icons'
import { TechRadar } from '@/components/sections/tech-radar'
import { Heading } from '@/components/ui/heading'
import { siteConfig, type Skill } from '@/lib/config'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'

/* ────────────────────────── types ────────────────────────── */

interface SkillsSectionProperties {
  readonly locale: Locale
}

/* ────────────────────── subcomponents ────────────────────── */

const SkillList: FCStrict<{
  readonly items: readonly Skill[]
  readonly title: string
}> = ({ items, title }): JSX.Element => {
  return (
    <div className="space-y-4 text-center lg:text-left">
      <h3 className="text-xl font-semibold text-foreground">{title}</h3>
      <div className="flex flex-wrap justify-center gap-2 lg:justify-start">
        {items.map((skill: Skill): JSX.Element => {
          const Icon: LucideIcon = getSkillIcon(skill.name)
          return (
            <div
              className="flex items-center gap-2 rounded-full border border-border bg-background px-3 py-1.5 text-sm transition-colors hover:border-primary/50 hover:bg-accent/50"
              key={skill.name}
            >
              <Icon className="h-3.5 w-3.5 text-muted-foreground" />
              <span>{skill.name}</span>
            </div>
          )
        })}
      </div>
    </div>
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

  const languages: readonly Skill[] = siteConfig.skills.languages
  const frameworks: readonly Skill[] = siteConfig.skills.frameworks
  const buildTools: readonly Skill[] = siteConfig.skills.buildTools
  const infrastructure: readonly Skill[] = siteConfig.skills.infrastructure

  return (
    <section
      className="relative overflow-hidden bg-muted/30 px-4 py-24"
      id="skills"
    >
      {/* Background Pattern */}
      <div className="absolute inset-0 -z-10 bg-[linear-gradient(to_right,#8080800a_1px,transparent_1px),linear-gradient(to_bottom,#8080800a_1px,transparent_1px)] bg-[size:24px_24px]" />
      <div className="absolute top-0 left-0 -z-10 h-full w-full bg-[radial-gradient(circle_800px_at_100%_200px,#8080800a,transparent)]" />

      <div className="mx-auto w-full max-w-6xl">
        <div className="mb-12 text-center md:mb-16">
          <Heading
            as="h2"
            className="mb-4 text-4xl font-bold tracking-tight text-foreground md:text-5xl"
          >
            {translations('title')}
          </Heading>
          <div className="mx-auto h-1.5 w-24 rounded-full bg-gradient-to-r from-primary to-primary/40" />
        </div>

        <div className="grid gap-12 lg:grid-cols-2 lg:items-center lg:gap-8">
          {/* Tech Radar (Hidden on mobile, visible on large screens) */}
          <div className="hidden lg:block">
            <TechRadar
              buildTools={buildTools}
              frameworks={frameworks}
              infrastructure={infrastructure}
              languages={languages}
            />
          </div>

          {/* Skill Lists (Visible on all screens, but styled differently on desktop) */}
          <div className="space-y-8 lg:pl-8">
            <div className="lg:hidden">
              {/* Mobile only intro or alternative view could go here, but the lists below are fine */}
            </div>

            <SkillList items={languages} title={translations('languages')} />
            <SkillList items={frameworks} title={translations('frameworks')} />
            <SkillList items={buildTools} title={translations('buildTools')} />
            <SkillList items={infrastructure} title={translations('infrastructure')} />
          </div>
        </div>
      </div>
    </section>
  )
}
