'use server'

import { type JSX } from 'react'

import { type Locale } from 'next-intl'

import { Quote } from 'lucide-react'
import Image from 'next/image'
import { getTranslations } from 'next-intl/server'

import { Card } from '@/components/ui/card'
import type { AsyncPageFC, FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'

/* ───────────────────────── types ───────────────────────── */

interface TestimonialsSectionProperties {
  readonly locale: Locale
}

interface TestimonialItem {
  readonly company: string
  readonly image: string
  readonly name: string
  readonly quote: string
  readonly role: string
}

/* ───────────────────── type guards/helpers ───────────────────── */

const isString: (value: unknown) => value is string = (
  value: unknown
): value is string => typeof value === 'string'

const isTestimonialItem: (v: unknown) => v is TestimonialItem = (
  value: unknown
): value is TestimonialItem => {
  if (value === null || typeof value !== 'object') {
    return false
  }
  const object: Record<string, unknown> = value as Record<string, unknown>
  return (
    isString(object['name']) &&
    isString(object['role']) &&
    isString(object['company']) &&
    isString(object['image']) &&
    isString(object['quote'])
  )
}

const makeKey: (testimonialItem: TestimonialItem) => string = (
  testimonialItem: TestimonialItem
): string =>
  `${testimonialItem.name}::${testimonialItem.company}::${testimonialItem.image}`

/* ───────────────────── subcomponents ───────────────────── */

interface TestimonialCardProperties {
  readonly item: TestimonialItem
}

const TestimonialCard: FCStrict<TestimonialCardProperties> = ({
  item,
}: TestimonialCardProperties): JSX.Element => {
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
  TestimonialsSectionProperties
> = async ({ locale }: TestimonialsSectionProperties): Promise<JSX.Element> => {
  const translations: Translations<'testimonials'> = await getTranslations({
    locale,
    namespace: 'testimonials',
  })

  // Safely parse raw items to avoid unsafe assignments
  const raw: unknown = translations.raw('items')
  const testimonials: readonly TestimonialItem[] = Array.isArray(raw)
    ? raw.filter(isTestimonialItem)
    : []

  const titleText: string = translations('title')
  const subtitleText: string = translations('subtitle')

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
