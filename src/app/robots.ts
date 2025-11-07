import type { MetadataRoute } from 'next'

import { siteConfig } from '@/lib/config'

export default function robots(): MetadataRoute.Robots {
  const baseUrl: string = siteConfig.url

  return {
    host: baseUrl,
    rules: [
      {
        allow: '/',
        disallow: ['/api/'],
        userAgent: '*',
      },
    ],
    sitemap: `${baseUrl}/sitemap.xml`,
  }
}
