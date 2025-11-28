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

interface InfoCardProperties {
  readonly country: string
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

// eslint-disable-next-line max-lines-per-function
const InfoCard: FCStrict<InfoCardProperties> = ({
  country,
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

        <address className="space-y-4 not-italic">
          <div className="group flex items-center gap-4 rounded-lg p-3 transition-all hover:bg-muted/50">
            <div
              aria-hidden="true"
              className="rounded-lg bg-gradient-to-br from-primary/10 to-primary/5 p-3 transition-transform duration-300 group-hover:scale-110"
            >
              <Mail className="h-6 w-6 text-primary" />
            </div>
            <p>
              <span className="block text-sm text-muted-foreground">
                {translations('email')}
              </span>
              <a
                className="text-lg font-medium text-foreground transition-colors hover:text-primary"
                href={`mailto:${siteConfig.email}`}
              >
                {siteConfig.email}
              </a>
            </p>
          </div>

          <div className="group flex items-center gap-4 rounded-lg p-3 transition-all hover:bg-muted/50">
            <div
              aria-hidden="true"
              className="rounded-lg bg-gradient-to-br from-primary/10 to-primary/5 p-3 transition-transform duration-300 group-hover:scale-110"
            >
              <GitBranch className="h-6 w-6 text-primary" />
            </div>
            <p>
              <span className="block text-sm text-muted-foreground">
                {translations('github')}
              </span>
              <a
                className="text-lg font-medium text-foreground transition-colors hover:text-primary"
                href={siteConfig.github}
                rel="noopener noreferrer"
                target="_blank"
              >
                {`@${siteConfig.githubUsername}`}
              </a>
            </p>
          </div>

          <div className="group flex items-center gap-4 rounded-lg p-3 transition-all hover:bg-muted/50">
            <div
              aria-hidden="true"
              className="rounded-lg bg-gradient-to-br from-primary/10 to-primary/5 p-3 transition-transform duration-300 group-hover:scale-110"
            >
              <MapPin className="h-6 w-6 text-primary" />
            </div>
            <p>
              <span className="block text-sm text-muted-foreground">
                {translations('location')}
              </span>
              <span className="text-lg font-medium text-foreground">
                {country}
              </span>
            </p>
          </div>
        </address>
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
  locale: Locale,
  translations: Translations<'contact'>
) => ResumeDetails = (
  locale: Locale,
  translations: Translations<'contact'>
): ResumeDetails => {
  const languageName: string = getResumeLanguageName(locale)
  const path: string = `/resume/resume-${locale}.pdf`
  const pdfLabel: string = translations('pdfVersion', {
    language: languageName,
  })

  return { languageName, path, pdfLabel }
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
          {/* Reader Mode / Screen Reader Fallback Link */}
          <p
            style={{
              height: '1px',
              left: '-10000px',
              overflow: 'hidden',
              position: 'absolute',
              top: 'auto',
              width: '1px',
            }}
          >
            <a download={true} href={resumePath}>
              {translations('downloadResume')}
            </a>
          </p>
        </div>
      </CardContent>
    </Card>
  )
}

/* ───────────────────────────── main FC ────────────────────────────── */

export const ContactSection: FCAsync<ContactSectionProperties> = async ({
  locale,
}: ContactSectionProperties): Promise<JSX.Element> => {
  const translation: Translations<''> = await getTranslations({ locale })

  const contactTranslations: Translations<'contact'> = await getTranslations({
    locale,
    namespace: 'contact',
  })

  const resumeDetails: ResumeDetails = getResumeDetails(
    locale,
    contactTranslations
  )

  return (
    <section className="relative bg-muted/30 px-4 py-20" id="contact">
      <div className="absolute inset-0 -z-10 bg-[linear-gradient(to_right,#8080800a_1px,transparent_1px),linear-gradient(to_bottom,#8080800a_1px,transparent_1px)] bg-[size:14px_24px]" />

      <div className="mx-auto w-full max-w-4xl">
        <div className="mb-12 text-center">
          <Heading as="h2" className="mb-3 text-4xl font-bold text-foreground">
            {contactTranslations('title')}
          </Heading>
          <div className="mx-auto h-1 w-20 rounded-full bg-gradient-to-r from-primary to-primary/60" />
        </div>

        <div className="mx-auto max-w-2xl space-y-6">
          <InfoCard
            country={translation('personalInfo.country')}
            translations={contactTranslations}
          />
          <ResumeCard
            details={resumeDetails}
            translations={contactTranslations}
          />
        </div>
      </div>
    </section>
  )
}
