import { type JSX, Suspense } from 'react'

import { type Locale } from 'next-intl'

import { setRequestLocale } from 'next-intl/server'

import { ScrollSnapPairController } from '@/components/scroll-snap-pair-controller'
import { DeferredSections } from '@/components/sections/deferred-sections'
import { HeroSection } from '@/components/sections/hero-section'
import { ensureLocaleFromParameters } from '@/i18n/locale'
import { fetchGitHubData, type GitHubData } from '@/lib/github'
import type { UnparsedLocalePageProperties } from '@/types/i18n'
import type { PageParameters, RoutePageFC } from '@/types/page'

type HomeProperties = UnparsedLocalePageProperties

const Home: RoutePageFC<HomeProperties> = async ({
  params,
}: PageParameters<HomeProperties>): Promise<JSX.Element> => {
  const locale: Locale = await ensureLocaleFromParameters(params)
  setRequestLocale(locale)

  const dataPromise: Promise<GitHubData> = fetchGitHubData()

  return (
    <main className="bg-background">
      <section className="min-h-screen" id="hero-section">
        <HeroSection locale={locale} />
      </section>

      <section id="main-section">
        <Suspense fallback={null}>
          <DeferredSections dataPromise={dataPromise} locale={locale} />
        </Suspense>
      </section>

      <ScrollSnapPairController
        bottomSectionId="main-section"
        topSectionId="hero-section"
      />
    </main>
  )
}

export default Home
