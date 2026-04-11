import { type JSX } from 'react'

import { BlueprintCorners } from '@/components/blueprint/blueprint-decoration'
import type { FCStrict } from '@/types/fc'

interface BlueprintItemProperties {
  readonly href: string
  readonly icon: JSX.Element
  readonly label: string
  readonly subLabel?: string
  readonly target?: string
}

export const BlueprintItem: FCStrict<BlueprintItemProperties> = ({
  href,
  icon,
  label,
  subLabel,
  target = '_blank',
}: BlueprintItemProperties): JSX.Element => (
  <a
    className="group hover:shadow-[0_0_10px_color-mix(in srgb, var(--brand), transparent 90%)] relative flex items-center gap-4 border border-brand/30 bg-brand/5 p-4 transition-all hover:bg-brand/10"
    href={href}
    rel={target === '_blank' ? 'noreferrer' : undefined}
    target={target}
  >
    <div className="flex h-10 w-10 items-center justify-center rounded-none border border-brand bg-blueprint-bg text-brand shadow-[0_0_5px_#60A5FA]">
      {icon}
    </div>
    <div className="flex flex-col">
      <span className="font-mono text-sm font-bold tracking-wide text-blueprint-text transition-colors group-hover:text-brand">
        {label}
      </span>
      {Boolean(subLabel) ? (
        <span className="font-mono text-xs tracking-wider text-blueprint-muted uppercase">
          {subLabel}
        </span>
      ) : null}
    </div>

    {/* Corner Accents */}
    <BlueprintCorners
      cornerLength={6}
      corners={['topRight', 'bottomLeft']}
      strokeWidth={1}
    />
  </a>
)
