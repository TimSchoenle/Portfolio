import { type JSX } from 'react'

import type { FCWithRequiredChildren, WithRequiredChildren } from '@/types/fc'

import {
  BlueprintCorners,
  BlueprintSideDecoration,
} from './blueprint-decoration'
import { BlueprintLabel } from './blueprint-label'
import { BlueprintCardBackground } from './blueprint-primitives'
import { MeasurementLine } from './measurement-line'

interface BlueprintCardProperties extends WithRequiredChildren {
  readonly className?: string
  readonly label?: string
  readonly noPadding?: boolean
}

export const BlueprintCard: FCWithRequiredChildren<BlueprintCardProperties> = ({
  children,
  className,
  label,
  noPadding = false,
}: BlueprintCardProperties): JSX.Element => (
  <div
    className={`relative ${noPadding ? '' : 'p-8 md:p-12'} w-full ${className ?? ''}`}
  >
    {/* Technical Frame for Content */}
    <BlueprintCardBackground />

    {/* Corner Markers */}
    <BlueprintCorners className="p-0" />

    {/* Side Decoration */}
    <BlueprintSideDecoration className="top-12 left-0" />
    <BlueprintSideDecoration className="right-0 bottom-12" />

    {Boolean(label) ? (
      <BlueprintLabel className="absolute -top-3 left-[var(--app-padding)] border border-brand/30 bg-blueprint-bg px-2 font-mono text-xs tracking-widest text-brand uppercase">
        {label}
      </BlueprintLabel>
    ) : null}

    <div className="relative z-10 flex h-full flex-col gap-8">{children}</div>

    {/* Bottom Measurement */}
    <MeasurementLine className="bottom-4 left-0 opacity-30" width="100%" />
  </div>
)
