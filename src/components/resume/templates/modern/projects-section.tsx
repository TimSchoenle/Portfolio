import { type FC, type ReactElement } from 'react'

import { Text, View } from '@react-pdf/renderer'

import type { ResumeProject, ResumeTranslations } from '@/types/resume'

import { styles } from './modern.styles'

interface ProjectsSectionProperties {
  readonly translations: ResumeTranslations
}

export const ProjectsSection: FC<ProjectsSectionProperties> = ({
  translations,
}: ProjectsSectionProperties): ReactElement => {
  const projects: ResumeProject[] = translations.raw(
    'resume.projects'
  ) as ResumeProject[]

  return (
    <>
      <Text style={styles.sectionTitle}>
        {translations('resume.sectionTitles.projects')}
      </Text>
      <View style={styles.sectionDivider} />
      {projects.map(
        (project: ResumeProject, index: number): ReactElement => (
          <View
            key={`${project.name}-${index.toString()}`}
            style={styles.projectItem}
          >
            <Text style={styles.projectName}>{project.name}</Text>
            <Text style={styles.projectDescription}>{project.description}</Text>
            <Text style={styles.projectTech}>
              {project.technologies.join(' • ')}
            </Text>
          </View>
        )
      )}
    </>
  )
}
