import { type Locale } from 'next-intl'
import { setRequestLocale } from 'next-intl/server'
import { type JSX } from 'react'

import AboutSection from '@/components/about-section'
import { ContactSection } from '@/components/contact-section'
import { ExperienceSection } from '@/components/experience-section'
import { HeroSection } from '@/components/hero-section'
import { ProjectsSection } from '@/components/projects-section'
import { SkillsSection } from '@/components/skills-section'
import { TestimonialsSection } from '@/components/testimonials-section'
import { ensureLocaleFromParams } from '@/i18n/locale'
import { siteConfig } from '@/lib/config'
import {
  getContributionData,
  getFeaturedProjects,
  getUserStats,
} from '@/lib/github'
import type { FCAsync, NoChildren } from '@/types/fc'
import type { UnparsedLocalePageProps } from '@/types/i18n'

interface HomeProps extends NoChildren {
  readonly params: Promise<UnparsedLocalePageProps>
}

interface GitHubData {
  projects: Awaited<ReturnType<typeof getFeaturedProjects>>
  stats: Awaited<ReturnType<typeof getUserStats>>
  contributionData: Awaited<ReturnType<typeof getContributionData>>
}

// Helper to fetch GitHub data
const fetchGitHubData: () => Promise<
  Readonly<GitHubData>
> = async (): Promise<GitHubData> => {
  const projects: Awaited<ReturnType<typeof getFeaturedProjects>> =
    await getFeaturedProjects(
      siteConfig.githubUsername,
      siteConfig.featuredRepos
    )
  const stats: Awaited<ReturnType<typeof getUserStats>> = await getUserStats(
    siteConfig.githubUsername
  )
  const contributionData: Awaited<ReturnType<typeof getContributionData>> =
    await getContributionData(siteConfig.githubUsername)

  return { projects, stats, contributionData }
}

const Home: FCAsync<HomeProps> = async ({
  params,
}: HomeProps): Promise<JSX.Element> => {
  const locale: Locale = await ensureLocaleFromParams(params)

  setRequestLocale(locale)

  const { projects, stats, contributionData }: GitHubData =
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
