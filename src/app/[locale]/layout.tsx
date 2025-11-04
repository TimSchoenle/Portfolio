import type React from 'react'
import type { Metadata } from 'next'
import { Toaster } from 'sonner'
import '../globals.css'
import { Geist, Geist_Mono, Source_Serif_4 } from 'next/font/google'
import { ThemeProvider } from '@/components/theme-provider'
import { ThemeToggle } from '@/components/theme-toggle'
import { CommandPalette } from '@/components/command-palette'
import { EasterEggs } from '@/components/easter-eggs'
import { LanguageSwitcher } from '@/components/language-switcher'
import { NextIntlClientProvider, hasLocale, type Locale } from 'next-intl'
import { getPathname, routing } from '@/i18n/routing'
import { siteConfig } from '@/lib/config'
import { LegalFooter } from '@/components/legal-footer'
import { notFound } from 'next/navigation'
import { setRequestLocale } from 'next-intl/server'
import ConsentManager from '@/components/consent-manager/consent-manager'

const geist = Geist({
  subsets: ['latin'],
  weight: ['100', '200', '300', '400', '500', '600', '700', '800', '900'],
  variable: '--font-geist',
})
const geistMono = Geist_Mono({
  subsets: ['latin'],
  weight: ['100', '200', '300', '400', '500', '600', '700', '800', '900'],
  variable: '--font-geist-mono',
})
const sourceSerif = Source_Serif_4({
  subsets: ['latin'],
  weight: ['200', '300', '400', '500', '600', '700', '800', '900'],
  variable: '--font-source-serif',
})

// Dynamic metadata based on locale
export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: Locale }>
}): Promise<Metadata> {
  const { locale } = await params
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
      locale: locale,
      alternateLocale: routing.locales
        .filter((loc: Locale) => loc !== locale)
        .join(', '),
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
      // canonical: `${siteConfig.url}/${locale}`,
      languages: Object.fromEntries(
        routing.locales.map((loc: Locale) => [
          loc,
          `${siteConfig.url}/${getPathname({ locale: loc, href: '/' })}`,
        ])
      ),
    },
  }
}

export async function generateStaticParams() {
  return routing.locales.map((locale: Locale) => ({ locale }))
}

export default async function RootLayout({
  children,
  params,
}: Readonly<{
  children: React.ReactNode
  params: Promise<{ locale: string }>
}>) {
  // Ensure that the incoming `locale` is valid
  const { locale } = await params
  if (!hasLocale(routing.locales, locale)) {
    notFound()
  }

  // Enable static rendering
  setRequestLocale(locale)

  return (
    <html lang={locale} className="dark">
      <body
        className={`${geist.variable} ${geistMono.variable} ${sourceSerif.variable} font-sans antialiased`}
      >
        <NextIntlClientProvider locale={locale}>
          <ThemeProvider defaultTheme="dark">
            <ConsentManager locale={locale}>
              <ThemeToggle />
              <LanguageSwitcher />
              <CommandPalette />
              <EasterEggs />
              {children}
              <LegalFooter locale={locale} />
              <Toaster position="bottom-right" />
            </ConsentManager>
          </ThemeProvider>
        </NextIntlClientProvider>
      </body>
    </html>
  )
}
