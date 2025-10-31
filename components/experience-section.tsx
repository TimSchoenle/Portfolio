import { Card, CardContent } from '@/components/ui/card'
import { Briefcase, Calendar } from 'lucide-react'
import Image from 'next/image'

interface Experience {
  company: string
  logo: string
  title: string
  dateRange: string
  description: string
}

export function ExperienceSection({ dict }: { dict: any }) {
  const experiences: Experience[] = dict.experience.items

  return (
    <section className="px-4 py-20">
      <div className="mx-auto w-full max-w-4xl">
        <div className="mb-12 text-center">
          <h2 className="text-foreground mb-3 text-4xl font-bold">
            {dict.experience.title}
          </h2>
          <div className="from-primary to-primary/60 mx-auto h-1 w-20 rounded-full bg-gradient-to-r" />
        </div>

        <div className="space-y-6">
          {experiences.map((exp, index) => (
            <Card
              key={index}
              className="group hover:border-primary/50 border-2 transition-all duration-300 hover:-translate-y-1 hover:shadow-xl"
            >
              <CardContent className="p-6">
                <div className="flex gap-6">
                  <div className="flex-shrink-0">
                    <div className="border-border bg-muted relative h-16 w-16 overflow-hidden rounded-xl border-2 shadow-md transition-transform duration-300 group-hover:scale-105 group-hover:shadow-lg">
                      {exp.logo ? (
                        <Image
                          src={exp.logo || '/placeholder.svg'}
                          alt={`${exp.company} logo`}
                          fill
                          className="object-cover"
                        />
                      ) : (
                        <div className="from-primary/10 to-primary/5 flex h-full w-full items-center justify-center bg-gradient-to-br">
                          <Briefcase className="text-primary h-8 w-8" />
                        </div>
                      )}
                    </div>
                  </div>

                  <div className="min-w-0 flex-1">
                    <h3 className="text-foreground group-hover:text-primary mb-1 text-xl font-semibold transition-colors">
                      {exp.title}
                    </h3>
                    <p className="text-foreground/80 mb-2 text-base font-medium">
                      {exp.company}
                    </p>
                    <div className="text-muted-foreground mb-4 flex items-center gap-2 text-sm">
                      <Calendar className="h-4 w-4" />
                      <p>{exp.dateRange}</p>
                    </div>

                    <div className="bg-muted/50 hover:bg-muted/70 scrollbar-thin scrollbar-thumb-primary/20 scrollbar-track-transparent max-h-28 overflow-y-auto rounded-lg border-2 p-4 shadow-inner transition-colors">
                      <p className="text-foreground/90 text-sm leading-relaxed whitespace-pre-line">
                        {exp.description}
                      </p>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </section>
  )
}
