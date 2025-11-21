/* eslint-disable no-console */
/* eslint-disable unicorn/prefer-top-level-await */
/* eslint-disable security/detect-non-literal-fs-filename */
import { mkdir, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'

import React from 'react'

import { createTranslator, type Messages } from 'next-intl'

import { renderToBuffer } from '@react-pdf/renderer'

import { ResumePDFDocument } from '@/components/resume/resume-pdf-document'
import { routing } from '@/i18n/routing'
import type { Translations } from '@/types/i18n'

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

  for (const locale of routing.locales) {
    console.log(`Generating resume for locale: ${locale}`)

    // Load messages for the locale
    const messages: Messages =
      // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access,unicorn/no-await-expression-member
      (await import(`../messages/${locale}.json`)).default as Messages

    const translations: Translations<''> = createTranslator({
      locale,
      messages,
    })

    const element: React.ReactElement = React.createElement(ResumePDFDocument, {
      translations: translations,
    })

    // @ts-expect-error - React PDF type mismatch
    const buffer: Buffer = await renderToBuffer(element)

    const filename: string = `resume-${locale}.pdf`
    await writeFile(path.join(publicDirectory, filename), buffer)

    console.log(`✓ Generated ${filename}`)
  }

  console.log('Resume generation complete!')
})()
