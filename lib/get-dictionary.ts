import 'server-only'
import type { Locale } from './i18n-config'

const dictionaries = {
  en: () => import('@/messages/en.json').then((module) => module.default),
  de: () => import('@/messages/de.json').then((module) => module.default),
}

export const getDictionary = async (locale: Locale) => {
  if (!(locale in dictionaries)) {
    console.error(`Invalid locale: ${locale}, falling back to 'en'`)
    return dictionaries.en()
  }

  return dictionaries[locale]()
}
