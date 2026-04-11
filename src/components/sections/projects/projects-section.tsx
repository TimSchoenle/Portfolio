import { type JSX } from 'react'

import { getTranslations } from 'next-intl/server'

import { BlueprintContainer } from '@/components/blueprint/blueprint-container'
import { BlueprintSectionDivider } from '@/components/blueprint/blueprint-section-divider'
import { BlueprintSectionTitle } from '@/components/blueprint/blueprint-section-title'
import { ContributionGraphClient } from '@/components/features/contribution-graph/contribution-graph-client'
import { FeaturedProjects } from '@/components/sections/projects/featured-projects'
import { ProjectStats } from '@/components/sections/projects/project-stats'
import { ViewAllButton } from '@/components/sections/projects/view-all-button'
import { getGithubUser, type GitHubData } from '@/lib/github/client'
import type { AsyncPageFC } from '@/types/fc'
import type { LocalePageProperties, Translations } from '@/types/i18n'

type ProjectsSectionProperties = LocalePageProperties

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
        {Object.keys(contributionData).length > 0 ? (
          <div className="mt-16 w-full rounded-lg border border-brand/30 bg-blueprint-card-bg/90 p-2 shadow-sm backdrop-blur-md md:p-6">
            <div className="w-full">
              <ContributionGraphClient
                data={contributionData}
                locale={locale}
                variant="blueprint"
              />
            </div>
          </div>
        ) : null}

        <BlueprintSectionDivider label="MODULES_LOADED" />
      </div>
    </BlueprintContainer>
  )
}

export default ProjectsSection
