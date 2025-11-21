import { type FC, type ReactElement } from 'react'

import { Link, Text, View } from '@react-pdf/renderer'

import { siteConfig } from '@/lib/config'
import type { ResumeTranslations } from '@/types/resume'

import { styles } from './modern.styles'

interface ContactSectionProperties {
  readonly translations: ResumeTranslations
}

export const ContactSection: FC<ContactSectionProperties> = ({
  translations,
}: ContactSectionProperties): ReactElement | null => {
  return (
    <>
      <Text style={styles.sectionTitleFirst}>
        {translations('contact.title')}
      </Text>
      <View style={styles.sectionDivider} />

      <Text style={styles.contactLabel}>{translations('contact.email')}</Text>
      <Text style={styles.contactItem}>{siteConfig.email}</Text>

      <Text style={styles.contactLabel}>
        {translations('contact.location')}
      </Text>
      <Text style={styles.contactItem}>
        {translations('personalInfo.country')}
      </Text>

      <Text style={styles.contactLabel}>
        {translations('common.socials.github')}
      </Text>
      <Link src={siteConfig.github} style={styles.contactItem}>
        {siteConfig.github.replace('https://', '')}
      </Link>

      {siteConfig.linkedin === undefined ? null : (
        <>
          <Text style={styles.contactLabel}>
            {translations('common.socials.linkedin')}
          </Text>
          <Link src={siteConfig.linkedin} style={styles.contactItem}>
            {siteConfig.linkedin.replace('https://', '')}
          </Link>
        </>
      )}
    </>
  )
}
