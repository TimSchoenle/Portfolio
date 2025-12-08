import { type ImageResponse } from 'next/og'

import { generateIconResponse, loadIconSvg } from '@/lib/icon-loader'

export const runtime: string = 'nodejs'

export function generateImageMetadata(): {
  contentType: string
  id: string
  size: { height: number; width: number }
}[] {
  return [
    {
      contentType: 'image/png',
      id: 'apple',
      size: { height: 180, width: 180 },
    },
  ]
}

export default async function AppleIcon(): Promise<ImageResponse> {
  const svg: string = await loadIconSvg()
  const size: { height: number; width: number } = { height: 180, width: 180 }
  return generateIconResponse(svg, size)
}
