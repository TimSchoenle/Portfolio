import { readFile } from 'node:fs/promises'
import path from 'node:path'

import { ImageResponse } from 'next/og'
import { describe, expect, it, vi } from 'vitest'

import {
  createIcon,
  generateDefaultIconResponse,
  generateIconResponse,
  getImageDimension,
  IconIds,
  loadIconSvg,
} from '@/lib/icon-creator'

const { mockedReadFile } = vi.hoisted(() => {
  return { mockedReadFile: vi.fn() }
})

// Mock node:fs/promises
vi.mock('node:fs/promises', async () => {
  return {
    default: {
      readFile: mockedReadFile,
    },
    readFile: mockedReadFile,
  }
})

// Mock node:path
vi.mock('node:path', async () => {
  const join = vi.fn((...args) => args.join('/'))
  return {
    default: {
      join,
    },
    join,
  }
})

describe('icon-creator', () => {
  describe('getImageDimension', () => {
    it('should return correct dimensions for known icons', () => {
      expect(getImageDimension(IconIds.FAV_ICON)).toEqual({
        height: 32,
        width: 32,
      })
      expect(getImageDimension(IconIds.ICON_192)).toEqual({
        height: 192,
        width: 192,
      })
      expect(getImageDimension(IconIds.ICON_512)).toEqual({
        height: 512,
        width: 512,
      })
      expect(getImageDimension(IconIds.SCREENSHOT_WIDE)).toEqual({
        height: 720,
        width: 1280,
      })
      expect(getImageDimension(IconIds.APPLE)).toEqual({
        height: 180,
        width: 180,
      })
    })

    it('should throw error for unhandled icon id', () => {
      // @ts-expect-error - Testing invalid ID at runtime
      expect(() => getImageDimension('invalid-id')).toThrow(
        'Unhandled iconId: invalid-id'
      )
    })
  })

  describe('createIcon', () => {
    it('should create valid icon properties', () => {
      const prop = createIcon(IconIds.FAV_ICON)
      expect(prop).toEqual({
        contentType: 'image/png',
        id: IconIds.FAV_ICON,
        size: { height: 32, width: 32 },
      })
    })
  })

  describe('loadIconSvg', () => {
    it('should read the SVG file from the correct path and return base64 string', async () => {
      const mockSvgContent = '<svg>test</svg>'
      const mockBuffer = Buffer.from(mockSvgContent)

      mockedReadFile.mockResolvedValue(mockBuffer as any)

      const result = await loadIconSvg()

      expect(path.join).toHaveBeenCalledWith(
        process.cwd(),
        'public',
        'favicon.svg'
      )
      expect(readFile).toHaveBeenCalled()
      expect(result).toBe(mockBuffer.toString('base64'))
    })
  })

  describe('generateDefaultIconResponse', () => {
    it('should generate response using loaded SVG and correct dimension', async () => {
      const mockSvgContent = '<svg>test</svg>'
      const mockBuffer = Buffer.from(mockSvgContent)
      mockedReadFile.mockResolvedValue(mockBuffer as any)

      const response = await generateDefaultIconResponse(IconIds.ICON_192)

      expect(response).toBeInstanceOf(ImageResponse)
      expect(path.join).toHaveBeenCalled()
    })
  })

  describe('generateIconResponse', () => {
    it('should return an ImageResponse with correct dimensions', () => {
      const svg = 'base64string'
      const size = { height: 100, width: 100 }

      const response = generateIconResponse(svg, size)

      expect(response).toBeInstanceOf(ImageResponse)
    })
  })
})
