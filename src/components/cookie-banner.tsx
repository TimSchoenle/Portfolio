'use client'

import { X } from 'lucide-react'
import { useTranslations } from 'next-intl'
import { useState, useEffect } from 'react'

import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'

export const CookieBanner = () => {
  const t = useTranslations('cookies')

  const [showBanner, setShowBanner] = useState(false)
  const [showCustomize, setShowCustomize] = useState(false)
  const [analytics, setAnalytics] = useState(false)

  useEffect(() => {
    const consent = localStorage.getItem('cookie-consent')
    if (!consent) {
      setShowBanner(true)
    }
  }, [])

  const handleAcceptAll = () => {
    localStorage.setItem(
      'cookie-consent',
      JSON.stringify({ essential: true, analytics: true })
    )
    setShowBanner(false)
  }

  const handleRejectAll = () => {
    localStorage.setItem(
      'cookie-consent',
      JSON.stringify({ essential: true, analytics: false })
    )
    setShowBanner(false)
  }

  const handleSavePreferences = () => {
    localStorage.setItem(
      'cookie-consent',
      JSON.stringify({ essential: true, analytics })
    )
    setShowBanner(false)
  }

  if (!showBanner) {return null}

  return (
    <div className="pointer-events-none fixed inset-0 z-50 flex items-end justify-center p-4">
      <Card className="bg-background/95 pointer-events-auto w-full max-w-2xl border-2 p-6 shadow-2xl backdrop-blur-sm">
        <div className="mb-4 flex items-start justify-between">
          <h3 className="text-xl font-semibold">{t('title')}</h3>
          <Button
            className="h-8 w-8"
            size="icon"
            variant="ghost"
            onClick={handleRejectAll}
          >
            <X className="h-4 w-4" />
          </Button>
        </div>

        {!showCustomize ? (
          <>
            <p className="text-muted-foreground mb-6 text-sm">
              {t('description')}
            </p>
            <div className="flex flex-col gap-3 sm:flex-row">
              <Button className="flex-1" onClick={handleAcceptAll}>
                {t('acceptAll')}
              </Button>
              <Button
                className="flex-1 bg-transparent"
                variant="outline"
                onClick={handleRejectAll}
              >
                {t('rejectAll')}
              </Button>
              <Button
                className="flex-1"
                variant="secondary"
                onClick={() => setShowCustomize(true)}
              >
                {t('customize')}
              </Button>
            </div>
          </>
        ) : (
          <>
            <div className="mb-6 space-y-4">
              <div className="bg-muted/50 flex items-start justify-between rounded-lg border p-4">
                <div className="flex-1">
                  <h4 className="mb-1 font-medium">{t('essential')}</h4>
                  <p className="text-muted-foreground text-sm">
                    {t('essentialDesc')}
                  </p>
                </div>
                <div className="text-muted-foreground ml-4 text-sm font-medium">
                  Required
                </div>
              </div>

              <div className="flex items-start justify-between rounded-lg border p-4">
                <div className="flex-1">
                  <h4 className="mb-1 font-medium">{t('analytics')}</h4>
                  <p className="text-muted-foreground text-sm">
                    {t('analyticsDesc')}
                  </p>
                </div>
                <label className="relative ml-4 inline-flex cursor-pointer items-center">
                  <input
                    checked={analytics}
                    className="peer sr-only"
                    type="checkbox"
                    onChange={(e) => setAnalytics(e.target.checked)}
                  />
                  <div className="bg-muted peer-focus:ring-primary/20 peer peer-checked:bg-primary h-6 w-11 rounded-full peer-focus:ring-4 peer-focus:outline-none after:absolute after:top-[2px] after:left-[2px] after:h-5 after:w-5 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-checked:after:translate-x-full peer-checked:after:border-white" />
                </label>
              </div>
            </div>

            <div className="flex flex-col gap-3 sm:flex-row">
              <Button className="flex-1" onClick={handleSavePreferences}>
                {t('save')}
              </Button>
              <Button
                className="flex-1"
                variant="outline"
                onClick={() => setShowCustomize(false)}
              >
                Back
              </Button>
            </div>
          </>
        )}
      </Card>
    </div>
  )
}
