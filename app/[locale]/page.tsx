import { HeroSection } from '@/components/hero-section'
import { AboutSection } from '@/components/about-section'
import { SkillsSection } from '@/components/skills-section'
import { ProjectsSection } from '@/components/projects-section'
import { ExperienceSection } from '@/components/experience-section'
import { TestimonialsSection } from '@/components/testimonials-section'
import { ContactSection } from '@/components/contact-section'
import { getDictionary } from '@/lib/get-dictionary'
import type { Locale } from '@/lib/i18n-config'
import {
  fetchContributionGraph,
  getFeaturedProjects,
  getUserStats,
} from '@/lib/github'
import { siteConfig } from '@/lib/config'

export default async function Home({
  params,
}: {
  params: Promise<{ locale: Locale }>
}) {
  const { locale } = await params
  const dict = await getDictionary(locale)

  // Fetch GitHub data in parallel for better performance
  const [projects, stats, contributionGraph] = await Promise.all([
    getFeaturedProjects(siteConfig.githubUsername, siteConfig.featuredRepos),
    getUserStats(siteConfig.githubUsername),
    fetchContributionGraph(siteConfig.githubUsername),
  ])

  // Structured data for SEO
  const jsonLd = {
    '@context': 'https://schema.org',
    '@type': 'Person',
    name: siteConfig.name,
    alternateName: siteConfig.username,
    url: siteConfig.url,
    image: `${siteConfig.url}/og-image.jpg`,
    sameAs: [siteConfig.github],
    jobTitle: siteConfig.title,
    worksFor: {
      '@type': 'Organization',
      name: 'Independent',
    },
    address: {
      '@type': 'PostalAddress',
      addressCountry: 'DE',
    },
    email: siteConfig.email,
    knowsAbout: [
      'Java',
      'Rust',
      'Next.js',
      'Software Development',
      'Open Source',
    ],
  }

  return (
    <>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
      />
      <main className="bg-background h-screen snap-y snap-mandatory overflow-y-scroll">
        <HeroSection dict={dict.hero} />
        <div className="snap-start">
          <AboutSection about={dict.about} />
          <SkillsSection dict={dict.skills} />
          <ProjectsSection
            dict={dict.projects}
            projects={projects}
            stats={stats}
            contributionGraph={contributionGraph}
          />
          <ExperienceSection dict={dict.experience} />
          <TestimonialsSection dict={dict.testimonials} />
          <ContactSection dict={dict.contact} locale={locale} />
        </div>
      </main>
    </>
  )
}

