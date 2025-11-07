import { hasLocale, type Locale } from 'next-intl'

import { notFound } from 'next/navigation'

import { routing } from '@/i18n/routing'
import type { UnparsedLocalePageProperties } from '@/types/i18n'

/**
 * Assertion-style validator. Refines a string to Locale in-place.
 */
export const assertLocale: (value: string) => asserts value is Locale = (
  value: string
): void => {
  if (!hasLocale(routing.locales as readonly Locale[], value)) {
    notFound()
  }
}

/**
 * Returns a validated Locale (throws notFound on invalid).
 */
export const ensureLocale: (value: string) => Locale = (
  value: string
): Locale => {
  if (!hasLocale(routing.locales as readonly Locale[], value)) {
    notFound()
  }
  return value
}

/**
 * Sugar for Next.js page/layout params → Locale.
 * Accepts a Promise to keep call-sites terse in async components.
 */
export const ensureLocaleFromParameters: (
  parametersPromise: Promise<UnparsedLocalePageProperties>
) => Promise<Locale> = async (
  parametersPromise: Promise<UnparsedLocalePageProperties>
): Promise<Locale> => {
  const raw: UnparsedLocalePageProperties = await parametersPromise
  return ensureLocale(raw.locale)
}

export const maybeLocale: (value: string) => Locale | null = (
  value: string
): Locale | null => {
  return hasLocale(routing.locales as readonly Locale[], value) ? value : null
}

export const maybeLocaleFromParameters: (
  parametersPromise: Promise<UnparsedLocalePageProperties>
) => Promise<Locale | null> = async (
  parametersPromise: Promise<UnparsedLocalePageProperties>
): Promise<Locale | null> => {
  const raw: UnparsedLocalePageProperties = await parametersPromise
  return maybeLocale(raw.locale)
}
