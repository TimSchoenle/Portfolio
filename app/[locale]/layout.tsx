import type React from 'react'
import type { Metadata } from 'next'
import { Toaster } from 'sonner'
import '../globals.css'
import {
  Geist,
  Geist_Mono,
  Source_Serif_4,
} from 'next/font/google'
import { locales, type Locale } from '@/lib/i18n-config'
import { LanguageSwitcher } from '@/components/language-switcher'
import { ThemeProvider } from '@/components/theme-provider'
import { ThemeToggle } from '@/components/theme-toggle'
import { CookieBanner } from '@/components/cookie-banner'
import { CommandPalette } from '@/components/command-palette'
import { EasterEggs } from '@/components/easter-eggs'
import { getDictionary } from '@/lib/get-dictionary'

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

  const localeMap = {
    en: 'en_US',
    de: 'de_DE',
  } as const

  return {
    metadataBase: new URL('https://timmi6790.de'),
    title: {
      default: 'Tim - Software Developer Portfolio',
      template: '%s | Tim - Software Developer',
    },
    description:
      'Portfolio of Tim (Timmi6790) - Software Developer specializing in Java, learning Rust and Next.js. Open-source contributor and passionate about building great software.',
    keywords: [
      'Tim',
      'Timmi6790',
      'Software Developer',
      'Java',
      'Rust',
      'Next.js',
      'Portfolio',
      'Open Source',
      'Germany',
    ],
    authors: [{ name: 'Tim', url: 'https://timmi6790.de' }],
    creator: 'Tim (Timmi6790)',
    publisher: 'Tim (Timmi6790)',
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
      locale: localeMap[locale],
      alternateLocale: locale === 'en' ? ['de_DE'] : ['en_US'],
      url: `https://timmi6790.de/${locale}`,
      title: 'Tim - Software Developer Portfolio',
      description:
        'Portfolio of Tim (Timmi6790) - Software Developer specializing in Java, learning Rust and Next.js',
      siteName: 'Tim Portfolio',
      images: [
        {
          url: '/og-image.png',
          width: 1200,
          height: 630,
          alt: 'Tim - Software Developer Portfolio',
        },
      ],
    },
    twitter: {
      card: 'summary_large_image',
      title: 'Tim - Software Developer Portfolio',
      description:
        'Portfolio of Tim (Timmi6790) - Software Developer specializing in Java, learning Rust and Next.js',
      images: ['/og-image.png'],
      creator: '@Timmi6790',
    },
    alternates: {
      canonical: `https://timmi6790.de/${locale}`,
      languages: {
        en: 'https://timmi6790.de/en',
        de: 'https://timmi6790.de/de',
      },
    },
  }
}

export async function generateStaticParams() {
  return locales.map((locale) => ({ locale }))
}

export default async function RootLayout({
  children,
  params,
}: Readonly<{
  children: React.ReactNode
  params: Promise<{ locale: Locale }>
}>) {
  const { locale } = await params
  const dict = await getDictionary(locale)

  return (
    <html lang={locale} className="dark">
      <body className={`${geist.variable} ${geistMono.variable} ${sourceSerif.variable} font-sans antialiased`}>
        <ThemeProvider defaultTheme="dark">
          <ThemeToggle />
          <LanguageSwitcher currentLocale={locale} />
          <CommandPalette locale={locale} dict={dict} />
          <EasterEggs />
          {children}
          <CookieBanner translations={dict.cookies} />
          <Toaster position="bottom-right" />
        </ThemeProvider>
      </body>
    </html>
  )
}
