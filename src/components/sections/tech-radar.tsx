/* eslint-disable max-params, max-lines-per-function, @typescript-eslint/typedef, react/jsx-no-literals */
'use client'

import React, { type JSX, useMemo, useState } from 'react'

import { type LucideIcon } from 'lucide-react'

import { getSkillIcon } from '@/components/sections/skill-icons'
import { type Skill } from '@/lib/config'
import { cn } from '@/lib/utilities'

interface TechRadarProperties {
  readonly languages: readonly Skill[]
  readonly frameworks: readonly Skill[]
  readonly buildTools: readonly Skill[]
  readonly infrastructure: readonly Skill[]
}

interface Blip {
  readonly angle: number
  readonly icon: LucideIcon
  readonly id: string
  readonly name: string
  readonly quadrant: 'languages' | 'frameworks' | 'buildTools' | 'infrastructure'
  readonly radius: number
  readonly x: number
  readonly y: number
}

// Seeded random number generator for deterministic layout
const seededRandom = (seed: number): number => {
  const temporary = Math.sin(seed) * 10_000
  return temporary - Math.floor(temporary)
}

/**
 * Calculate position for a blip within a quadrant.
 * `startAngle` and `endAngle` define the angular range (radians).
 */
const calculateBlipPosition = (
  index: number,
  total: number,
  startAngle: number,
  endAngle: number,
  seedOffset: number,
  confidence: number
): { angle: number; radius: number; x: number; y: number } => {
  const angleStep = (endAngle - startAngle) / (total + 1)
  const angleJitter = (seededRandom(index + seedOffset) - 0.5) * 0.2
  const angle = startAngle + (index + 1) * angleStep + angleJitter

  // Map confidence (0-1) to radius (inner to outer)
  // High confidence -> closer to center
  // Low confidence -> closer to edge
  const minRadius = 25
  const maxRadius = 85
  const radiusJitter = (seededRandom(index + seedOffset + 100) - 0.5) * 5
  const radius =
    minRadius + (1 - confidence) * (maxRadius - minRadius) + radiusJitter

  const x = Math.cos(angle) * radius
  const y = Math.sin(angle) * radius
  return { angle, radius, x, y }
}

export const TechRadar: React.FC<TechRadarProperties> = ({
  languages,
  frameworks,
  buildTools,
  infrastructure,
}): JSX.Element => {
  const [hoveredBlip, setHoveredBlip] = useState<string | null>(null)

  const blips = useMemo((): Blip[] => {
    const all: Blip[] = []
    const quadrants = {
      languages: { end: (3 * Math.PI) / 2, start: Math.PI }, // top-left
      frameworks: { end: 2 * Math.PI, start: (3 * Math.PI) / 2 }, // top-right
      buildTools: { end: Math.PI, start: Math.PI / 2 }, // bottom-left
      infrastructure: { end: Math.PI / 2, start: 0 }, // bottom-right
    }

    const pushCategory = (
      items: readonly Skill[],
      quadrantKey: Blip['quadrant'],
      start: number,
      end: number
    ) => {
      for (const [index, skill] of items.entries()) {
        const { angle, radius, x, y } = calculateBlipPosition(
          index,
          items.length,
          start,
          end,
          1000 + index,
          skill.confidence
        )
        all.push({
          angle,
          icon: getSkillIcon(skill.name),
          id: `${quadrantKey}-${index}`,
          name: skill.name,
          quadrant: quadrantKey,
          radius,
          x,
          y,
        })
      }
    }

    pushCategory(
      languages,
      'languages',
      quadrants.languages.start,
      quadrants.languages.end
    )
    pushCategory(
      frameworks,
      'frameworks',
      quadrants.frameworks.start,
      quadrants.frameworks.end
    )
    pushCategory(
      buildTools,
      'buildTools',
      quadrants.buildTools.start,
      quadrants.buildTools.end
    )
    pushCategory(
      infrastructure,
      'infrastructure',
      quadrants.infrastructure.start,
      quadrants.infrastructure.end
    )

    return all
  }, [languages, frameworks, buildTools, infrastructure])

  return (
    <div className="relative mx-auto aspect-square w-full max-w-2xl">
      <svg
        className="h-full w-full overflow-visible"
        viewBox="-100 -100 200 200"
      >
        {/* Clip path for axes */}
        <defs>
          <clipPath id="radarClip">
            <circle cx="0" cy="0" r="97.5" />
          </clipPath>
          {/* Curved text paths for quadrant labels - moved outside at 105px radius */}
          <path
            d="M -105,0 A 105,105 0 0,1 0,-105"
            fill="none"
            id="languagesPath"
          />
          <path d="M 0,-105 A 105,105 0 0,1 105,0" fill="none" id="frameworksPath" />
          {/* Bottom paths reversed to keep text upright - radius increased to 114 to match visual distance of top labels */}
          <path
            d="M -114,0 A 114,114 0 0,0 0,114"
            fill="none"
            id="buildToolsPath"
          />
          <path
            d="M 0,114 A 114,114 0 0,0 114,0"
            fill="none"
            id="infrastructurePath"
          />
        </defs>

        {/* Background circles */}
        <circle
          className="fill-muted/20 stroke-border stroke-1"
          cx="0"
          cy="0"
          r="32.5"
        />
        <circle
          className="fill-muted/10 stroke-border stroke-1"
          cx="0"
          cy="0"
          r="67.5"
        />
        <circle
          className="fill-transparent stroke-border stroke-1"
          cx="0"
          cy="0"
          r="97.5"
        />

        {/* Sonar sweep animation - increased opacity */}
        <g
          className="animate-[spin_10s_linear_infinite]"
          style={{ transformOrigin: '0 0' }}
        >
          <path
            className="fill-primary/20"
            d="M 0 0 L 100 0 A 100 100 0 0 1 70.7 70.7 Z"
            style={{ filter: 'blur(8px)' }}
          />
        </g>

        {/* Axes - clipped to circle */}
        <g clipPath="url(#radarClip)">
          <line
            className="stroke-border/50 stroke-1"
            x1="-100"
            x2="100"
            y1="0"
            y2="0"
          />
          <line
            className="stroke-border/50 stroke-1"
            x1="0"
            x2="0"
            y1="-100"
            y2="100"
          />
        </g>

        {/* Curved Quadrant Labels - Top quadrants */}
        <text className="fill-primary text-[8px] font-bold tracking-wider uppercase">
          <textPath href="#languagesPath" startOffset="50%" textAnchor="middle">
            Languages
          </textPath>
        </text>
        <text className="fill-sky-500 text-[8px] font-bold tracking-wider uppercase">
          <textPath href="#frameworksPath" startOffset="50%" textAnchor="middle">
            Frameworks
          </textPath>
        </text>

        {/* Curved Quadrant Labels - Bottom quadrants with upright text */}
        <text className="fill-slate-500 text-[8px] font-bold tracking-wider uppercase">
          <textPath href="#buildToolsPath" startOffset="50%" textAnchor="middle">
            Build & Tools
          </textPath>
        </text>
        <text className="fill-violet-500 text-[8px] font-bold tracking-wider uppercase">
          <textPath href="#infrastructurePath" startOffset="50%" textAnchor="middle">
            Infrastructure
          </textPath>
        </text>
        {/* Blips */}
        {blips.map((blip): JSX.Element => {
          const isHovered = hoveredBlip === blip.id
          let blipColorClass = ''
          switch (blip.quadrant) {
            case 'languages': {
              blipColorClass = 'fill-primary stroke-primary-foreground'
              break;
            }
            case 'frameworks': {
              blipColorClass = 'fill-sky-500 stroke-sky-100'
              break;
            }
            case 'buildTools': {
              blipColorClass = 'fill-slate-500 stroke-slate-100'
              break;
            }
            case 'infrastructure': {
              blipColorClass = 'fill-violet-500 stroke-violet-100'
              break;
            }
            default: {
              blipColorClass = 'fill-foreground stroke-background'
            }
          }

          return (
            <g
              className="cursor-pointer transition-all duration-300"
              key={blip.id}
              style={{
                transform: `translate(${blip.x}px, ${blip.y}px) scale(${isHovered ? '1.5' : '1'})`,
                zIndex: isHovered ? 50 : 1,
              }}
              onMouseEnter={() => { setHoveredBlip(blip.id); }}
              onMouseLeave={() => { setHoveredBlip(null); }}
            >
              <circle
                className={cn('transition-colors duration-300', blipColorClass)}
                r="3"
                strokeWidth="0.5"
              />
            </g>
          )
        })}
      </svg>
      {/* Tooltip */}
      {hoveredBlip && (
        <div
          className="pointer-events-none absolute z-50 flex flex-col items-center justify-center rounded-lg border border-border bg-popover px-3 py-2 text-sm shadow-md transition-opacity animate-in fade-in zoom-in-95"
          style={(() => {
            const hoveredBlipData = blips.find((b) => b.id === hoveredBlip)
            const xPos = 50 + (hoveredBlipData?.x ?? 0) / 2
            const yPos = 50 + (hoveredBlipData?.y ?? 0) / 2
            return {
              left: `${xPos}%`,
              top: `${yPos}%`,
              transform: 'translate(-50%, -120%)',
            }
          })()}
        >
          {(() => {
            const blip = blips.find((b) => b.id === hoveredBlip)
            if (!blip) { return null }
            const Icon = blip.icon
            return (
              <>
                <Icon className="mb-1 h-4 w-4 text-primary" />
                <span className="font-semibold text-popover-foreground">
                  {blip.name}
                </span>
                <span className="text-xs text-muted-foreground capitalize">
                  {blip.quadrant}
                </span>
              </>
            )
          })()}
        </div>
      )}
      {/* Center hub */}
      <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
        <div className="h-2 w-2 animate-pulse rounded-full bg-primary" />
      </div>
    </div>
  )
}
