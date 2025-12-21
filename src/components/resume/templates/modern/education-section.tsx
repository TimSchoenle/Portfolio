import { type FC, type ReactElement } from 'react'

import { type createFormatter } from 'next-intl'

import { Text, View } from '@react-pdf/renderer'

import type { ResumeEducation, ResumeTranslations } from '@/types/resume'

import { styles } from './modern.styles'

interface EducationSectionProperties {
  readonly formatDate: ReturnType<typeof createFormatter>
  readonly translations: ResumeTranslations
}

export const EducationSection: FC<EducationSectionProperties> = ({
  formatDate,
  translations,
}: EducationSectionProperties): ReactElement => {
  const educations: ResumeEducation[] = translations.raw(
    'resume.education'
  ) as ResumeEducation[]

  return (
    <>
      <Text style={styles.sectionTitle}>
        {translations('resume.sectionTitles.education')}
      </Text>
      <View style={styles.sectionDivider} />
      {educations.map((edu: ResumeEducation, index: number): ReactElement => {
        const startDate: Date = new Date(
          Date.UTC(edu.start.year, edu.start.month - 1, 1)
        )
        const start: string = formatDate.dateTime(startDate, {
          month: 'short',
          year: 'numeric',
        })

        let end: string = 'Present'
        if (edu.end !== null) {
          const endDate: Date = new Date(
            Date.UTC(edu.end.year, edu.end.month - 1, 1)
          )
          end = formatDate.dateTime(endDate, {
            month: 'short',
            year: 'numeric',
          })
        }

        return (
          <View
            key={`${edu.institution}-${index.toString()}`}
            style={styles.educationItem}
          >
            <Text style={styles.educationDegree}>{edu.degree}</Text>
            <Text style={styles.educationInstitution}>{edu.institution}</Text>
            <Text style={styles.dateText}>
              {start}
              {' - '}
              {end}
            </Text>
          </View>
        )
      })}
    </>
  )
}
