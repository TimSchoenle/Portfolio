'use server'

import { BookOpen, Code2 } from 'lucide-react'
import { type Locale } from 'next-intl'
import { getTranslations } from 'next-intl/server'

import { Card, CardContent } from '@/components/ui/card'

export const AboutSection = ({ locale }: { locale: Locale }) => {
  const t = await getTranslations({ locale, namespace: 'about' })

  return (
    <section className="px-4 py-20" id="about">
      <div className="mx-auto w-full max-w-6xl">
        <div className="mb-12 text-center">
          <h2 className="text-foreground mb-3 text-4xl font-bold">
            {t('title')}
          </h2>
          <div className="from-primary to-primary/60 mx-auto h-1 w-20 rounded-full bg-gradient-to-r" />
        </div>

        <div className="grid gap-6 md:grid-cols-2">
          <Card className="group hover:border-primary/50 border-2 transition-all duration-300 hover:shadow-xl">
            <CardContent className="flex items-start gap-4 p-6">
              <div className="from-primary/10 to-primary/5 rounded-lg bg-gradient-to-br p-3 transition-transform duration-300 group-hover:scale-110">
                <BookOpen className="text-primary h-6 w-6" />
              </div>
              <div>
                <h3 className="text-foreground mb-2 text-xl font-semibold">
                  {t('learning.title')}
                </h3>
                <p className="text-muted-foreground leading-relaxed">
                  {t.rich('learning.description', {
                    highlight: (chunks) => (
                      <span className="text-foreground font-medium">
                        {chunks}
                      </span>
                    ),
                  })}
                </p>
              </div>
            </CardContent>
          </Card>

          <Card className="group hover:border-primary/50 border-2 transition-all duration-300 hover:shadow-xl">
            <CardContent className="flex items-start gap-4 p-6">
              <div className="from-primary/10 to-primary/5 rounded-lg bg-gradient-to-br p-3 transition-transform duration-300 group-hover:scale-110">
                <Code2 className="text-primary h-6 w-6" />
              </div>
              <div>
                <h3 className="text-foreground mb-2 text-xl font-semibold">
                  {t('expertise.title')}
                </h3>
                <p className="text-muted-foreground leading-relaxed">
                  {t.rich('expertise.description', {
                    highlight: (chunks) => (
                      <span className="text-foreground font-medium">
                        {chunks}
                      </span>
                    ),
                  })}
                </p>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </section>
  )
}
