/* eslint-disable max-lines */
'use server'

import { type JSX } from 'react'

import { type Locale } from 'next-intl'

import { getTranslations } from 'next-intl/server'

import { Card } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import { panic } from '@/shared/utils'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import {
  CONTRIBUTION_LEVELS,
  type ContributionLevel,
  type ContributionPoint,
} from '@/types/github'
import type { LocalePageProperties, Translations } from '@/types/i18n'

import styles from './contribution-graph.module.css'

/* =============================== Types =============================== */

interface ContributionGraphProperties extends LocalePageProperties {
  readonly data: readonly ContributionPoint[]
}

interface MonthLabel {
  readonly month: string
  readonly weekIndex: number
}

interface WeekModel {
  readonly days: readonly (ContributionPoint | null)[]
  readonly key: string
}

interface CalendarModel {
  readonly monthLabels: readonly MonthLabel[]
  readonly weeks: readonly WeekModel[]
}

/* ============================ Pure Helpers =========================== */

const levelClass: (level: ContributionLevel) => string = (
  level: ContributionLevel
): string => {
  if (level <= 0) {
    return 'bg-muted/40 dark:bg-muted/20 hover:bg-muted/60 dark:hover:bg-muted/40 border border-border/50 dark:border-border/30'
  }
  if (level === 1) {
    return 'bg-emerald-100 dark:bg-emerald-900/40 hover:bg-emerald-200 dark:hover:bg-emerald-900/60 border border-emerald-200/50 dark:border-emerald-800/30'
  }
  if (level === 2) {
    return 'bg-emerald-300 dark:bg-emerald-700/60 hover:bg-emerald-400 dark:hover:bg-emerald-700/80 border border-emerald-400/50 dark:border-emerald-600/30'
  }
  if (level === 3) {
    return 'bg-emerald-500 dark:bg-emerald-600/80 hover:bg-emerald-600 dark:hover:bg-emerald-600 border border-emerald-600/50 dark:border-emerald-500/30'
  }
  return 'bg-emerald-700 dark:bg-emerald-500 hover:bg-emerald-800 dark:hover:bg-emerald-400 border border-emerald-800/50 dark:border-emerald-400/30'
}

const isoDate: (date: Date) => string = (date: Date): string =>
  date.toISOString().split('T')[0] ?? date.toISOString()

const sundayOfWeekUTC: (date: Date) => Date = (date: Date): Date => {
  const utcDate: Date = new Date(
    Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate())
  )
  const day: number = utcDate.getUTCDay() // 0..6 (Sun..Sat)
  utcDate.setUTCDate(utcDate.getUTCDate() - day)
  return utcDate
}

const makeDataMap: (
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

const computeMonthLabel: (context: MonthContext) => ComputeMonthLabelResult = ({
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

const buildCalendar: (properties: BuildCalendarProperties) => CalendarModel = ({
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

interface DayLabelTripleResult {
  readonly dayFive: string
  readonly dayOne: string
  readonly dayThree: string
}
const dayLabelTriple: (p: DayLabelTripleProperties) => DayLabelTripleResult = ({
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

const sumCount: (input: readonly ContributionPoint[]) => number = (
  input: readonly ContributionPoint[]
): number => {
  let total: number = 0
  for (const contributionPoint of input) {
    total += contributionPoint.count
  }
  return total
}

/* =============================== Subviews =============================== */

interface LegendSquareProperties {
  readonly level: ContributionLevel
}

const LegendSquare: FCStrict<LegendSquareProperties> = ({
  level,
}: LegendSquareProperties): JSX.Element => {
  const cls: string = levelClass(level).split(' ').slice(0, 3).join(' ')
  return <div className={`h-4 w-4 rounded-sm ${cls}`} />
}

interface LegendBarProperties {
  readonly less: string
  readonly more: string
}

const LegendBar: FCStrict<LegendBarProperties> = ({
  less,
  more,
}: LegendBarProperties): JSX.Element => {
  return (
    <div className="text-muted-foreground flex items-center gap-2 text-xs">
      <span>{less}</span>
      {CONTRIBUTION_LEVELS.map(
        (lvl: ContributionLevel): JSX.Element => (
          <LegendSquare key={`lg-${String(lvl)}`} level={lvl} />
        )
      )}
      <span>{more}</span>
    </div>
  )
}

interface MonthLabelsRowProperties {
  readonly labels: readonly MonthLabel[]
}

const MonthLabelsRow: FCStrict<MonthLabelsRowProperties> = ({
  labels,
}: MonthLabelsRowProperties): JSX.Element => {
  return (
    <div className="relative mb-3 h-5">
      {labels.map(
        (monthLabel: MonthLabel): JSX.Element => (
          <div
            className="text-muted-foreground absolute text-sm font-medium"
            key={`${monthLabel.month}-${String(monthLabel.weekIndex)}`}
            style={{ left: String(48 + monthLabel.weekIndex * 20) + 'px' }}
          >
            {monthLabel.month}
          </div>
        )
      )}
    </div>
  )
}

// one-time module constant (tree-shakable)
const WEEKDAY_INDICES: readonly [0, 1, 2, 3, 4, 5, 6] = [
  0, 1, 2, 3, 4, 5, 6,
] as const

interface WeekdayLabelColProperties {
  readonly dayFive: string
  readonly dayOne: string
  readonly dayThree: string
}

const WeekdayLabelCol: FCStrict<WeekdayLabelColProperties> = ({
  dayFive,
  dayOne,
  dayThree,
}: WeekdayLabelColProperties): JSX.Element => {
  return (
    <div className="flex flex-shrink-0 flex-col gap-1 pr-3">
      {WEEKDAY_INDICES.map(
        (index: (typeof WEEKDAY_INDICES)[number]): JSX.Element => {
          let label: string

          switch (index) {
            case 1: {
              label = dayOne
              break
            }
            case 3: {
              label = dayThree
              break
            }
            case 5: {
              label = dayFive
              break
            }
            default: {
              label = ''
            }
          }

          return (
            <div
              className="text-muted-foreground flex h-4 w-8 items-center text-xs font-medium"
              key={`wd-${String(index)}`}
            >
              {label}
            </div>
          )
        }
      )}
    </div>
  )
}

interface DayCellProperties {
  readonly day: ContributionPoint
  readonly dayIndex: number
  readonly locale: Locale
  readonly weekIndex: number
}

const DayCell: FCStrict<DayCellProperties> = ({
  day,
  dayIndex,
  locale,
  weekIndex,
}: DayCellProperties): JSX.Element => {
  const delayMs: string = String(weekIndex * 7 + dayIndex) + 'ms'

  const commitsText: string = day.count.toLocaleString(locale)
  const dateText: string = new Date(day.date).toLocaleDateString(locale, {
    day: 'numeric',
    month: 'short',
    timeZone: 'UTC',
    weekday: 'short',
    year: 'numeric',
  })

  return (
    <div
      className={`h-4 w-4 rounded-sm transition-all duration-200 ${levelClass(day.level)} ${styles['cell'] ?? ''}`}
      data-commits={commitsText}
      data-date={dateText}
      data-row={dayIndex}
      key={day.date}
      style={{ animationDelay: delayMs }}
    />
  )
}

const EmptyCell: FCStrict = (): JSX.Element => (
  <div className="border-border/30 bg-muted/20 dark:border-border/20 dark:bg-muted/10 h-4 w-4 rounded-sm border" />
)

interface WeekColumnProperties {
  readonly locale: Locale
  readonly week: WeekModel
  readonly weekIndex: number
}

const WeekColumn: FCStrict<WeekColumnProperties> = ({
  locale,
  week,
  weekIndex,
}: WeekColumnProperties): JSX.Element => {
  return (
    <div className="flex flex-shrink-0 flex-col gap-1">
      {week.days.map(
        (
          contributionPoint: ContributionPoint | null,
          dayIndex: number
        ): JSX.Element =>
          contributionPoint === null ? (
            <EmptyCell key={`${week.key}-${String(dayIndex)}`} />
          ) : (
            <DayCell
              day={contributionPoint}
              dayIndex={dayIndex}
              key={contributionPoint.date}
              locale={locale}
              weekIndex={weekIndex}
            />
          )
      )}
    </div>
  )
}

interface WeeksGridProperties {
  readonly dayFive: string
  readonly dayOne: string
  readonly dayThree: string
  readonly locale: Locale
  readonly weeks: readonly WeekModel[]
}

const WeeksGrid: FCStrict<WeeksGridProperties> = ({
  dayFive,
  dayOne,
  dayThree,
  locale,
  weeks,
}: WeeksGridProperties): JSX.Element => {
  return (
    <div className="flex gap-1">
      <WeekdayLabelCol dayFive={dayFive} dayOne={dayOne} dayThree={dayThree} />
      <div className="flex gap-1">
        {weeks.map(
          (weekModel: WeekModel, index: number): JSX.Element => (
            <WeekColumn
              key={weekModel.key}
              locale={locale}
              week={weekModel}
              weekIndex={index}
            />
          )
        )}
      </div>
    </div>
  )
}

/* ================================ Main FC ================================ */
export const ContributionGraph: AsyncPageFC<
  ContributionGraphProperties
> = async ({
  data,
  locale,
}: ContributionGraphProperties): Promise<JSX.Element> => {
  const translations: Translations<'projects.contributions'> =
    await getTranslations({
      locale,
      namespace: 'projects.contributions',
    })

  const calendar: CalendarModel = buildCalendar({ data, locale })
  const labels: DayLabelTripleResult = dayLabelTriple({ data, locale })
  const total: number = sumCount(data)

  return (
    <Card className="hover:border-primary/50 dark:bg-card/50 w-full overflow-hidden border-2 p-6 transition-all duration-300 hover:shadow-lg">
      <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <Heading as="h3" className="text-2xl font-bold">
            {translations('title')}
          </Heading>
          <p className="text-muted-foreground text-sm">
            {translations('totalAmount', {
              count: total.toLocaleString(locale),
            })}
          </p>
        </div>
        <LegendBar less={translations('less')} more={translations('more')} />
      </div>

      <div className="relative w-full overflow-x-auto pb-2">
        <div className="inline-block min-w-full">
          <MonthLabelsRow labels={calendar.monthLabels} />
          <WeeksGrid
            dayFive={labels.dayFive}
            dayOne={labels.dayOne}
            dayThree={labels.dayThree}
            locale={locale}
            weeks={calendar.weeks}
          />
        </div>
      </div>
    </Card>
  )
}
