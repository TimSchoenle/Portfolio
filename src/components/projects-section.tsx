'use server'

import { ExternalLink, Github, Star, GitFork } from 'lucide-react'
import { type Locale } from 'next-intl'
import { getTranslations } from 'next-intl/server'

import { ContributionGraph } from '@/components/contribution-graph'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { type GitHubProject } from '@/lib/github'


interface ProjectsSectionProps {
  locale: Locale
  githubUsername: string
  projects: GitHubProject[]
  stats: {
    repositories: number
    stars: number
    forks: number
  }
  contributionData: Array<{
    date: string
    count: number
    level: 0 | 1 | 2 | 3 | 4
  }>
}

export const ProjectsSection = ({
  locale,
  githubUsername,
  projects,
  stats,
  contributionData,
}: ProjectsSectionProps) => {
  const t = await getTranslations({ locale, namespace: 'projects' })

  return (
    <section
      className="from-background to-muted/20 min-h-screen bg-gradient-to-b px-4 py-20 md:px-8"
      id="projects"
    >
      <div className="mx-auto max-w-7xl">
        <div className="mb-16 text-center">
          <h2 className="from-primary to-primary/60 mb-4 bg-gradient-to-r bg-clip-text text-4xl font-bold text-transparent md:text-5xl">
            {t('title')}
          </h2>
          <p className="text-muted-foreground mx-auto max-w-2xl text-lg">
            {t('subtitle')}
          </p>
        </div>

        {/* GitHub Stats Cards */}
        <div className="mb-16 grid grid-cols-1 gap-6 md:grid-cols-3">
          <Card className="hover:border-primary/50 border-2 p-6 transition-all duration-300 hover:shadow-lg">
            <div className="flex items-center gap-4">
              <div className="bg-primary/10 rounded-lg p-3">
                <Github className="text-primary h-8 w-8" />
              </div>
              <div>
                <p className="text-3xl font-bold">{stats.repositories}</p>
                <p className="text-muted-foreground text-sm">
                  {t('stats.repositories')}
                </p>
              </div>
            </div>
          </Card>

          <Card className="hover:border-primary/50 border-2 p-6 transition-all duration-300 hover:shadow-lg">
            <div className="flex items-center gap-4">
              <div className="bg-primary/10 rounded-lg p-3">
                <Star className="text-primary h-8 w-8" />
              </div>
              <div>
                <p className="text-3xl font-bold">{stats.stars}</p>
                <p className="text-muted-foreground text-sm">
                  {t('stats.stars')}
                </p>
              </div>
            </div>
          </Card>

          <Card className="hover:border-primary/50 border-2 p-6 transition-all duration-300 hover:shadow-lg">
            <div className="flex items-center gap-4">
              <div className="bg-primary/10 rounded-lg p-3">
                <GitFork className="text-primary h-8 w-8" />
              </div>
              <div>
                <p className="text-3xl font-bold">{stats.forks}</p>
                <p className="text-muted-foreground text-sm">
                  {t('stats.forks')}
                </p>
              </div>
            </div>
          </Card>
        </div>

        {/* Featured Projects */}
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
          {projects.map((project, index) => (
            <Card
              className="group hover:border-primary/50 flex flex-col overflow-hidden border-2 transition-all duration-300 hover:shadow-xl"
              key={index}
            >
              <div className="from-primary/20 to-primary/5 relative h-48 overflow-hidden bg-gradient-to-br">
                <div className="absolute inset-0 flex items-center justify-center">
                  <Github className="text-primary/40 h-20 w-20 transition-transform duration-300 group-hover:scale-110" />
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
                  {project.topics.map((topic) => (
                    <Badge className="text-xs" key={topic} variant="secondary">
                      {topic}
                    </Badge>
                  ))}
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
                        <Github className="h-4 w-4" />
                      </a>
                    </Button>
                    {project.homepage && (
                      <Button asChild={true} size="sm" variant="ghost">
                        <a
                          href={project.homepage}
                          rel="noopener noreferrer"
                          target="_blank"
                        >
                          <ExternalLink className="h-4 w-4" />
                        </a>
                      </Button>
                    )}
                  </div>
                </div>
              </div>
            </Card>
          ))}
        </div>

        {/* GitHub Contribution Graph */}
        <div className="mt-16">
          <ContributionGraph data={contributionData} locale={locale} />
        </div>

        {/* View All Projects Button */}
        <div className="mt-12 text-center">
          <Button asChild={true} className="group" size="lg">
            <a
              href={`https://github.com/${githubUsername}`}
              rel="noopener noreferrer"
              target="_blank"
            >
              {t('viewAll')}
              <ExternalLink className="ml-2 h-4 w-4 transition-transform group-hover:translate-x-1" />
            </a>
          </Button>
        </div>
      </div>
    </section>
  )
}
