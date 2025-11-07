'use server'

import { Quote } from 'lucide-react'
import Image from 'next/image'
import { type Locale } from 'next-intl'
import { getTranslations } from 'next-intl/server'
import { type JSX } from 'react'

import { Card } from '@/components/ui/card'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'

/* ───────────────────────── types ───────────────────────── */

interface TestimonialsSectionProps {
  readonly locale: Locale
}

interface TestimonialItem {
  readonly name: string
  readonly role: string
  readonly company: string
  readonly image: string
  readonly quote: string
}

/* ───────────────────── type guards/helpers ───────────────────── */

const isString: (v: unknown) => v is string = (v: unknown): v is string =>
  typeof v === 'string'

const isTestimonialItem: (v: unknown) => v is TestimonialItem = (
  v: unknown
): v is TestimonialItem => {
  if (v === null || typeof v !== 'object') {
    return false
  }
  const o: Record<string, unknown> = v as Record<string, unknown>
  return (
    isString(o['name']) &&
    isString(o['role']) &&
    isString(o['company']) &&
    isString(o['image']) &&
    isString(o['quote'])
  )
}

const makeKey: (t: TestimonialItem) => string = (t: TestimonialItem): string =>
  `${t.name}::${t.company}::${t.image}`

/* ───────────────────── subcomponents ───────────────────── */

interface TestimonialCardProps {
  readonly item: TestimonialItem
}

const TestimonialCard: FCStrict<TestimonialCardProps> = ({
  item,
}: TestimonialCardProps): JSX.Element => {
  return (
    <Card className="group hover:border-primary/50 relative overflow-hidden border-2 p-8 transition-all duration-300 hover:shadow-2xl">
      <div className="absolute top-4 right-4 opacity-10 transition-opacity group-hover:opacity-20">
        <Quote className="text-primary h-16 w-16" />
      </div>

      <div className="relative z-10">
        <div className="mb-6 flex items-center gap-4">
          <div className="ring-primary/20 group-hover:ring-primary/50 relative h-16 w-16 overflow-hidden rounded-full ring-2 transition-all">
            <Image
              alt={`${item.name} avatar`}
              className="object-cover"
              fill={true}
              src={item.image || '/placeholder.svg'}
            />
          </div>
          <div>
            <h3 className="text-lg font-bold">{item.name}</h3>
            <p className="text-muted-foreground text-sm">{item.role}</p>
            <p className="text-primary text-xs">{item.company}</p>
          </div>
        </div>

        <blockquote className="text-muted-foreground leading-relaxed italic">
          {`“${item.quote}”`}
        </blockquote>
      </div>
    </Card>
  )
}

/* ───────────────────── main component ───────────────────── */

export const TestimonialsSection: AsyncPageFC<
  TestimonialsSectionProps
> = async ({ locale }: TestimonialsSectionProps): Promise<JSX.Element> => {
  const t: Translations<'testimonials'> = await getTranslations({
    locale,
    namespace: 'testimonials',
  })

  // Safely parse raw items to avoid unsafe assignments
  const raw: unknown = t.raw('items')
  const testimonials: readonly TestimonialItem[] = Array.isArray(raw)
    ? raw.filter(isTestimonialItem)
    : []

  const titleText: string = t('title')
  const subtitleText: string = t('subtitle')

  return (
    <section className="from-muted/20 to-background min-h-screen bg-gradient-to-b px-4 py-20 md:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="mb-16 text-center">
          <h2 className="from-primary to-primary/60 mb-4 bg-gradient-to-r bg-clip-text text-4xl font-bold text-transparent md:text-5xl">
            {titleText}
          </h2>
          <p className="text-muted-foreground mx-auto max-w-2xl text-lg">
            {subtitleText}
          </p>
        </div>

        <div className="grid grid-cols-1 gap-8 md:grid-cols-2 lg:grid-cols-3">
          {testimonials.map(
            (item: TestimonialItem): JSX.Element => (
              <TestimonialCard item={item} key={makeKey(item)} />
            )
          )}
        </div>
      </div>
    </section>
  )
}
