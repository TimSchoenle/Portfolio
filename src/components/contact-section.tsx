'use server'

import { access, constants } from 'node:fs/promises'
import path from 'node:path'

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

interface ResumeDetails {
  readonly languageName: string
  readonly path: string
  readonly pdfLabel: string
}

interface ResumeCardProperties {
  readonly details: ResumeDetails
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
    <Card className="group border-2 transition-all duration-300 hover:border-primary/50 hover:shadow-xl">
      <CardContent className="p-6">
        <Heading
          as="h3"
          className="mb-4 text-2xl font-semibold text-foreground"
        >
          {translations('infoTitle')}
        </Heading>

        <div className="space-y-4">
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

const getResumeLanguageName: (locale: Locale) => string = (
  locale: Locale
): string => {
  switch (locale) {
    case 'de': {
      return 'Deutsch'
    }
    case 'en': {
      return 'English'
    }
    default: {
      return locale.toUpperCase()
    }
  }
}

const getResumeDetails: (
  locale: Locale
) => Promise<ResumeDetails | null> = async (
  locale: Locale
): Promise<ResumeDetails | null> => {
  const fileName: string = `${locale}.pdf`
  const languageName: string = getResumeLanguageName(locale)

  const publicPath: string = `/${siteConfig.resumeDirectory}/${fileName}`
  const fileSystemPath: string = path.join(
    process.cwd(),
    'public',
    siteConfig.resumeDirectory,
    fileName
  )

  try {
    await access(fileSystemPath, constants.F_OK)
    const pdfLabel: string = `PDF • ${languageName}`

    return {
      languageName,
      path: publicPath,
      pdfLabel,
    }
  } catch {
    return null
  }
}

const ResumeCard: FCStrict<ResumeCardProperties> = ({
  details,
  translations,
}: ResumeCardProperties): JSX.Element => {
  const { path: resumePath, pdfLabel }: ResumeDetails = details

  return (
    <Card className="group border-2 transition-all duration-300 hover:border-primary/50 hover:shadow-xl">
      <CardContent className="flex items-start gap-4 p-6">
        <div className="rounded-lg bg-gradient-to-br from-primary/10 to-primary/5 p-3 transition-transform duration-300 group-hover:scale-110">
          <FileText className="h-6 w-6 text-primary" />
        </div>

        <div className="flex-1">
          <Heading
            as="h3"
            className="mb-1 text-xl font-semibold text-foreground"
          >
            {translations('downloadResume')}
          </Heading>
          <p className="mb-4 text-sm text-muted-foreground">{pdfLabel}</p>

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
      </CardContent>
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

  const resumeDetails: ResumeDetails | null = await getResumeDetails(locale)

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
          {resumeDetails === null ? null : (
            <ResumeCard details={resumeDetails} translations={translations} />
          )}
        </div>
      </div>
    </section>
  )
}
