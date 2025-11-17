'use client'

import type { JSX } from 'react'

import { useTranslations } from 'next-intl'

import { AlertTriangle } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import type { FCNullable, FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'

// Hoisted utility to satisfy unicorn/consistent-function-scoping
function goHome(): void {
  window.location.assign('/')
}

interface ErrorHeaderProperties {
  readonly title: string
}

const ErrorHeader: FCStrict<ErrorHeaderProperties> = ({
  title,
}: ErrorHeaderProperties): JSX.Element => {
  return (
    <CardHeader>
      <div className="flex items-center gap-2">
        <AlertTriangle className="h-6 w-6 text-destructive" />
        <CardTitle>{title}</CardTitle>
      </div>
    </CardHeader>
  )
}

interface ErrorInfoProperties {
  readonly digest?: string | undefined
  readonly label: string
}

const ErrorInfo: FCNullable<ErrorInfoProperties> = ({
  digest,
  label,
}: ErrorInfoProperties): JSX.Element | null => {
  if (typeof digest !== 'string' || digest.length === 0) {
    return null
  }

  return (
    <p className="text-xs text-muted-foreground">
      <span>{label}</span> {digest}
    </p>
  )
}

interface ErrorActionsLabels {
  readonly goHome: string
  readonly tryAgain: string
}

interface ErrorActionsProperties {
  readonly labels: ErrorActionsLabels
  readonly reset: () => void
}

const ErrorActions: FCStrict<ErrorActionsProperties> = ({
  labels,
  reset,
}: Readonly<ErrorActionsProperties>): JSX.Element => {
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

interface ErrorPageProperties {
  readonly error: Readonly<Error> & { readonly digest?: string }
  readonly reset: () => void
}

const ErrorPage: FCStrict<ErrorPageProperties> = ({
  error,
  reset,
}: ErrorPageProperties): JSX.Element => {
  const translations: Translations<'error'> = useTranslations('error')

  return (
    <div className="flex min-h-screen items-center justify-center bg-background p-4">
      <Card className="w-full max-w-md">
        <ErrorHeader title={translations('title')} />
        <CardContent className="space-y-4">
          <p className="text-sm text-muted-foreground">
            {translations('description')}
          </p>
          <ErrorInfo
            digest={error.digest}
            label={translations('errorIdLabel')}
          />
          <ErrorActions
            labels={{
              goHome: translations('goHome'),
              tryAgain: translations('tryAgain'),
            }}
            reset={reset}
          />
        </CardContent>
      </Card>
    </div>
  )
}

export default ErrorPage
