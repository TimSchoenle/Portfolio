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
import {
  Card,
  CARD_DECORATIONS,
  CARD_HOVERS,
  CARD_VARIANTS,
} from '@/components/ui/card'
import { GridPattern } from '@/components/ui/grid-pattern'
import { Heading } from '@/components/ui/heading'
import { Section, SECTION_BACKGROUNDS } from '@/components/ui/section'
import { SectionContainer } from '@/components/ui/section-container'
import { SectionHeader } from '@/components/ui/section-header'
import { siteConfig } from '@/lib/config'
import { getGithubUser, type GitHubData } from '@/lib/github/client'
import type { GitHubProject } from '@/models/github'
import type { FCAsync, FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'

/* --------------------------------- Constants -------------------------------- */

const VISIBLE_PROJECTS_LIMIT: number = 3
const BG_GRID_SIZE: number = 32
const BG_PATTERN_SIZE: number = 16
const BG_PATTERN_OPACITY: number = 50

/* --------------------------------- pieces --------------------------------- */

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
    <Card
      className="p-6"
      decorative={CARD_DECORATIONS.OVERLAY}
      hover={CARD_HOVERS.MODERATE}
      variant={CARD_VARIANTS.INTERACTIVE}
    >
      <div className="relative flex items-center gap-4">
        <div className="rounded-xl bg-gradient-to-br from-primary/20 to-primary/10 p-3 shadow-lg ring-2 ring-primary/10 transition-all duration-300 group-hover:scale-110 group-hover:ring-primary/30">
          {icon}
        </div>
        <p className="flex flex-col">
          <span className="bg-gradient-to-r from-foreground to-foreground/80 bg-clip-text text-3xl font-bold text-transparent">
            {value}
          </span>
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
    <Card
      className="flex h-full flex-col transition-all duration-500"
      hover={CARD_HOVERS.MODERATE}
      variant={CARD_VARIANTS.INTERACTIVE}
    >
      <div className="relative h-48 overflow-hidden bg-gradient-to-br from-primary/30 via-primary/20 to-primary/10">
        {/* Animated background pattern */}
        <div
          className="absolute inset-0 bg-[linear-gradient(to_right,#ffffff08_1px,transparent_1px),linear-gradient(to_bottom,#ffffff08_1px,transparent_1px)]"
          style={{
            backgroundSize: `${BG_PATTERN_SIZE.toString()}px ${BG_PATTERN_SIZE.toString()}px`,
            opacity: BG_PATTERN_OPACITY / 100,
          }}
        />

        {/* Code icon with better animation */}
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="relative">
            {/* Main icon */}
            <Code2 className="relative h-20 w-20 text-primary/60 transition-all duration-500 group-hover:scale-125 group-hover:rotate-6 group-hover:text-primary" />
          </div>
        </div>

        {/* Subtle gradient overlay */}
        <div className="absolute inset-0 bg-gradient-to-t from-background/80 via-transparent to-transparent" />
      </div>

      <div className="relative flex flex-1 flex-col p-6">
        <Heading
          as="h3"
          className="mb-2 bg-gradient-to-r from-foreground via-foreground to-foreground/70 bg-clip-text text-xl font-bold text-transparent transition-all duration-300"
        >
          {project.name}
        </Heading>
        <p className="mb-4 flex-1 leading-relaxed text-muted-foreground">
          {project.description}
        </p>

        <div className="mb-4 flex flex-wrap gap-2">
          {project.topics.map(
            (topic: string): JSX.Element => (
              <Badge
                className="border-primary/20 bg-primary/5 text-xs transition-all hover:border-primary/40 hover:bg-primary/10"
                key={topic}
                variant="secondary"
              >
                {topic}
              </Badge>
            )
          )}
        </div>

        <div className="flex items-center justify-between border-t border-border/50 pt-4">
          <div
            aria-hidden="true"
            className="flex items-center gap-4 text-sm text-muted-foreground print:hidden"
          >
            <span className="flex items-center gap-1 transition-colors group-hover:text-foreground">
              <Star className="h-4 w-4" />
              {project.stargazers_count}
            </span>
            <span className="flex items-center gap-1 transition-colors group-hover:text-foreground">
              <GitFork className="h-4 w-4" />
              {project.forks_count}
            </span>
          </div>

          <div className="flex gap-2">
            <Button
              asChild={true}
              className="transition-all hover:scale-110 hover:bg-primary/10"
              size="sm"
              variant="ghost"
            >
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
              <Button
                asChild={true}
                className="transition-all hover:scale-110 hover:bg-primary/10"
                size="sm"
                variant="ghost"
              >
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
  const hasManyProjects: boolean = projects.length > VISIBLE_PROJECTS_LIMIT
  const mobileProjects: readonly GitHubProject[] = projects.slice(
    0,
    VISIBLE_PROJECTS_LIMIT
  )

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
      <Button
        asChild={true}
        className="group shadow-lg transition-all hover:scale-105 hover:shadow-2xl"
        size="lg"
      >
        <a
          href={`https://github.com/${githubUsername}`}
          rel="noopener noreferrer"
          target="_blank"
        >
          {cta}
          <ExternalLink className="ml-2 h-4 w-4 transition-all duration-300 group-hover:translate-x-1 group-hover:scale-110" />
        </a>
      </Button>
    </div>
  )
}

interface StatsGridProperties {
  readonly stats: GitHubData['stats']
  readonly translations: Translations<'projects'>
}

const StatsGrid: FCStrict<StatsGridProperties> = ({
  stats,
  translations,
}: StatsGridProperties): JSX.Element => (
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
)

/* ------------------------------- main export ------------------------------ */

interface ProjectsSectionProperties {
  readonly locale: Locale
  readonly performance?: boolean
}

export const ProjectsSection: FCAsync<ProjectsSectionProperties> = async ({
  locale,
  performance,
}: ProjectsSectionProperties): Promise<JSX.Element> => {
  const { contributionData, projects, stats }: GitHubData =
    await getGithubUser()

  const translations: Translations<'projects'> = await getTranslations({
    locale,
    namespace: 'projects',
  })

  // Contribution graph logic
  const hasContributions: boolean = Object.keys(contributionData).length > 0

  return (
    <Section
      background={SECTION_BACKGROUNDS.GRADIENT}
      className="min-h-screen"
      id="projects"
      isEmpty={projects.length === 0}
      performance={performance ?? false}
    >
      {/* Background patterns */}
      <GridPattern size={BG_GRID_SIZE} />

      <SectionContainer className="relative" size="xl">
        <SectionHeader
          gradient={true}
          subtitle={translations('subtitle')}
          title={translations('title')}
        />

        {/* GitHub Stats Cards */}
        <StatsGrid stats={stats} translations={translations} />

        {/* Featured Projects */}
        <ProjectsGrid projects={projects} translations={translations} />

        {/* GitHub Contribution Graph - Hidden in reader mode/print, or if no data */}
        <aside aria-hidden="true" className="mt-16 print:hidden">
          {hasContributions ? (
            <ContributionGraph data={contributionData} locale={locale} />
          ) : null}
        </aside>

        <SectionFooter
          cta={translations('viewAll')}
          githubUsername={siteConfig.socials.githubUsername}
        />
      </SectionContainer>
    </Section>
  )
}
