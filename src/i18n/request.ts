import { type Locale } from 'next-intl'
import { getRequestConfig, type GetRequestConfigParams } from 'next-intl/server'

import type en from '../../messages/en.json'

import { routing } from './routing'

// Infer the messages schema from a known file (requires "resolveJsonModule": true)
type Messages = typeof en

const isSupportedLocale: (value: unknown) => value is Locale = (
  value: unknown
): value is Locale => {
  return (
    typeof value === 'string' &&
    (routing.locales as readonly string[]).includes(value)
  )
}

export default getRequestConfig(
  async (
    params: GetRequestConfigParams
  ): Promise<{ locale: Locale; messages: Messages }> => {
    const requested: string | undefined = await params.requestLocale
    const locale: Locale = isSupportedLocale(requested)
      ? requested
      : routing.defaultLocale

    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    const mod: { readonly default: Messages } = await import(
      `../../messages/${locale}.json`
    )
    const messages: Messages = mod.default

    return { locale, messages }
  }
)
