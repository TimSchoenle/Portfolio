import { createSerwistRoute } from '@serwist/turbopack'

import { environment } from '@/environment'

const revision: string = environment.NEXT_PUBLIC_REVISION ?? crypto.randomUUID()

// eslint-disable-next-line @typescript-eslint/typedef,perfectionist/sort-objects
export const { dynamic, dynamicParams, revalidate, generateStaticParams, GET } =
  createSerwistRoute({
    additionalPrecacheEntries: [
      { revision, url: '/en' },
      { revision, url: '/de' },
      { revision, url: '/en/imprint' },
      { revision, url: '/de/imprint' },
      { revision, url: '/en/privacy' },
      { revision, url: '/de/privacy' },
    ],
    nextConfig: {
      basePath: '/',
    },
    swSrc: 'src/app/sw.ts',
  })
