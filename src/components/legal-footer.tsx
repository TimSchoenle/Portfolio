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
          className="text-sm text-muted-foreground transition-colors hover:text-primary hover:underline"
          href="/imprint"
          prefetch={false}
        >
          {translations('imprint.title')}
        </Link>
        <Link
          className="text-sm text-muted-foreground transition-colors hover:text-primary hover:underline"
          href="/privacy"
          prefetch={false}
        >
          {translations('privacy.title')}
        </Link>
      </nav>
    </footer>
  )
}
