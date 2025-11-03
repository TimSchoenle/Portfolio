import { HeroSection } from '@/components/hero-section'
import { AboutSection } from '@/components/about-section'
import { SkillsSection } from '@/components/skills-section'
import { ProjectsSection } from '@/components/projects-section'
import { ExperienceSection } from '@/components/experience-section'
import { TestimonialsSection } from '@/components/testimonials-section'
import { ContactSection } from '@/components/contact-section'
import {
  getContributionData,
  getFeaturedProjects,
  getUserStats,
} from '@/lib/github'
import { siteConfig } from '@/lib/config'
import type { Metadata } from 'next'

export async function generateMetadata(): Promise<Metadata> {
  return {
    metadataBase: new URL(siteConfig.url),
    title: siteConfig.name,
    description: siteConfig.description,
    alternates: {
      canonical: siteConfig.url,
      languages: {
        'en-US': `${siteConfig.url}/en`,
        'de-DE': `${siteConfig.url}/de`,
      },
    },
    openGraph: {
      type: 'website',
      url: siteConfig.url,
      siteName: siteConfig.name,
      images: [`${siteConfig.url}/og-image.jpg`],
    },
    other: {
      'application-ld+json': JSON.stringify({
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
      }),
    },
  }
}

export default async function Home() {
  // Fetch GitHub data in parallel for better performance
  const [projects, stats, contributionData] = await Promise.all([
    getFeaturedProjects(siteConfig.githubUsername, siteConfig.featuredRepos),
    getUserStats(siteConfig.githubUsername),
    getContributionData(siteConfig.githubUsername),
  ])

  return (
    <>
      <main className="bg-background h-screen snap-y snap-mandatory overflow-y-scroll">
        <HeroSection />
        <div className="snap-start">
          <AboutSection />
          <SkillsSection />
          <ProjectsSection
            githubUsername={siteConfig.githubUsername}
            projects={projects}
            stats={stats}
            contributionData={contributionData}
          />
          <ExperienceSection />
          <TestimonialsSection />
          <ContactSection />
        </div>
      </main>
    </>
  )
}
