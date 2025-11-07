'use server'

import { type JSX } from 'react'

import { getTranslations } from 'next-intl/server'

import { Link } from '@/i18n/routing'
import type { AsyncPageFC } from '@/types/fc'
import type { LocalePageProperties, Translations } from '@/types/i18n'

type LegalFooterProperties = LocalePageProperties

export const LegalFooter: AsyncPageFC<LegalFooterProperties> = async ({
  locale,
}: LegalFooterProperties): Promise<JSX.Element> => {
  const translations: Translations<''> = await getTranslations({ locale })

  return (
    <footer className="mt-8 text-center">
      <nav aria-label="Legal navigation" className="flex justify-center gap-4">
        <Link
          className="text-muted-foreground hover:text-primary text-sm transition-colors hover:underline"
          href="/imprint"
        >
          {translations('imprint.title')}
        </Link>
        <Link
          className="text-muted-foreground hover:text-primary text-sm transition-colors hover:underline"
          href="/privacy"
        >
          {translations('privacy.title')}
        </Link>
      </nav>
    </footer>
  )
}
