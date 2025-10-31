'use client'

import { Card } from '@/components/ui/card'
import { Quote } from 'lucide-react'
import Image from 'next/image'
import type { TestimonialsSectionProps } from '@/types/testimonials-section'

export function TestimonialsSection({ dict }: TestimonialsSectionProps) {
  return (
    <section className="from-muted/20 to-background min-h-screen bg-gradient-to-b px-4 py-20 md:px-8">
      <div className="mx-auto max-w-7xl">
        <div className="mb-16 text-center">
          <h2 className="from-primary to-primary/60 mb-4 bg-gradient-to-r bg-clip-text text-4xl font-bold text-transparent md:text-5xl">
            {dict.testimonials.title}
          </h2>
          <p className="text-muted-foreground mx-auto max-w-2xl text-lg">
            {dict.testimonials.subtitle}
          </p>
        </div>

        <div className="grid grid-cols-1 gap-8 md:grid-cols-2 lg:grid-cols-3">
          {dict.testimonials.items.map((testimonial: any, index: number) => (
            <Card
              key={index}
              className="group hover:border-primary/50 relative overflow-hidden border-2 p-8 transition-all duration-300 hover:shadow-2xl"
            >
              <div className="absolute top-4 right-4 opacity-10 transition-opacity group-hover:opacity-20">
                <Quote className="text-primary h-16 w-16" />
              </div>

              <div className="relative z-10">
                <div className="mb-6 flex items-center gap-4">
                  <div className="ring-primary/20 group-hover:ring-primary/50 relative h-16 w-16 overflow-hidden rounded-full ring-2 transition-all">
                    <Image
                      src={testimonial.image || '/placeholder.svg'}
                      alt={testimonial.name}
                      fill
                      className="object-cover"
                    />
                  </div>
                  <div>
                    <h3 className="text-lg font-bold">{testimonial.name}</h3>
                    <p className="text-muted-foreground text-sm">
                      {testimonial.role}
                    </p>
                    <p className="text-primary text-xs">
                      {testimonial.company}
                    </p>
                  </div>
                </div>

                <blockquote className="text-muted-foreground leading-relaxed italic">
                  "{testimonial.quote}"
                </blockquote>
              </div>
            </Card>
          ))}
        </div>
      </div>
    </section>
  )
}
