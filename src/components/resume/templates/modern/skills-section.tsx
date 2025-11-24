import { type FC, type ReactElement } from 'react'

import { Text, View } from '@react-pdf/renderer'

import { siteConfig, type Skill } from '@/lib/config'
import type { ResumeTranslations } from '@/types/resume'

import { styles } from './modern.styles'

interface SkillsSectionProperties {
  readonly translations: ResumeTranslations
}

export const SkillsSection: FC<SkillsSectionProperties> = ({
  translations,
}: SkillsSectionProperties): ReactElement => (
  <>
    <Text style={styles.sectionTitle}>
      {translations('resume.sectionTitles.skills')}
    </Text>
    <View style={styles.sectionDivider} />

    <Text style={styles.contactLabel}>
      {translations('resume.sectionTitles.skillsSubTypes.languages')}
    </Text>
    <View style={styles.skillsContainer}>
      {siteConfig.skills.languages.map(
        (skill: Skill): ReactElement => (
          <Text key={skill.name} style={styles.skillTag}>
            {skill.name}
          </Text>
        )
      )}
    </View>

    <Text style={styles.contactLabel}>
      {translations('resume.sectionTitles.skillsSubTypes.frameworks')}
    </Text>
    <View style={styles.skillsContainer}>
      {siteConfig.skills.frameworks.map(
        (skill: Skill): ReactElement => (
          <Text key={skill.name} style={styles.skillTag}>
            {skill.name}
          </Text>
        )
      )}
    </View>

    <Text style={styles.contactLabel}>
      {translations('resume.sectionTitles.skillsSubTypes.buildTools')}
    </Text>
    <View style={styles.skillsContainer}>
      {siteConfig.skills.buildTools.map(
        (skill: Skill): ReactElement => (
          <Text key={skill.name} style={styles.skillTag}>
            {skill.name}
          </Text>
        )
      )}
    </View>

    <Text style={styles.contactLabel}>
      {translations('resume.sectionTitles.skillsSubTypes.infrastructure')}
    </Text>
    <View style={styles.skillsContainer}>
      {siteConfig.skills.infrastructure.map(
        (skill: Skill): ReactElement => (
          <Text key={skill.name} style={styles.skillTag}>
            {skill.name}
          </Text>
        )
      )}
    </View>
  </>
)
