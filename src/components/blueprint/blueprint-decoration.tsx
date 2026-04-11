/* eslint-disable complexity */
import { type JSX, memo, type MemoExoticComponent } from 'react'

import { cn } from '@/lib/utilities'
import type { FCStrict } from '@/types/fc'

interface BlueprintCornersProperties {
  readonly className?: string
  readonly cornerLength?: number
  readonly corners?: readonly (
    | 'bottomLeft'
    | 'bottomRight'
    | 'topLeft'
    | 'topRight'
  )[]
  readonly strokeWidth?: number
  readonly variant?: 'all' | 'bracket' | 'lines'
}

interface BlueprintSideDecorationProperties {
  readonly className?: string
  readonly orientation?: 'horizontal' | 'vertical'
}

interface CornerSegmentProperties {
  readonly cornerLength: number
  readonly strokeWidth: number
  readonly variant: 'all' | 'bracket' | 'lines'
}

const TopLeftCorner: FCStrict<CornerSegmentProperties> = ({
  cornerLength,
  strokeWidth,
  variant,
}: CornerSegmentProperties): JSX.Element => (
  <>
    {variant === 'all' || variant === 'bracket' ? (
      <path
        className="fill-none stroke-brand"
        d={`M0 ${String(cornerLength)} V0 H${String(cornerLength)}`}
        strokeWidth={strokeWidth}
        vectorEffect="non-scaling-stroke"
      />
    ) : null}
    {variant === 'lines' ? (
      <line
        className="stroke-brand"
        strokeWidth={strokeWidth}
        vectorEffect="non-scaling-stroke"
        x1="6"
        x2="6"
        y1="-4"
        y2="4"
      />
    ) : null}
  </>
)

const TopRightCorner: FCStrict<CornerSegmentProperties> = ({
  cornerLength,
  strokeWidth,
  variant,
}: CornerSegmentProperties): JSX.Element => (
  <>
    {variant === 'all' ? (
      <path
        className="fill-none stroke-brand"
        d={`M-${String(cornerLength)} 0 H0 V${String(cornerLength)}`}
        strokeWidth={strokeWidth}
        vectorEffect="non-scaling-stroke"
      />
    ) : null}
    {variant === 'lines' ? (
      <line
        className="stroke-brand"
        strokeWidth={strokeWidth}
        vectorEffect="non-scaling-stroke"
        x1="-6"
        x2="-6"
        y1="-4"
        y2="4"
      />
    ) : null}
  </>
)

const BottomLeftCorner: FCStrict<CornerSegmentProperties> = ({
  cornerLength,
  strokeWidth,
  variant,
}: CornerSegmentProperties): JSX.Element => (
  <>
    {variant === 'all' ? (
      <path
        className="fill-none stroke-brand"
        d={`M0 -${String(cornerLength)} V0 H${String(cornerLength)}`}
        strokeWidth={strokeWidth}
        vectorEffect="non-scaling-stroke"
      />
    ) : null}
    {variant === 'lines' ? (
      <line
        className="stroke-brand"
        strokeWidth={strokeWidth}
        vectorEffect="non-scaling-stroke"
        x1="6"
        x2="6"
        y1="-4"
        y2="4"
      />
    ) : null}
  </>
)

const BottomRightCorner: FCStrict<CornerSegmentProperties> = ({
  cornerLength,
  strokeWidth,
  variant,
}: CornerSegmentProperties): JSX.Element => (
  <>
    {variant === 'all' || variant === 'bracket' ? (
      <path
        className="fill-none stroke-brand"
        d={`M-${String(cornerLength)} 0 H0 V-${String(cornerLength)}`}
        strokeWidth={strokeWidth}
        vectorEffect="non-scaling-stroke"
      />
    ) : null}
    {variant === 'lines' ? (
      <line
        className="stroke-brand"
        strokeWidth={strokeWidth}
        vectorEffect="non-scaling-stroke"
        x1="-6"
        x2="-6"
        y1="-4"
        y2="4"
      />
    ) : null}
  </>
)

const BlueprintCornersComponent: FCStrict<BlueprintCornersProperties> = ({
  className,
  cornerLength = 8,
  corners = ['topLeft', 'topRight', 'bottomLeft', 'bottomRight'],
  strokeWidth = 2,
  variant = 'all',
}: BlueprintCornersProperties): JSX.Element => {
  const showTopLeft: boolean = corners.includes('topLeft')
  const showTopRight: boolean = corners.includes('topRight')
  const showBottomLeft: boolean = corners.includes('bottomLeft')
  const showBottomRight: boolean = corners.includes('bottomRight')

  return (
    <svg
      aria-hidden="true"
      className={cn(
        'pointer-events-none absolute inset-0 h-full w-full overflow-visible',
        className
      )}
    >
      {showTopLeft ? (
        <TopLeftCorner
          cornerLength={cornerLength}
          strokeWidth={strokeWidth}
          variant={variant}
        />
      ) : null}

      {showTopRight ? (
        <svg overflow="visible" x="100%" y="0">
          <TopRightCorner
            cornerLength={cornerLength}
            strokeWidth={strokeWidth}
            variant={variant}
          />
        </svg>
      ) : null}

      {showBottomLeft ? (
        <svg overflow="visible" x="0" y="100%">
          <BottomLeftCorner
            cornerLength={cornerLength}
            strokeWidth={strokeWidth}
            variant={variant}
          />
        </svg>
      ) : null}

      {showBottomRight ? (
        <svg overflow="visible" x="100%" y="100%">
          <BottomRightCorner
            cornerLength={cornerLength}
            strokeWidth={strokeWidth}
            variant={variant}
          />
        </svg>
      ) : null}
    </svg>
  )
}

export const BlueprintCorners: MemoExoticComponent<
  FCStrict<BlueprintCornersProperties>
> = memo(BlueprintCornersComponent)

export const BlueprintSideDecoration: MemoExoticComponent<
  FCStrict<BlueprintSideDecorationProperties>
> = memo(
  ({
    className,
    orientation = 'vertical',
  }: BlueprintSideDecorationProperties): JSX.Element => {
    const isVertical: boolean = orientation === 'vertical'
    return (
      <svg
        aria-hidden="true"
        className={cn(
          'absolute overflow-visible',
          isVertical ? 'h-16 w-1' : 'h-1 w-16',
          className
        )}
      >
        <rect
          className="fill-current text-brand/40"
          height="100%"
          width="100%"
        />
      </svg>
    )
  }
)

BlueprintSideDecoration.displayName = 'BlueprintSideDecoration'
