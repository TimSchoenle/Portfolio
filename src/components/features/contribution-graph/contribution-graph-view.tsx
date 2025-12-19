import { type JSX } from 'react'

import { type Locale, useTranslations } from 'next-intl'

import { Card } from '@/components/ui/card'
import type {
  CalendarModel,
  DayLabelTripleResult,
} from '@/lib/github/contribution-calendar'
import type { FCStrict } from '@/types/fc'

import {
  GraphSection,
  HeaderSection,
} from './contribution-graph-view-components'

interface ContributionGraphViewProperties {
  readonly calendar: CalendarModel
  readonly labels: DayLabelTripleResult
  readonly locale: Locale
  readonly onYearChange: (year: number) => void
  readonly selectedYear: number
  readonly total: number
  readonly variant?: 'blueprint' | 'default'
  readonly years: readonly number[]
}

export const ContributionGraphView: FCStrict<
  ContributionGraphViewProperties
> = ({
  calendar,
  labels,
  locale,
  onYearChange,
  selectedYear,
  total,
  variant = 'default',
  years,
}: ContributionGraphViewProperties): JSX.Element => {
  const translate: ReturnType<typeof useTranslations> = useTranslations(
    'projects.contributions'
  )

  const Wrapper: JSX.ElementType = variant === 'blueprint' ? 'div' : Card
  const wrapperClassName: string =
    variant === 'blueprint'
      ? 'w-full overflow-hidden p-6 transition-all duration-300'
      : 'w-full overflow-hidden border-2 p-6 transition-all duration-300 hover:border-primary/50 hover:shadow-lg dark:bg-card/50'

  return (
    <Wrapper className={wrapperClassName}>
      <HeaderSection
        selectedYear={selectedYear}
        total={total}
        translate={translate}
        variant={variant}
        years={years}
        onYearChange={onYearChange}
      />
      <GraphSection
        dayFive={labels.dayFive}
        dayOne={labels.dayOne}
        dayThree={labels.dayThree}
        locale={locale}
        variant={variant}
        weeks={calendar.weeks}
      />
    </Wrapper>
  )
}
