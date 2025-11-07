'use server'

import type { JSX } from 'react'

import { type Locale } from 'next-intl'

import { Code2, ExternalLink, GitFork, Globe, Link, Star } from 'lucide-react'
import { getTranslations } from 'next-intl/server'

import { ContributionGraph } from '@/components/contribution-graph'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import type { FCAsync, FCStrict } from '@/types/fc'
import type {
  ContributionPoint,
  GitHubProject,
  UserStats,
} from '@/types/github'
import type { Translations } from '@/types/i18n'

/* --------------------------------- pieces --------------------------------- */

interface SectionHeaderProperties {
  readonly subtitle: string
  readonly title: string
}
const SectionHeader: FCStrict<SectionHeaderProperties> = ({
  subtitle,
  title,
}: SectionHeaderProperties): JSX.Element => {
  return (
    <div className="mb-16 text-center">
      <h2 className="from-primary to-primary/60 mb-4 bg-gradient-to-r bg-clip-text text-4xl font-bold text-transparent md:text-5xl">
        {title}
      </h2>
      <p className="text-muted-foreground mx-auto max-w-2xl text-lg">
        {subtitle}
      </p>
    </div>
  )
}

interface StatsCardProperties {
  readonly icon: JSX.Element
  readonly label: string
  readonly value: number
}
const StatsCard: FCStrict<StatsCardProperties> = ({
  icon,
  label,
  value,
}: StatsCardProperties): JSX.Element => {
  return (
    <Card className="hover:border-primary/50 border-2 p-6 transition-all duration-300 hover:shadow-lg">
      <div className="flex items-center gap-4">
        <div className="bg-primary/10 rounded-lg p-3">{icon}</div>
        <div>
          <p className="text-3xl font-bold">{value}</p>
          <p className="text-muted-foreground text-sm">{label}</p>
        </div>
      </div>
    </Card>
  )
}

interface ProjectCardProperties {
  readonly project: GitHubProject
}
// eslint-disable-next-line max-lines-per-function
const ProjectCard: FCStrict<ProjectCardProperties> = ({
  project,
}: ProjectCardProperties): JSX.Element => {
  const hasHomepage: boolean =
    typeof project.homepage === 'string' && project.homepage.length > 0

  return (
    <Card className="group hover:border-primary/50 flex flex-col overflow-hidden border-2 transition-all duration-300 hover:shadow-xl">
      <div className="from-primary/20 to-primary/5 relative h-48 overflow-hidden bg-gradient-to-br">
        <div className="absolute inset-0 flex items-center justify-center">
          <Code2 className="text-primary/40 h-20 w-20 transition-transform duration-300 group-hover:scale-110" />
        </div>
      </div>

      <div className="flex flex-1 flex-col p-6">
        <h3 className="group-hover:text-primary mb-2 text-xl font-bold transition-colors">
          {project.name}
        </h3>
        <p className="text-muted-foreground mb-4 flex-1">
          {project.description}
        </p>

        <div className="mb-4 flex flex-wrap gap-2">
          {project.topics.map(
            (topic: string): JSX.Element => (
              <Badge className="text-xs" key={topic} variant="secondary">
                {topic}
              </Badge>
            )
          )}
        </div>

        <div className="flex items-center justify-between border-t pt-4">
          <div className="text-muted-foreground flex items-center gap-4 text-sm">
            <span className="flex items-center gap-1">
              <Star className="h-4 w-4" />
              {project.stargazers_count}
            </span>
            <span className="flex items-center gap-1">
              <GitFork className="h-4 w-4" />
              {project.forks_count}
            </span>
          </div>

          <div className="flex gap-2">
            <Button asChild={true} size="sm" variant="ghost">
              <a
                href={project.html_url}
                rel="noopener noreferrer"
                target="_blank"
              >
                <Link className="h-4 w-4" />
              </a>
            </Button>
            {hasHomepage ? (
              <Button asChild={true} size="sm" variant="ghost">
                <a
                  href={project.homepage}
                  rel="noopener noreferrer"
                  target="_blank"
                >
                  <Globe className="h-4 w-4" />
                </a>
              </Button>
            ) : null}
          </div>
        </div>
      </div>
    </Card>
  )
}

interface ProjectsGridProperties {
  readonly projects: readonly GitHubProject[]
}
const ProjectsGrid: FCStrict<ProjectsGridProperties> = ({
  projects,
}: ProjectsGridProperties): JSX.Element => {
  return (
    <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
      {projects.map(
        (project: GitHubProject): JSX.Element => (
          <ProjectCard key={project.html_url} project={project} />
        )
      )}
    </div>
  )
}

interface SectionFooterProperties {
  readonly cta: string
  readonly githubUsername: string
}
const SectionFooter: FCStrict<SectionFooterProperties> = ({
  cta,
  githubUsername,
}: SectionFooterProperties): JSX.Element => {
  return (
    <div className="mt-12 text-center">
      <Button asChild={true} className="group" size="lg">
        <a
          href={`https://github.com/${githubUsername}`}
          rel="noopener noreferrer"
          target="_blank"
        >
          {cta}
          <ExternalLink className="ml-2 h-4 w-4 transition-transform group-hover:translate-x-1" />
        </a>
      </Button>
    </div>
  )
}

/* ------------------------------- main export ------------------------------ */

interface ProjectsSectionProperties {
  readonly contributionData: readonly ContributionPoint[]
  readonly githubUsername: string
  readonly locale: Locale
  readonly projects: readonly GitHubProject[]
  readonly stats: UserStats
}
export const ProjectsSection: FCAsync<ProjectsSectionProperties> = async ({
  contributionData,
  githubUsername,
  locale,
  projects,
  stats,
}: ProjectsSectionProperties): Promise<JSX.Element> => {
  const translations: Translations<'projects'> = await getTranslations({
    locale,
    namespace: 'projects',
  })

  return (
    <section
      className="from-background to-muted/20 min-h-screen bg-gradient-to-b px-4 py-20 md:px-8"
      id="projects"
    >
      <div className="mx-auto max-w-7xl">
        <SectionHeader
          subtitle={translations('subtitle')}
          title={translations('title')}
        />

        {/* GitHub Stats Cards */}
        <div className="mb-16 grid grid-cols-1 gap-6 md:grid-cols-3">
          <StatsCard
            icon={<Code2 className="text-primary h-8 w-8" />}
            label={translations('stats.repositories')}
            value={stats.repositories}
          />
          <StatsCard
            icon={<Star className="text-primary h-8 w-8" />}
            label={translations('stats.stars')}
            value={stats.stars}
          />
          <StatsCard
            icon={<GitFork className="text-primary h-8 w-8" />}
            label={translations('stats.forks')}
            value={stats.forks}
          />
        </div>

        {/* Featured Projects */}
        <ProjectsGrid projects={projects} />

        {/* GitHub Contribution Graph */}
        <div className="mt-16">
          <ContributionGraph data={contributionData} locale={locale} />
        </div>

        {/* View All Projects Button */}
        <SectionFooter
          cta={translations('viewAll')}
          githubUsername={githubUsername}
        />
      </div>
    </section>
  )
}
