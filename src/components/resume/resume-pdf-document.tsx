import { type ReactElement } from 'react'

import { Document } from '@react-pdf/renderer'

import { ModernTemplate } from '@/components/resume/templates/modern'
import type { ResumeTranslations } from '@/types/resume'

interface ResumePDFDocumentProperties {
  readonly translations: ResumeTranslations
}

// eslint-disable-next-line @typescript-eslint/typedef
export const ResumePDFDocument = ({
  translations,
}: ResumePDFDocumentProperties): ReactElement => {
  return (
    <Document>
      <ModernTemplate translations={translations} />
    </Document>
  )
}
