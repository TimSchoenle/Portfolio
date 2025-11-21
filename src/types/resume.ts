import type { Translations } from '@/types/i18n'

import type en from '../../messages/en.json'

export interface ResumeEducation {
  readonly degree: string
  readonly institution: string
  readonly year: string
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

export type ResumeData = typeof en.resume

export type ResumeTranslations = Translations<''>
