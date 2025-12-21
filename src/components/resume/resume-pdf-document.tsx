import { type ReactElement } from 'react'

import { type createFormatter } from 'next-intl'

import { Document } from '@react-pdf/renderer'

import { ModernTemplate } from '@/components/resume/templates/modern'
import type { ResumeTranslations } from '@/types/resume'

interface ResumePDFDocumentProperties {
  readonly formatDate: ReturnType<typeof createFormatter>
  readonly translations: ResumeTranslations
}

export const ResumePDFDocument: (
  properties: ResumePDFDocumentProperties
) => ReactElement = ({
  formatDate,
  translations,
}: ResumePDFDocumentProperties): ReactElement => {
  return (
    <Document>
      <ModernTemplate formatDate={formatDate} translations={translations} />
    </Document>
  )
}
