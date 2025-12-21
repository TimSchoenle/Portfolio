import type { Translations } from '@/types/i18n'

import type en from '../../messages/en.json'

export interface ResumeEducation {
  readonly degree: string
  readonly end: ResumeDate | null
  readonly institution: string
  readonly start: ResumeDate
}

export interface ResumeDate {
  readonly month: number
  readonly year: number
}

export interface ResumeExperience {
  readonly achievements: readonly string[]
  readonly company: string
  readonly end: ResumeDate | null
  readonly location: string
  readonly start: ResumeDate
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
