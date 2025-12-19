import React, { type JSX } from 'react'

import {
  type Locale,
  type TranslationValues,
  type useTranslations,
} from 'next-intl'

import { ContributionGraphSvg } from '@/components/features/contribution-graph/contribution-graph-svg'
import { Heading } from '@/components/ui/heading'
import type {
  CalendarModel,
  DayLabelTripleResult,
} from '@/lib/github/contribution-calendar'
import type { FCStrict } from '@/types/fc'

interface LegendBarProperties {
  readonly less: string
  readonly more: string
}

const LegendBar: FCStrict<LegendBarProperties> = ({
  less,
  more,
}: LegendBarProperties): JSX.Element => (
  <div className="flex items-center gap-2 text-xs text-muted-foreground">
    <span>{less}</span>
    <div aria-hidden="true" className="flex gap-1">
      {[0, 1, 2, 3, 4].map(
        (level: number): JSX.Element => (
          <div
            className="h-4 w-4 rounded-[2px]"
            key={level}
            style={{ backgroundColor: `var(--color-level-${String(level)})` }}
          />
        )
      )}
    </div>
    <span>{more}</span>
  </div>
)

interface HeaderSectionProperties {
  readonly onYearChange: (year: number) => void
  readonly selectedYear: number
  readonly total: number
  readonly translate: ReturnType<typeof useTranslations>
  readonly variant?: 'blueprint' | 'default'
  readonly years: readonly number[]
}

export const HeaderSection: FCStrict<HeaderSectionProperties> = ({
  onYearChange,
  selectedYear,
  total,
  translate,
  variant = 'default',
  years,
}: HeaderSectionProperties): JSX.Element => {
  const selectClassName: string =
    variant === 'blueprint'
      ? 'h-9 w-24 rounded-md border border-brand/30 bg-blueprint-bg px-3 py-1 text-sm text-brand shadow-sm focus-visible:ring-1 focus-visible:ring-brand focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50'
      : 'h-9 w-24 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50'

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
          className={selectClassName}
          value={selectedYear}
          onChange={(event: React.ChangeEvent<HTMLSelectElement>): void => {
            onYearChange(Number(event.target.value))
          }}
        >
          {years.map(
            (year: number): JSX.Element => (
              <option
                className="bg-blueprint-bg text-brand"
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
  readonly dayFive: DayLabelTripleResult['dayFive']
  readonly dayOne: DayLabelTripleResult['dayOne']
  readonly dayThree: DayLabelTripleResult['dayThree']
  readonly locale: Locale
  readonly variant?: 'blueprint' | 'default'
  readonly weeks: CalendarModel['weeks']
}

export const GraphSection: FCStrict<GraphSectionProperties> = ({
  dayFive,
  dayOne,
  dayThree,
  locale,
  variant = 'default',
  weeks,
}: GraphSectionProperties): JSX.Element => {
  // We removed overflow-x-auto because SVG scales fit-to-width
  return (
    <div
      className={`mt-4 w-full ${variant === 'blueprint' ? 'overflow-hidden' : ''}`}
    >
      <ContributionGraphSvg
        dayFive={dayFive}
        dayOne={dayOne}
        dayThree={dayThree}
        locale={locale}
        weeks={weeks}
      />
    </div>
  )
}
