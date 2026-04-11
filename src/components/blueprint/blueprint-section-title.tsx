import { type JSX } from 'react'

import type { FCStrict } from '@/types/fc'

import {
  BlueprintHeading,
  BlueprintSubheading,
  BlueprintTinyLabel,
} from './blueprint-primitives'
import { MeasurementLine } from './measurement-line'

interface BlueprintSectionTitleProperties {
  readonly as?: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6' | undefined
  readonly greeting?: string
  readonly sectionLabel?: string
  readonly subtitle?: string
  readonly title: string
}

export const BlueprintSectionTitle: FCStrict<
  BlueprintSectionTitleProperties
> = ({
  as: headingAs,
  greeting,
  sectionLabel = '// SECTION_HEADER',
  subtitle,
  title,
}: BlueprintSectionTitleProperties): JSX.Element => (
  <div className="relative z-10 mx-auto max-w-4xl p-12 text-center">
    {/* Framing Box for text */}
    <div className="absolute inset-0 border border-brand/30" />
    <div className="absolute inset-0 scale-[1.02] border border-brand/10" />

    {/* Section Identifier */}
    <div className="absolute -top-3 left-4 bg-blueprint-bg px-2">
      <BlueprintTinyLabel>{sectionLabel}</BlueprintTinyLabel>
    </div>

    {/* Title Block */}
    <div className="flex flex-col items-center gap-2">
      {Boolean(greeting) ? (
        <BlueprintSubheading className="text-xl md:text-2xl">
          {greeting}
        </BlueprintSubheading>
      ) : null}
      <BlueprintHeading as={headingAs}>{title.toUpperCase()}</BlueprintHeading>
      <div className="my-4 h-px w-32 bg-gradient-to-r from-transparent via-brand to-transparent" />
      {Boolean(subtitle) ? (
        <p className="font-mono text-lg tracking-widest text-blueprint-muted uppercase sm:text-2xl">
          {subtitle}
        </p>
      ) : null}
    </div>

    {/* Measurements around the box */}
    <MeasurementLine
      className="-top-8 left-0"
      label="ZONE_WIDTH"
      width="100%"
    />
    <MeasurementLine
      className="top-0 -right-8 hidden lg:flex"
      label="HEIGHT"
      orientation="vertical"
      width="100%"
    />
  </div>
)
