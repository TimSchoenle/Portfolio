import { type FC, type ReactElement } from 'react'

import { Link, Text, View } from '@react-pdf/renderer'

import type { ResumeData } from '@/types/resume-types'

import { styles } from './modern.styles'

interface ContactSectionProperties {
  readonly personalInfo: ResumeData['personalInfo']
  readonly titles: Record<string, string>
}

export const ContactSection: FC<ContactSectionProperties> = ({
  personalInfo,
  titles,
}: ContactSectionProperties): ReactElement | null => {
  if (personalInfo === undefined) {
    return null
  }

  return (
    <>
      <Text style={styles.sectionTitleFirst}>
        {titles['contact'] ?? 'Contact'}
      </Text>
      <View style={styles.sectionDivider} />

      <Text style={styles.contactLabel}>{titles['email'] ?? 'Email'}</Text>
      <Text style={styles.contactItem}>{personalInfo.email}</Text>

      <Text style={styles.contactLabel}>
        {titles['location'] ?? 'Location'}
      </Text>
      <Text style={styles.contactItem}>{personalInfo.location}</Text>

      <Text style={styles.contactLabel}>{titles['github'] ?? 'GitHub'}</Text>
      <Link src={personalInfo.github} style={styles.contactItem}>
        {personalInfo.github.replace('https://', '')}
      </Link>

      {personalInfo.linkedin === undefined ? null : (
        <>
          <Text style={styles.contactLabel}>
            {titles['linkedin'] ?? 'LinkedIn'}
          </Text>
          <Link src={personalInfo.linkedin} style={styles.contactItem}>
            {personalInfo.linkedin.replace('https://', '')}
          </Link>
        </>
      )}
    </>
  )
}
