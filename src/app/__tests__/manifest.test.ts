import { describe, expect, it } from 'vitest'

import manifest from '@/app/manifest'
import { siteConfig } from '@/lib/config'

describe('manifest', () => {
  it('should return valid manifest object', () => {
    const result = manifest()

    expect(result).toEqual({
      background_color: '#FFFFFF',
      categories: ['productivity', 'portfolio', 'developer'],
      description: siteConfig.description,
      display: 'standalone',
      icons: expect.arrayContaining([
        expect.objectContaining({ sizes: '32x32', src: '/icon/favicon' }),
        expect.objectContaining({ sizes: '192x192', src: '/icon/icon-192' }),
        expect.objectContaining({ sizes: '512x512', src: '/icon/icon-512' }),
      ]),
      id: 'portfolio',
      name: siteConfig.title,
      orientation: 'portrait',
      screenshots: expect.arrayContaining([
        expect.objectContaining({
          form_factor: 'wide',
          sizes: '1280x720',
          src: '/icon/screenshot-wide',
        }),
        expect.objectContaining({ sizes: '512x512', src: '/icon/icon-512' }),
      ]),
      short_name: siteConfig.title,
      start_url: '/',
      theme_color: '#FFFFFF',
    })
  })
})
