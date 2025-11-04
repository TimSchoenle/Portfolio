'use server'

import { getTranslations } from 'next-intl/server'
import { type Locale } from 'next-intl'
import { Link } from '@/i18n/routing'

export async function LegalFooter({ locale }: { locale: Locale }) {
  const t = await getTranslations({ locale })

  return (
    <footer className="mt-8 text-center">
      <nav aria-label="Legal navigation" className="flex justify-center gap-4">
        <Link
          href="/imprint"
          className="text-muted-foreground hover:text-primary text-sm transition-colors hover:underline"
        >
          {t('imprint.title')}
        </Link>
        <Link
          href="/privacy"
          className="text-muted-foreground hover:text-primary text-sm transition-colors hover:underline"
        >
          {t('privacy.title')}
        </Link>
      </nav>
    </footer>
  )
}
