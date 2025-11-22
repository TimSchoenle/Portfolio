import { type JSX, use } from 'react'

import { type Locale } from 'next-intl'

import AboutSection from '@/components/sections/about-section'
import { ContactSection } from '@/components/sections/contact-section'
import { ExperienceSection } from '@/components/sections/experience-section'
import { ProjectsSection } from '@/components/sections/projects-section'
import { SkillsSection } from '@/components/sections/skills-section'
import { TestimonialsSection } from '@/components/sections/testimonials-section'
import { siteConfig } from '@/lib/config'
import { type GitHubData } from '@/lib/github'

interface DeferredSectionsProperties {
  dataPromise: Promise<GitHubData>
  locale: Locale
}

export const DeferredSections: (
  properties: DeferredSectionsProperties
) => JSX.Element = ({
  dataPromise,
  locale,
}: DeferredSectionsProperties): JSX.Element => {
  const { contributionData, projects, stats }: GitHubData = use(dataPromise)
  return (
    <>
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
    </>
  )
}
