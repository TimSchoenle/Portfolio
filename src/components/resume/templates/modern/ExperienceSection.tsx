import { type FC, type ReactElement } from 'react'

import { Text, View } from '@react-pdf/renderer'

import type { ResumeExperience } from '@/types/resume-types'

import { styles } from './modern.styles'

interface ExperienceSectionProperties {
  readonly experience: readonly ResumeExperience[]
  readonly titles: Record<string, string>
}

export const ExperienceSection: FC<ExperienceSectionProperties> = ({
  experience,
  titles,
}: ExperienceSectionProperties): ReactElement => (
  <>
    <Text style={styles.sectionTitleFirst}>
      {titles['experience'] ?? 'Professional Experience'}
    </Text>
    <View style={styles.sectionDivider} />
    {experience.map(
      (exp: ResumeExperience, index: number): ReactElement => (
        <View
          key={`${exp.company}-${index.toString()}`}
          style={styles.experienceItem}
        >
          <View style={styles.jobHeader}>
            <Text style={styles.jobTitle}>{exp.title}</Text>
            <Text style={styles.dateText}>
              {exp.startDate}
              {' - '}
              {exp.endDate}
            </Text>
          </View>
          <View style={styles.companyRow}>
            <Text style={styles.companyText}>
              {exp.company}
              {' • '}
              {exp.location}
            </Text>
          </View>
          {exp.achievements.map(
            (achievement: string, achievementIndex: number): ReactElement => (
              <Text
                key={`achievement_${achievementIndex.toString()}`}
                style={styles.achievement}
              >
                {'• '}
                {achievement}
              </Text>
            )
          )}
        </View>
      )
    )}
  </>
)
