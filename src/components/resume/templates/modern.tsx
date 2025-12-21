import { type FC, type JSX, type ReactElement, type ReactNode } from 'react'

import { type createFormatter } from 'next-intl'

import { Page, Text, View } from '@react-pdf/renderer'

import { SkillsSection } from '@/components/resume/templates/modern/skills-section'
import { siteConfig } from '@/lib/config'
import type { ResumeTranslations } from '@/types/resume'

import { ContactSection } from './modern/contact-section'
import { EducationSection } from './modern/education-section'
import { ExperienceSection } from './modern/experience-section'
import { styles } from './modern/modern.styles'

interface ModernTemplateProperties {
  readonly formatDate: ReturnType<typeof createFormatter>
  readonly translations: ResumeTranslations
}

export const ModernTemplate: FC<ModernTemplateProperties> = ({
  formatDate,
  translations,
}: ModernTemplateProperties): ReactElement => {
  return (
    <Page size="A4" style={styles.page}>
      <View style={styles.topHeader}>
        <Text style={styles.name}>{siteConfig.fullName}</Text>
        <Text style={styles.title}>
          {translations('personalInfo.jobTitle')}
        </Text>
        <Text style={styles.summary}>
          {translations.rich('about.summary', {
            highlight: (chunks: ReactNode): JSX.Element =>
              chunks as JSX.Element,
          })}
        </Text>
      </View>

      <View style={styles.mainContainer}>
        {/* Left sidebar */}
        <View style={styles.leftColumn}>
          <ContactSection translations={translations} />
          <SkillsSection translations={translations} />
          <EducationSection
            formatDate={formatDate}
            translations={translations}
          />
        </View>

        {/* Main content */}
        <View style={styles.rightColumn}>
          <ExperienceSection
            formatDate={formatDate}
            translations={translations}
          />
        </View>
      </View>
    </Page>
  )
}
