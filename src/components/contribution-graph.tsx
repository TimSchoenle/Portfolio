/* eslint-disable max-lines */
'use client'

import { type Locale, useTranslations } from 'next-intl'
import React, {
  type Dispatch,
  type JSX,
  type SetStateAction,
  useMemo,
  useState,
} from 'react'

import { Card } from '@/components/ui/card'
import type { FCStrict } from '@/types/fc'
import {
  CONTRIBUTION_LEVELS,
  type ContributionLevel,
  type ContributionPoint,
} from '@/types/github'
import type { LocalePageProps, Translations } from '@/types/i18n'

/* =============================== Types =============================== */

interface ContributionGraphProps extends LocalePageProps {
  readonly data: readonly ContributionPoint[]
}

interface MousePos {
  readonly x: number
  readonly y: number
}

interface MonthLabel {
  readonly month: string
  readonly weekIndex: number
}

interface WeekModel {
  readonly key: string
  readonly days: readonly (ContributionPoint | null)[]
}

interface CalendarModel {
  readonly weeks: readonly WeekModel[]
  readonly monthLabels: readonly MonthLabel[]
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

const isoDate: (d: Date) => string = (d: Date): string =>
  d.toISOString().split('T')[0] ?? d.toISOString()

const sundayOfWeekUTC: (d: Date) => Date = (d: Date): Date => {
  const c: Date = new Date(
    Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate())
  )
  const day: number = c.getUTCDay() // 0..6 (Sun..Sat)
  c.setUTCDate(c.getUTCDate() - day)
  return c
}

const makeDataMap: (
  input: readonly ContributionPoint[]
) => Map<string, ContributionPoint> = (
  input: readonly ContributionPoint[]
): Map<string, ContributionPoint> => {
  const entries: readonly (readonly [string, ContributionPoint])[] = input.map(
    (pt: ContributionPoint): readonly [string, ContributionPoint] => [
      pt.date,
      pt,
    ]
  )
  return new Map<string, ContributionPoint>(entries)
}

interface MonthCtx {
  readonly weekDays: readonly (ContributionPoint | null)[]
  readonly locale: string
  readonly weeksLength: number
  readonly currentMonth: string
}

interface ComputeMonthLabelResult {
  readonly next: string
  readonly label: MonthLabel | null
}

const computeMonthLabel: (ctx: MonthCtx) => ComputeMonthLabelResult = ({
  weekDays,
  locale,
  weeksLength,
  currentMonth,
}: MonthCtx): ComputeMonthLabelResult => {
  const first: ContributionPoint | undefined = weekDays.find(
    (d: ContributionPoint | null): d is ContributionPoint => d !== null
  )
  if (first === undefined) {
    return { next: currentMonth, label: null }
  }

  const monthName: string = new Date(first.date).toLocaleDateString(locale, {
    month: 'short',
  })
  if (monthName === currentMonth) {
    return { next: currentMonth, label: null }
  }

  return {
    next: monthName,
    label: { month: monthName, weekIndex: weeksLength },
  }
}

interface BuildCalendarProps {
  readonly locale: string
  readonly data: readonly ContributionPoint[]
}

const buildCalendar: (props: BuildCalendarProps) => CalendarModel = ({
  locale,
  data,
  // eslint-disable-next-line complexity
}: BuildCalendarProps): CalendarModel => {
  if (data.length === 0) {
    return { weeks: [], monthLabels: [] }
  }

  const dataMap: Map<string, ContributionPoint> = makeDataMap(data)
  const sorted: readonly ContributionPoint[] = [...data].sort(
    (a: ContributionPoint, b: ContributionPoint): number =>
      new Date(a.date).getTime() - new Date(b.date).getTime()
  )

  const firstDate: Date = new Date(sorted[0]?.date ?? Date.now())
  const lastDate: Date = new Date(sorted[sorted.length - 1]?.date ?? Date.now())

  const cursor: Date = sundayOfWeekUTC(firstDate)
  const weeks: WeekModel[] = []
  const monthLabels: MonthLabel[] = []
  let currentMonth: string = ''

  while (cursor <= lastDate) {
    const start: Date = new Date(cursor)
    const days: (ContributionPoint | null)[] = []
    for (let i: number = 0; i < 7; i++) {
      const key: string = isoDate(cursor)
      const day: ContributionPoint | null = dataMap.get(key) ?? null
      days.push(day)
      cursor.setDate(cursor.getDate() + 1)
    }

    const res: { readonly next: string; readonly label: MonthLabel | null } =
      computeMonthLabel({
        weekDays: days,
        locale,
        weeksLength: weeks.length,
        currentMonth,
      })
    currentMonth = res.next
    if (res.label !== null) {
      monthLabels.push(res.label)
    }

    weeks.push({ key: isoDate(start), days })
  }
  return { weeks, monthLabels }
}

interface DayLabelTripleProps {
  readonly locale: Locale
  readonly data: readonly ContributionPoint[]
}

interface DayLabelTripleResult {
  readonly d1: string
  readonly d3: string
  readonly d5: string
}
const dayLabelTriple: (p: DayLabelTripleProps) => DayLabelTripleResult = ({
  locale,
  data,
}: DayLabelTripleProps): DayLabelTripleResult => {
  if (data.length === 0) {
    return { d1: '', d3: '', d5: '' }
  }

  const firstUTC: Date = new Date(data[0]!.date + 'T00:00:00Z')
  const start: Date = sundayOfWeekUTC(firstUTC)

  const labelUTC: (offset: number) => string = (offset: number): string => {
    const d: Date = new Date(start)
    d.setUTCDate(d.getUTCDate() + offset)
    return d.toLocaleDateString(locale, { weekday: 'short', timeZone: 'UTC' })
  }

  return { d1: labelUTC(1), d3: labelUTC(3), d5: labelUTC(5) }
}

const sumCount: (input: readonly ContributionPoint[]) => number = (
  input: readonly ContributionPoint[]
): number => {
  let total: number = 0
  for (const d of input) {
    total += d.count
  }
  return total
}

/* =============================== Subviews =============================== */

interface LegendSquareProps {
  readonly level: ContributionLevel
}

const LegendSquare: FCStrict<LegendSquareProps> = ({
  level,
}: LegendSquareProps): JSX.Element => {
  const cls: string = levelClass(level).split(' ').slice(0, 3).join(' ')
  return <div className={`h-4 w-4 rounded-sm ${cls}`} />
}

interface LegendBarProps {
  readonly less: string
  readonly more: string
}

const LegendBar: FCStrict<LegendBarProps> = ({
  less,
  more,
}: LegendBarProps): JSX.Element => {
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

interface MonthLabelsRowProps {
  readonly labels: readonly MonthLabel[]
}

const MonthLabelsRow: FCStrict<MonthLabelsRowProps> = ({
  labels,
}: MonthLabelsRowProps): JSX.Element => {
  return (
    <div className="relative mb-3 h-5">
      {labels.map(
        (ml: MonthLabel): JSX.Element => (
          <div
            className="text-muted-foreground absolute text-sm font-medium"
            key={`${ml.month}-${String(ml.weekIndex)}`}
            style={{ left: String(48 + ml.weekIndex * 20) + 'px' }}
          >
            {ml.month}
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

interface WeekdayLabelColProps {
  readonly d1: string
  readonly d3: string
  readonly d5: string
}

const WeekdayLabelCol: FCStrict<WeekdayLabelColProps> = ({
  d1,
  d3,
  d5,
}: WeekdayLabelColProps): JSX.Element => {
  return (
    <div className="flex flex-shrink-0 flex-col gap-1 pr-3">
      {WEEKDAY_INDICES.map(
        (i: (typeof WEEKDAY_INDICES)[number]): JSX.Element => {
          const label: string = i === 1 ? d1 : i === 3 ? d3 : i === 5 ? d5 : ''
          return (
            <div
              className="text-muted-foreground flex h-4 w-8 items-center text-xs font-medium"
              key={`wd-${String(i)}`}
            >
              {label}
            </div>
          )
        }
      )}
    </div>
  )
}

interface DayCellProps {
  readonly day: ContributionPoint
  readonly weekIndex: number
  readonly dayIndex: number
  readonly onEnter: (props: CellEnterProps) => void
  readonly onLeave: () => void
}

const DayCell: FCStrict<DayCellProps> = ({
  day,
  weekIndex,
  dayIndex,
  onEnter,
  onLeave,
}: DayCellProps): JSX.Element => {
  const delayMs: string = String(weekIndex * 7 + dayIndex) + 'ms'
  const aria: string = String(day.count) + ' on ' + day.date
  return (
    <div
      aria-label={aria}
      className={`h-4 w-4 cursor-pointer rounded-sm transition-all duration-200 ${levelClass(
        day.level
      )}`}
      key={day.date}
      role="button"
      style={{ animationDelay: delayMs }}
      tabIndex={0}
      onBlur={onLeave}
      onFocus={(e: React.FocusEvent<HTMLDivElement>): void => {
        onEnter({
          element: e as unknown as React.MouseEvent<HTMLDivElement>,
          point: day,
        })
      }}
      onKeyDown={(e: React.KeyboardEvent<HTMLDivElement>): void => {
        if (e.key === 'Enter' || e.key === ' ') {
          onEnter({
            element: e as unknown as React.MouseEvent<HTMLDivElement>,
            point: day,
          })
        }
      }}
      onMouseEnter={(e: React.MouseEvent<HTMLDivElement>): void => {
        onEnter({ element: e, point: day })
      }}
      onMouseLeave={onLeave}
    />
  )
}

const EmptyCell: FCStrict = (): JSX.Element => (
  <div className="border-border/30 bg-muted/20 dark:border-border/20 dark:bg-muted/10 h-4 w-4 rounded-sm border" />
)

interface CellEnterProps {
  readonly element: React.MouseEvent<HTMLDivElement>
  readonly point: ContributionPoint
}

interface WeekColumnProps {
  readonly week: WeekModel
  readonly weekIndex: number
  readonly onEnter: (props: CellEnterProps) => void
  readonly onLeave: () => void
}

const WeekColumn: FCStrict<WeekColumnProps> = ({
  week,
  weekIndex,
  onEnter,
  onLeave,
}: WeekColumnProps): JSX.Element => {
  return (
    <div className="flex flex-shrink-0 flex-col gap-1">
      {week.days.map(
        (d: ContributionPoint | null, dayIndex: number): JSX.Element =>
          d === null ? (
            <EmptyCell key={`${week.key}-${String(dayIndex)}`} />
          ) : (
            <DayCell
              day={d}
              dayIndex={dayIndex}
              key={d.date}
              weekIndex={weekIndex}
              onEnter={onEnter}
              onLeave={onLeave}
            />
          )
      )}
    </div>
  )
}

interface WeeksGridProps {
  readonly weeks: readonly WeekModel[]
  readonly d1: string
  readonly d3: string
  readonly d5: string
  readonly onEnter: (props: CellEnterProps) => void
  readonly onLeave: () => void
}

const WeeksGrid: FCStrict<WeeksGridProps> = ({
  weeks,
  d1,
  d3,
  d5,
  onEnter,
  onLeave,
}: WeeksGridProps): JSX.Element => {
  return (
    <div className="flex gap-1">
      <WeekdayLabelCol d1={d1} d3={d3} d5={d5} />
      <div className="flex gap-1">
        {weeks.map(
          (w: WeekModel, i: number): JSX.Element => (
            <WeekColumn
              key={w.key}
              week={w}
              weekIndex={i}
              onEnter={onEnter}
              onLeave={onLeave}
            />
          )
        )}
      </div>
    </div>
  )
}

interface TooltipProps {
  readonly day: ContributionPoint
  readonly mouse: MousePos
  readonly locale: string
}

const Tooltip: FCStrict<TooltipProps> = ({
  day,
  mouse,
  locale,
}: TooltipProps): JSX.Element => {
  const cnt: string = day.count.toLocaleString(locale)
  const dateTxt: string = new Date(day.date).toLocaleDateString(locale, {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  })
  return (
    <div
      className="bg-popover dark:border-border/50 dark:bg-popover/95 pointer-events-none fixed z-50 -translate-x-1/2 -translate-y-full rounded-lg border-2 px-3 py-2 text-sm shadow-xl dark:shadow-2xl"
      style={{ left: mouse.x, top: mouse.y - 8 }}
    >
      <p className="font-semibold">{cnt}</p>
      <p className="text-muted-foreground text-xs">{dateTxt}</p>
    </div>
  )
}

/* ================================ Main FC ================================ */

// eslint-disable-next-line max-lines-per-function
export const ContributionGraph: FCStrict<ContributionGraphProps> = ({
  locale,
  data,
}: ContributionGraphProps): JSX.Element => {
  const t: Translations<'projects.contributions'> = useTranslations(
    'projects.contributions'
  )
  const [hoveredDay, setHoveredDay]: [
    ContributionPoint | null,
    Dispatch<SetStateAction<ContributionPoint | null>>,
  ] = useState<ContributionPoint | null>(null)
  const [mouse, setMouse]: [MousePos, Dispatch<SetStateAction<MousePos>>] =
    useState<MousePos>({ x: 0, y: 0 })

  const calendar: CalendarModel = useMemo<CalendarModel>(
    (): CalendarModel => buildCalendar({ locale, data }),
    [data, locale]
  )
  const labels: DayLabelTripleResult = useMemo<DayLabelTripleResult>(
    (): DayLabelTripleResult => dayLabelTriple({ locale, data }),
    [data, locale]
  )

  const total: number = useMemo<number>((): number => sumCount(data), [data])

  const lessText: string = t('less')
  const moreText: string = t('more')
  const titleText: string = t('title')
  const totalText: string = t('totalAmount', {
    count: total.toLocaleString(locale),
  })

  const handleEnter: (props: CellEnterProps) => void = ({
    element,
    point,
  }: CellEnterProps): void => {
    const rect: DOMRect = element.currentTarget.getBoundingClientRect()
    const x: number = rect.left + rect.width / 2
    const y: number = rect.top
    setMouse({ x, y })
    setHoveredDay(point)
  }

  const handleLeave: () => void = (): void => {
    setHoveredDay(null)
  }

  return (
    <Card className="hover:border-primary/50 dark:bg-card/50 w-full overflow-hidden border-2 p-6 transition-all duration-300 hover:shadow-lg">
      <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h3 className="text-2xl font-bold">{titleText}</h3>
          <p className="text-muted-foreground text-sm">{totalText}</p>
        </div>
        <LegendBar less={lessText} more={moreText} />
      </div>

      <div className="relative w-full overflow-x-auto pb-2">
        <div className="inline-block min-w-full">
          <MonthLabelsRow labels={calendar.monthLabels} />
          <WeeksGrid
            d1={labels.d1}
            d3={labels.d3}
            d5={labels.d5}
            weeks={calendar.weeks}
            onEnter={handleEnter}
            onLeave={handleLeave}
          />
        </div>
      </div>

      {hoveredDay !== null ? (
        <Tooltip day={hoveredDay} locale={locale} mouse={mouse} />
      ) : null}
    </Card>
  )
}
