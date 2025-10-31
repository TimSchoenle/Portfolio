import type { Metadata } from 'next'

import './globals.css'

import {
  Geist,
  Geist_Mono,
  Source_Serif_4,
} from 'next/font/google'

// Initialize fonts
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

export const metadata: Metadata = {
  title: 'Tim - Software Developer Portfolio',
  description: 'Portfolio of Tim (Timmi6790) - Software Developer specializing in Java, learning Rust and Next.js',
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en">
      <body className={`${geist.variable} ${geistMono.variable} ${sourceSerif.variable} font-sans antialiased`}>{children}</body>
    </html>
  )
}
