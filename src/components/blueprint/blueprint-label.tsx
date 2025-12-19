import {
  type CSSProperties,
  type ElementType,
  type JSX,
  type ReactNode,
} from 'react'

import type { FCWithRequiredChildren } from '@/types/fc'

interface BlueprintLabelProperties {
  /** The element type to render (default: 'div'). */
  readonly as?: 'div' | 'p' | 'span'
  /** Additional CSS classes. */
  readonly className?: string
  /** Inline styles (crucial for dynamic positioning/sizing). */
  readonly style?: CSSProperties
}

/**
 * A wrapper for decorative text elements in the Blueprint design system.
 *
 * **Accessibility:**
 * - Applies `aria-hidden="true"` to remove content from the accessibility tree.
 * - Applies `role="img"` to signal to heuristics that this constitutes a graphical element / texture,
 *   preventing false positives in color contrast audits for purely decorative text.
 */
export const BlueprintLabel: FCWithRequiredChildren<
  BlueprintLabelProperties
> = ({
  // eslint-disable-next-line @typescript-eslint/naming-convention
  as = 'div',
  children,
  className,
  style,
}: BlueprintLabelProperties & {
  readonly children: ReactNode
}): JSX.Element => {
  const Component: ElementType = as

  return (
    <Component
      aria-hidden="true"
      className={className}
      role="img"
      style={style}
    >
      {children}
    </Component>
  )
}
