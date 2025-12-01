import { ImageResponse } from 'next/og'

import type { ImageResponseOptions } from 'next/dist/compiled/@vercel/og/types'

// Image metadata
export const size: { height: number; width: number } = {
  height: 180,
  width: 180,
}
export const contentType: string = 'image/png'

// Image generation
export default function Icon(): Response {
  return new ImageResponse(
    // ImageResponse JSX element
    <div
      style={{
        alignItems: 'center',
        background: 'black',
        borderRadius: '20%',
        color: 'white',
        display: 'flex',
        fontSize: 120,
        height: '100%',
        justifyContent: 'center',
        width: '100%',
      }}
    >
      {/* eslint-disable-next-line react/jsx-curly-brace-presence */}
      {'P'}
    </div>,
    // ImageResponse options
    {
      ...size,
    } as ImageResponseOptions
  )
}
