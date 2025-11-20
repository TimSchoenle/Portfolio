import type {
  ResumeData,
  ResumeEducation,
  ResumeExperience,
  ResumePersonalInfo,
  ResumeProject,
  ResumeSkills,
} from '@/types/resume-types'

/* ────────────────────────── type guards ────────────────────────── */

function isResumeContactInfo(object: Record<string, unknown>): boolean {
  return (
    typeof object['email'] === 'string' &&
    typeof object['location'] === 'string' &&
    typeof object['github'] === 'string' &&
    (object['linkedin'] === undefined || typeof object['linkedin'] === 'string')
  )
}

function isResumePersonalInfo(data: unknown): data is ResumePersonalInfo {
  if (typeof data !== 'object' || data === null) {
    return false
  }

  const object: Record<string, unknown> = data as Record<string, unknown>

  return (
    typeof object['name'] === 'string' &&
    typeof object['title'] === 'string' &&
    isResumeContactInfo(object)
  )
}

function isStringArray(data: unknown): data is readonly string[] {
  return (
    Array.isArray(data) &&
    data.every((item: unknown): boolean => typeof item === 'string')
  )
}

function isResumeSkills(data: unknown): data is ResumeSkills {
  if (typeof data !== 'object' || data === null) {
    return false
  }

  const object: Record<string, unknown> = data as Record<string, unknown>

  return (
    isStringArray(object['expertise']) &&
    isStringArray(object['learning']) &&
    isStringArray(object['tools'])
  )
}

function isResumeExperience(data: unknown): data is ResumeExperience {
  if (typeof data !== 'object' || data === null) {
    return false
  }

  const object: Record<string, unknown> = data as Record<string, unknown>

  return (
    typeof object['title'] === 'string' &&
    typeof object['company'] === 'string' &&
    typeof object['location'] === 'string' &&
    typeof object['startDate'] === 'string' &&
    typeof object['endDate'] === 'string' &&
    isStringArray(object['achievements'])
  )
}

function isResumeProject(data: unknown): data is ResumeProject {
  if (typeof data !== 'object' || data === null) {
    return false
  }

  const object: Record<string, unknown> = data as Record<string, unknown>

  return (
    typeof object['name'] === 'string' &&
    typeof object['description'] === 'string' &&
    isStringArray(object['technologies']) &&
    (object['url'] === undefined || typeof object['url'] === 'string')
  )
}

function isResumeEducation(data: unknown): data is ResumeEducation {
  if (typeof data !== 'object' || data === null) {
    return false
  }

  const object: Record<string, unknown> = data as Record<string, unknown>

  return (
    typeof object['degree'] === 'string' &&
    typeof object['institution'] === 'string' &&
    typeof object['year'] === 'string'
  )
}

function areResumeArraysValid(object: Record<string, unknown>): boolean {
  return (
    Array.isArray(object['experience']) &&
    (object['experience'] as unknown[]).every((item: unknown): boolean =>
      isResumeExperience(item)
    ) &&
    Array.isArray(object['projects']) &&
    (object['projects'] as unknown[]).every((item: unknown): boolean =>
      isResumeProject(item)
    ) &&
    Array.isArray(object['education']) &&
    (object['education'] as unknown[]).every((item: unknown): boolean =>
      isResumeEducation(item)
    )
  )
}

function isResumeData(data: unknown): data is ResumeData {
  if (typeof data !== 'object' || data === null) {
    return false
  }

  const object: Record<string, unknown> = data as Record<string, unknown>

  // Check simple fields first
  if (
    !isResumePersonalInfo(object['personalInfo']) ||
    typeof object['summary'] !== 'string' ||
    !isResumeSkills(object['skills'])
  ) {
    return false
  }

  return areResumeArraysValid(object)
}

/* ──────────────────────── parsing function ────────────────────── */

export function parseResumeData(raw: unknown): ResumeData {
  if (!isResumeData(raw)) {
    throw new Error('Invalid resume data structure')
  }

  return raw
}
