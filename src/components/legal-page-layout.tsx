import { type JSX } from 'react'

import { BackToHome } from '@/components/back-to-home'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import type { FCWithChildren, WithChildren } from '@/types/fc'
import type { LocalePageProperties } from '@/types/i18n'

interface LegalPageLayoutProperties extends WithChildren, LocalePageProperties {
  readonly title: string
}

export const LegalPageLayout: FCWithChildren<LegalPageLayoutProperties> = ({
  children,
  locale,
  title,
}: LegalPageLayoutProperties): JSX.Element => {
  return (
    <main className="min-h-screen bg-background px-4 py-12">
      <div className="mx-auto max-w-3xl">
        <BackToHome locale={locale} />

        <Card>
          <CardHeader>
            <CardTitle className="text-3xl">{title}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-6">{children}</CardContent>
        </Card>
      </div>
    </main>
  )
}
