import { readFile } from 'node:fs/promises'
import path from 'node:path'

import { ImageResponse } from 'next/og'

import type { Buffer } from 'node:buffer'

export interface IconDimension {
  readonly height: number
  readonly width: number
}

export interface IconProperties {
  readonly contentType: string
  readonly id: IconType
  readonly size: IconDimension
}

export const IconIds: {
  readonly FAV_ICON: 'favicon'
  readonly ICON_192: 'icon-192'
  readonly ICON_512: 'icon-512'
  readonly SCREENSHOT_WIDE: 'screenshot-wide'
  readonly APPLE: 'apple'
} = {
  APPLE: 'apple',
  FAV_ICON: 'favicon',
  ICON_192: 'icon-192',
  ICON_512: 'icon-512',
  SCREENSHOT_WIDE: 'screenshot-wide',
} as const

export type IconType = (typeof IconIds)[keyof typeof IconIds]

export function getImageDimension(iconId: IconType): IconDimension {
  switch (iconId) {
    case IconIds.SCREENSHOT_WIDE: {
      return { height: 720, width: 1280 }
    }
    case IconIds.ICON_512: {
      return { height: 512, width: 512 }
    }
    case IconIds.ICON_192: {
      return { height: 192, width: 192 }
    }
    case IconIds.FAV_ICON: {
      return { height: 32, width: 32 }
    }
    case IconIds.APPLE: {
      return { height: 180, width: 180 }
    }
    default: {
      // Exhaustive check: if we ever get here, iconId is not handled
      throw new Error(`Unhandled iconId: ${String(iconId)}`)
    }
  }
}

export function createIcon(iconId: IconType): IconProperties {
  const dimension: IconDimension = getImageDimension(iconId)
  return { contentType: 'image/png', id: iconId, size: dimension }
}

export async function loadIconSvg(): Promise<string> {
  const filePath: string = path.join(process.cwd(), 'public', 'favicon.svg')
  const buffer: Buffer<ArrayBuffer> = await readFile(filePath)
  return buffer.toString('base64')
}

export async function generateDefaultIconResponse(
  iconId: IconType
): Promise<ImageResponse> {
  const svg: string = await loadIconSvg()
  const iconDimension: IconDimension = getImageDimension(iconId)
  return generateIconResponse(svg, iconDimension)
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
