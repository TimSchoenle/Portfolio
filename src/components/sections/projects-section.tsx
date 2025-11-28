import type { JSX } from 'react'

import { type Locale } from 'next-intl'

import {
  Code2,
  ExternalLink,
  GitFork,
  Globe,
  Link as LinkIcon,
  Star,
} from 'lucide-react'
import { getTranslations } from 'next-intl/server'

import { ContributionGraph } from '@/components/features/contribution-graph/contribution-graph'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import { siteConfig } from '@/lib/config'
import { getGithubUser, type GitHubData } from '@/lib/github/client'
import type { FCAsync, FCStrict } from '@/types/fc'
import type { GitHubProject } from '@/types/github'
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
      <Heading
        as="h2"
        className="mb-4 inline-block bg-gradient-to-r from-primary to-primary/60 bg-clip-text py-px text-4xl leading-[1.15] font-bold [text-wrap:balance] text-transparent md:text-5xl md:leading-[1.2]"
      >
        {title}
      </Heading>
      <p className="mx-auto max-w-2xl text-lg text-muted-foreground">
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
    <Card className="border-2 p-6 transition-all duration-300 hover:border-primary/50 hover:shadow-lg">
      <div className="flex items-center gap-4">
        <div className="rounded-lg bg-primary/10 p-3">{icon}</div>
        <p className="flex flex-col">
          <span className="text-3xl font-bold">{value}</span>
          <span className="text-sm text-muted-foreground">{label}</span>
        </p>
      </div>
    </Card>
  )
}

interface ProjectCardProperties {
  readonly project: GitHubProject
  readonly translations: Translations<'projects'>
}
// eslint-disable-next-line max-lines-per-function
const ProjectCard: FCStrict<ProjectCardProperties> = ({
  project,
  translations,
}: ProjectCardProperties): JSX.Element => {
  const hasHomepage: boolean =
    typeof project.homepage === 'string' && project.homepage.length > 0

  return (
    <Card className="group flex h-full flex-col overflow-hidden border-2 transition-all duration-300 hover:border-primary/50 hover:shadow-xl">
      <div className="relative h-48 overflow-hidden bg-gradient-to-br from-primary/20 to-primary/5">
        <div className="absolute inset-0 flex items-center justify-center">
          <Code2 className="h-20 w-20 text-primary/40 transition-transform duration-300 group-hover:scale-110" />
        </div>
      </div>

      <div className="flex flex-1 flex-col p-6">
        <Heading
          as="h3"
          className="mb-2 text-xl font-bold transition-colors group-hover:text-primary"
        >
          {project.name}
        </Heading>
        <p className="mb-4 flex-1 text-muted-foreground">
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
          <div
            aria-hidden="true"
            className="flex items-center gap-4 text-sm text-muted-foreground print:hidden"
          >
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
                aria-label={translations('view')}
                href={project.html_url}
                rel="noopener noreferrer"
                target="_blank"
              >
                <LinkIcon className="h-4 w-4" />
              </a>
            </Button>
            {hasHomepage ? (
              <Button asChild={true} size="sm" variant="ghost">
                <a
                  aria-label={translations('view')}
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
  readonly translations: Translations<'projects'>
}

// eslint-disable-next-line max-lines-per-function
const ProjectsGrid: FCStrict<ProjectsGridProperties> = ({
  projects,
  translations,
}: ProjectsGridProperties): JSX.Element => {
  const hasManyProjects: boolean = projects.length > 3
  const mobileProjects: readonly GitHubProject[] = projects.slice(0, 3)

  const renderProject: (
    project: GitHubProject,
    wrapperClassName?: string
  ) => JSX.Element = (
    project: GitHubProject,
    wrapperClassName?: string
  ): JSX.Element => (
    <li className={wrapperClassName} key={project.html_url}>
      <ProjectCard project={project} translations={translations} />
    </li>
  )

  return (
    <div className="space-y-6">
      {/* Mobile: limit to the first 3 projects in a simple grid */}
      <ul className="grid grid-cols-1 gap-6 sm:grid-cols-2 md:hidden">
        {mobileProjects.map(
          (project: GitHubProject): JSX.Element => (
            <li key={project.html_url}>
              <ProjectCard project={project} translations={translations} />
            </li>
          )
        )}
      </ul>

      {/* Tablet / Desktop */}
      <div className="hidden md:block">
        {hasManyProjects ? (
          // Slider on md+ when we have more than 3 projects
          <div className="relative">
            <ul className="flex snap-x snap-mandatory gap-6 overflow-x-auto scroll-smooth pb-4">
              {projects.map(
                (project: GitHubProject): JSX.Element =>
                  renderProject(
                    project,
                    'snap-start flex-none w-[18rem] md:w-[20rem] lg:w-[22rem]'
                  )
              )}
            </ul>
          </div>
        ) : (
          // No slider needed: regular grid
          <ul className="grid grid-cols-2 gap-6 lg:grid-cols-3">
            {projects.map(
              (project: GitHubProject): JSX.Element => (
                <li key={project.html_url}>
                  <ProjectCard project={project} translations={translations} />
                </li>
              )
            )}
          </ul>
        )}
      </div>
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
  readonly locale: Locale
}

export const ProjectsSection: FCAsync<ProjectsSectionProperties> = async ({
  locale,
}: ProjectsSectionProperties): Promise<JSX.Element> => {
  const { contributionData, projects, stats }: GitHubData =
    await getGithubUser()

  const translations: Translations<'projects'> = await getTranslations({
    locale,
    namespace: 'projects',
  })

  return (
    <section
      className="min-h-screen bg-gradient-to-b from-background to-muted/20 px-4 py-20 md:px-8"
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
            icon={<Code2 className="h-8 w-8 text-primary" />}
            label={translations('stats.repositories')}
            value={stats.repositories}
          />
          <StatsCard
            icon={<Star className="h-8 w-8 text-primary" />}
            label={translations('stats.stars')}
            value={stats.stars}
          />
          <StatsCard
            icon={<GitFork className="h-8 w-8 text-primary" />}
            label={translations('stats.forks')}
            value={stats.forks}
          />
        </div>

        {/* Featured Projects */}
        <ProjectsGrid projects={projects} translations={translations} />

        {/* GitHub Contribution Graph - Hidden in reader mode/print */}
        <aside aria-hidden="true" className="mt-16 print:hidden">
          <ContributionGraph data={contributionData} locale={locale} />
        </aside>

        {/* View All Projects Button */}
        <SectionFooter
          cta={translations('viewAll')}
          githubUsername={siteConfig.githubUsername}
        />
      </div>
    </section>
  )
}
