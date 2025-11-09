import { type HTMLAttributes, type JSX } from 'react'

import type { FCWithChildren } from '@/types/fc'

type HeadingTag = 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6'

type NativeHeadingProperties = HTMLAttributes<HTMLHeadingElement>

type HeadingProperties = Omit<NativeHeadingProperties, 'tw'> & {
  readonly as?: HeadingTag
  readonly tw?: string
}

export const Heading: FCWithChildren<HeadingProperties> = ({
  as: asTag,
  tw: twProperty,
  ...rest
}: HeadingProperties): JSX.Element => {
  const Element: HeadingTag = asTag ?? 'h2'
  const dataHeadingTag: Uppercase<HeadingTag> =
    Element.toUpperCase() as Uppercase<HeadingTag>

  return (
    <Element
      {...rest}
      {...(twProperty === undefined ? {} : { tw: twProperty })}
      data-heading-tag={dataHeadingTag}
    />
  )
}
