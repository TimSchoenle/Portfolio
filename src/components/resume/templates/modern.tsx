import { type FC, type ReactElement } from 'react'

import { Page, Text, View } from '@react-pdf/renderer'

import { SkillsSection } from '@/components/resume/templates/modern/skills-section'
import { siteConfig } from '@/lib/config'
import type { ResumeTranslations } from '@/types/resume'

import { ContactSection } from './modern/contact-section'
import { EducationSection } from './modern/education-section'
import { ExperienceSection } from './modern/experience-section'
import { styles } from './modern/modern.styles'
import { ProjectsSection } from './modern/projects-section'

interface ModernTemplateProperties {
  readonly translations: ResumeTranslations
}

export const ModernTemplate: FC<ModernTemplateProperties> = ({
  translations,
}: ModernTemplateProperties): ReactElement => {
  return (
    <Page size="A4" style={styles.page}>
      <View style={styles.topHeader}>
        <Text style={styles.name}>{siteConfig.name}</Text>
        <Text style={styles.title}>
          {translations('personalInfo.jobTitle')}
        </Text>
        <Text style={styles.summary}>{translations('resume.summary')}</Text>
      </View>

      <View style={styles.mainContainer}>
        {/* Left sidebar */}
        <View style={styles.leftColumn}>
          <ContactSection translations={translations} />
          <SkillsSection translations={translations} />
          <EducationSection translations={translations} />
        </View>

        {/* Main content */}
        <View style={styles.rightColumn}>
          <ExperienceSection translations={translations} />
          <ProjectsSection translations={translations} />
        </View>
      </View>
    </Page>
  )
}
