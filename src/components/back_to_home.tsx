'use server'

import { ArrowLeft } from 'lucide-react'
import { getTranslations } from 'next-intl/server'
import { type JSX } from 'react'

import { Button } from '@/components/ui/button'
import { Link } from '@/i18n/routing'
import type { AsyncPageFC } from '@/types/fc'
import type { LocalePageProps, Translations } from '@/types/i18n'

type BackToHomeProps = LocalePageProps

export const BackToHome: AsyncPageFC<BackToHomeProps> = async ({
  locale,
}: BackToHomeProps): Promise<JSX.Element> => {
  const t: Translations<'imprint'> = await getTranslations({
    locale,
    namespace: 'imprint',
  })

  return (
    <Link href="/">
      <Button className="mb-8" variant="ghost">
        <ArrowLeft className="mr-2 h-4 w-4" />
        {t('backHome')}
      </Button>
    </Link>
  )
}
