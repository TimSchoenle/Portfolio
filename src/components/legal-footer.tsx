'use server'

import { type Locale } from 'next-intl'
import { getTranslations } from 'next-intl/server'

import { Link } from '@/i18n/routing'

export const LegalFooter = ({ locale }: { locale: Locale }) => {
  const t = await getTranslations({ locale })

  return (
    <footer className="mt-8 text-center">
      <nav aria-label="Legal navigation" className="flex justify-center gap-4">
        <Link
          className="text-muted-foreground hover:text-primary text-sm transition-colors hover:underline"
          href="/imprint"
        >
          {t('imprint.title')}
        </Link>
        <Link
          className="text-muted-foreground hover:text-primary text-sm transition-colors hover:underline"
          href="/privacy"
        >
          {t('privacy.title')}
        </Link>
      </nav>
    </footer>
  )
}
