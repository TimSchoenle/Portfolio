'use client'

import React, {
  type Context,
  createContext,
  type Dispatch,
  type JSX,
  type ReactNode,
  type SetStateAction,
  useContext,
  useState,
} from 'react'

export interface HoverPosition {
  readonly x: number
  readonly y: number
}

export type HoverPositionOrNull = HoverPosition | null

export interface HoverContextValue {
  readonly hoveredDate: string | null
  readonly hoverPosition: HoverPositionOrNull
  readonly setHoveredDate: Dispatch<SetStateAction<string | null>>
  readonly setHoverPosition: Dispatch<SetStateAction<HoverPosition | null>>
}

const HoverContext: Context<HoverContextValue | null> =
  createContext<HoverContextValue | null>(null)

export const useContributionHover: () => HoverContextValue =
  (): HoverContextValue => {
    const context: HoverContextValue | null = useContext(HoverContext)
    if (context === null) {
      throw new Error(
        'useContributionHover must be used within ContributionHoverProvider'
      )
    }
    return context
  }

interface ContributionHoverProviderProperties {
  readonly children: ReactNode
}

export const ContributionHoverProvider: React.FC<
  ContributionHoverProviderProperties
> = ({ children }: ContributionHoverProviderProperties): JSX.Element => {
  const [hoveredDate, setHoveredDate]: [
    string | null,
    Dispatch<SetStateAction<string | null>>,
  ] = useState<string | null>(null)

  const [hoverPosition, setHoverPosition]: [
    HoverPositionOrNull,
    Dispatch<SetStateAction<HoverPositionOrNull>>,
  ] = useState<HoverPositionOrNull>(null)

  return (
    <HoverContext.Provider
      value={{
        hoveredDate,
        hoverPosition,
        setHoveredDate,
        setHoverPosition,
      }}
    >
      {children}
    </HoverContext.Provider>
  )
}
