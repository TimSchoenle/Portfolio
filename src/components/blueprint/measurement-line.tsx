import { type CSSProperties, type JSX } from 'react'

import type { FCStrict } from '@/types/fc'

interface MeasurementLineProperties {
  readonly className?: string
  readonly label?: string
  readonly orientation?: 'horizontal' | 'vertical'
  readonly width?: string
}

const STYLES: Record<
  'horizontal' | 'vertical',
  {
    container: string
    endTick: string
    labelRotation: string
    line: string
    startTick: string
  }
> = {
  horizontal: {
    container: 'flex-row',
    endTick: 'right-0 h-2 -translate-y-[0.5px] border-r',
    labelRotation: '',
    line: 'h-px w-full',
    startTick: 'left-0 h-2 -translate-y-[0.5px] border-l',
  },
  vertical: {
    container: 'flex-col',
    endTick: 'bottom-0 w-2 -translate-x-[0.5px] border-b',
    labelRotation: '-rotate-90',
    line: 'h-full w-px',
    startTick: 'top-0 w-2 -translate-x-[0.5px] border-t',
  },
}

const MeasurementLine: FCStrict<MeasurementLineProperties> = ({
  className,
  label,
  orientation = 'horizontal',
  width = '100px',
}: MeasurementLineProperties): JSX.Element => {
  const currentStyle: {
    container: string
    endTick: string
    labelRotation: string
    line: string
    startTick: string
  } = STYLES[orientation] // eslint-disable-line security/detect-object-injection

  const style: CSSProperties = {
    [orientation === 'vertical' ? 'height' : 'width']: width,
  }

  return (
    <div
      aria-hidden="true"
      className={`absolute flex items-center justify-center opacity-60 ${className ?? ''} ${currentStyle.container}`}
      style={style}
    >
      <div className={`${currentStyle.line} bg-brand`} />
      {Boolean(label) && (
        <span
          className={`absolute bg-blueprint-bg px-1 font-mono text-[9px] tracking-wider text-brand uppercase ${currentStyle.labelRotation}`}
        >
          {label}
        </span>
      )}
      <div className={`absolute border-brand ${currentStyle.startTick}`} />
      <div className={`absolute border-brand ${currentStyle.endTick}`} />
    </div>
  )
}

export { MeasurementLine }
