'use client'

import React, {
  type Dispatch,
  type JSX,
  type SetStateAction,
  useCallback,
  useEffect,
  useState,
} from 'react'

import { useTranslations } from 'next-intl'

import { X } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import type { FCNullable, FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'

/* ───────────────────────────── Types & consts ───────────────────────────── */

interface CookieConsent {
  readonly analytics: boolean
  readonly essential: true
}

const CONSENT_KEY: string = 'cookie-consent'

/* ───────────────────────────── Helpers (pure) ───────────────────────────── */

const loadConsent: () => CookieConsent | null = (): CookieConsent | null => {
  const raw: string | null =
    typeof window !== 'undefined'
      ? window.localStorage.getItem(CONSENT_KEY)
      : null
  if (raw === null) {
    return null
  }

  try {
    const parsed: unknown = JSON.parse(raw)
    // Narrow carefully
    if (
      typeof parsed === 'object' &&
      parsed !== null &&
      (parsed as { essential?: unknown }).essential === true &&
      typeof (parsed as { analytics?: unknown }).analytics === 'boolean'
    ) {
      return {
        analytics: (parsed as { analytics: boolean }).analytics,
        essential: true,
      }
    }
  } catch {
    // ignore parse errors and treat as no consent
  }
  return null
}

const saveConsent: (consent: CookieConsent) => void = (
  consent: CookieConsent
): void => {
  window.localStorage.setItem(CONSENT_KEY, JSON.stringify(consent))
}

/* ───────────────────────────── Subcomponents ───────────────────────────── */

interface HeaderProperties {
  readonly onClose: () => void
  readonly title: string
}
const Header: FCStrict<HeaderProperties> = ({
  onClose,
  title,
}: HeaderProperties): JSX.Element => (
  <div className="mb-4 flex items-start justify-between">
    <h3 className="text-xl font-semibold">{title}</h3>
    <Button className="h-8 w-8" size="icon" variant="ghost" onClick={onClose}>
      <X className="h-4 w-4" />
    </Button>
  </div>
)

interface SummaryActionsProperties {
  readonly onAcceptAll: () => void
  readonly onCustomize: () => void
  readonly onRejectAll: () => void
  readonly translations: Translations<'cookies'>
}
const SummaryActions: FCStrict<SummaryActionsProperties> = ({
  onAcceptAll,
  onCustomize,
  onRejectAll,
  translations,
}: SummaryActionsProperties): JSX.Element => (
  <div className="flex flex-col gap-3 sm:flex-row">
    <Button className="flex-1" onClick={onAcceptAll}>
      {translations('acceptAll')}
    </Button>
    <Button
      className="flex-1 bg-transparent"
      variant="outline"
      onClick={onRejectAll}
    >
      {translations('rejectAll')}
    </Button>
    <Button className="flex-1" variant="secondary" onClick={onCustomize}>
      {translations('customize')}
    </Button>
  </div>
)

interface EssentialRowProperties {
  readonly requiredLabel: string
  readonly translations: Translations<'cookies'>
}
const EssentialRow: FCStrict<EssentialRowProperties> = ({
  requiredLabel,
  translations,
}: EssentialRowProperties): JSX.Element => (
  <div className="bg-muted/50 flex items-start justify-between rounded-lg border p-4">
    <div className="flex-1">
      <h4 className="mb-1 font-medium">{translations('essential')}</h4>
      <p className="text-muted-foreground text-sm">
        {translations('essentialDesc')}
      </p>
    </div>
    <div className="text-muted-foreground ml-4 text-sm font-medium">
      {requiredLabel}
    </div>
  </div>
)

interface AnalyticsRowProperties {
  readonly checked: boolean
  readonly onChange: (next: boolean) => void
  readonly translations: Translations<'cookies'>
}
const AnalyticsRow: FCStrict<AnalyticsRowProperties> = ({
  checked,
  onChange,
  translations,
}: AnalyticsRowProperties): JSX.Element => (
  <div className="flex items-start justify-between rounded-lg border p-4">
    <div className="flex-1">
      <h4 className="mb-1 font-medium">{translations('analytics')}</h4>
      <p className="text-muted-foreground text-sm">
        {translations('analyticsDesc')}
      </p>
    </div>

    <label className="relative ml-4 inline-flex cursor-pointer items-center">
      <input
        checked={checked}
        className="peer sr-only"
        type="checkbox"
        onChange={(error: React.ChangeEvent<HTMLInputElement>): void => {
          onChange(error.target.checked)
        }}
      />
      <div className="bg-muted peer-focus:ring-primary/20 peer peer-checked:bg-primary h-6 w-11 rounded-full peer-focus:ring-4 peer-focus:outline-none after:absolute after:top-[2px] after:left-[2px] after:h-5 after:w-5 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-checked:after:translate-x-full peer-checked:after:border-white" />
    </label>
  </div>
)

interface CustomizeActionsProperties {
  readonly backLabel: string
  readonly onBack: () => void
  readonly onSave: () => void
  readonly translations: Translations<'cookies'>
}
const CustomizeActions: FCStrict<CustomizeActionsProperties> = ({
  backLabel,
  onBack,
  onSave,
  translations,
}: CustomizeActionsProperties): JSX.Element => (
  <div className="flex flex-col gap-3 sm:flex-row">
    <Button className="flex-1" onClick={onSave}>
      {translations('save')}
    </Button>
    <Button className="flex-1" variant="outline" onClick={onBack}>
      {backLabel}
    </Button>
  </div>
)

/* ───────────────────────────── Main component ───────────────────────────── */

// eslint-disable-next-line max-lines-per-function
export const CookieBanner: FCNullable = (): JSX.Element | null => {
  const translations: Translations<'cookies'> = useTranslations('cookies')

  const [showBanner, setShowBanner]: [
    boolean,
    Dispatch<SetStateAction<boolean>>,
  ] = useState<boolean>(false)
  const [customize, setCustomize]: [
    boolean,
    Dispatch<SetStateAction<boolean>>,
  ] = useState<boolean>(false)
  const [analytics, setAnalytics]: [
    boolean,
    Dispatch<SetStateAction<boolean>>,
  ] = useState<boolean>(false)

  // bootstrap from localStorage
  useEffect((): void => {
    const existing: CookieConsent | null = loadConsent()
    if (existing === null) {
      setShowBanner(true)
      return
    }
    setAnalytics(existing.analytics)
  }, [])

  const handleAcceptAll: () => void = useCallback((): void => {
    saveConsent({ analytics: true, essential: true })
    setShowBanner(false)
  }, [])

  const handleRejectAll: () => void = useCallback((): void => {
    saveConsent({ analytics: false, essential: true })
    setShowBanner(false)
  }, [])

  const handleSavePreferences: () => void = useCallback((): void => {
    saveConsent({ analytics, essential: true })
    setShowBanner(false)
  }, [analytics])

  if (!showBanner) {
    return null
  }

  const requiredLabel: string = translations('required')
  const backLabel: string = translations('back')

  return (
    <div className="pointer-events-none fixed inset-0 z-50 flex items-end justify-center p-4">
      <Card className="bg-background/95 pointer-events-auto w-full max-w-2xl border-2 p-6 shadow-2xl backdrop-blur-sm">
        <Header title={translations('title')} onClose={handleRejectAll} />

        {!customize ? (
          <>
            <p className="text-muted-foreground mb-6 text-sm">
              {translations('description')}
            </p>
            <SummaryActions
              translations={translations}
              onAcceptAll={handleAcceptAll}
              onCustomize={(): void => {
                setCustomize(true)
              }}
              onRejectAll={handleRejectAll}
            />
          </>
        ) : (
          <>
            <div className="mb-6 space-y-4">
              <EssentialRow
                requiredLabel={requiredLabel}
                translations={translations}
              />
              <AnalyticsRow
                checked={analytics}
                translations={translations}
                onChange={(next: boolean): void => {
                  setAnalytics(next)
                }}
              />
            </div>
            <CustomizeActions
              backLabel={backLabel}
              translations={translations}
              onBack={(): void => {
                setCustomize(false)
              }}
              onSave={handleSavePreferences}
            />
          </>
        )}
      </Card>
    </div>
  )
}
