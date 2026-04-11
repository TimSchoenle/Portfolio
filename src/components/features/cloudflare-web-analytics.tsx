import type { FC, JSX } from 'react'

import Script from 'next/script'

import { environment } from '@/environment'

/**
 * Cloudflare Web Analytics Integration
 *
 * Privacy-first analytics that tracks Core Web Vitals without cookies or user tracking.
 * Only loads in production environments.
 *
 * @see https://www.cloudflare.com/web-analytics/
 */
export const CloudflareWebAnalytics: FC = (): JSX.Element | null => {
  const token: string | undefined =
    environment.NEXT_PUBLIC_CLOUDFLARE_WEB_ANALYTICS_TOKEN
  const shouldRenderAnalytics: boolean =
    environment.NODE_ENV === 'production' &&
    token !== undefined &&
    token.length > 0

  return shouldRenderAnalytics ? (
    <Script
      data-cf-beacon={JSON.stringify({ send: '/cf/rum/beacon', token })}
      defer={true}
      src="/cf/rum/script.js"
      // Use afterInteractive to avoid blocking the page load
      strategy="afterInteractive"
    />
  ) : null
}
