import { type Locale } from 'next-intl'

import { getRequestConfig, type GetRequestConfigParams } from 'next-intl/server'

import { routing } from './routing'

import type en from '../../messages/en.json'

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
    parameters: GetRequestConfigParams
  ): Promise<{ locale: Locale; messages: Messages }> => {
    const requested: string | undefined = await parameters.requestLocale
    const locale: Locale = isSupportedLocale(requested)
      ? requested
      : routing.defaultLocale

    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    const module_: { readonly default: Messages } = await import(
      `../../messages/${locale}.json`
    )
    const messages: Messages = module_.default

    return { locale, messages }
  }
)
