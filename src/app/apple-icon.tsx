import { type ImageResponse } from 'next/og'

import {
  createIcon,
  generateDefaultIconResponse,
  IconIds,
  type IconProperties,
} from '@/lib/icon-creator'

export const runtime: string = 'nodejs'

export function generateImageMetadata(): IconProperties[] {
  return [createIcon(IconIds.APPLE)]
}

export default async function AppleIcon(): Promise<ImageResponse> {
  return generateDefaultIconResponse(IconIds.APPLE)
}
