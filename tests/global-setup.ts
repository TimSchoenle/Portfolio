import { ensureSitemapPaths } from './global-setup/sitemap-collect'

export default async function globalSetup(): Promise<void> {
  const origin: string =
    process.env['SITEMAP_ORIGIN'] ??
    process.env['PLAYWRIGHT_BASE_URL'] ??
    'http://localhost:3000'

  // Build the cache before tests are collected
  await ensureSitemapPaths({ origin })
}
