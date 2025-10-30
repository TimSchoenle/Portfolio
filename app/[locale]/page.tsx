import { HeroSection } from "@/components/hero-section"
import { AboutSection } from "@/components/about-section"
import { SkillsSection } from "@/components/skills-section"
import { ProjectsSection } from "@/components/projects-section"
import { ExperienceSection } from "@/components/experience-section"
import { TestimonialsSection } from "@/components/testimonials-section"
import { ContactSection } from "@/components/contact-section"
import { getDictionary } from "@/lib/get-dictionary"
import type { Locale } from "@/lib/i18n-config"

export default async function Home({ params }: { params: Promise<{ locale: Locale }> }) {
  const { locale } = await params
  const dict = await getDictionary(locale)

  const jsonLd = {
    "@context": "https://schema.org",
    "@type": "Person",
    name: "Tim",
    alternateName: "Timmi6790",
    url: "https://timmi6790.de",
    image: "https://timmi6790.de/og-image.jpg",
    sameAs: ["https://github.com/Timmi6790"],
    jobTitle: "Software Developer",
    worksFor: {
      "@type": "Organization",
      name: "Independent",
    },
    address: {
      "@type": "PostalAddress",
      addressCountry: "DE",
    },
    email: "contact@timmi6790.de",
    knowsAbout: ["Java", "Rust", "Next.js", "Software Development", "Open Source"],
  }

  return (
    <>
      <script type="application/ld+json" dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }} />
      <main className="h-screen overflow-y-scroll snap-y snap-mandatory bg-background">
        <HeroSection dict={dict} />
        <div className="snap-start">
          <AboutSection dict={dict} />
          <SkillsSection dict={dict} />
          <ProjectsSection dict={dict} />
          <ExperienceSection dict={dict} />
          <TestimonialsSection dict={dict} />
          <ContactSection dict={dict} locale={locale} />
        </div>
      </main>
    </>
  )
}
