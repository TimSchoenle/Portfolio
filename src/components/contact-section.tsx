'use server'

import { type JSX } from 'react'

import { type Locale } from 'next-intl'

import { Download, FileText, GitBranch, Mail, MapPin } from 'lucide-react'
import { getTranslations } from 'next-intl/server'

import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Heading } from '@/components/ui/heading'
import { siteConfig } from '@/lib/config'
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
    <div className="group flex items-center gap-4 rounded-lg p-3 transition-all hover:bg-muted/50">
      <div className="rounded-lg bg-gradient-to-br from-primary/10 to-primary/5 p-3 transition-transform duration-300 group-hover:scale-110">
        {icon}
      </div>
      <div>
        <p className="text-sm text-muted-foreground">{label}</p>
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
                className="text-lg font-medium text-foreground transition-colors hover:text-primary"
                href={`mailto:${siteConfig.email}`}
              >
                {siteConfig.email}
              </a>
            }
            icon={<Mail className="h-6 w-6 text-primary" />}
            label={translations('email')}
          />

          <InfoItem
            content={
              <a
                className="text-lg font-medium text-foreground transition-colors hover:text-primary"
                href={siteConfig.github}
                rel="noopener noreferrer"
                target="_blank"
              >
                {`@${siteConfig.githubUsername}`}
              </a>
            }
            icon={<GitBranch className="h-6 w-6 text-primary" />}
            label={translations('github')}
          />

          <InfoItem
            content={
              <p className="text-lg font-medium text-foreground">
                {translations('locationValue')}
              </p>
            }
            icon={<MapPin className="h-6 w-6 text-primary" />}
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
      <div className="bg-gradient-to-br from-primary/20 via-primary/10 to-primary/5 p-8">
        <div className="mb-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="rounded-lg bg-primary/10 p-3">
              <FileText className="h-6 w-6 text-primary" />
            </div>
            <div>
              <Heading as="h3" className="text-xl font-bold text-foreground">
                {translations('downloadResume')}
              </Heading>
              <p className="text-sm text-muted-foreground">{pdfLabel}</p>
            </div>
          </div>
        </div>

        <Button
          asChild={true}
          className="group w-full bg-primary shadow-lg transition-all hover:bg-primary/90 hover:shadow-xl"
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
    <section className="relative bg-muted/30 px-4 py-20" id="contact">
      <div className="absolute inset-0 -z-10 bg-[linear-gradient(to_right,#8080800a_1px,transparent_1px),linear-gradient(to_bottom,#8080800a_1px,transparent_1px)] bg-[size:14px_24px]" />

      <div className="mx-auto w-full max-w-4xl">
        <div className="mb-12 text-center">
          <Heading as="h2" className="mb-3 text-4xl font-bold text-foreground">
            {translations('title')}
          </Heading>
          <div className="mx-auto h-1 w-20 rounded-full bg-gradient-to-r from-primary to-primary/60" />
        </div>

        <div className="mx-auto max-w-2xl space-y-6">
          <InfoCard translations={translations} />
          <ResumeCard locale={locale} translations={translations} />
        </div>
      </div>
    </section>
  )
}
