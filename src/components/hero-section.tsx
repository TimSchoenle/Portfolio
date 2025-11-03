'use client'

import { Button } from '@/components/ui/button'
import { Mail, Github, ArrowDown } from 'lucide-react'
import { useTranslations } from 'next-intl'

export function HeroSection() {
  const t = useTranslations('hero')

  return (
    <section className="relative flex h-screen min-h-screen snap-start items-center justify-center px-4 py-20">
      <div className="from-primary/5 absolute inset-0 -z-10 bg-gradient-to-b via-transparent to-transparent" />

      <div className="max-w-4xl text-center">
        <h1 className="text-foreground animate-in fade-in slide-in-from-bottom-4 mb-6 text-5xl font-bold tracking-tight text-balance duration-1000 md:text-7xl">
          {t('greeting')}{' '}
          <span
            className="gradient-text-protected from-primary to-primary/60 bg-gradient-to-r bg-clip-text text-transparent"
            data-darkreader-inline-bgcolor=""
            data-darkreader-inline-color=""
          >
            {t('name')}
          </span>
        </h1>
        <p className="text-muted-foreground animate-in fade-in slide-in-from-bottom-4 mb-3 text-xl text-pretty delay-150 duration-1000 md:text-2xl">
          {t('title')}
        </p>
        <p className="text-muted-foreground animate-in fade-in slide-in-from-bottom-4 mb-10 text-lg delay-300 duration-1000">
          {t('location')} • {t('tagline')}
        </p>

        <div className="animate-in fade-in slide-in-from-bottom-4 flex flex-wrap items-center justify-center gap-4 delay-500 duration-1000">
          <Button
            asChild
            size="lg"
            className="group shadow-lg transition-all hover:shadow-xl"
          >
            <a
              href="https://github.com/Timmi6790"
              target="_blank"
              rel="noopener noreferrer"
            >
              <Github className="mr-2 h-5 w-5 transition-transform group-hover:scale-110" />
              {t('github')}
            </a>
          </Button>
          <Button
            asChild
            variant="outline"
            size="lg"
            className="group bg-transparent shadow-md transition-all hover:shadow-lg"
          >
            <a href="mailto:contact@timmi6790.de">
              <Mail className="mr-2 h-5 w-5 transition-transform group-hover:scale-110" />
              {t('contact')}
            </a>
          </Button>
        </div>

        <div className="animate-in fade-in mt-16 delay-700 duration-1000">
          <ArrowDown className="text-muted-foreground mx-auto h-6 w-6 animate-bounce" />
        </div>
      </div>
    </section>
  )
}
