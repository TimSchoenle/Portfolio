'use client'

import { type JSX, useEffect } from 'react'

import { useTranslations } from 'next-intl'

import * as Sentry from '@sentry/nextjs'
import { AlertTriangle, Home, RotateCcw } from 'lucide-react'

import { BlueprintCard } from '@/components/blueprint/blueprint-card'
import type { FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'

const ERROR_SEPARATOR: string = '::'

// Hoisted utility to satisfy unicorn/consistent-function-scoping
function goHome(): void {
  window.location.assign('/')
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
    <div className="mt-6 flex gap-4">
      <button
        className="flex items-center gap-2 border border-brand bg-brand/10 px-6 py-3 font-mono text-xs tracking-wider text-brand uppercase transition-all hover:bg-brand/20 hover:text-blueprint-text"
        onClick={reset}
      >
        <RotateCcw className="h-4 w-4" />
        {labels.tryAgain}
      </button>

      <button
        className="flex items-center gap-2 border border-brand/30 bg-transparent px-6 py-3 font-mono text-xs tracking-wider text-brand-readable uppercase transition-all hover:border-brand hover:bg-brand/5 hover:text-blueprint-text"
        onClick={goHome}
      >
        <Home className="h-4 w-4" />
        {labels.goHome}
      </button>
    </div>
  )
}

interface ErrorPageProperties {
  readonly error: Readonly<Error> & { readonly digest?: string }
  readonly reset: () => void
}

// eslint-disable-next-line max-lines-per-function
const ErrorPage: FCStrict<ErrorPageProperties> = ({
  error,
  reset,
}: ErrorPageProperties): JSX.Element => {
  const translations: Translations<'error'> = useTranslations('error')

  useEffect((): void => {
    Sentry.captureException(error)
  }, [error])

  return (
    <div className="relative flex min-h-screen w-full flex-col items-center justify-center overflow-hidden bg-blueprint-bg p-4 text-blueprint-text">
      {/* Blueprint Grid Background */}
      <div
        className="pointer-events-none absolute inset-0 opacity-[0.03]"
        style={{
          backgroundImage: `linear-gradient(#60A5FA 1px, transparent 1px), linear-gradient(90deg, #60A5FA 1px, transparent 1px)`,
          backgroundSize: '40px 40px',
        }}
      />

      <div className="relative z-10 w-full max-w-lg">
        <BlueprintCard
          className="border-destructive/30 p-8 md:p-12"
          label="SYSTEM FAILING"
          noPadding={true}
        >
          <div className="flex flex-col items-center gap-6 text-center">
            {/* Error Icon */}
            <div className="relative flex h-20 w-20 items-center justify-center">
              <div className="absolute inset-0 animate-pulse rounded-full border border-red-500/30" />
              <div className="absolute inset-2 rounded-full border border-red-500/20" />
              <AlertTriangle className="h-10 w-10 text-red-400" />
            </div>

            <div className="flex flex-col gap-2">
              <h1 className="font-mono text-2xl font-bold tracking-wider text-red-400 uppercase">
                {translations('title')}
              </h1>
              <div className="w- full my-2 h-px bg-gradient-to-r from-transparent via-red-500/30 to-transparent" />
              <p className="font-mono text-sm text-blueprint-muted">
                {translations('description')}
              </p>
            </div>

            {Boolean(error.digest) ? (
              <div className="w-full rounded border border-red-500/20 bg-red-950/20 p-3 font-mono text-[10px] tracking-wider text-red-300/70 uppercase">
                {translations('errorIdLabel')} {ERROR_SEPARATOR} {error.digest}
              </div>
            ) : null}

            <ErrorActions
              labels={{
                goHome: translations('goHome'),
                tryAgain: translations('tryAgain'),
              }}
              reset={reset}
            />
          </div>
        </BlueprintCard>
      </div>
    </div>
  )
}

export default ErrorPage
