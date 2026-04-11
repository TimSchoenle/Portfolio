import { type JSX } from 'react'

import { BlueprintText } from '@/components/blueprint/blueprint-text'
import type { FCWithRequiredChildren, WithRequiredChildren } from '@/types/fc'

import type { LucideIcon } from 'lucide-react'

// Removed unused FCStrict

interface BlueprintButtonProperties extends WithRequiredChildren {
  readonly href?: string
  readonly icon?: LucideIcon
  readonly onClick?: () => void
  readonly target?: string
  readonly variant?: 'outline' | 'primary'
}

interface BlueprintButtonContentProperties {
  readonly children: React.ReactNode
  readonly icon: LucideIcon | undefined
  readonly isPrimary: boolean
}

const BlueprintButtonContent: FCWithRequiredChildren<
  BlueprintButtonContentProperties
> = ({
  children,
  // eslint-disable-next-line @typescript-eslint/naming-convention
  icon: Icon,
  isPrimary,
}: BlueprintButtonContentProperties): JSX.Element => (
  <>
    <span className="relative z-10 flex items-center gap-2 font-bold">
      {Boolean(Icon) && Icon ? <Icon className="h-4 w-4" /> : null}
      {children}
    </span>
    {/* Button Frame */}
    <div
      className={`absolute inset-0 skew-x-[-12deg] border border-brand transition-all ${
        isPrimary
          ? 'shadow-[0_0_15px_color-mix(in srgb, var(--brand), transparent 60%)] bg-brand group-hover:bg-transparent group-hover:text-brand'
          : 'group-hover:shadow-[0_0_15px_color-mix(in srgb, var(--brand), transparent 70%)] bg-brand/5 group-hover:bg-brand/20'
      }`}
    />
    {isPrimary ? null : (
      <>
        <div className="absolute right-0 bottom-0 h-1 w-1 bg-brand" />
        <div className="absolute top-0 left-0 h-1 w-1 bg-brand" />
      </>
    )}
  </>
)

export const BlueprintButton: FCWithRequiredChildren<
  BlueprintButtonProperties
> = ({
  children,
  href,
  icon,
  onClick,
  target,
  variant = 'primary',
}: BlueprintButtonProperties): JSX.Element => {
  const isPrimary: boolean = variant === 'primary'

  const className: string = `group relative px-8 py-3 transition-colors hover:text-blueprint-text ${
    isPrimary ? 'text-blueprint-bg' : 'text-brand'
  }`

  const content: JSX.Element = (
    <BlueprintButtonContent icon={icon} isPrimary={isPrimary}>
      {children}
    </BlueprintButtonContent>
  )

  if (href !== undefined) {
    return (
      <BlueprintText
        as="a"
        className={className}
        href={href}
        rel={target === '_blank' ? 'noreferrer' : undefined}
        target={target}
        variant="brand"
      >
        {content}
      </BlueprintText>
    )
  }

  return (
    <BlueprintText
      as="button"
      className={className}
      type="button"
      uppercase={true}
      variant="brand"
      onClick={onClick}
    >
      {content}
    </BlueprintText>
  )
}
