import { locales } from '@/lib/i18n-config'
import type { MetadataRoute } from 'next'

export default function sitemap(): MetadataRoute.Sitemap {
  const baseUrl = 'https://timmi6790.de'
  const currentDate = new Date()

  // Generate sitemap entries for all locales
  const routes = locales.flatMap((locale) => [
    {
      url: `${baseUrl}/${locale}`,
      lastModified: currentDate,
      changeFrequency: 'weekly' as const,
      priority: 1.0,
      alternates: {
        languages: Object.fromEntries(
          locales.map((loc) => [loc, `${baseUrl}/${loc}`])
        ),
      },
    },
    {
      url: `${baseUrl}/${locale}/imprint`,
      lastModified: currentDate,
      changeFrequency: 'monthly' as const,
      priority: 0.5,
      alternates: {
        languages: Object.fromEntries(
          locales.map((loc) => [loc, `${baseUrl}/${loc}/imprint`])
        ),
      },
    },
  ])

  return routes
}

