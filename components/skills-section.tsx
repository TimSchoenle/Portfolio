import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

const skills = {
  expertise: ['Java', 'Spring Boot', 'Maven', 'Gradle'],
  learning: ['Rust', 'Next.js', 'React', 'TypeScript'],
  tools: ['Git', 'GitHub', 'Docker', 'Linux'],
}

export function SkillsSection({ dict }: { dict: any }) {
  return (
    <section id="skills" className="bg-muted/30 relative px-4 py-20">
      <div className="absolute inset-0 -z-10 bg-[linear-gradient(to_right,#8080800a_1px,transparent_1px),linear-gradient(to_bottom,#8080800a_1px,transparent_1px)] bg-[size:14px_24px]" />

      <div className="mx-auto w-full max-w-6xl">
        <div className="mb-12 text-center">
          <h2 className="text-foreground mb-3 text-4xl font-bold">
            {dict.skills.title}
          </h2>
          <div className="from-primary to-primary/60 mx-auto h-1 w-20 rounded-full bg-gradient-to-r" />
        </div>

        <div className="grid gap-6 md:grid-cols-3">
          <Card className="group hover:border-primary/50 border-2 transition-all duration-300 hover:-translate-y-1 hover:shadow-xl">
            <CardHeader>
              <CardTitle className="text-foreground flex items-center gap-2">
                <div className="bg-primary h-2 w-2 rounded-full" />
                {dict.skills.expertise}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2">
                {skills.expertise.map((skill, index) => (
                  <Badge
                    key={skill}
                    variant="default"
                    className="transition-all hover:scale-105 hover:shadow-md"
                    style={{ animationDelay: `${index * 50}ms` }}
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
                {dict.skills.learning}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2">
                {skills.learning.map((skill, index) => (
                  <Badge
                    key={skill}
                    variant="secondary"
                    className="transition-all hover:scale-105 hover:shadow-md"
                    style={{ animationDelay: `${index * 50}ms` }}
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
                {dict.skills.tools}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2">
                {skills.tools.map((skill, index) => (
                  <Badge
                    key={skill}
                    variant="outline"
                    className="hover:bg-primary/5 transition-all hover:scale-105 hover:shadow-md"
                    style={{ animationDelay: `${index * 50}ms` }}
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
