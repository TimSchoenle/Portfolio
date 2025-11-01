'use client'

import { Card } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { ExternalLink, Github, Star, GitFork } from 'lucide-react'
import { type GitHubProject } from '@/lib/github'
import { type ProjectsDictionary } from '@/lib/dictionary'
import { ContributionGraph } from '@/components/contribution-graph'
import { type Locale } from '@/lib/i18n-config'

interface ProjectsSectionProps {
  locale: Locale
  githubUsername: string
  dict: ProjectsDictionary
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

export function ProjectsSection({
  locale,
  githubUsername,
  dict,
  projects,
  stats,
  contributionData,
}: ProjectsSectionProps) {
  return (
    <section
      id="projects"
      className="from-background to-muted/20 min-h-screen bg-gradient-to-b px-4 py-20 md:px-8"
    >
      <div className="mx-auto max-w-7xl">
        <div className="mb-16 text-center">
          <h2 className="from-primary to-primary/60 mb-4 bg-gradient-to-r bg-clip-text text-4xl font-bold text-transparent md:text-5xl">
            {dict.title}
          </h2>
          <p className="text-muted-foreground mx-auto max-w-2xl text-lg">
            {dict.subtitle}
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
                  {dict.stats.repositories}
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
                  {dict.stats.stars}
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
                  {dict.stats.forks}
                </p>
              </div>
            </div>
          </Card>
        </div>

        {/* Featured Projects */}
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
          {projects.map((project, index) => (
            <Card
              key={index}
              className="group hover:border-primary/50 flex flex-col overflow-hidden border-2 transition-all duration-300 hover:shadow-xl"
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
                    <Badge key={topic} variant="secondary" className="text-xs">
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
                    <Button size="sm" variant="ghost" asChild>
                      <a
                        href={project.html_url}
                        target="_blank"
                        rel="noopener noreferrer"
                      >
                        <Github className="h-4 w-4" />
                      </a>
                    </Button>
                    {project.homepage && (
                      <Button size="sm" variant="ghost" asChild>
                        <a
                          href={project.homepage}
                          target="_blank"
                          rel="noopener noreferrer"
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
          <ContributionGraph
            title={dict.contributions}
            data={contributionData}
            locale={locale}
            dict={dict.contributions}
          />
        </div>

        {/* View All Projects Button */}
        <div className="mt-12 text-center">
          <Button size="lg" asChild className="group">
            <a
              href={`https://github.com/${githubUsername}`}
              target="_blank"
              rel="noopener noreferrer"
            >
              {dict.viewAll}
              <ExternalLink className="ml-2 h-4 w-4 transition-transform group-hover:translate-x-1" />
            </a>
          </Button>
        </div>
      </div>
    </section>
  )
}
