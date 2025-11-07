'use client'

import { type JSX } from 'react'

import { type Locale, useLocale } from 'next-intl'

import { Globe } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { usePathname, useRouter } from '@/i18n/routing'
import type { FCStrict } from '@/types/fc'

export const LanguageSwitcher: FCStrict = (): JSX.Element => {
  const locale: Locale = useLocale()

  const pathname: string = usePathname()
  const router: ReturnType<typeof useRouter> = useRouter()

  const switchLanguage: () => void = (): void => {
    const newLocale: Locale = locale === 'en' ? 'de' : 'en'
    router.replace(pathname, { locale: newLocale })
  }

  return (
    <Button
      className="fixed top-4 right-4 z-50 bg-transparent"
      size="sm"
      variant="outline"
      onClick={switchLanguage}
    >
      <Globe className="mr-2 h-4 w-4" />
      {locale === 'en' ? 'DE' : 'EN'}
    </Button>
  )
}
