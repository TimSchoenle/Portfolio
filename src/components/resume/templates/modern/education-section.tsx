import { type FC, type ReactElement } from 'react'

import { Text, View } from '@react-pdf/renderer'

import type { ResumeEducation, ResumeTranslations } from '@/types/resume'

import { styles } from './modern.styles'

interface EducationSectionProperties {
  readonly translations: ResumeTranslations
}

export const EducationSection: FC<EducationSectionProperties> = ({
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
      {educations.map(
        (edu: ResumeEducation, index: number): ReactElement => (
          <View
            key={`${edu.institution}-${index.toString()}`}
            style={styles.educationItem}
          >
            <Text style={styles.educationDegree}>{edu.degree}</Text>
            <Text style={styles.educationInstitution}>{edu.institution}</Text>
            <Text style={styles.dateText}>{edu.year}</Text>
          </View>
        )
      )}
    </>
  )
}
