// E:\NextJs\Portfolio\src\app\[locale]\layout.tsx

import '../globals.css'

import type { Metadata } from 'next'
import type { NextFontWithVariable } from 'next/dist/compiled/@next/font'
import { Geist, Geist_Mono, Source_Serif_4 } from 'next/font/google'
import { NextIntlClientProvider, type Locale } from 'next-intl'
import { setRequestLocale } from 'next-intl/server'
import type { JSX } from 'react'
import { Toaster } from 'sonner'

import { CommandPalette } from '@/components/command-palette'
import { CookieBanner } from '@/components/cookie-banner'
import { EasterEggs } from '@/components/easter-eggs'
import { LanguageSwitcher } from '@/components/language-switcher'
import { LegalFooter } from '@/components/legal-footer'
import { ThemeProvider } from '@/components/theme-provider'
import { ThemeToggle } from '@/components/theme-toggle'
import { ensureLocaleFromParams } from '@/i18n/locale'
import { getPathname, routing } from '@/i18n/routing'
import { siteConfig } from '@/lib/config'
import type { FCAsyncWithChildren, WithChildren } from '@/types/fc'
import type { LocalePageProps, UnparsedLocalePageProps } from '@/types/i18n'

/* ---------- fonts ---------- */
const geist: NextFontWithVariable = Geist({
  subsets: ['latin'],
  variable: '--font-geist',
  weight: ['100', '200', '300', '400', '500', '600', '700', '800', '900'],
})

const geistMono: NextFontWithVariable = Geist_Mono({
  subsets: ['latin'],
  variable: '--font-geist-mono',
  weight: ['100', '200', '300', '400', '500', '600', '700', '800', '900'],
})

const sourceSerif: NextFontWithVariable = Source_Serif_4({
  subsets: ['latin'],
  variable: '--font-source-serif',
  weight: ['200', '300', '400', '500', '600', '700', '800', '900'],
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
        `${siteConfig.url}/${getPathname({ locale: loc, href: '/' })}`,
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

interface GenerateMetadataParams {
  readonly params: Promise<LocalePageProps>
}

type GenerateMetadataFn = (
  args: Readonly<GenerateMetadataParams>
) => Promise<Metadata>

export const generateMetadata: GenerateMetadataFn = async (
  args: Readonly<GenerateMetadataParams>
): Promise<Metadata> => {
  const { locale }: { locale: Locale } = await args.params

  return {
    metadataBase: new URL(siteConfig.url),
    title: {
      default: siteConfig.title,
      template: `%s | ${siteConfig.title}`,
    },
    description: siteConfig.description,
    keywords: siteConfig.seo.keywords.join(', '),
    authors: [{ name: siteConfig.fullName, url: siteConfig.url }],
    creator: siteConfig.fullName,
    publisher: siteConfig.fullName,
    robots: {
      index: true,
      follow: true,
      googleBot: {
        index: true,
        follow: true,
        'max-video-preview': -1,
        'max-image-preview': 'large',
        'max-snippet': -1,
      },
    },
    openGraph: {
      type: 'website',
      locale,
      alternateLocale: buildAlternateLocales(locale),
      url: `${siteConfig.url}/${locale}`,
      title: siteConfig.title,
      description: siteConfig.description,
      siteName: siteConfig.title,
      images: [
        {
          url: '/og-image.png',
          width: 1200,
          height: 630,
          alt: siteConfig.title,
        },
      ],
    },
    twitter: {
      card: 'summary_large_image',
      title: siteConfig.title,
      description: siteConfig.description,
      images: ['/og-image.png'],
      creator: siteConfig.twitter,
    },
    alternates: {
      languages: buildLanguages(),
    },
  }
}

/* ---------- generateStaticParams ---------- */

interface StaticParam {
  readonly locale: Locale
}
type GenerateStaticParams = () => readonly StaticParam[]

export const generateStaticParams: GenerateStaticParams =
  (): readonly StaticParam[] => {
    const out: StaticParam[] = []
    for (const loc of routing.locales as readonly Locale[]) {
      out.push({ locale: loc })
    }
    return out
  }

/* ---------- layout ---------- */
interface RootLayoutProps extends WithChildren {
  readonly params: Readonly<Promise<UnparsedLocalePageProps>>
}

const RootLayout: FCAsyncWithChildren<RootLayoutProps> = async ({
  children,
  params,
}: RootLayoutProps): Promise<JSX.Element> => {
  const locale: Locale = await ensureLocaleFromParams(params)

  setRequestLocale(locale)

  return (
    <html className="dark" lang={locale}>
      <body
        className={`${geist.variable} ${geistMono.variable} ${sourceSerif.variable} font-sans antialiased`}
      >
        <NextIntlClientProvider locale={locale}>
          <ThemeProvider defaultTheme="dark">
            <ThemeToggle />
            <LanguageSwitcher />
            <CommandPalette />
            <EasterEggs />
            {children}
            <CookieBanner />
            <LegalFooter locale={locale} />
            <Toaster position="bottom-right" />
          </ThemeProvider>
        </NextIntlClientProvider>
      </body>
    </html>
  )
}

export default RootLayout
