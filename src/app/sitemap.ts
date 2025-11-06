import type { MetadataRoute } from 'next'
import { type Locale } from 'next-intl'

import { getPathname, routing } from '@/i18n/routing'
import { siteConfig } from '@/lib/config'

type ChangeFreq = MetadataRoute.Sitemap[0]['changeFrequency']

interface CreateUrlEntryOptions {
  readonly baseUrl: string
  readonly now: Readonly<Date>
  readonly path: string
  readonly changeFreq: ChangeFreq
  readonly priority: number
}

function createUrlEntry({
  baseUrl,
  now,
  path,
  changeFreq,
  priority,
}: CreateUrlEntryOptions): MetadataRoute.Sitemap {
  const adjustedBasePath: string = path.startsWith('/') ? path : `/${path}`

  const alternates: Record<string, string> = Object.fromEntries(
    routing.locales.map((loc: Locale): [string, string] => [
      loc,
      `${baseUrl}${getPathname({ locale: loc, href: adjustedBasePath })}`,
    ])
  )

  return [
    {
      url: `${baseUrl}${adjustedBasePath}`,
      lastModified: now,
      changeFrequency: changeFreq,
      priority,
      alternates: { languages: alternates },
    },
  ]
}

export default function sitemap(): MetadataRoute.Sitemap {
  const baseUrl: string = siteConfig.url.replace(/\/$/, '')
  const now: Date = new Date()

  const staticPages: readonly string[] = ['imprint', 'privacy']

  const urls: MetadataRoute.Sitemap = []

  // Root page
  urls.push(
    ...createUrlEntry({
      baseUrl,
      now,
      path: '/',
      changeFreq: 'weekly',
      priority: 1.0,
    })
  )

  // Static pages
  for (const page of staticPages) {
    urls.push(
      ...createUrlEntry({
        baseUrl,
        now,
        path: page,
        changeFreq: 'monthly',
        priority: 0.5,
      })
    )
  }

  return urls
}
