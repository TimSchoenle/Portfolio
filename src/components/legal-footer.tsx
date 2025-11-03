import Link from 'next/link'
import { useTranslations } from 'next-intl'

export function LegalFooter() {
  const t = useTranslations()

  return (
    <footer className="mt-8 text-center">
      <nav aria-label="Legal navigation" className="flex justify-center gap-4">
        <Link
          href="/imprint"
          className="text-muted-foreground hover:text-primary text-sm transition-colors hover:underline"
        >
          {t('imprint.title')}
        </Link>
        <Link
          href="/privacy"
          className="text-muted-foreground hover:text-primary text-sm transition-colors hover:underline"
        >
          {t('privacy.title')}
        </Link>
      </nav>
    </footer>
  )
}
