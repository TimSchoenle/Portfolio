import type { MetadataRoute } from 'next'
import { type Locale } from 'next-intl'

import { getPathname, routing } from '@/i18n/routing'
import { siteConfig } from '@/lib/config'

type ChangeFreq = MetadataRoute.Sitemap[0]['changeFrequency']

interface CreateUrlEntryOptions {
  readonly baseUrl: string
  readonly changeFreq: ChangeFreq
  readonly now: Readonly<Date>
  readonly path: string
  readonly priority: number
}

function createUrlEntry({
  baseUrl,
  changeFreq,
  now,
  path,
  priority,
}: CreateUrlEntryOptions): MetadataRoute.Sitemap {
  const adjustedBasePath: string = path.startsWith('/') ? path : `/${path}`

  const alternates: Record<string, string> = Object.fromEntries(
    routing.locales.map((loc: Locale): [string, string] => [
      loc,
      `${baseUrl}${getPathname({ href: adjustedBasePath, locale: loc })}`,
    ])
  )

  return [
    {
      alternates: { languages: alternates },
      changeFrequency: changeFreq,
      lastModified: now,
      priority,
      url: `${baseUrl}${adjustedBasePath}`,
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
      changeFreq: 'weekly',
      now,
      path: '/',
      priority: 1.0,
    })
  )

  // Static pages
  for (const page of staticPages) {
    urls.push(
      ...createUrlEntry({
        baseUrl,
        changeFreq: 'monthly',
        now,
        path: page,
        priority: 0.5,
      })
    )
  }

  return urls
}
