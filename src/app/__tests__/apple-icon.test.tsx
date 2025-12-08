import { describe, expect, it, vi } from 'vitest'

import AppleIcon, { generateImageMetadata } from '@/app/apple-icon'
import * as iconCreator from '@/lib/icon-creator'

// Mock the icon-creator dependency
vi.mock('@/lib/icon-creator', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/lib/icon-creator')>()
  return {
    ...actual,
    generateDefaultIconResponse: vi
      .fn()
      .mockImplementation(() => 'mock-response'),
  }
})

describe('AppleIcon', () => {
  describe('generateImageMetadata', () => {
    it('should return correct metadata', () => {
      const metadata = generateImageMetadata()

      expect(metadata).toEqual([
        {
          contentType: 'image/png',
          id: 'apple',
          size: { height: 180, width: 180 },
        },
      ])
    })
  })

  describe('AppleIcon Component', () => {
    it('should generate apple icon with correct params', async () => {
      await AppleIcon()

      expect(iconCreator.generateDefaultIconResponse).toHaveBeenCalledWith(
        'apple'
      )
    })
  })
})
