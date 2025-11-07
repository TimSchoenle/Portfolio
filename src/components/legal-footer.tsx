'use server'

import { getTranslations } from 'next-intl/server'
import { type JSX } from 'react'

import { Link } from '@/i18n/routing'
import type { AsyncPageFC } from '@/types/fc'
import type { LocalePageProps, Translations } from '@/types/i18n'

type LegalFooterProps = LocalePageProps

export const LegalFooter: AsyncPageFC<LegalFooterProps> = async ({
  locale,
}: LegalFooterProps): Promise<JSX.Element> => {
  const t: Translations<''> = await getTranslations({ locale })

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
