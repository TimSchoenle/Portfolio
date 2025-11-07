import { type JSX } from 'react'

import { type Locale } from 'next-intl'

import { setRequestLocale } from 'next-intl/server'

import AboutSection from '@/components/about-section'
import { ContactSection } from '@/components/contact-section'
import { ExperienceSection } from '@/components/experience-section'
import { HeroSection } from '@/components/hero-section'
import { ProjectsSection } from '@/components/projects-section'
import { SkillsSection } from '@/components/skills-section'
import { TestimonialsSection } from '@/components/testimonials-section'
import { ensureLocaleFromParameters } from '@/i18n/locale'
import { siteConfig } from '@/lib/config'
import {
  getContributionData,
  getFeaturedProjects,
  getUserStats,
} from '@/lib/github'
import type {
  ContributionPoint,
  GitHubProject,
  UserStats,
} from '@/types/github'
import type { UnparsedLocalePageProperties } from '@/types/i18n'
import type { PageParameters, RoutePageFC } from '@/types/page'

interface GitHubData {
  contributionData: ContributionPoint[]
  projects: GitHubProject[]
  stats: UserStats
}

// Helper to fetch GitHub data
const fetchGitHubData: () => Promise<
  Readonly<GitHubData>
> = async (): Promise<GitHubData> => {
  const projects: GitHubProject[] = await getFeaturedProjects()
  const stats: UserStats = await getUserStats()
  const contributionData: ContributionPoint[] = await getContributionData()

  return { contributionData, projects, stats }
}

type HomeProperties = UnparsedLocalePageProperties

const Home: RoutePageFC<HomeProperties> = async ({
  params,
}: PageParameters<HomeProperties>): Promise<JSX.Element> => {
  const locale: Locale = await ensureLocaleFromParameters(params)

  setRequestLocale(locale)

  const { contributionData, projects, stats }: GitHubData =
    await fetchGitHubData()

  return (
    <main className="bg-background h-screen snap-y snap-mandatory overflow-y-scroll">
      <HeroSection locale={locale} />
      <div className="snap-start">
        <AboutSection locale={locale} />
        <SkillsSection locale={locale} />
        <ProjectsSection
          contributionData={contributionData}
          githubUsername={siteConfig.githubUsername}
          locale={locale}
          projects={projects}
          stats={stats}
        />
        <ExperienceSection locale={locale} />
        <TestimonialsSection locale={locale} />
        <ContactSection locale={locale} />
      </div>
    </main>
  )
}

export default Home
