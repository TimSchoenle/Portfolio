import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Mail, Github, MapPin, Download, FileText } from 'lucide-react'
import Link from 'next/link'
import type { Locale } from '@/lib/i18n-config'
import { ContactDictionary } from '@/lib/dictionary'

export function ContactSection({
  dict,
  locale,
}: {
  dict: ContactDictionary
  locale: Locale
}) {
  const resumePath = locale === 'de' ? '/resume-de.pdf' : '/resume-en.pdf'

  return (
    <section id="contact" className="bg-muted/30 relative px-4 py-20">
      <div className="absolute inset-0 -z-10 bg-[linear-gradient(to_right,#8080800a_1px,transparent_1px),linear-gradient(to_bottom,#8080800a_1px,transparent_1px)] bg-[size:14px_24px]" />

      <div className="mx-auto w-full max-w-4xl">
        <div className="mb-12 text-center">
          <h2 className="text-foreground mb-3 text-4xl font-bold">
            {dict.title}
          </h2>
          <div className="from-primary to-primary/60 mx-auto h-1 w-20 rounded-full bg-gradient-to-r" />
        </div>

        <div className="mx-auto max-w-2xl space-y-6">
          <Card className="border-2 shadow-xl">
            <CardContent className="p-8">
              <h3 className="mb-6 text-2xl font-bold">
                {dict.infoTitle}
              </h3>
              <div className="space-y-6">
                <div className="group hover:bg-muted/50 flex items-center gap-4 rounded-lg p-3 transition-all">
                  <div className="from-primary/10 to-primary/5 rounded-lg bg-gradient-to-br p-3 transition-transform duration-300 group-hover:scale-110">
                    <Mail className="text-primary h-6 w-6" />
                  </div>
                  <div>
                    <p className="text-muted-foreground text-sm">
                      {dict.email}
                    </p>
                    <a
                      href="mailto:contact@timmi6790.de"
                      className="text-foreground hover:text-primary text-lg font-medium transition-colors"
                    >
                      contact@timmi6790.de
                    </a>
                  </div>
                </div>

                <div className="group hover:bg-muted/50 flex items-center gap-4 rounded-lg p-3 transition-all">
                  <div className="from-primary/10 to-primary/5 rounded-lg bg-gradient-to-br p-3 transition-transform duration-300 group-hover:scale-110">
                    <Github className="text-primary h-6 w-6" />
                  </div>
                  <div>
                    <p className="text-muted-foreground text-sm">
                      {dict.github}
                    </p>
                    <a
                      href="https://github.com/Timmi6790"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-foreground hover:text-primary text-lg font-medium transition-colors"
                    >
                      @Timmi6790
                    </a>
                  </div>
                </div>

                <div className="group hover:bg-muted/50 flex items-center gap-4 rounded-lg p-3 transition-all">
                  <div className="from-primary/10 to-primary/5 rounded-lg bg-gradient-to-br p-3 transition-transform duration-300 group-hover:scale-110">
                    <MapPin className="text-primary h-6 w-6" />
                  </div>
                  <div>
                    <p className="text-muted-foreground text-sm">
                      {dict.location}
                    </p>
                    <p className="text-foreground text-lg font-medium">
                      {dict.locationValue}
                    </p>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card className="overflow-hidden border-2 shadow-xl">
            <div className="from-primary/20 via-primary/10 to-primary/5 bg-gradient-to-br p-8">
              <div className="mb-4 flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className="bg-primary/10 rounded-lg p-3">
                    <FileText className="text-primary h-6 w-6" />
                  </div>
                  <div>
                    <h3 className="text-foreground text-xl font-bold">
                      {dict.downloadResume}
                    </h3>
                    <p className="text-muted-foreground text-sm">
                      PDF • {locale === 'de' ? 'Deutsch' : 'English'}
                    </p>
                  </div>
                </div>
              </div>
              <Button
                asChild
                className="group bg-primary hover:bg-primary/90 w-full shadow-lg transition-all hover:shadow-xl"
                size="lg"
              >
                <a href={resumePath} download>
                  <Download className="mr-2 h-5 w-5 transition-transform group-hover:translate-y-0.5 group-hover:scale-110" />
                  {dict.downloadResume}
                </a>
              </Button>
            </div>
          </Card>
        </div>

        <div className="mt-8 text-center">
          <Link
            href={`/${locale}/imprint`}
            className="text-muted-foreground hover:text-primary text-sm transition-colors hover:underline"
          >
            {dict.imprint}
          </Link>
        </div>
      </div>
    </section>
  )
}
