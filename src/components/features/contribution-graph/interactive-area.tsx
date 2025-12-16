import React, { type JSX } from 'react'

import {
  type HoverContextValue,
  useContributionHover,
} from '@/components/features/contribution-graph/contribution-hover-context'
import { type FCWithRequiredChildren } from '@/types/fc'

export const InteractiveArea: FCWithRequiredChildren = ({
  children,
}: {
  children: React.ReactNode
}): JSX.Element => {
  const { setHoveredDate, setHoverPosition }: HoverContextValue =
    useContributionHover()

  const handleMouseMove: (event: React.MouseEvent<HTMLDivElement>) => void = (
    event: React.MouseEvent<HTMLDivElement>
  ): void => {
    const target: HTMLElement = event.target as HTMLElement
    // eslint-disable-next-line unicorn/prefer-dom-node-dataset
    const date: string | null = target.getAttribute('data-date')

    if (date === null) {
      setHoveredDate(null)
      setHoverPosition(null)
    } else {
      setHoveredDate(date)
      setHoverPosition({
        x: event.clientX,
        y: event.clientY,
      })
    }
  }

  const handleMouseLeave: () => void = (): void => {
    setHoveredDate(null)
    setHoverPosition(null)
  }

  return (
    <div
      role="none"
      onMouseLeave={handleMouseLeave}
      onMouseMove={handleMouseMove}
    >
      {children}
    </div>
  )
}
