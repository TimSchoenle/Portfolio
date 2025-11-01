'use client'

import React, { useState, useMemo } from 'react'
import { Card } from '@/components/ui/card'
import { type Locale } from '@/lib/i18n-config'
import { type GithubContribution } from '@/lib/dictionary'

interface ContributionDay {
  date: string
  count: number
  level: 0 | 1 | 2 | 3 | 4
}

interface ContributionGraphProps {
  data: ContributionDay[]
  locale: Locale
  dict: GithubContribution
}

export function ContributionGraph({
  data,
  locale,
  dict,
}: ContributionGraphProps) {
  const [hoveredDay, setHoveredDay] = useState<ContributionDay | null>(null)
  const [mousePos, setMousePos] = useState({ x: 0, y: 0 })

  // Organize data by calendar weeks
  const { weeks, monthLabels } = useMemo(() => {
    const weeks: Array<Array<ContributionDay | null>> = []
    const monthLabels: Array<{ month: string; weekIndex: number }> = []
    let currentMonth = ''

    if (data.length === 0) return { weeks: [], monthLabels: [] }

    const dataMap = new Map(data.map((d) => [d.date, d]))

    const sortedData = [...data].sort(
      (a, b) => new Date(a.date).getTime() - new Date(b.date).getTime()
    )

    const firstDate = new Date(sortedData[0]?.date ?? Date.now())
    const lastDate = new Date(
      sortedData[sortedData.length - 1]?.date ?? Date.now()
    )

    const currentDate = new Date(firstDate)
    currentDate.setDate(currentDate.getDate() - currentDate.getDay())

    while (currentDate <= lastDate) {
      const week: Array<ContributionDay | null> = []

      for (let dayOfWeek = 0; dayOfWeek < 7; dayOfWeek++) {
        const dateStr =
          currentDate.toISOString().split('T')[0] ?? currentDate.toISOString()
        const dayData = dataMap.get(dateStr) ?? null

        week.push(dayData)
        currentDate.setDate(currentDate.getDate() + 1)
      }

      weeks.push(week)

      const firstDayOfWeek = week.find((d) => d !== null)
      if (firstDayOfWeek) {
        const monthName = new Date(firstDayOfWeek.date).toLocaleDateString(
          locale,
          { month: 'short' }
        )

        if (monthName !== currentMonth) {
          monthLabels.push({ month: monthName, weekIndex: weeks.length - 1 })
          currentMonth = monthName
        }
      }
    }

    return { weeks, monthLabels }
  }, [data, locale])

  const getLevelColor = (level: number) => {
    const colors = [
      'bg-muted/40 dark:bg-muted/20 hover:bg-muted/60 dark:hover:bg-muted/40 border border-border/50 dark:border-border/30',
      'bg-emerald-100 dark:bg-emerald-900/40 hover:bg-emerald-200 dark:hover:bg-emerald-900/60 border border-emerald-200/50 dark:border-emerald-800/30',
      'bg-emerald-300 dark:bg-emerald-700/60 hover:bg-emerald-400 dark:hover:bg-emerald-700/80 border border-emerald-400/50 dark:border-emerald-600/30',
      'bg-emerald-500 dark:bg-emerald-600/80 hover:bg-emerald-600 dark:hover:bg-emerald-600 border border-emerald-600/50 dark:border-emerald-500/30',
      'bg-emerald-700 dark:bg-emerald-500 hover:bg-emerald-800 dark:hover:bg-emerald-400 border border-emerald-800/50 dark:border-emerald-400/30',
    ]

    // clamp level to valid range
    const index = Math.max(0, Math.min(level, colors.length - 1))
    return colors[index]
  }

  const handleMouseMove = (
    e: React.MouseEvent<HTMLDivElement>,
    day: ContributionDay
  ) => {
    const rect = e.currentTarget.getBoundingClientRect()
    setMousePos({
      x: rect.left + rect.width / 2,
      y: rect.top,
    })
    setHoveredDay(day)
  }

  const totalContributions = data.reduce((sum, day) => sum + day.count, 0)

  const dayLabels = useMemo(() => {
    if (data.length === 0) return []

    const sortedData = [...data].sort(
      (a, b) => new Date(a.date).getTime() - new Date(b.date).getTime()
    )

    const firstDate = new Date(sortedData[0]?.date ?? Date.now())
    const startOfWeek = new Date(firstDate)
    startOfWeek.setDate(startOfWeek.getDate() - startOfWeek.getDay())

    return [1, 3, 5].map((offset) => {
      const date = new Date(startOfWeek)
      date.setDate(date.getDate() + offset)
      return date.toLocaleDateString(locale, { weekday: 'short' })
    })
  }, [data, locale])

  const dayIndices = [1, 3, 5]

  return (
    <Card className="hover:border-primary/50 dark:bg-card/50 w-full overflow-hidden border-2 p-6 transition-all duration-300 hover:shadow-lg">
      <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h3 className="text-2xl font-bold">{dict.title}</h3>
          <p className="text-muted-foreground text-sm">
            {dict.totalAmount.replace(
              '{count}',
              totalContributions.toLocaleString(locale)
            )}
          </p>
        </div>
        <div className="text-muted-foreground flex items-center gap-2 text-xs">
          <span>Less</span>
          {[0, 1, 2, 3, 4].map((level) => (
            <div
              key={level}
              className={`h-4 w-4 rounded-sm ${getLevelColor(level).split(' ').slice(0, 3).join(' ')}`}
            />
          ))}
          <span>More</span>
        </div>
      </div>

      <div className="relative w-full overflow-x-auto pb-2">
        <div className="inline-block min-w-full">
          {/* Month labels */}
          <div className="relative mb-3 h-5">
            {monthLabels.map(({ month, weekIndex }) => (
              <div
                key={`${month}-${weekIndex}`}
                className="text-muted-foreground absolute text-sm font-medium"
                style={{
                  left: `${48 + weekIndex * 20}px`,
                }}
              >
                {month}
              </div>
            ))}
          </div>

          {/* Contribution grid */}
          <div className="flex gap-1">
            <div className="flex flex-shrink-0 flex-col gap-1 pr-3">
              {[0, 1, 2, 3, 4, 5, 6].map((day) => (
                <div
                  key={day}
                  className="text-muted-foreground flex h-4 w-8 items-center text-xs font-medium"
                >
                  {dayIndices.includes(day)
                    ? dayLabels[dayIndices.indexOf(day)]
                    : ''}
                </div>
              ))}
            </div>

            <div className="flex gap-1">
              {weeks.map((week, weekIndex) => (
                <div
                  key={weekIndex}
                  className="flex flex-shrink-0 flex-col gap-1"
                >
                  {week.map((day, dayIndex) => {
                    const uniqueKey = `${weekIndex}-${dayIndex}`

                    if (!day) {
                      return (
                        <div
                          key={uniqueKey}
                          className="border-border/30 bg-muted/20 dark:border-border/20 dark:bg-muted/10 h-4 w-4 rounded-sm border"
                        />
                      )
                    }

                    return (
                      <div
                        key={uniqueKey}
                        className={`h-4 w-4 cursor-pointer rounded-sm transition-all duration-200 ${getLevelColor(day.level)}`}
                        onMouseEnter={(e) => handleMouseMove(e, day)}
                        onMouseLeave={() => setHoveredDay(null)}
                        style={{
                          animationDelay: `${(weekIndex * 7 + dayIndex) * 2}ms`,
                        }}
                      />
                    )
                  })}
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>

      {hoveredDay && (
        <div
          className="bg-popover dark:border-border/50 dark:bg-popover/95 pointer-events-none fixed z-50 -translate-x-1/2 -translate-y-full rounded-lg border-2 px-3 py-2 text-sm shadow-xl dark:shadow-2xl"
          style={{
            left: mousePos.x,
            top: mousePos.y - 8,
          }}
        >
          <p className="font-semibold">
            {hoveredDay.count.toLocaleString(locale)} contribution
            {hoveredDay.count !== 1 ? 's' : ''}
          </p>
          <p className="text-muted-foreground text-xs">
            {new Date(hoveredDay.date).toLocaleDateString(locale, {
              weekday: 'short',
              month: 'short',
              day: 'numeric',
              year: 'numeric',
            })}
          </p>
        </div>
      )}
    </Card>
  )
}
