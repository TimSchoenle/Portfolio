import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { ArrowLeft } from 'lucide-react'
import Link from 'next/link'
import { getDictionary } from '@/lib/get-dictionary'
import type { Locale } from '@/lib/i18n-config'
import type { Metadata } from 'next'

export async function generateMetadata({
  params,
}: {
  params: { locale: Locale }
}): Promise<Metadata> {
  const dict = await getDictionary(params.locale)

  return {
    title: dict.imprint.title,
    description:
      params.locale === 'de'
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
  params: { locale: Locale }
}) {
  const dict = await getDictionary(params.locale)

  return (
    <main className="bg-background min-h-screen px-4 py-12">
      <div className="mx-auto max-w-3xl">
        <Link href={`/${params.locale}`}>
          <Button variant="ghost" className="mb-8">
            <ArrowLeft className="mr-2 h-4 w-4" />
            {dict.imprint.backHome}
          </Button>
        </Link>

        <Card>
          <CardHeader>
            <CardTitle className="text-3xl">{dict.imprint.title}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-6">
            <div>
              <h2 className="mb-2 text-xl font-semibold">
                {dict.imprint.infoTitle}
              </h2>
              <p className="text-muted-foreground">
                Tim
                <br />
                Germany
              </p>
            </div>

            <div>
              <h2 className="mb-2 text-xl font-semibold">
                {dict.imprint.contactTitle}
              </h2>
              <p className="text-muted-foreground">
                {dict.contact.email}:{' '}
                <a
                  href="mailto:contact@timmi6790.de"
                  className="text-primary hover:underline"
                >
                  contact@timmi6790.de
                </a>
              </p>
            </div>

            <div>
              <h2 className="mb-2 text-xl font-semibold">
                {dict.imprint.responsibleTitle}
              </h2>
              <p className="text-muted-foreground">
                Tim
                <br />
                Germany
              </p>
            </div>

            <div>
              <h2 className="mb-2 text-xl font-semibold">
                {dict.imprint.liabilityContentTitle}
              </h2>
              <p className="text-muted-foreground text-sm leading-relaxed">
                {dict.imprint.liabilityContent}
              </p>
            </div>

            <div>
              <h2 className="mb-2 text-xl font-semibold">
                {dict.imprint.liabilityLinksTitle}
              </h2>
              <p className="text-muted-foreground text-sm leading-relaxed">
                {dict.imprint.liabilityLinks}
              </p>
            </div>

            <div>
              <h2 className="mb-2 text-xl font-semibold">
                {dict.imprint.copyrightTitle}
              </h2>
              <p className="text-muted-foreground text-sm leading-relaxed">
                {dict.imprint.copyright}
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    </main>
  )
}
