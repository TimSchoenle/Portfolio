import { type ImageResponse } from 'next/og'

import {
  generateIconResponse,
  type IconDimension,
  loadIconSvg,
} from '@/lib/icon-loader'

export const runtime: string = 'nodejs'

const IconIds: {
  readonly FAV_ICONS: 'favicon'
  readonly ICON_192: 'icon-192'
  readonly ICON_512: 'icon-512'
} = {
  FAV_ICONS: 'favicon',
  ICON_192: 'icon-192',
  ICON_512: 'icon-512',
} as const

type IconType = (typeof IconIds)[keyof typeof IconIds]

interface IconProperties {
  readonly contentType: string
  readonly id: IconType
  readonly size: IconDimension
}

function getImageDimension(iconId: IconType): IconDimension {
  switch (iconId) {
    case IconIds.ICON_512: {
      return { height: 512, width: 512 }
    }
    case IconIds.ICON_192: {
      return { height: 192, width: 192 }
    }
    case IconIds.FAV_ICONS: {
      return { height: 32, width: 32 }
    }
  }
}

function createIcon(iconId: IconType): IconProperties {
  const dimension: IconDimension = getImageDimension(iconId)
  return { contentType: 'image/png', id: iconId, size: dimension }
}

export function generateImageMetadata(): IconProperties[] {
  return [
    createIcon(IconIds.FAV_ICONS),
    createIcon(IconIds.ICON_192),
    createIcon(IconIds.ICON_512),
  ]
}

export default async function Icon({
  // eslint-disable-next-line @typescript-eslint/naming-convention
  id,
}: {
  id: Promise<IconType>
}): Promise<ImageResponse> {
  const iconId: IconType = await id

  const svg: string = await loadIconSvg()
  const iconDimension: IconDimension = getImageDimension(iconId)

  return generateIconResponse(svg, iconDimension)
}
