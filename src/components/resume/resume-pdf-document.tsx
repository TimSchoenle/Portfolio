import { type ReactElement } from 'react'

import { Document } from '@react-pdf/renderer'

import { ModernTemplate } from '@/components/resume/templates/modern'
import type { ResumeData } from '@/types/resume-types'

interface ResumePDFDocumentProperties {
  readonly data: ResumeData
  readonly titles?: Record<string, string>
}

// eslint-disable-next-line @typescript-eslint/typedef
export const ResumePDFDocument = ({
  data,
  titles = {},
}: ResumePDFDocumentProperties): ReactElement => {
  return (
    <Document>
      <ModernTemplate data={data} titles={titles} />
    </Document>
  )
}
