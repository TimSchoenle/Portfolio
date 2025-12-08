import { type ImageResponse } from 'next/og'

import {
  createIcon,
  generateDefaultIconResponse,
  IconIds,
  type IconProperties,
  type IconType,
} from '@/lib/icon-creator'

export const runtime: string = 'nodejs'

export function generateImageMetadata(): IconProperties[] {
  return [
    createIcon(IconIds.FAV_ICON),
    createIcon(IconIds.ICON_192),
    createIcon(IconIds.ICON_512),
    createIcon(IconIds.SCREENSHOT_WIDE),
  ]
}

export default async function Icon({
  // eslint-disable-next-line @typescript-eslint/naming-convention
  id,
}: {
  id: Promise<IconType>
}): Promise<ImageResponse> {
  const iconId: IconType = await id
  return generateDefaultIconResponse(iconId)
}
