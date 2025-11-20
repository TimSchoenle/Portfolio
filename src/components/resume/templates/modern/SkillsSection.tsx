/* eslint-disable react/jsx-curly-brace-presence */
import { type FC, type ReactElement } from 'react'

import { Text, View } from '@react-pdf/renderer'

import type { ResumeSkills } from '@/types/resume-types'

import { styles } from './modern.styles'

interface SkillsSectionProperties {
  readonly skills: ResumeSkills
  readonly titles: Record<string, string>
}

export const SkillsSection: FC<SkillsSectionProperties> = ({
  skills,
  titles,
}: SkillsSectionProperties): ReactElement => (
  <>
    <Text style={styles.sectionTitle}>
      {titles['skills'] ?? 'Technical Skills'}
    </Text>
    <View style={styles.sectionDivider} />

    <Text style={styles.contactLabel}>{'Expertise'}</Text>
    <View style={styles.skillsContainer}>
      {skills.expertise.map(
        (skill: string): ReactElement => (
          <Text key={skill} style={styles.skillTag}>
            {skill}
          </Text>
        )
      )}
    </View>

    <Text style={styles.contactLabel}>{'Currently Learning'}</Text>
    <View style={styles.skillsContainer}>
      {skills.learning.map(
        (skill: string): ReactElement => (
          <Text key={skill} style={styles.skillTag}>
            {skill}
          </Text>
        )
      )}
    </View>

    <Text style={styles.contactLabel}>{'Tools & Platforms'}</Text>
    <View style={styles.skillsContainer}>
      {skills.tools.map(
        (skill: string): ReactElement => (
          <Text key={skill} style={styles.skillTag}>
            {skill}
          </Text>
        )
      )}
    </View>
  </>
)
