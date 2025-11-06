import { type Locale, type useTranslations } from 'next-intl'

export type Translations = ReturnType<typeof useTranslations>

export interface UnparsedLocalePageProps {
  // This needs to stay "local" to match the type of the `locale` prop
  readonly locale: string
}

export interface LocalePageProps {
  readonly locale: Locale
}
