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
import type { FCStrict } from '@/types/fc'

export interface ContributionGraphClientProperties {
  readonly data: ContributionCollection
  readonly locale: Locale
  readonly variant?: 'blueprint' | 'default'
}

interface ContributionGraphData {
  calendar: CalendarModel
  currentYearData: ContributionPoint[]
  labels: DayLabelTripleResult
  selectedYear: number
  setSelectedYear: React.Dispatch<React.SetStateAction<number>>
  total: number
  years: number[]
}

const useContributionGraphData: (properties: {
  data: ContributionCollection
  locale: Locale
}) => ContributionGraphData = ({
  data,
  locale,
}: {
  data: ContributionCollection
  locale: Locale
}): ContributionGraphData => {
  // Sort years descending
  const years: number[] = useMemo(
    (): number[] =>
      Object.keys(data)
        .map(Number)
        .toSorted((yearA: number, yearB: number): number => yearB - yearA),
    [data]
  )

  const initialYear: number = years[0] ?? new Date().getFullYear()

  // eslint-disable-next-line @typescript-eslint/typedef
  const [selectedYear, setSelectedYear] = useState<number>(initialYear)

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

  return {
    calendar,
    currentYearData,
    labels,
    selectedYear,
    setSelectedYear,
    total,
    years,
  }
}

export const ContributionGraphClient: FCStrict<
  ContributionGraphClientProperties
> = ({
  data,
  locale,
  variant = 'default',
}: ContributionGraphClientProperties): JSX.Element => {
  // eslint-disable-next-line @typescript-eslint/typedef
  const { calendar, labels, selectedYear, setSelectedYear, total, years } =
    useContributionGraphData({ data, locale })

  return (
    <ContributionGraphView
      calendar={calendar}
      labels={labels}
      locale={locale}
      selectedYear={selectedYear}
      total={total}
      variant={variant}
      years={years}
      onYearChange={setSelectedYear}
    />
  )
}
