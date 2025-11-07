import { createNavigation } from 'next-intl/navigation'
import { defineRouting } from 'next-intl/routing'

// eslint-disable-next-line @typescript-eslint/typedef
export const routing = defineRouting({
  // Used when no locale matches
  defaultLocale: 'en',

  // The locale prefix strategy
  localePrefix: {
    mode: 'always',
  },

  // A list of all locales that are supported
  locales: ['en', 'de'],
})

// Lightweight wrappers around Next.js' navigation APIs
// that will consider the routing configuration
// eslint-disable-next-line @typescript-eslint/typedef
export const { getPathname, Link, redirect, usePathname, useRouter } =
  createNavigation(routing)
