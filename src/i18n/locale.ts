// src/lib/locale.ts
import { notFound } from 'next/navigation'
import { hasLocale, type Locale } from 'next-intl'

import { routing } from '@/i18n/routing'
import type { UnparsedLocalePageProps } from '@/types/i18n'

/**
 * Assertion-style validator. Refines a string to Locale in-place.
 */
export const assertLocale: (value: string) => asserts value is Locale = (
  value: string
): void => {
  if (!hasLocale(routing.locales as readonly Locale[], value as Locale)) {
    notFound()
  }
}

/**
 * Returns a validated Locale (throws notFound on invalid).
 */
export const ensureLocale: (value: string) => Locale = (
  value: string
): Locale => {
  if (!hasLocale(routing.locales as readonly Locale[], value as Locale)) {
    notFound()
  }
  return value as Locale
}

/**
 * Sugar for Next.js page/layout params → Locale.
 * Accepts a Promise to keep call-sites terse in async components.
 */
export const ensureLocaleFromParams: (
  paramsPromise: Promise<UnparsedLocalePageProps>
) => Promise<Locale> = async (
  paramsPromise: Promise<UnparsedLocalePageProps>
): Promise<Locale> => {
  const raw: UnparsedLocalePageProps = await paramsPromise
  return ensureLocale(raw.locale)
}

export const maybeLocale: (value: string) => Locale | null = (
  value: string
): Locale | null => {
  return hasLocale(routing.locales as readonly Locale[], value as Locale)
    ? (value as Locale)
    : null
}

export const maybeLocaleFromParams: (
  paramsPromise: Promise<UnparsedLocalePageProps>
) => Promise<Locale | null> = async (
  paramsPromise: Promise<UnparsedLocalePageProps>
): Promise<Locale | null> => {
  const raw: UnparsedLocalePageProps = await paramsPromise
  return maybeLocale(raw.locale)
}
