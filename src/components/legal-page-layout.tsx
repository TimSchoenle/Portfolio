import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import BackToHome from '@/components/back_to_home'
import type { ReactNode } from 'react'

interface LegalPageLayoutProps {
  title: string
  children: ReactNode
}

export function LegalPageLayout({ title, children }: LegalPageLayoutProps) {
  return (
    <main className="bg-background min-h-screen px-4 py-12">
      <div className="mx-auto max-w-3xl">
        <BackToHome />

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
