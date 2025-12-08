import type { MetadataRoute } from 'next'

import { siteConfig } from '@/lib/config'

const icons: MetadataRoute.Manifest['icons'] = [
  {
    purpose: 'any',
    sizes: '32x32',
    src: '/icon/favicon',
    type: 'image/png',
  },
  {
    purpose: 'any',
    sizes: '192x192',
    src: '/icon/icon-192',
    type: 'image/png',
  },
  {
    purpose: 'maskable',
    sizes: '192x192',
    src: '/icon/icon-192',
    type: 'image/png',
  },
  {
    purpose: 'any',
    sizes: '512x512',
    // eslint-disable-next-line sonarjs/no-duplicate-string
    src: '/icon/icon-512',
    type: 'image/png',
  },
  {
    purpose: 'maskable',
    sizes: '512x512',
    src: '/icon/icon-512',
    type: 'image/png',
  },
]

export default function manifest(): MetadataRoute.Manifest {
  return {
    background_color: '#FFFFFF',
    categories: ['productivity', 'portfolio', 'developer'],
    description: siteConfig.description,
    display: 'standalone',
    icons,
    id: 'portfolio',
    name: siteConfig.title,
    orientation: 'portrait',
    screenshots: [
      {
        form_factor: 'wide',
        sizes: '1280x720',
        src: '/icon/screenshot-wide',
        type: 'image/png',
      },
      {
        sizes: '512x512',
        src: '/icon/icon-512',
        type: 'image/png',
      },
    ],
    short_name: siteConfig.title,
    start_url: '/',
    theme_color: '#FFFFFF',
  }
}
