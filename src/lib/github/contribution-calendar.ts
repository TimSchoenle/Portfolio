import { type Locale } from 'next-intl'

import { panic } from '@/lib/utilities'
import type { ContributionPoint } from '@/models/github'

/* =============================== Types =============================== */

export interface MonthLabel {
  readonly month: string
  readonly weekIndex: number
}

export interface WeekModel {
  readonly days: readonly (ContributionPoint | null)[]
  readonly key: string
}

export interface CalendarModel {
  readonly monthLabels: readonly MonthLabel[]
  readonly weeks: readonly WeekModel[]
}

/* ============================ Pure Helpers =========================== */

export const isoDate: (date: Date) => string = (date: Date): string =>
  date.toISOString().split('T')[0] ?? date.toISOString()

export const sundayOfWeekUTC: (date: Date) => Date = (date: Date): Date => {
  const utcDate: Date = new Date(
    Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate())
  )
  const day: number = utcDate.getUTCDay() // 0..6 (Sun..Sat)
  utcDate.setUTCDate(utcDate.getUTCDate() - day)
  return utcDate
}

export const makeDataMap: (
  input: readonly ContributionPoint[]
) => Map<string, ContributionPoint> = (
  input: readonly ContributionPoint[]
): Map<string, ContributionPoint> => {
  const entries: readonly (readonly [string, ContributionPoint])[] = input.map(
    (
      contributionPoint: ContributionPoint
    ): readonly [string, ContributionPoint] => [
      contributionPoint.date,
      contributionPoint,
    ]
  )
  return new Map<string, ContributionPoint>(entries)
}

interface MonthContext {
  readonly currentMonth: string
  readonly locale: string
  readonly weekDays: readonly (ContributionPoint | null)[]
  readonly weeksLength: number
}

interface ComputeMonthLabelResult {
  readonly label: MonthLabel | null
  readonly next: string
}

export const computeMonthLabel: (
  context: MonthContext
) => ComputeMonthLabelResult = ({
  currentMonth,
  locale,
  weekDays,
  weeksLength,
}: MonthContext): ComputeMonthLabelResult => {
  const first: ContributionPoint | undefined = weekDays.find(
    (
      contributionPoint: ContributionPoint | null
    ): contributionPoint is ContributionPoint => contributionPoint !== null
  )
  if (first === undefined) {
    return { label: null, next: currentMonth }
  }

  const monthName: string = new Date(first.date).toLocaleDateString(locale, {
    month: 'short',
    timeZone: 'UTC',
  })
  if (monthName === currentMonth) {
    return { label: null, next: currentMonth }
  }

  return {
    label: { month: monthName, weekIndex: weeksLength },
    next: monthName,
  }
}

interface BuildCalendarProperties {
  readonly data: readonly ContributionPoint[]
  readonly locale: string
}

export const buildCalendar: (
  properties: BuildCalendarProperties
) => CalendarModel = ({
  data,
  locale,
  // eslint-disable-next-line complexity
}: BuildCalendarProperties): CalendarModel => {
  if (data.length === 0) {
    return { monthLabels: [], weeks: [] }
  }

  const dataMap: Map<string, ContributionPoint> = makeDataMap(data)
  const sorted: readonly ContributionPoint[] = [...data].toSorted(
    (pointOne: ContributionPoint, pointTwo: ContributionPoint): number =>
      new Date(pointOne.date).getTime() - new Date(pointTwo.date).getTime()
  )

  // normalize to UTC midnight to avoid TZ/DST drift
  const firstISO: string =
    sorted[0]?.date ?? panic('buildCalendar: empty contributions array')

  const lastISO: string =
    sorted.at(-1)?.date ?? panic('buildCalendar: missing last item')

  const firstDate: Date = new Date(`${firstISO}T00:00:00Z`)
  const lastDate: Date = new Date(`${lastISO}T00:00:00Z`)

  const cursor: Date = sundayOfWeekUTC(firstDate)
  const weeks: WeekModel[] = []
  const monthLabels: MonthLabel[] = []
  let currentMonth: string = ''

  while (cursor.getTime() <= lastDate.getTime()) {
    const start: Date = new Date(cursor)
    const days: (ContributionPoint | null)[] = []

    for (let index: number = 0; index < 7; index++) {
      const key: string = isoDate(cursor) // YYYY-MM-DD (UTC)
      const day: ContributionPoint | null = dataMap.get(key) ?? null
      days.push(day)
      // 🔧 advance in UTC, not local time
      cursor.setUTCDate(cursor.getUTCDate() + 1)
    }

    const response: ComputeMonthLabelResult = computeMonthLabel({
      currentMonth,
      locale,
      weekDays: days,
      weeksLength: weeks.length,
    })
    currentMonth = response.next
    if (response.label !== null) {
      monthLabels.push(response.label)
    }

    weeks.push({ days, key: isoDate(start) })
  }

  return { monthLabels, weeks }
}

interface DayLabelTripleProperties {
  readonly data: readonly ContributionPoint[]
  readonly locale: Locale
}

export interface DayLabelTripleResult {
  readonly dayFive: string
  readonly dayOne: string
  readonly dayThree: string
}
export const dayLabelTriple: (
  p: DayLabelTripleProperties
) => DayLabelTripleResult = ({
  data,
  locale,
}: DayLabelTripleProperties): DayLabelTripleResult => {
  if (data.length === 0) {
    return { dayFive: '', dayOne: '', dayThree: '' }
  }

  const iso: string =
    data[0]?.date ?? panic('dayLabelTriple: empty contributions array')
  const firstUTC: Date = new Date(`${iso}T00:00:00Z`)
  const start: Date = sundayOfWeekUTC(firstUTC)

  const labelUTC: (offset: number) => string = (offset: number): string => {
    const date: Date = new Date(start)
    date.setUTCDate(date.getUTCDate() + offset)
    return date.toLocaleDateString(locale, {
      timeZone: 'UTC',
      weekday: 'short',
    })
  }

  return { dayFive: labelUTC(5), dayOne: labelUTC(1), dayThree: labelUTC(3) }
}

export const sumCount: (input: readonly ContributionPoint[]) => number = (
  input: readonly ContributionPoint[]
): number => {
  let total: number = 0
  for (const contributionPoint of input) {
    total += contributionPoint.count
  }
  return total
}
