'use server'

import { type Locale } from 'next-intl'
import type { ReactNode } from 'react'

import { BackToHome } from '@/components/back_to_home'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'


interface LegalPageLayoutProps {
  locale: Locale
  title: string
  children: ReactNode
}

export default async function LegalPageLayout({
  locale,
  title,
  children,
}: LegalPageLayoutProps) {
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
