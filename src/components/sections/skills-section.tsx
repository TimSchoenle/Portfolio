import { type JSX } from 'react'

import { type LucideIcon } from 'lucide-react'
import { getTranslations } from 'next-intl/server'

import { BlueprintCard } from '@/components/blueprint/blueprint-card'
import { BlueprintContainer } from '@/components/blueprint/blueprint-container'
import { BlueprintSectionDivider } from '@/components/blueprint/blueprint-section-divider'
import { BlueprintSectionTitle } from '@/components/blueprint/blueprint-section-title'
import { LazyLoad } from '@/components/common/lazy-load'
import { getSkillIcon } from '@/components/sections/skill-icons'
import { TechRadar } from '@/components/sections/tech-radar/tech-radar'
import { siteConfig, type Skill, SKILL_RENDER_AREAS } from '@/lib/config'
import { shouldShowSkill } from '@/lib/tech-radar-utilities'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { LocalePageProperties, Translations } from '@/types/i18n'

/* ── types ─────────────────────────────────────────────────────────────── */

type SkillsSectionProperties = LocalePageProperties

interface SkillListProperties {
  readonly items: readonly Skill[]
  readonly title: string
}

/* ── subcomponents ─────────────────────────────────────────────────────── */

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
      <div className="inline-block border-l-2 border-brand pl-3">
        <h3 className="font-mono text-sm font-bold tracking-[0.2em] text-blueprint-text uppercase">
          {title}
        </h3>
      </div>

      <ul
        aria-hidden="true"
        className="flex flex-wrap justify-center gap-2 lg:justify-start"
      >
        {filteredItems.map((skill: Skill): JSX.Element => {
          const Icon: LucideIcon = getSkillIcon(skill.name)
          return (
            <li
              className="hover:shadow-[0_0_10px_color-mix(in srgb, var(--brand), transparent 80%)] flex items-center gap-2 border border-brand/30 bg-brand/5 px-3 py-1.5 font-mono text-xs tracking-wider text-blueprint-muted uppercase transition-all hover:bg-brand/20 hover:text-blueprint-text"
              key={skill.name}
            >
              <Icon className="h-3 w-3" />
              <span>{skill.name}</span>
            </li>
          )
        })}
      </ul>
    </div>
  )
}

const TechRadarContainer: FCStrict<{
  readonly buildTools: readonly Skill[]
  readonly frameworks: readonly Skill[]
  readonly infrastructure: readonly Skill[]
  readonly languages: readonly Skill[]
  readonly locale: string
}> = ({
  buildTools,
  frameworks,
  infrastructure,
  languages,
  locale,
}: {
  readonly buildTools: readonly Skill[]
  readonly frameworks: readonly Skill[]
  readonly infrastructure: readonly Skill[]
  readonly languages: readonly Skill[]
  readonly locale: string
}): JSX.Element => (
  <BlueprintCard
    className="relative hidden h-full items-center justify-center lg:flex"
    label="RADAR_SCAN"
    noPadding={true}
  >
    {/* Tech Radar (Hidden on small mobile if needed, but keeping logic similar) */}
    <div className="h-full w-full">
      <LazyLoad className="h-full w-full">
        <TechRadar
          buildTools={buildTools}
          frameworks={frameworks}
          infrastructure={infrastructure}
          languages={languages}
          locale={locale}
        />
      </LazyLoad>
    </div>
  </BlueprintCard>
)

const SkillMatrix: FCStrict<{
  readonly buildTools: readonly Skill[]
  readonly frameworks: readonly Skill[]
  readonly infrastructure: readonly Skill[]
  readonly languages: readonly Skill[]
  readonly translations: Translations<'skills'>
}> = ({
  buildTools,
  frameworks,
  infrastructure,
  languages,
  translations,
}: {
  readonly buildTools: readonly Skill[]
  readonly frameworks: readonly Skill[]
  readonly infrastructure: readonly Skill[]
  readonly languages: readonly Skill[]
  readonly translations: Translations<'skills'>
}): JSX.Element => (
  <BlueprintCard className="h-full" label="SKILL_MATRIX">
    <div className="space-y-8">
      <SkillList items={languages} title={translations('languages')} />
      <SkillList items={frameworks} title={translations('frameworks')} />
      <SkillList items={buildTools} title={translations('buildTools')} />
      <SkillList
        items={infrastructure}
        title={translations('infrastructure')}
      />
    </div>
  </BlueprintCard>
)

/* ── main ──────────────────────────────────────────────────── */

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
    <BlueprintContainer id="skills" isLazy={true}>
      <div className="mx-auto flex w-full max-w-6xl flex-col items-center">
        <BlueprintSectionTitle
          sectionLabel="// TECH_STACK_ANALYSIS"
          title={translations('title')}
        />

        <div className="mt-8 grid w-full gap-8 lg:grid-cols-[1.5fr_1fr] lg:items-start">
          {/* Left Column: Tech Radar */}
          <TechRadarContainer
            buildTools={buildTools}
            frameworks={frameworks}
            infrastructure={infrastructure}
            languages={languages}
            locale={locale}
          />

          {/* Right Column: Skill Lists */}
          <SkillMatrix
            buildTools={buildTools}
            frameworks={frameworks}
            infrastructure={infrastructure}
            languages={languages}
            translations={translations}
          />
        </div>

        <BlueprintSectionDivider label="SYSTEM_ANALYSIS_COMPLETE" />
      </div>
    </BlueprintContainer>
  )
}

export default SkillsSection
