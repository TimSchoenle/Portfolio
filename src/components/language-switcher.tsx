'use client'

import { Globe } from 'lucide-react'
import { type Route } from 'next'
import { usePathname, useRouter } from 'next/navigation'
import { useLocale } from 'next-intl'

import { Button } from '@/components/ui/button'
import { getPathname } from '@/i18n/routing'


export const LanguageSwitcher = () => {
  const locale = useLocale()

  const pathname = usePathname()
  const router = useRouter()

  const switchLanguage = () => {
    const newLocale = locale === 'en' ? 'de' : 'en'
    router.push(
      getPathname({
        href: pathname,
        locale: newLocale,
      }) as Route
    )
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
