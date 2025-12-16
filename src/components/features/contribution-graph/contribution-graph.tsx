import { type JSX } from 'react'

import { ContributionGraphClient } from '@/components/features/contribution-graph/contribution-graph-client'
import { type ContributionCollection } from '@/models/github'
import type { FCStrict } from '@/types/fc'
import type { LocalePageProperties } from '@/types/i18n'

/* =============================== Types =============================== */

interface ContributionGraphProperties extends LocalePageProperties {
  readonly data: ContributionCollection
}

/* ================================ Main FC ================================ */

export const ContributionGraph: FCStrict<ContributionGraphProperties> = ({
  data,
  locale,
}: ContributionGraphProperties): JSX.Element => {
  return <ContributionGraphClient data={data} locale={locale} />
}
