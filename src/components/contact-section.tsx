'use server'

import { type JSX } from 'react'

import { type Locale } from 'next-intl'

import { Download, FileText, GitBranch, Mail, MapPin } from 'lucide-react'
import { getTranslations } from 'next-intl/server'

import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import { siteConfig } from '@/config'
import type { FCAsync, FCStrict, NoChildren } from '@/types/fc'
import type { Translations } from '@/types/i18n'

/* ─────────────────────────────── types ─────────────────────────────── */

interface ContactSectionProperties extends NoChildren {
  readonly locale: Locale
}

interface InfoItemProperties {
  readonly content: JSX.Element
  readonly icon: JSX.Element
  readonly label: string
}

interface InfoCardProperties {
  readonly translations: Translations<'contact'>
}

interface ResumeCardProperties {
  readonly locale: Locale
  readonly translations: Translations<'contact'>
}

/* ───────────────────────────── subviews ───────────────────────────── */

const InfoItem: FCStrict<InfoItemProperties> = ({
  content,
  icon,
  label,
}: InfoItemProperties): JSX.Element => {
  return (
    <div className="group hover:bg-muted/50 flex items-center gap-4 rounded-lg p-3 transition-all">
      <div className="from-primary/10 to-primary/5 rounded-lg bg-gradient-to-br p-3 transition-transform duration-300 group-hover:scale-110">
        {icon}
      </div>
      <div>
        <p className="text-muted-foreground text-sm">{label}</p>
        {content}
      </div>
    </div>
  )
}

const InfoCard: FCStrict<InfoCardProperties> = ({
  translations,
}: InfoCardProperties): JSX.Element => {
  return (
    <Card className="border-2 shadow-xl">
      <CardContent className="p-8">
        <Heading as="h3" className="mb-6 text-2xl font-bold">
          {translations('infoTitle')}
        </Heading>

        <div className="space-y-6">
          <InfoItem
            content={
              <a
                className="text-foreground hover:text-primary text-lg font-medium transition-colors"
                href={`mailto:${siteConfig.email}`}
              >
                {siteConfig.email}
              </a>
            }
            icon={<Mail className="text-primary h-6 w-6" />}
            label={translations('email')}
          />

          <InfoItem
            content={
              <a
                className="text-foreground hover:text-primary text-lg font-medium transition-colors"
                href={siteConfig.github}
                rel="noopener noreferrer"
                target="_blank"
              >
                {`@${siteConfig.githubUsername}`}
              </a>
            }
            icon={<GitBranch className="text-primary h-6 w-6" />}
            label={translations('github')}
          />

          <InfoItem
            content={
              <p className="text-foreground text-lg font-medium">
                {translations('locationValue')}
              </p>
            }
            icon={<MapPin className="text-primary h-6 w-6" />}
            label={translations('location')}
          />
        </div>
      </CardContent>
    </Card>
  )
}

const ResumeCard: FCStrict<ResumeCardProperties> = ({
  locale,
  translations,
}: ResumeCardProperties): JSX.Element => {
  const resumePath: string =
    locale === 'de' ? '/resume-de.pdf' : '/resume-en.pdf'
  const languageName: string = locale === 'de' ? 'Deutsch' : 'English'
  const pdfLabel: string = `PDF • ${languageName}`

  return (
    <Card className="overflow-hidden border-2 shadow-xl">
      <div className="from-primary/20 via-primary/10 to-primary/5 bg-gradient-to-br p-8">
        <div className="mb-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="bg-primary/10 rounded-lg p-3">
              <FileText className="text-primary h-6 w-6" />
            </div>
            <div>
              <Heading as="h3" className="text-foreground text-xl font-bold">
                {translations('downloadResume')}
              </Heading>
              <p className="text-muted-foreground text-sm">{pdfLabel}</p>
            </div>
          </div>
        </div>

        <Button
          asChild={true}
          className="group bg-primary hover:bg-primary/90 w-full shadow-lg transition-all hover:shadow-xl"
          size="lg"
        >
          <a download={true} href={resumePath}>
            <Download className="mr-2 h-5 w-5 transition-transform group-hover:translate-y-0.5 group-hover:scale-110" />
            {translations('downloadResume')}
          </a>
        </Button>
      </div>
    </Card>
  )
}

/* ───────────────────────────── main FC ────────────────────────────── */

export const ContactSection: FCAsync<ContactSectionProperties> = async ({
  locale,
}: ContactSectionProperties): Promise<JSX.Element> => {
  const translations: Translations<'contact'> = await getTranslations({
    locale,
    namespace: 'contact',
  })

  return (
    <section className="bg-muted/30 relative px-4 py-20" id="contact">
      <div className="absolute inset-0 -z-10 bg-[linear-gradient(to_right,#8080800a_1px,transparent_1px),linear-gradient(to_bottom,#8080800a_1px,transparent_1px)] bg-[size:14px_24px]" />

      <div className="mx-auto w-full max-w-4xl">
        <div className="mb-12 text-center">
          <Heading as="h2" className="text-foreground mb-3 text-4xl font-bold">
            {translations('title')}
          </Heading>
          <div className="from-primary to-primary/60 mx-auto h-1 w-20 rounded-full bg-gradient-to-r" />
        </div>

        <div className="mx-auto max-w-2xl space-y-6">
          <InfoCard translations={translations} />
          <ResumeCard locale={locale} translations={translations} />
        </div>
      </div>
    </section>
  )
}
