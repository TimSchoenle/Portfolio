'use server'

import { type JSX } from 'react'

import { type Locale } from 'next-intl'

import { type LucideIcon } from 'lucide-react'
import { getTranslations } from 'next-intl/server'

import { getSkillIcon } from '@/components/sections/skill-icons'
import { TechRadar } from '@/components/sections/tech-radar/tech-radar'
import { GridPattern } from '@/components/ui/grid-pattern'
import { Section, SECTION_BACKGROUNDS } from '@/components/ui/section'
import { SectionContainer } from '@/components/ui/section-container'
import { SectionHeader } from '@/components/ui/section-header'
import { siteConfig, type Skill, SKILL_RENDER_AREAS } from '@/lib/config'
import { shouldShowSkill } from '@/lib/tech-radar-utilities'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'

/* ────────────────────────── types ────────────────────────── */

interface SkillsSectionProperties {
  readonly locale: Locale
  readonly performance?: boolean
}

interface SkillListProperties {
  readonly items: readonly Skill[]
  readonly title: string
}
/* ────────────────────── subcomponents ────────────────────── */

const SkillList: FCStrict<SkillListProperties> = ({
  items,
  title,
}: SkillListProperties): JSX.Element => {
  const filteredItems: Skill[] = items
    .filter((skill: Skill): boolean =>
      shouldShowSkill({ renderArea: SKILL_RENDER_AREAS.SECTION, skill })
    )
    .toSorted(
      (skillOne: Skill, skillTwo: Skill): number =>
        skillTwo.confidence - skillOne.confidence
    )

  return (
    <div className="space-y-4 text-center lg:text-left">
      <h3 className="text-xl font-semibold text-foreground">{title}</h3>
      {/* Visual List - Hidden from screen readers/reader mode to avoid duplication/issues */}
      <ul
        aria-hidden="true"
        className="flex flex-wrap justify-center gap-2 lg:justify-start"
      >
        {filteredItems.map((skill: Skill): JSX.Element => {
          const Icon: LucideIcon = getSkillIcon(skill.name)
          return (
            <li
              className="flex items-center gap-2 rounded-full border border-border bg-background px-3 py-1.5 text-sm transition-colors hover:border-primary/50 hover:bg-accent/50"
              key={skill.name}
            >
              <Icon className="h-3.5 w-3.5 text-muted-foreground" />
              <span>{skill.name}</span>
            </li>
          )
        })}
      </ul>

      {/* Screen Reader / Reader Mode Content - Visible to text parsers */}
      <p
        style={{
          height: '1px',
          left: '-10000px',
          overflow: 'hidden',
          position: 'absolute',
          top: 'auto',
          width: '1px',
        }}
      >
        {filteredItems.map((skill: Skill): string => skill.name).join(', ')}
      </p>
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
    <Section
      background={SECTION_BACKGROUNDS.MUTED}
      className="py-24"
      id="skills"
    >
      {/* Background Pattern */}
      <GridPattern size={24} />
      <div className="absolute top-0 left-0 -z-10 h-full w-full bg-[radial-gradient(circle_800px_at_100%_200px,#8080800a,transparent)]" />

      <SectionContainer size="lg">
        <SectionHeader title={translations('title')} underline={true} />

        <div className="grid gap-12 lg:grid-cols-2 lg:items-center lg:gap-8">
          {/* Tech Radar (Hidden on mobile, visible on large screens) */}
          <div aria-hidden="true" className="hidden lg:block print:hidden">
            <TechRadar
              buildTools={buildTools}
              frameworks={frameworks}
              infrastructure={infrastructure}
              languages={languages}
              locale={locale}
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
            <SkillList
              items={infrastructure}
              title={translations('infrastructure')}
            />
          </div>
        </div>
      </SectionContainer>
    </Section>
  )
}
