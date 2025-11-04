'use server'

import { type Locale } from 'next-intl'
import { getTranslations } from 'next-intl/server'

import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { siteConfig } from '@/lib/config'

export const SkillsSection = ({ locale }: { locale: Locale }) => {
  const t = await getTranslations({ locale, namespace: 'skills' })

  return (
    <section className="bg-muted/30 relative px-4 py-20" id="skills">
      <div className="absolute inset-0 -z-10 bg-[linear-gradient(to_right,#8080800a_1px,transparent_1px),linear-gradient(to_bottom,#8080800a_1px,transparent_1px)] bg-[size:14px_24px]" />

      <div className="mx-auto w-full max-w-6xl">
        <div className="mb-12 text-center">
          <h2 className="text-foreground mb-3 text-4xl font-bold">
            {t('title')}
          </h2>
          <div className="from-primary to-primary/60 mx-auto h-1 w-20 rounded-full bg-gradient-to-r" />
        </div>

        <div className="grid gap-6 md:grid-cols-3">
          <Card className="group hover:border-primary/50 border-2 transition-all duration-300 hover:-translate-y-1 hover:shadow-xl">
            <CardHeader>
              <CardTitle className="text-foreground flex items-center gap-2">
                <div className="bg-primary h-2 w-2 rounded-full" />
                {t('expertise')}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2">
                {siteConfig.skills.expertise.map((skill, index) => (
                  <Badge
                    className="transition-all hover:scale-105 hover:shadow-md"
                    key={skill}
                    style={{ animationDelay: `${index * 50}ms` }}
                    variant="default"
                  >
                    {skill}
                  </Badge>
                ))}
              </div>
            </CardContent>
          </Card>

          <Card className="group hover:border-primary/50 border-2 transition-all duration-300 hover:-translate-y-1 hover:shadow-xl">
            <CardHeader>
              <CardTitle className="text-foreground flex items-center gap-2">
                <div className="bg-primary h-2 w-2 rounded-full" />
                {t('learning')}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2">
                {siteConfig.skills.learning.map((skill, index) => (
                  <Badge
                    className="transition-all hover:scale-105 hover:shadow-md"
                    key={skill}
                    style={{ animationDelay: `${index * 50}ms` }}
                    variant="secondary"
                  >
                    {skill}
                  </Badge>
                ))}
              </div>
            </CardContent>
          </Card>

          <Card className="group hover:border-primary/50 border-2 transition-all duration-300 hover:-translate-y-1 hover:shadow-xl">
            <CardHeader>
              <CardTitle className="text-foreground flex items-center gap-2">
                <div className="bg-primary h-2 w-2 rounded-full" />
                {t('tools')}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2">
                {siteConfig.skills.tools.map((skill, index) => (
                  <Badge
                    className="hover:bg-primary/5 transition-all hover:scale-105 hover:shadow-md"
                    key={skill}
                    style={{ animationDelay: `${index * 50}ms` }}
                    variant="outline"
                  >
                    {skill}
                  </Badge>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </section>
  )
}
