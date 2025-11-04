'use server'

import { getTranslations, setRequestLocale } from 'next-intl/server'
import type { Metadata } from 'next'
import { siteConfig } from '@/lib/config'
import LegalPageLayout from '@/components/legal-page-layout'
import { type Locale } from 'next-intl'

export async function generateMetadata({
  params,
}: Readonly<{
  params: Promise<{ locale: Locale }>
}>): Promise<Metadata> {
  const { locale } = await params
  const t = await getTranslations({ locale, namespace: 'imprint' })

  return {
    title: t('title'),
    description: t('description'),
  }
}

export default async function ImprintPage({
  params,
}: Readonly<{
  params: Promise<{ locale: Locale }>
}>) {
  const { locale } = await params

  // Enable static rendering
  setRequestLocale(locale)

  const t = await getTranslations({ locale, namespace: 'imprint' })
  const tContact = await getTranslations({ locale, namespace: 'contact' })

  return (
    <LegalPageLayout title={t('title')} locale={locale}>
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
