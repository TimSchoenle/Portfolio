'use client'

import { usePathname, useRouter } from 'next/navigation'
import { Button } from '@/components/ui/button'
import { Globe } from 'lucide-react'
import { useLocale } from 'next-intl'

export function LanguageSwitcher() {
  const locale = useLocale()

  const pathname = usePathname()
  const router = useRouter()

  const switchLanguage = () => {
    const newLocale = locale === 'en' ? 'de' : 'en'
    const newPath = pathname.replace(`/${locale}`, `/${newLocale}`)
    router.push(newPath)
  }

  return (
    <Button
      variant="outline"
      size="sm"
      onClick={switchLanguage}
      className="fixed top-4 right-4 z-50 bg-transparent"
    >
      <Globe className="mr-2 h-4 w-4" />
      {locale === 'en' ? 'DE' : 'EN'}
    </Button>
  )
}
