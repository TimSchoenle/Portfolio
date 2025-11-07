import { createNavigation } from 'next-intl/navigation'
import { defineRouting } from 'next-intl/routing'

// eslint-disable-next-line @typescript-eslint/typedef
export const routing = defineRouting({
  // A list of all locales that are supported
  locales: ['en', 'de'],

  // Used when no locale matches
  defaultLocale: 'en',

  // The locale prefix strategy
  localePrefix: {
    mode: 'always',
  },
})

// Lightweight wrappers around Next.js' navigation APIs
// that will consider the routing configuration
// eslint-disable-next-line @typescript-eslint/typedef
export const { Link, redirect, usePathname, useRouter, getPathname } =
  createNavigation(routing)
