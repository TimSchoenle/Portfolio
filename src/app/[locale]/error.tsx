'use client'

import { AlertTriangle } from 'lucide-react'
import { useTranslations } from 'next-intl'
import type { JSX } from 'react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import type { FCNullable, FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'

// Hoisted utility to satisfy unicorn/consistent-function-scoping
function goHome(): void {
  window.location.assign('/')
}

interface ErrorHeaderProps {
  readonly title: string
}

const ErrorHeader: FCStrict<ErrorHeaderProps> = ({
  title,
}: ErrorHeaderProps): JSX.Element => {
  return (
    <CardHeader>
      <div className="flex items-center gap-2">
        <AlertTriangle className="text-destructive h-6 w-6" />
        <CardTitle>{title}</CardTitle>
      </div>
    </CardHeader>
  )
}

interface ErrorInfoProps {
  readonly digest?: string | undefined
  readonly label: string
}

const ErrorInfo: FCNullable<ErrorInfoProps> = ({
  digest,
  label,
}: ErrorInfoProps): JSX.Element | null => {
  if (typeof digest !== 'string' || digest.length === 0) {
    return null
  }

  return (
    <p className="text-muted-foreground text-xs">
      <span>{label}</span> {digest}
    </p>
  )
}

interface ErrorActionsLabels {
  readonly goHome: string
  readonly tryAgain: string
}

interface ErrorActionsProps {
  readonly labels: ErrorActionsLabels
  readonly reset: () => void
}

const ErrorActions: FCStrict<ErrorActionsProps> = ({
  labels,
  reset,
}: Readonly<ErrorActionsProps>): JSX.Element => {
  return (
    <div className="flex gap-2">
      <Button className="w-full" onClick={reset}>
        {labels.tryAgain}
      </Button>
      <Button className="w-full" variant="outline" onClick={goHome}>
        {labels.goHome}
      </Button>
    </div>
  )
}

interface ErrorPageProps {
  readonly error: Readonly<Error> & { readonly digest?: string }
  readonly reset: () => void
}

const ErrorPage: FCStrict<ErrorPageProps> = ({
  error,
  reset,
}: ErrorPageProps): JSX.Element => {
  const t: Translations<'error'> = useTranslations('error')

  return (
    <div className="bg-background flex min-h-screen items-center justify-center p-4">
      <Card className="w-full max-w-md">
        <ErrorHeader title={t('title')} />
        <CardContent className="space-y-4">
          <p className="text-muted-foreground text-sm">{t('description')}</p>
          <ErrorInfo digest={error.digest} label={t('errorIdLabel')} />
          <ErrorActions
            labels={{ goHome: t('goHome'), tryAgain: t('tryAgain') }}
            reset={reset}
          />
        </CardContent>
      </Card>
    </div>
  )
}

export default ErrorPage
