'use client'

import React, {
  type Dispatch,
  type JSX,
  type RefObject,
  type SetStateAction,
  useLayoutEffect,
  useState,
} from 'react'

import { getSkillIcon } from '@/components/sections/skill-icons'
import { RADAR_CONFIG } from '@/lib/radar-config'
import type { Blip } from '@/types/tech-radar'

import { type HoverContextValue, useHover } from './hover-context'

import type { LucideIcon } from 'lucide-react'

interface TooltipCoordinates {
  x: number
  y: number
}

interface CalculateCoordinatesParameters {
  readonly blip: Blip
  readonly parentRect: DOMRect
  readonly svgRect: DOMRect
}

const calculateCoordinates: (
  properties: CalculateCoordinatesParameters
) => TooltipCoordinates = ({
  blip,
  parentRect,
  svgRect,
}: CalculateCoordinatesParameters): TooltipCoordinates => {
  const viewBoxWidth: number = RADAR_CONFIG.viewBox.width
  const viewBoxHeight: number = RADAR_CONFIG.viewBox.height
  const svgRatio: number = viewBoxWidth / viewBoxHeight
  const elementRatio: number = svgRect.width / svgRect.height

  let drawWidth: number
  let drawHeight: number
  let offsetX: number
  let offsetY: number

  if (elementRatio > svgRatio) {
    // Constrained by height
    drawHeight = svgRect.height
    drawWidth = drawHeight * svgRatio
    offsetX = (svgRect.width - drawWidth) / 2
    offsetY = 0
  } else {
    // Constrained by width
    drawWidth = svgRect.width
    drawHeight = drawWidth / svgRatio
    offsetX = 0
    offsetY = (svgRect.height - drawHeight) / 2
  }

  const scale: number = drawWidth / viewBoxWidth
  const blipRadius: number =
    RADAR_CONFIG.blips.size * RADAR_CONFIG.blips.hoverScale

  // Calculate blip position in pixels relative to the SVG element
  const blipRelativeX: number =
    offsetX + (blip.xCoordinate - RADAR_CONFIG.viewBox.min) * scale
  // Calculate top edge of blip
  const blipRelativeY: number =
    offsetY +
    (blip.yCoordinate - RADAR_CONFIG.viewBox.min) * scale -
    blipRadius * scale

  // Convert to coordinates relative to the parent container
  return {
    x: svgRect.left - parentRect.left + blipRelativeX,
    y: svgRect.top - parentRect.top + blipRelativeY,
  }
}

interface UseTooltipPositionParameters {
  readonly blip: Blip | undefined
  readonly ref: RefObject<HTMLElement | null>
}

const useTooltipPosition: (
  properties: UseTooltipPositionParameters
) => TooltipCoordinates | null = ({
  blip,
  ref,
}: UseTooltipPositionParameters): TooltipCoordinates | null => {
  const [position, setPosition]: [
    TooltipCoordinates | null,
    Dispatch<SetStateAction<TooltipCoordinates | null>>,
  ] = useState<TooltipCoordinates | null>(null)

  useLayoutEffect((): (() => void) => {
    if (!blip || !ref.current) {
      return (): null => null
    }

    const updatePosition: () => void = (): void => {
      const parent: HTMLElement | null | undefined = ref.current?.parentElement
      const svg: SVGSVGElement | null | undefined = parent?.querySelector('svg')

      if (!parent || !svg) {
        return
      }

      const parentRect: DOMRect = parent.getBoundingClientRect()
      const svgRect: DOMRect = svg.getBoundingClientRect()

      setPosition(
        calculateCoordinates({
          blip,
          parentRect,
          svgRect,
        })
      )
    }

    updatePosition()
    window.addEventListener('resize', updatePosition)

    return (): void => {
      window.removeEventListener('resize', updatePosition)
    }
  }, [blip, ref])

  return position
}

interface TechRadarTooltipProperties {
  readonly blips: readonly Blip[]
}

export const TechRadarTooltip: React.FC<TechRadarTooltipProperties> = ({
  blips,
}: TechRadarTooltipProperties): JSX.Element | null => {
  const { hoveredBlip }: HoverContextValue = useHover()
  const tooltipReference: RefObject<HTMLDivElement | null> =
    React.useRef<HTMLDivElement>(null)

  const hoveredBlipData: Blip | undefined = React.useMemo(
    (): Blip | undefined =>
      blips.find((blip: Blip): boolean => blip.id === hoveredBlip),
    [blips, hoveredBlip]
  )

  const position: TooltipCoordinates | null = useTooltipPosition({
    blip: hoveredBlipData,
    ref: tooltipReference,
  })

  if (hoveredBlip === null) {
    return null
  }

  if (!hoveredBlipData) {
    return null
  }

  const Icon: LucideIcon = getSkillIcon(hoveredBlipData.iconName)

  if (!position) {
    // Render hidden to allow ref attachment
    return <div className="invisible absolute" ref={tooltipReference} />
  }

  return (
    <div
      className="pointer-events-none absolute z-50 flex flex-col items-center justify-center rounded-lg border border-border bg-popover px-3 py-2 text-sm shadow-md transition-opacity animate-in fade-in zoom-in-95"
      ref={tooltipReference}
      style={{
        left: position.x,
        top: position.y,
        transform: 'translate(-50%, calc(-100% - 8px))',
      }}
    >
      <Icon className="mb-1 h-4 w-4 text-primary" />
      <span className="font-semibold text-popover-foreground">
        {hoveredBlipData.name}
      </span>
      <span className="text-xs text-muted-foreground capitalize">
        {hoveredBlipData.quadrant}
      </span>
    </div>
  )
}
