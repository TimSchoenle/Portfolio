import type { MetadataRoute } from 'next'
import { siteConfig } from '@/lib/config'
import { getPathname, routing } from '@/i18n/routing'
import { type Locale } from 'next-intl'

export default async function sitemap(): Promise<MetadataRoute.Sitemap> {
  const baseUrl = siteConfig.url.replace(/\/$/, '')
  const now = new Date()

  const staticPages = ['imprint', 'privacy']

  // Function to create URL entries
  function createUrlEntry(
    path: string,
    changeFreq: MetadataRoute.Sitemap[0]['changeFrequency'],
    priority: number
  ): MetadataRoute.Sitemap {
    const adjustedBasePath = path.startsWith('/') ? path : `/${path}`
    return [
      {
        url: `${baseUrl}${adjustedBasePath}`,
        lastModified: now,
        changeFrequency: changeFreq,
        priority,
        alternates: {
          languages: Object.fromEntries(
            routing.locales.map((loc: Locale) => [
              loc,
              `${baseUrl}${getPathname({ locale: loc, href: adjustedBasePath })}`,
            ])
          ),
        },
      },
    ]
  }

  // Build sitemap array
  const urls: MetadataRoute.Sitemap = []

  // Root pages
  urls.push(...createUrlEntry('/', 'weekly', 1.0))

  // Static pages
  for (const page of staticPages) {
    urls.push(...createUrlEntry(page, 'monthly', 0.5))
  }

  return urls
}
