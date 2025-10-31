import 'server-only'
import type { Locale } from './i18n-config'
import type { Dictionary } from './dictionary'

const dictionaries = {
  en: () => import('@/messages/en.json').then((module) => module.default as Dictionary),
  de: () => import('@/messages/de.json').then((module) => module.default as Dictionary),
} satisfies Record<Locale, () => Promise<Dictionary>>

export const getDictionary = async (locale: Locale): Promise<Dictionary> => {
  if (!(locale in dictionaries)) {
    console.error(`Invalid locale: ${locale}, falling back to 'en'`)
    return dictionaries.en()
  }

  return dictionaries[locale]()
}
