import { type FC, type ReactElement } from 'react'

import { type createFormatter } from 'next-intl'

import { Text, View } from '@react-pdf/renderer'

import type { FCStrict } from '@/types/fc'
import type { ResumeExperience, ResumeTranslations } from '@/types/resume'

import { styles } from './modern.styles'

interface ExperienceSectionProperties {
  readonly formatDate: ReturnType<typeof createFormatter>
  readonly translations: ResumeTranslations
}

interface ExperienceItemHeaderProperties {
  readonly end: string
  readonly isCompact: boolean
  readonly start: string
  readonly title: string
}

interface CompanyRowProperties {
  readonly company: string
  readonly isCompact: boolean
  readonly location: string
}

interface ExperienceItemProperties {
  readonly exp: ResumeExperience
  readonly formatDate: ReturnType<typeof createFormatter>
  readonly index: number
  readonly isCompact: boolean
  readonly maxAchievementsPerEntry: number
  readonly presentLabel: string
}

/**
 * Calculates the maximum number of achievements to display per experience entry
 * based on total content density to prevent overflow.
 */
function calculateMaxAchievementsPerEntry(
  experiences: ResumeExperience[]
): number {
  const totalAchievements: number = experiences.reduce(
    (sum: number, exp: ResumeExperience): number =>
      sum + exp.achievements.length,
    0
  )

  // Heavy content: 4+ experiences with 12+ total achievements
  if (experiences.length >= 4 && totalAchievements > 12) {
    return 2
  }

  // Moderate content: 3 experiences with 9+ total achievements
  if (experiences.length === 3 && totalAchievements > 9) {
    return 3
  }

  // Light content: show all achievements
  return Number.POSITIVE_INFINITY
}

const ExperienceItemHeader: FCStrict<ExperienceItemHeaderProperties> = ({
  end,
  isCompact,
  start,
  title,
}: ExperienceItemHeaderProperties): ReactElement => (
  <View style={isCompact ? styles.jobHeaderCompact : styles.jobHeader}>
    <Text style={isCompact ? styles.jobTitleCompact : styles.jobTitle}>
      {title}
    </Text>
    <Text style={isCompact ? styles.dateTextCompact : styles.dateText}>
      {start}
      {' - '}
      {end}
    </Text>
  </View>
)

const CompanyRow: FCStrict<CompanyRowProperties> = ({
  company,
  isCompact,
  location,
}: CompanyRowProperties): ReactElement => (
  <View style={styles.companyRow}>
    <Text style={isCompact ? styles.companyTextCompact : styles.companyText}>
      {company}
      {' • '}
      {location}
    </Text>
  </View>
)

const ExperienceItem: FCStrict<ExperienceItemProperties> = ({
  exp,
  formatDate,
  index,
  isCompact,
  maxAchievementsPerEntry,
  presentLabel,
}: ExperienceItemProperties): ReactElement => {
  const startDate: Date = new Date(
    Date.UTC(exp.start.year, exp.start.month - 1, 1)
  )
  const start: string = formatDate.dateTime(startDate, {
    month: 'short',
    year: 'numeric',
  })

  let end: string = presentLabel
  if (exp.end !== null) {
    const endDate: Date = new Date(Date.UTC(exp.end.year, exp.end.month - 1, 1))
    end = formatDate.dateTime(endDate, {
      month: 'short',
      year: 'numeric',
    })
  }

  return (
    <View
      key={`${exp.company}-${index.toString()}`}
      style={isCompact ? styles.experienceItemCompact : styles.experienceItem}
    >
      <ExperienceItemHeader
        end={end}
        isCompact={isCompact}
        start={start}
        title={exp.title}
      />
      <CompanyRow
        company={exp.company}
        isCompact={isCompact}
        location={exp.location}
      />
      {exp.achievements.slice(0, maxAchievementsPerEntry).map(
        (achievement: string, achievementIndex: number): ReactElement => (
          <Text
            key={`achievement_${achievementIndex.toString()}`}
            style={isCompact ? styles.achievementCompact : styles.achievement}
          >
            {'• '}
            {achievement}
          </Text>
        )
      )}
    </View>
  )
}

export const ExperienceSection: FC<ExperienceSectionProperties> = ({
  formatDate,
  translations,
}: ExperienceSectionProperties): ReactElement => {
  const experiences: ResumeExperience[] = translations.raw(
    'resume.experience'
  ) as ResumeExperience[]

  // Use compact styles if there are more than 2 experience entries
  const isCompact: boolean = experiences.length > 2

  // Calculate max achievements per entry to prevent overflow
  const maxAchievementsPerEntry: number =
    calculateMaxAchievementsPerEntry(experiences)

  return (
    <>
      <Text style={styles.sectionTitleFirst}>
        {translations('resume.sectionTitles.experience')}
      </Text>
      <View style={styles.sectionDivider} />
      {experiences.map(
        (exp: ResumeExperience, index: number): ReactElement => (
          <ExperienceItem
            exp={exp}
            formatDate={formatDate}
            index={index}
            isCompact={isCompact}
            key={`${exp.company}-${index.toString()}`}
            maxAchievementsPerEntry={maxAchievementsPerEntry}
            presentLabel={translations('resume.present')}
          />
        )
      )}
    </>
  )
}
