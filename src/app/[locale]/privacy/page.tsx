import type { Metadata } from 'next'
import { getTranslations } from 'next-intl/server'
import { LegalPageLayout } from '@/components/legal-page-layout'

export async function generateStaticParams() {
  return [{ locale: 'en' }, { locale: 'de' }]
}

export async function generateMetadata(): Promise<Metadata> {
  const t = await getTranslations('privacy')

  return {
    title: t('title'),
    description: t('description'),
    robots: {
      index: true,
      follow: true,
    },
  }
}

export default async function PrivacyPolicyPage() {
  const t = await getTranslations('privacy')

  const controller = {
    title: t('controller.title'),
    name: t('controller.name'),
    address: t('controller.address'),
    email: t('controller.email'),
  }

  const cloudflare = {
    title: t('cloudflare.title'),
    pre: t('cloudflare.pre'),
    strong: t('cloudflare.strong'),
    post: t('cloudflare.post'),
    provider: t('cloudflare.provider'),
    policyLink: t('cloudflare.policyLink'),
    text: t('cloudflare.text'),
  }

  const sections = [
    'general',
    'logs',
    'cloudflare',
    'contact',
    'rights',
    'nocookies',
    'changes',
  ]

  return (
    <LegalPageLayout title={t('title')}>
      <div>
        <h2 className="mb-2 text-xl font-semibold">{controller.title}</h2>
        <p className="text-muted-foreground">
          <strong>Name:</strong> {controller.name}
          <br />
          <strong>Address:</strong> {controller.address}
          <br />
          <strong>Email:</strong>{' '}
          <a
            href={`mailto:${controller.email}`}
            className="text-primary hover:underline"
          >
            {controller.email}
          </a>
        </p>
      </div>

      {sections.map((key) => {
        if (key === 'cloudflare') {
          return (
            <div key={key}>
              <h2 className="mb-2 text-xl font-semibold">{cloudflare.title}</h2>
              <p className="text-muted-foreground text-sm leading-relaxed">
                {cloudflare.pre} <strong>{cloudflare.strong}</strong>{' '}
                {cloudflare.post}
              </p>
              <p className="text-muted-foreground mt-1 text-sm leading-relaxed">
                {cloudflare.provider}
              </p>
              <p className="text-muted-foreground mt-1 text-sm leading-relaxed">
                <a
                  href={cloudflare.policyLink}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline"
                >
                  {cloudflare.policyLink}
                </a>
              </p>
              <p className="text-muted-foreground mt-2 text-sm leading-relaxed">
                {cloudflare.text}
              </p>
            </div>
          )
        }

        return (
          <div key={key}>
            <h2 className="mb-2 text-xl font-semibold">{t(`${key}.title`)}</h2>
            <p className="text-muted-foreground text-sm leading-relaxed">
              {t(`${key}.text`)}
            </p>
          </div>
        )
      })}
    </LegalPageLayout>
  )
}
