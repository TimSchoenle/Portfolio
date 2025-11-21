import { type FC, type ReactElement } from 'react'

import { Text, View } from '@react-pdf/renderer'

import type { ResumeExperience, ResumeTranslations } from '@/types/resume'

import { styles } from './modern.styles'

interface ExperienceSectionProperties {
  readonly translations: ResumeTranslations
}

export const ExperienceSection: FC<ExperienceSectionProperties> = ({
  translations,
}: ExperienceSectionProperties): ReactElement => {
  const experiences: ResumeExperience[] = translations.raw(
    'resume.experience'
  ) as ResumeExperience[]

  return (
    <>
      <Text style={styles.sectionTitleFirst}>
        {translations('resume.sectionTitles.experience')}
      </Text>
      <View style={styles.sectionDivider} />
      {experiences.map(
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
}
