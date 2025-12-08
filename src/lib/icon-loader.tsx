import { readFile } from 'node:fs/promises'
import path from 'node:path'

import { ImageResponse } from 'next/og'

import type { Buffer } from 'node:buffer'

export interface IconDimension {
  readonly height: number
  readonly width: number
}

export async function loadIconSvg(): Promise<string> {
  const filePath: string = path.join(process.cwd(), 'public', 'favicon.svg')
  const buffer: Buffer<ArrayBuffer> = await readFile(filePath)
  return buffer.toString('base64')
}

export function generateIconResponse(
  svg: string,
  size: IconDimension
): ImageResponse {
  return new ImageResponse(
    <div
      style={{
        alignItems: 'center',
        background: 'transparent',
        display: 'flex',
        height: '100%',
        justifyContent: 'center',
        width: '100%',
      }}
    >
      {/* eslint-disable-next-line next/no-img-element */}
      <img
        alt="icon"
        height={size.height}
        src={`data:image/svg+xml;base64,${svg}`}
        width={size.width}
      />
    </div>,
    {
      ...size,
    }
  )
}
