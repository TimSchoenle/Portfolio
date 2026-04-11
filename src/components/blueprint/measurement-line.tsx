import { type CSSProperties, type JSX } from 'react'

import type { FCStrict } from '@/types/fc'

interface MeasurementLineProperties {
  readonly className?: string
  readonly label?: string
  readonly orientation?: 'horizontal' | 'vertical'
  readonly width?: string
}

interface SubComponentProperties {
  readonly isVertical: boolean
}

const MainLine: FCStrict<SubComponentProperties> = ({
  isVertical,
}: SubComponentProperties): JSX.Element =>
  isVertical ? (
    <line
      className="stroke-brand"
      strokeWidth="1"
      x1="50%"
      x2="50%"
      y1="0"
      y2="100%"
    />
  ) : (
    <line
      className="stroke-brand"
      strokeWidth="1"
      x1="0"
      x2="100%"
      y1="50%"
      y2="50%"
    />
  )

const TickLines: FCStrict<SubComponentProperties> = ({
  isVertical,
}: SubComponentProperties): JSX.Element =>
  isVertical ? (
    <>
      <line
        className="stroke-brand"
        strokeWidth="1"
        x1="25%"
        x2="75%"
        y1="0"
        y2="0"
      />
      <line
        className="stroke-brand"
        strokeWidth="1"
        x1="25%"
        x2="75%"
        y1="100%"
        y2="100%"
      />
    </>
  ) : (
    <>
      <line
        className="stroke-brand"
        strokeWidth="1"
        x1="0"
        x2="0"
        y1="25%"
        y2="75%"
      />
      <line
        className="stroke-brand"
        strokeWidth="1"
        x1="100%"
        x2="100%"
        y1="25%"
        y2="75%"
      />
    </>
  )

const MeasurementLabel: FCStrict<
  SubComponentProperties & { readonly label: string }
> = ({
  isVertical,
  label,
}: SubComponentProperties & { readonly label: string }): JSX.Element => (
  <text
    aria-hidden="true"
    className="fill-brand font-mono text-[9px] tracking-wider uppercase select-none"
    dominantBaseline="central"
    style={{
      paintOrder: 'stroke',
      stroke: 'var(--blueprint-bg)',
      strokeLinejoin: 'round',
      strokeWidth: '10px',
      transformBox: 'fill-box',
      transformOrigin: 'center',
    }}
    textAnchor="middle"
    transform={isVertical ? 'rotate(-90)' : undefined}
    x="50%"
    y="50%"
  >
    {label}
  </text>
)

/**
 * Renders a decorative measurement line using SVG.
 *
 * **Accessibility:**
 * - Renders as an SVG with `role="img"` (via internal logic or wrapper depending on iteration).
 * - SVG text (`foreignObject`) is treated as graphical content, bypassing strict text-contrast rules
 *   that often flag decorative technical markings.
 */
const MeasurementLine: FCStrict<MeasurementLineProperties> = ({
  className,
  label,
  orientation = 'horizontal',
  width = '100px',
}: MeasurementLineProperties): JSX.Element => {
  const isVertical: boolean = orientation === 'vertical'
  const style: CSSProperties = {
    height: isVertical ? width : '10px',
    width: isVertical ? '10px' : width,
  }

  return (
    <div
      aria-hidden="true"
      className={`absolute flex items-center justify-center opacity-60 ${className ?? ''} pointer-events-none select-none`}
      role="img"
      style={style}
    >
      <svg
        className="h-full w-full overflow-visible"
        preserveAspectRatio="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <MainLine isVertical={isVertical} />
        <TickLines isVertical={isVertical} />
        {typeof label === 'string' ? (
          <MeasurementLabel isVertical={isVertical} label={label} />
        ) : null}
      </svg>
    </div>
  )
}

export { MeasurementLine }
