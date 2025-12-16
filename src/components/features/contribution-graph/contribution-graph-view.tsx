import { type JSX } from 'react'

import { type Locale, useTranslations } from 'next-intl'

import { ContributionHoverProvider } from '@/components/features/contribution-graph/contribution-hover-context'
import { ContributionTooltip } from '@/components/features/contribution-graph/contribution-tooltip'
import { Card } from '@/components/ui/card'
import type {
  CalendarModel,
  DayLabelTripleResult,
} from '@/lib/github/contribution-calendar'
import type { ContributionPoint } from '@/models/github'
import type { FCStrict } from '@/types/fc'

import {
  GraphSection,
  HeaderSection,
} from './contribution-graph-view-components'

interface ContributionGraphViewProperties {
  readonly calendar: CalendarModel
  readonly currentYearData: ContributionPoint[]
  readonly labels: DayLabelTripleResult
  readonly locale: Locale
  readonly onYearChange: (year: number) => void
  readonly selectedYear: number
  readonly total: number
  readonly years: readonly number[]
}

export const ContributionGraphView: FCStrict<
  ContributionGraphViewProperties
> = ({
  calendar,
  currentYearData,
  labels,
  locale,
  onYearChange,
  selectedYear,
  total,
  years,
}: ContributionGraphViewProperties): JSX.Element => {
  const translate: ReturnType<typeof useTranslations> = useTranslations(
    'projects.contributions'
  )

  return (
    <Card className="w-full overflow-hidden border-2 p-6 transition-all duration-300 hover:border-primary/50 hover:shadow-lg dark:bg-card/50">
      <ContributionHoverProvider>
        <HeaderSection
          selectedYear={selectedYear}
          total={total}
          translate={translate}
          years={years}
          onYearChange={onYearChange}
        />
        <GraphSection calendar={calendar} labels={labels} locale={locale} />
        <ContributionTooltip data={currentYearData} locale={locale} />
      </ContributionHoverProvider>
    </Card>
  )
}
