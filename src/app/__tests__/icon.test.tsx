import { describe, expect, it, vi } from 'vitest'

import Icon, { generateImageMetadata } from '@/app/icon'
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

describe('Icon', () => {
  describe('generateImageMetadata', () => {
    it('should return metadata for all icon types', () => {
      const metadata = generateImageMetadata()

      expect(metadata).toHaveLength(4)
      expect(metadata).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            id: 'favicon',
            size: { height: 32, width: 32 },
          }),
          expect.objectContaining({
            id: 'icon-192',
            size: { height: 192, width: 192 },
          }),
          expect.objectContaining({
            id: 'icon-512',
            size: { height: 512, width: 512 },
          }),
          expect.objectContaining({
            id: 'screenshot-wide',
            size: { height: 720, width: 1280 },
          }),
        ])
      )
    })
  })

  describe('Icon Component', () => {
    it('should respond for favicon', async () => {
      await Icon({ id: Promise.resolve('favicon' as any) })
      expect(iconCreator.generateDefaultIconResponse).toHaveBeenCalledWith(
        'favicon'
      )
    })

    it('should respond for icon-192', async () => {
      await Icon({ id: Promise.resolve('icon-192' as any) })
      expect(iconCreator.generateDefaultIconResponse).toHaveBeenCalledWith(
        'icon-192'
      )
    })

    it('should respond for icon-512', async () => {
      await Icon({ id: Promise.resolve('icon-512' as any) })
      expect(iconCreator.generateDefaultIconResponse).toHaveBeenCalledWith(
        'icon-512'
      )
    })

    it('should respond for screenshot-wide', async () => {
      await Icon({ id: Promise.resolve('screenshot-wide' as any) })
      expect(iconCreator.generateDefaultIconResponse).toHaveBeenCalledWith(
        'screenshot-wide'
      )
    })
  })
})
