'use client'

import { type JSX, useMemo, useState } from 'react'

import { type Locale } from 'next-intl'

import { ContributionGraphView } from '@/components/features/contribution-graph/contribution-graph-view'
import {
  buildCalendar,
  type CalendarModel,
  dayLabelTriple,
  type DayLabelTripleResult,
  sumCount,
} from '@/lib/github/contribution-calendar'
import type { ContributionCollection, ContributionPoint } from '@/models/github'

export interface ContributionGraphClientProperties {
  readonly data: ContributionCollection
  readonly locale: Locale
}

export const ContributionGraphClient: React.FC<
  ContributionGraphClientProperties
> = ({ data, locale }: ContributionGraphClientProperties): JSX.Element => {
  // Sort years descending
  const years: number[] = useMemo(
    (): number[] =>
      Object.keys(data)
        .map(Number)
        .toSorted((yearA: number, yearB: number): number => yearB - yearA),
    [data]
  )

  const initialYear: number = years[0] ?? new Date().getFullYear()

  const [selectedYear, setSelectedYear]: [
    number,
    React.Dispatch<React.SetStateAction<number>>,
  ] = useState<number>(initialYear)

  const currentYearData: ContributionPoint[] =
    useMemo((): ContributionPoint[] => {
      const entry: [string, ContributionPoint[]] | undefined = Object.entries(
        data
      ).find(
        ([year]: [string, ContributionPoint[]]): boolean =>
          Number(year) === selectedYear
      )
      return entry ? entry[1] : []
    }, [data, selectedYear])

  const calendar: CalendarModel = useMemo(
    (): CalendarModel => buildCalendar({ data: currentYearData, locale }),
    [currentYearData, locale]
  )

  const labels: DayLabelTripleResult = useMemo(
    (): DayLabelTripleResult =>
      dayLabelTriple({ data: currentYearData, locale }),
    [currentYearData, locale]
  )

  const total: number = useMemo(
    (): number => sumCount(currentYearData),
    [currentYearData]
  )

  return (
    <ContributionGraphView
      calendar={calendar}
      currentYearData={currentYearData}
      labels={labels}
      locale={locale}
      selectedYear={selectedYear}
      total={total}
      years={years}
      onYearChange={setSelectedYear}
    />
  )
}
