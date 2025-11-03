import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { ArrowLeft } from 'lucide-react'
import Link from 'next/link'
import { getTranslations } from 'next-intl/server'
import type { Metadata } from 'next'
import { siteConfig } from '@/lib/config'

export async function generateMetadata({
  params,
}: {
  params: { locale: string }
}): Promise<Metadata> {
  const { locale } = params
  const t = await getTranslations('imprint')

  return {
    title: t('title'),
    description:
      locale === 'de'
        ? 'Impressum und rechtliche Informationen für Tim - Software Developer Portfolio'
        : 'Imprint and legal information for Tim - Software Developer Portfolio',
    robots: {
      index: true,
      follow: true,
    },
  }
}

export default async function ImprintPage({
  params,
}: {
  params: { locale: string }
}) {
  const { locale } = params
  const t = await getTranslations('imprint')
  const tContact = await getTranslations('contact')

  return (
    <main className="bg-background min-h-screen px-4 py-12">
      <div className="mx-auto max-w-3xl">
        <Link href={`/${locale}`}>
          <Button variant="ghost" className="mb-8">
            <ArrowLeft className="mr-2 h-4 w-4" />
            {t('backHome')}
          </Button>
        </Link>

        <Card>
          <CardHeader>
            <CardTitle className="text-3xl">{t('title')}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-6">
            <div>
              <h2 className="mb-2 text-xl font-semibold">{t('infoTitle')}</h2>
              <p className="text-muted-foreground">
                Tim
                <br />
                Germany
              </p>
            </div>

            <div>
              <h2 className="mb-2 text-xl font-semibold">
                {t('contactTitle')}
              </h2>
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
              <h2 className="mb-2 text-xl font-semibold">
                {t('responsibleTitle')}
              </h2>
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
              <h2 className="mb-2 text-xl font-semibold">
                {t('copyrightTitle')}
              </h2>
              <p className="text-muted-foreground text-sm leading-relaxed">
                {t('copyright')}
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    </main>
  )
}
