import { type FC, type ReactElement } from 'react'

import { Page, Text, View } from '@react-pdf/renderer'

import type { ResumeData, ResumePersonalInfo } from '@/types/resume-types'

import { ContactSection } from './modern/ContactSection'
import { EducationSection } from './modern/EducationSection'
import { ExperienceSection } from './modern/ExperienceSection'
import { styles } from './modern/modern.styles'
import { ProjectsSection } from './modern/ProjectsSection'
import { SkillsSection } from './modern/SkillsSection'

interface ModernTemplateProperties {
  readonly data: ResumeData
  readonly titles?: Record<string, string>
}

export const ModernTemplate: FC<ModernTemplateProperties> = ({
  data,
  titles = {},
}: ModernTemplateProperties): ReactElement => {
  const personalInfo: ResumePersonalInfo = data.personalInfo ?? {
    email: '',
    github: '',
    location: '',
    name: '',
    title: '',
  }

  return (
    <Page size="A4" style={styles.page}>
      <View style={styles.topHeader}>
        <Text style={styles.name}>{personalInfo.name}</Text>
        <Text style={styles.title}>{personalInfo.title}</Text>
        <Text style={styles.summary}>{data.summary}</Text>
      </View>

      <View style={styles.mainContainer}>
        {/* Left sidebar */}
        <View style={styles.leftColumn}>
          <ContactSection personalInfo={personalInfo} titles={titles} />
          <SkillsSection skills={data.skills} titles={titles} />
          <EducationSection education={data.education} titles={titles} />
        </View>

        {/* Main content */}
        <View style={styles.rightColumn}>
          <ExperienceSection experience={data.experience} titles={titles} />
          <ProjectsSection projects={data.projects} titles={titles} />
        </View>
      </View>
    </Page>
  )
}
