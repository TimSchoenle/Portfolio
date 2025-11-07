import { type JSX } from 'react'

import { BackToHome } from '@/components/back_to_home'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import type { FCWithChildren, WithChildren } from '@/types/fc'
import type { LocalePageProps } from '@/types/i18n'

interface LegalPageLayoutProps extends WithChildren, LocalePageProps {
  readonly title: string
}

export const LegalPageLayout: FCWithChildren<LegalPageLayoutProps> = ({
  locale,
  title,
  children,
}: LegalPageLayoutProps): JSX.Element => {
  return (
    <main className="bg-background min-h-screen px-4 py-12">
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
