'use client'

import React, { type JSX, useEffect, useState } from 'react'

import { type Locale, useTranslations } from 'next-intl'

import { createPortal } from 'react-dom'

import {
  type HoverContextValue,
  useContributionHover,
} from '@/components/features/contribution-graph/contribution-hover-context'
import type { ContributionPoint } from '@/models/github'
import type { FCNullable } from '@/types/fc'
import type { Translations } from '@/types/i18n'

interface ContributionTooltipProperties {
  readonly data: readonly ContributionPoint[]
  readonly locale: Locale
}

// eslint-disable-next-line max-lines-per-function
export const ContributionTooltip: FCNullable<ContributionTooltipProperties> = ({
  data,
  locale,
}: ContributionTooltipProperties): JSX.Element | null => {
  const { hoveredDate, hoverPosition }: HoverContextValue =
    useContributionHover()
  const translations: Translations<'projects.contributions'> = useTranslations(
    'projects.contributions'
  )
  const [mounted, setMounted]: [
    boolean,
    React.Dispatch<React.SetStateAction<boolean>>,
  ] = useState<boolean>(false)

  useEffect((): void => {
    setMounted(true)
  }, [])

  if (!mounted || hoveredDate === null || hoverPosition === null) {
    return null
  }

  const point: ContributionPoint | undefined = data.find(
    (contributionItem: ContributionPoint): boolean =>
      contributionItem.date === hoveredDate
  )

  if (!point) {
    return null
  }

  const dateText: string = new Date(point.date).toLocaleDateString(locale, {
    day: 'numeric',
    month: 'short',
    timeZone: 'UTC',
    weekday: 'short',
    year: 'numeric',
  })

  // Offset tooltip to be above the cursor
  const style: React.CSSProperties = {
    left: hoverPosition.x,
    pointerEvents: 'none',
    position: 'fixed',
    top: hoverPosition.y - 10,
    transform: 'translate(-50%, -100%)',
    zIndex: 50,
  }

  return createPortal(
    <div
      className="flex flex-col items-center justify-center rounded-lg border border-border bg-popover px-3 py-2 text-sm shadow-md transition-opacity animate-in fade-in zoom-in-95"
      style={style}
    >
      <span className="font-semibold text-popover-foreground">
        {translations('tooltip', { count: point.count })}
      </span>
      <span className="text-xs text-muted-foreground">{dateText}</span>
    </div>,
    document.body
  )
}
