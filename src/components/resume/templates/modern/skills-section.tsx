import { type ReactElement } from 'react'

import { Text, View } from '@react-pdf/renderer'

import { siteConfig } from '@/data/config'
import { shouldShowSkill } from '@/lib/tech-radar-utilities'
import type { FCNullable, FCStrict } from '@/types/fc'
import type { ResumeTranslations } from '@/types/resume'
import { type Skill, SKILL_RENDER_AREAS } from '@/types/skill'

import { styles } from './modern.styles'

interface SkillSectionProperties {
  readonly isCompact?: boolean
  readonly limit?: number
  readonly skills: readonly Skill[]
  readonly title: string
}

const SkillSection: FCNullable<SkillSectionProperties> = ({
  isCompact = false,
  limit = 12,
  skills,
  title,
}: SkillSectionProperties): ReactElement | null => {
  const visibleSkills: Skill[] = skills
    .filter((skill: Skill): boolean =>
      shouldShowSkill({ renderArea: SKILL_RENDER_AREAS.RESUME, skill })
    )
    .toSorted(
      (skillOne: Skill, skillTwo: Skill): number =>
        skillTwo.confidence - skillOne.confidence
    )
    .slice(0, limit)

  return visibleSkills.length > 0 ? (
    <>
      <Text style={styles.contactLabel}>{title}</Text>
      <View
        style={
          isCompact ? styles.skillsContainerCompact : styles.skillsContainer
        }
      >
        {visibleSkills.map(
          (skill: Skill): ReactElement => (
            <Text
              key={skill.name}
              style={isCompact ? styles.skillTagCompact : styles.skillTag}
            >
              {skill.name}
            </Text>
          )
        )}
      </View>
    </>
  ) : null
}

interface SkillsSectionProperties {
  readonly translations: ResumeTranslations
}

export const SkillsSection: FCStrict<SkillsSectionProperties> = ({
  translations,
}: SkillsSectionProperties): ReactElement => {
  // Check total skill count to decide on compactness
  const allSkills: Skill[] = [
    ...siteConfig.skills.languages,
    ...siteConfig.skills.frameworks,
    ...siteConfig.skills.buildTools,
    ...siteConfig.skills.infrastructure,
  ]
  const totalSkills: number = allSkills.filter((skill: Skill): boolean =>
    shouldShowSkill({ renderArea: SKILL_RENDER_AREAS.RESUME, skill: skill })
  ).length

  // Enable compact mode if we have a lot of skills (> 20)
  const isCompact: boolean = totalSkills > 20
  // Tighter limit per category if we have many categories populated
  const categoryLimit: number = isCompact ? 15 : 10

  return (
    <>
      <Text style={styles.sectionTitle}>
        {translations('resume.sectionTitles.skills')}
      </Text>
      <View style={styles.sectionDivider} />

      <SkillSection
        isCompact={isCompact}
        limit={categoryLimit}
        skills={siteConfig.skills.languages}
        title={translations('resume.sectionTitles.skillsSubTypes.languages')}
      />

      <SkillSection
        isCompact={isCompact}
        limit={categoryLimit}
        skills={siteConfig.skills.frameworks}
        title={translations('resume.sectionTitles.skillsSubTypes.frameworks')}
      />

      <SkillSection
        isCompact={isCompact}
        limit={categoryLimit}
        skills={siteConfig.skills.buildTools}
        title={translations('resume.sectionTitles.skillsSubTypes.buildTools')}
      />

      <SkillSection
        isCompact={isCompact}
        limit={categoryLimit}
        skills={siteConfig.skills.infrastructure}
        title={translations(
          'resume.sectionTitles.skillsSubTypes.infrastructure'
        )}
      />
    </>
  )
}
