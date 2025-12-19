import { type ComponentType, type JSX } from 'react'

import { Folder, GitFork, Star } from 'lucide-react'
import dynamic from 'next/dynamic'
import { getTranslations } from 'next-intl/server'

import { BlueprintCard } from '@/components/blueprint/blueprint-card'
import { BlueprintContainer } from '@/components/blueprint/blueprint-container'
import { BlueprintSectionDivider } from '@/components/blueprint/blueprint-section-divider'
import { BlueprintSectionTitle } from '@/components/blueprint/blueprint-section-title'
import type * as ContributionGraphModule from '@/components/features/contribution-graph/contribution-graph-client'
import { Card } from '@/components/ui/card'
import { siteConfig } from '@/lib/config'
import { getGithubUser, type GitHubData } from '@/lib/github/client'
import type { GitHubProject } from '@/models/github'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { LocalePageProperties, Translations } from '@/types/i18n'

/* ── types ─────────────────────────────────────────────────────────────── */

type ProjectsSectionProperties = LocalePageProperties

interface ProjectCardProperties {
  readonly description: string
  readonly language: string
  readonly name: string
  readonly stats: {
    readonly forks: number
    readonly stars: number
  }
  readonly url: string
  readonly viewProject: string
}

const ContributionGraph: ComponentType<ContributionGraphModule.ContributionGraphClientProperties> =
  dynamic(
    async (): Promise<
      FCStrict<ContributionGraphModule.ContributionGraphClientProperties>
    > =>
      import('@/components/features/contribution-graph/contribution-graph-client').then(
        (
          module_: typeof ContributionGraphModule
        ): FCStrict<ContributionGraphModule.ContributionGraphClientProperties> =>
          module_.ContributionGraphClient
      ),
    {
      loading: (): JSX.Element => (
        <div className="h-[180px] w-full animate-pulse rounded-lg bg-blueprint-card-bg/50" />
      ),
    }
  )

/* ── subcomponents ─────────────────────────────────────────────────────── */

const BlueprintProjectCard: FCStrict<ProjectCardProperties> = ({
  description,
  language,
  name,
  stats,
  url,
  viewProject,
}: ProjectCardProperties): JSX.Element => (
  <BlueprintCard
    className="flex h-full flex-col"
    label="REPO_DATA"
    noPadding={true}
  >
    <div className="flex h-full flex-col p-6">
      {/* Header */}
      <div className="mb-4 flex items-start justify-between">
        <div className="flex items-center gap-3">
          <Folder className="h-5 w-5 text-brand" />
          <h3 className="font-mono text-lg font-bold tracking-tight text-blueprint-text">
            {name}
          </h3>
        </div>
      </div>

      {/* Description */}
      <p className="mb-6 line-clamp-3 flex-grow font-mono text-xs leading-relaxed text-blueprint-muted">
        {description}
      </p>

      {/* Footer / Stats */}
      <div className="mt-auto flex items-center justify-between border-t border-brand/30 pt-4 font-mono text-xs text-brand/80 uppercase">
        <div className="flex items-center gap-4">
          <span className="flex items-center gap-1">
            <div className="h-2 w-2 rounded-full bg-brand opacity-70" />
            {language}
          </span>
          <span className="flex items-center gap-1">
            <Star className="h-3 w-3" />
            {stats.stars}
          </span>
          <span className="flex items-center gap-1">
            <GitFork className="h-3 w-3" />
            {stats.forks}
          </span>
        </div>

        <a
          className="decoration-brand underline-offset-4 transition-colors hover:text-blueprint-text hover:underline"
          href={url}
          rel="noreferrer"
          target="_blank"
        >
          {viewProject} {'->'}
        </a>
      </div>
    </div>
  </BlueprintCard>
)

interface ProjectStatsProperties {
  readonly stats: {
    readonly forks: number
    readonly repositories: number
    readonly stars: number
  }
}

const REPO_TEXT: string = 'Repositories'
const STARS_TEXT: string = 'Total Stars'
const FORKS_TEXT: string = 'Total Forks'

const ProjectStats: FCStrict<ProjectStatsProperties> = ({
  stats,
}: ProjectStatsProperties): JSX.Element => (
  <div className="mt-8 grid w-full grid-cols-3 gap-4 md:w-2/3 lg:w-1/2">
    <Card className="flex flex-col items-center justify-center border-brand/30 bg-blueprint-bg/50 p-4">
      <span className="text-3xl font-bold text-blueprint-text">
        {stats.repositories}
      </span>
      <span className="font-mono text-xs text-blueprint-muted uppercase">
        {REPO_TEXT}
      </span>
    </Card>
    <Card className="flex flex-col items-center justify-center border-brand/30 bg-blueprint-bg/50 p-4">
      <span className="text-3xl font-bold text-blueprint-text">
        {stats.stars}
      </span>
      <span className="font-mono text-xs text-blueprint-muted uppercase">
        {STARS_TEXT}
      </span>
    </Card>
    <Card className="flex flex-col items-center justify-center border-brand/30 bg-blueprint-bg/50 p-4">
      <span className="text-3xl font-bold text-blueprint-text">
        {stats.forks}
      </span>
      <span className="font-mono text-xs text-blueprint-muted uppercase">
        {FORKS_TEXT}
      </span>
    </Card>
  </div>
)

interface ViewAllButtonProperties {
  readonly label: string
}

const VIEW_ALL_ARROW: string = '->'

const ViewAllButton: FCStrict<ViewAllButtonProperties> = ({
  label,
}: ViewAllButtonProperties): JSX.Element => (
  <div className="mt-12 flex justify-center">
    <a
      className="group relative inline-flex items-center justify-center"
      href={siteConfig.socials.github}
      rel="noreferrer"
      target="_blank"
    >
      {/* Glow Effect */}
      <div className="absolute inset-0 rounded-sm bg-brand/20 blur-md transition-all duration-300 group-hover:bg-brand/40" />

      {/* Button Content */}
      <div className="group-hover:shadow-[0_0_15px_color-mix(in srgb, var(--brand), transparent 70%)] relative flex items-center gap-2 border border-brand bg-blueprint-bg/90 px-8 py-3 font-mono text-sm tracking-wider text-brand backdrop-blur-sm transition-all duration-300 group-hover:text-blueprint-text hover:bg-brand/10">
        <span>{label}</span>
        <span className="transition-transform duration-300 group-hover:translate-x-1">
          {VIEW_ALL_ARROW}
        </span>
      </div>

      {/* Corner Accents */}
      <div className="absolute -top-[1px] -left-[1px] h-2 w-2 border-t border-l border-brand" />
      <div className="absolute -right-[1px] -bottom-[1px] h-2 w-2 border-r border-b border-brand" />
    </a>
  </div>
)

interface FeaturedProjectsProperties {
  readonly projects: readonly GitHubProject[]
  readonly translations: Translations<'projects'>
}

const FeaturedProjects: FCStrict<FeaturedProjectsProperties> = ({
  projects,
  translations,
}: FeaturedProjectsProperties): JSX.Element => (
  <div className="mt-12 grid w-full gap-6 md:grid-cols-2 lg:grid-cols-3">
    {projects.map(
      (project: GitHubProject): JSX.Element => (
        <BlueprintProjectCard
          description={project.description ?? ''}
          key={project.name}
          language={project.language ?? 'Unknown'}
          name={project.name}
          stats={{
            forks: project.forks_count,
            stars: project.stargazers_count,
          }}
          url={project.html_url}
          viewProject={translations('view')}
        />
      )
    )}
  </div>
)

/* ── main ──────────────────────────────────────────────────── */

export const ProjectsSection: AsyncPageFC<ProjectsSectionProperties> = async ({
  locale,
}: ProjectsSectionProperties): Promise<JSX.Element> => {
  const translations: Translations<'projects'> = await getTranslations({
    locale,
    namespace: 'projects',
  })

  // Fetch all user data including featured projects, stats, and contributions
  const { contributionData, projects, stats }: GitHubData =
    await getGithubUser()

  return (
    <BlueprintContainer id="projects" isLazy={true}>
      <div className="mx-auto flex w-full max-w-6xl flex-col items-center">
        <BlueprintSectionTitle
          sectionLabel="// OPEN_SOURCE_MODULES"
          title={translations('title')}
        />

        <ProjectStats stats={stats} />

        <FeaturedProjects
          projects={projects.slice(0, 6)}
          translations={translations}
        />

        <ViewAllButton label={translations('viewAll')} />

        {/* Contribution Graph - Scaled to fit container without scroll */}
        <div className="mt-16 w-full rounded-lg border border-brand/30 bg-blueprint-card-bg/90 p-2 shadow-sm backdrop-blur-md md:p-6">
          <div className="w-full">
            <ContributionGraph
              data={contributionData}
              locale={locale}
              variant="blueprint"
            />
          </div>
        </div>

        <BlueprintSectionDivider label="MODULES_LOADED" />
      </div>
    </BlueprintContainer>
  )
}

export default ProjectsSection
