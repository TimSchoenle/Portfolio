import '../globals.css'

import type { JSX } from 'react'

import type { Metadata, Viewport } from 'next'
import { type Locale, type Messages, NextIntlClientProvider } from 'next-intl'

import { Inter } from 'next/font/google'
import { getMessages, setRequestLocale } from 'next-intl/server'

import DeferredClientUi from '@/app/[locale]/deferred-client-ui'
import { LanguageSwitcher } from '@/components/language-switcher'
import { LegalFooter } from '@/components/legal-footer'
import { ThemeProvider } from '@/components/theme-provider'
import { ThemeToggle } from '@/components/theme-toggle'
import {
  ensureLocaleFromParameters,
  maybeLocaleFromParameters,
} from '@/i18n/locale'
import { getPathname, routing } from '@/i18n/routing'
import { siteConfig } from '@/lib/config'
import type { UnparsedLocalePageProperties } from '@/types/i18n'
import type {
  GenerateMetadataFC,
  PageParameters,
  PageParametersWithChildren,
  RoutePageWithChildrenFC,
} from '@/types/page'

import type { NextFontWithVariable } from 'next/dist/compiled/@next/font'
import type { DeepPartial } from 'react-hook-form'

/* ---------- fonts (auto-fetched via next/font/google) ---------- */
const inter: NextFontWithVariable = Inter({
  adjustFontFallback: true,
  display: 'swap',
  subsets: ['latin'],
  variable: '--font-inter',
})

/* ---------- helpers (readonly params, no inline callbacks) ---------- */
const buildLanguages: () => Record<string, string> = (): Record<
  string,
  string
> => {
  return Object.fromEntries(
    (routing.locales as readonly Locale[]).map(
      (loc: Locale): readonly [Locale, string] => [
        loc,
        `${siteConfig.url}/${getPathname({ href: '/', locale: loc })}`,
      ]
    )
  )
}

const buildAlternateLocales: (current: Locale) => string = (
  current: Locale
): string => {
  return (routing.locales as readonly Locale[])
    .filter((loc: Locale): boolean => loc !== current)
    .join(', ')
}

/* ---------- generateMetadata ---------- */
export const generateMetadata: GenerateMetadataFC<
  UnparsedLocalePageProperties
> = async ({
  params,
}: PageParameters<UnparsedLocalePageProperties>): Promise<Metadata> => {
  const locale: Locale | null = await maybeLocaleFromParameters(params)
  if (locale === null) {
    return {}
  }

  return {
    alternates: {
      languages: buildLanguages(),
    },
    authors: [{ name: siteConfig.fullName, url: siteConfig.url }],
    creator: siteConfig.fullName,
    description: siteConfig.description,
    keywords: siteConfig.seo.keywords.join(', '),
    metadataBase: new URL(siteConfig.url),
    openGraph: {
      alternateLocale: buildAlternateLocales(locale),
      description: siteConfig.description,
      images: [
        {
          alt: siteConfig.title,
          height: 630,
          url: '/og-image.jpg',
          width: 1200,
        },
      ],
      locale,
      siteName: siteConfig.title,
      title: siteConfig.title,
      type: 'website',
      url: `${siteConfig.url}/${locale}`,
    },
    publisher: siteConfig.fullName,
    robots: {
      follow: true,
      googleBot: {
        follow: true,
        index: true,
        'max-image-preview': 'large',
        'max-snippet': -1,
        'max-video-preview': -1,
      },
      index: true,
    },
    title: {
      default: siteConfig.title,
      template: `%s | ${siteConfig.title}`,
    },
    twitter: {
      card: 'summary_large_image',
      creator: siteConfig.twitter,
      description: siteConfig.description,
      images: ['/og-image.jpg'],
      title: siteConfig.title,
    },
  }
}

/* ---------- viewport ---------- */
export const viewport: Viewport = {
  initialScale: 1,
  // Avoids auto-detecting phone numbers/emails as links on iOS:
  interactiveWidget: 'resizes-content',
  themeColor: [
    { color: '#000000', media: '(prefers-color-scheme: dark)' },
    { color: '#ffffff', media: '(prefers-color-scheme: light)' },
  ],
  // Optional but helpful:
  viewportFit: 'cover',
  width: 'device-width',
}

/* ---------- generateStaticParams ---------- */
interface StaticParameter {
  readonly locale: Locale
}
type GenerateStaticParameters = () => readonly StaticParameter[]

export const generateStaticParams: GenerateStaticParameters =
  (): readonly StaticParameter[] => {
    const out: StaticParameter[] = []
    for (const loc of routing.locales as readonly Locale[]) {
      out.push({ locale: loc })
    }
    return out
  }

/* ---------- layout ---------- */
type RootLayoutProperties = UnparsedLocalePageProperties

const RootLayout: RoutePageWithChildrenFC<RootLayoutProperties> = async ({
  children,
  params,
}: PageParametersWithChildren<RootLayoutProperties>): Promise<JSX.Element> => {
  const locale: Locale = await ensureLocaleFromParameters(params)
  setRequestLocale(locale)

  const messages: DeepPartial<Messages> = await getMessages()

  return (
    <html className={`dark ${inter.variable}`} lang={locale}>
      <body className="font-sans antialiased">
        <NextIntlClientProvider locale={locale} messages={messages}>
          <ThemeProvider defaultTheme="dark">
            <ThemeToggle />
            <LanguageSwitcher />

            {/* Non-critical client UI mounts after idle inside this wrapper */}
            <DeferredClientUi />
            <main id="content">{children}</main>
            <LegalFooter locale={locale} />
          </ThemeProvider>
        </NextIntlClientProvider>
      </body>
    </html>
  )
}

export default RootLayout
