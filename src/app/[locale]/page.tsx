import { type JSX } from 'react'

import { type Locale } from 'next-intl'

import { setRequestLocale } from 'next-intl/server'

import { ScrollSnapPairController } from '@/components/features/scroll-snap/scroll-snap-pair-controller'
import AboutSection from '@/components/sections/about-section'
import { ContactSection } from '@/components/sections/contact-section'
import { ExperienceSection } from '@/components/sections/experience-section'
import { HeroSection } from '@/components/sections/hero-section'
import { ProjectsSection } from '@/components/sections/projects-section'
import { SkillsSection } from '@/components/sections/skills-section'
import { TestimonialsSection } from '@/components/sections/testimonials-section'
import { ensureLocaleFromParameters } from '@/i18n/locale'
import type { UnparsedLocalePageProperties } from '@/types/i18n'
import type { PageParameters, RoutePageFC } from '@/types/page'

type HomeProperties = UnparsedLocalePageProperties

const Home: RoutePageFC<HomeProperties> = async ({
  params,
}: PageParameters<HomeProperties>): Promise<JSX.Element> => {
  const locale: Locale = await ensureLocaleFromParameters(params)
  setRequestLocale(locale)

  return (
    <main className="bg-background">
      <section className="min-h-screen" id="hero-section">
        <HeroSection locale={locale} />
      </section>

      <article
        id="main-section"
        itemScope={true}
        itemType="https://schema.org/Person"
      >
        <AboutSection locale={locale} />
        <SkillsSection locale={locale} />
        <ProjectsSection locale={locale} />
        <ExperienceSection locale={locale} />
        <TestimonialsSection locale={locale} />
        <ContactSection locale={locale} />
      </article>

      <ScrollSnapPairController
        bottomSectionId="main-section"
        topSectionId="hero-section"
      />
    </main>
  )
}

export default Home
