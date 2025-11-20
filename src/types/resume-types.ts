export interface ResumePersonalInfo {
  readonly email: string
  readonly github: string
  readonly linkedin?: string
  readonly location: string
  readonly name: string
  readonly title: string
}

export interface ResumeExperience {
  readonly achievements: readonly string[]
  readonly company: string
  readonly endDate: string
  readonly location: string
  readonly startDate: string
  readonly title: string
}

export interface ResumeProject {
  readonly description: string
  readonly name: string
  readonly technologies: readonly string[]
  readonly url?: string
}

export interface ResumeEducation {
  readonly degree: string
  readonly institution: string
  readonly year: string
}

export interface ResumeSkills {
  readonly expertise: readonly string[]
  readonly learning: readonly string[]
  readonly tools: readonly string[]
}

export interface ResumeData {
  readonly education: readonly ResumeEducation[]
  readonly experience: readonly ResumeExperience[]
  readonly personalInfo?: ResumePersonalInfo
  readonly projects: readonly ResumeProject[]
  readonly skills: ResumeSkills
  readonly summary: string
}
