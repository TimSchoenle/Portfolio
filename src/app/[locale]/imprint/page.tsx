import { getTranslations } from 'next-intl/server'
import type { Metadata } from 'next'
import { siteConfig } from '@/lib/config'
import { LegalPageLayout } from '@/components/legal-page-layout'

export async function generateMetadata(): Promise<Metadata> {
  const t = await getTranslations('imprint')

  return {
    title: t('title'),
    description: t('description'),
    robots: {
      index: true,
      follow: true,
    },
  }
}

export default async function ImprintPage() {
  const t = await getTranslations('imprint')
  const tContact = await getTranslations('contact')

  return (
    <LegalPageLayout title={t('title')}>
      <div>
        <h2 className="mb-2 text-xl font-semibold">{t('infoTitle')}</h2>
        <p className="text-muted-foreground">
          Tim
          <br />
          Germany
        </p>
      </div>

      <div>
        <h2 className="mb-2 text-xl font-semibold">{t('contactTitle')}</h2>
        <p className="text-muted-foreground">
          {tContact('email')}:{' '}
          <a
            href={`mailto:${siteConfig.email}`}
            className="text-primary hover:underline"
          >
            {siteConfig.email}
          </a>
        </p>
      </div>

      <div>
        <h2 className="mb-2 text-xl font-semibold">{t('responsibleTitle')}</h2>
        <p className="text-muted-foreground">
          Tim
          <br />
          Germany
        </p>
      </div>

      <div>
        <h2 className="mb-2 text-xl font-semibold">
          {t('liabilityContentTitle')}
        </h2>
        <p className="text-muted-foreground text-sm leading-relaxed">
          {t('liabilityContent')}
        </p>
      </div>

      <div>
        <h2 className="mb-2 text-xl font-semibold">
          {t('liabilityLinksTitle')}
        </h2>
        <p className="text-muted-foreground text-sm leading-relaxed">
          {t('liabilityLinks')}
        </p>
      </div>

      <div>
        <h2 className="mb-2 text-xl font-semibold">{t('copyrightTitle')}</h2>
        <p className="text-muted-foreground text-sm leading-relaxed">
          {t('copyright')}
        </p>
      </div>
    </LegalPageLayout>
  )
}
