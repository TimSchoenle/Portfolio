import { type FC, type ReactElement } from 'react'

import { Text, View } from '@react-pdf/renderer'

import type { ResumeProject } from '@/types/resume-types'

import { styles } from './modern.styles'

interface ProjectsSectionProperties {
  readonly projects: readonly ResumeProject[]
  readonly titles: Record<string, string>
}

export const ProjectsSection: FC<ProjectsSectionProperties> = ({
  projects,
  titles,
}: ProjectsSectionProperties): ReactElement => (
  <>
    <Text style={styles.sectionTitle}>
      {titles['projects'] ?? 'Notable Projects'}
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
