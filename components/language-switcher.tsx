'use client'

import { usePathname, useRouter } from 'next/navigation'
import { Button } from '@/components/ui/button'
import { Globe } from 'lucide-react'
import type { Locale } from '@/lib/i18n-config'

export function LanguageSwitcher({ currentLocale }: { currentLocale: Locale }) {
  const pathname = usePathname()
  const router = useRouter()

  const switchLanguage = () => {
    const newLocale = currentLocale === 'en' ? 'de' : 'en'
    const newPath = pathname.replace(`/${currentLocale}`, `/${newLocale}`)
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
      {currentLocale === 'en' ? 'DE' : 'EN'}
    </Button>
  )
}
