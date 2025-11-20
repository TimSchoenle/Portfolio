/* eslint-disable no-console */
/* eslint-disable unicorn/prefer-top-level-await */
/* eslint-disable security/detect-non-literal-fs-filename */
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'

import React from 'react'

import { renderToBuffer } from '@react-pdf/renderer'

import { ResumePDFDocument } from '@/components/resume/resume-pdf-document'
import {
  getResumeSummary,
  resumeEducation,
  resumeExperience,
  resumeProjects,
  resumeSkills,
} from '@/data/resume'
import { siteConfig } from '@/lib/config'
import type { ResumeData, ResumePersonalInfo } from '@/types/resume-types'

type Locale = 'de' | 'en'

const LOCALES: readonly Locale[] = ['en', 'de']

// Main execution wrapped in async IIFE to support CJS output format
void (async (): Promise<void> => {
  const publicDirectory: string = path.join(process.cwd(), 'public', 'resume')

  // Delete existing directory to ensure clean slate
  try {
    await rm(publicDirectory, { force: true, recursive: true })
    console.log('✓ Cleaned resume directory')
  } catch {
    // Directory doesn't exist, that's fine
  }

  // Create fresh directory
  await mkdir(publicDirectory, { recursive: true })

  // Build personal info from siteConfig
  const personalInfo: ResumePersonalInfo = {
    email: siteConfig.email,
    github: siteConfig.github,
    location: siteConfig.location,
    name: siteConfig.fullName,
    title: siteConfig.jobTitle,
  }

  for (const locale of LOCALES) {
    console.log(`Generating resume for locale: ${locale}`)

    // Read i18n file for section titles
    const messagesPath: string = path.join(
      process.cwd(),
      'messages',
      `${locale}.json`
    )
    const messagesContent: string = await readFile(messagesPath, 'utf8')
    const messages: Record<string, unknown> = JSON.parse(
      messagesContent
    ) as Record<string, unknown>
    const contactSection: unknown = messages['contact']

    // Extract section titles
    const sectionTitles: Record<string, string> =
      typeof contactSection === 'object' && contactSection !== null
        ? ((contactSection as Record<string, unknown>)[
            'sectionTitles'
          ] as Record<string, string>)
        : {}

    // Build complete resume data
    const resumeData: ResumeData = {
      education: resumeEducation,
      experience: resumeExperience,
      personalInfo,
      projects: resumeProjects,
      skills: resumeSkills,
      summary: getResumeSummary(locale),
    }

    const element: React.ReactElement = React.createElement(ResumePDFDocument, {
      data: resumeData,
      titles: sectionTitles,
    })

    // @ts-expect-error - React PDF type mismatch
    const buffer: Buffer = await renderToBuffer(element)

    const filename: string = `resume-${locale}.pdf`
    await writeFile(path.join(publicDirectory, filename), buffer)

    console.log(`✓ Generated ${filename}`)
  }

  console.log('Resume generation complete!')
})()
