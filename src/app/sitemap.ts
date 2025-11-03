import type { MetadataRoute } from 'next'
import { routing } from '@/i18n/routing'

export default function sitemap(): MetadataRoute.Sitemap {
  const baseUrl = 'https://timmi6790.de'
  const currentDate = new Date()

  // Generate sitemap entries for all locales
  return routing.locales.flatMap((locale) => [
    {
      url: `${baseUrl}/${locale}`,
      lastModified: currentDate,
      changeFrequency: 'weekly' as const,
      priority: 1.0,
      alternates: {
        languages: Object.fromEntries(
          routing.locales.map((loc) => [loc, `${baseUrl}/${loc}`])
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
          routing.locales.map((loc) => [loc, `${baseUrl}/${loc}/imprint`])
        ),
      },
    },
  ])
}
