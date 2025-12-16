import React, { type JSX } from 'react'

import {
  type Locale,
  type TranslationValues,
  type useTranslations,
} from 'next-intl'

import {
  LegendBar,
  MonthLabelsRow,
  WeeksGrid,
} from '@/components/features/contribution-graph/contribution-graph-components'
import { Heading } from '@/components/ui/heading'
import type {
  CalendarModel,
  DayLabelTripleResult,
} from '@/lib/github/contribution-calendar'
import type { FCStrict } from '@/types/fc'

import { InteractiveArea } from './interactive-area'

interface HeaderSectionProperties {
  readonly onYearChange: (year: number) => void
  readonly selectedYear: number
  readonly total: number
  readonly translate: ReturnType<typeof useTranslations>
  readonly years: readonly number[]
}

export const HeaderSection: FCStrict<HeaderSectionProperties> = ({
  onYearChange,
  selectedYear,
  total,
  translate,
  years,
}: HeaderSectionProperties): JSX.Element => {
  return (
    <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
      <div className="flex flex-col gap-2">
        <Heading as="h3" className="text-2xl font-bold">
          {translate('title')}
        </Heading>
        <p className="text-sm text-muted-foreground">
          {translate('totalAmount', {
            count: total,
            year: selectedYear,
          } as TranslationValues)}
        </p>
      </div>

      <div className="flex items-center gap-4">
        <select
          aria-label="Select Year"
          className="h-9 w-24 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50"
          value={selectedYear}
          onChange={(event: React.ChangeEvent<HTMLSelectElement>): void => {
            onYearChange(Number(event.target.value))
          }}
        >
          {years.map(
            (year: number): JSX.Element => (
              <option
                className="bg-popover text-popover-foreground"
                key={year}
                value={year}
              >
                {year}
              </option>
            )
          )}
        </select>
        <LegendBar less={translate('less')} more={translate('more')} />
      </div>
    </div>
  )
}

interface GraphSectionProperties {
  readonly calendar: CalendarModel
  readonly labels: DayLabelTripleResult
  readonly locale: Locale
}

export const GraphSection: FCStrict<GraphSectionProperties> = ({
  calendar,
  labels,
  locale,
}: GraphSectionProperties): JSX.Element => {
  return (
    <div className="relative w-full overflow-x-auto pb-2">
      <div className="inline-block min-w-full">
        <MonthLabelsRow labels={calendar.monthLabels} />
        <InteractiveArea>
          <WeeksGrid
            dayFive={labels.dayFive}
            dayOne={labels.dayOne}
            dayThree={labels.dayThree}
            locale={locale}
            weeks={calendar.weeks}
          />
        </InteractiveArea>
      </div>
    </div>
  )
}
