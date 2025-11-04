'use server'

import { ArrowLeft } from 'lucide-react'
import { type Locale } from 'next-intl'
import { getTranslations } from 'next-intl/server'

import { Button } from '@/components/ui/button'
import { Link } from '@/i18n/routing'

export const BackToHome = ({ locale }: { locale: Locale }) => {
  const t = await getTranslations({ locale, namespace: 'imprint' })

  return (
    <Link href="/">
      <Button className="mb-8" variant="ghost">
        <ArrowLeft className="mr-2 h-4 w-4" />
        {t('backHome')}
      </Button>
    </Link>
  )
}
