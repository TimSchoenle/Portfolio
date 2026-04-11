'use client'

import {
  type Dispatch,
  type JSX,
  type SetStateAction,
  useEffect,
  useState,
} from 'react'

import { useReportWebVitals } from 'next/web-vitals'

import {
  formatMetricValue,
  getMetricColorClass,
  METRIC_THRESHOLDS,
  type MetricName,
  type MetricRating,
  reportToConsole,
  type WebVitalsMetric,
} from '@/lib/web-vitals'
import { type FCNullable } from '@/types/fc'

interface DisplayMetric {
  readonly colorClass: string
  readonly formattedValue: string
  readonly name: string
}

const HEADER_TEXT: string = 'SYS_METRICS :: LIVE'
const LOADING_TEXT: string = 'GATHERING_DATA...'

/**
 * Hook to collect and manage web vitals metrics
 */

const usePerformanceMetrics: () => DisplayMetric[] = (): DisplayMetric[] => {
  const [metrics, setMetrics]: [
    Record<string, WebVitalsMetric>,
    Dispatch<SetStateAction<Record<string, WebVitalsMetric>>>,
  ] = useState<Record<string, WebVitalsMetric>>({})

  // Track web vitals using Next.js hook
  useReportWebVitals((metric: object): void => {
    // Cast to WebVitalsMetric (which matches web-vitals Metric type)
    const webVitalsMetric: WebVitalsMetric = metric as WebVitalsMetric

    // Only track core vitals we care about
    const validMetrics: readonly MetricName[] = [
      'CLS',
      'FCP',
      'INP',
      'LCP',
      'TTFB',
    ] as const

    if (!validMetrics.includes(webVitalsMetric.name)) {
      return
    }

    // Report to console in development
    reportToConsole(webVitalsMetric)

    setMetrics(
      (
        previousMetrics: Record<string, WebVitalsMetric>
      ): Record<string, WebVitalsMetric> => ({
        ...previousMetrics,
        [webVitalsMetric.name]: webVitalsMetric,
      })
    )
  })

  // Convert metrics to display format
  return Object.entries(metrics).map(
    ([, metric]: [string, WebVitalsMetric]): DisplayMetric => {
      const displayName: string = metric.name
      const metricName: MetricName = metric.name
      const metricRating: MetricRating = metric.rating

      const colorClass: string = getMetricColorClass(metricRating)
      const formattedValue: string = formatMetricValue(metricName, metric.value)

      return {
        colorClass,
        formattedValue,
        name: displayName,
      }
    }
  )
}

export const PerformanceMonitor: FCNullable = (): JSX.Element | null => {
  const metrics: DisplayMetric[] = usePerformanceMetrics()
  const [mounted, setMounted]: [boolean, Dispatch<SetStateAction<boolean>>] =
    useState<boolean>(false)

  useEffect((): void => {
    setMounted(true)
  }, [])

  const sortedMetrics: DisplayMetric[] = metrics.toSorted(
    (metricA: DisplayMetric, metricB: DisplayMetric): number =>
      metricA.name.localeCompare(metricB.name)
  )

  return mounted ? (
    <div className="flex flex-col gap-1 font-mono text-[10px] tracking-wider text-brand/40 uppercase tabular-nums transition-opacity hover:opacity-100 md:text-right">
      <div className="mb-1 font-bold text-brand-readable">{HEADER_TEXT}</div>

      <div className="grid grid-cols-2 gap-x-4 gap-y-0.5">
        {sortedMetrics.map(
          (metric: DisplayMetric): JSX.Element => (
            <div
              className="flex justify-between gap-4"
              key={metric.name}
              title={`Threshold: Good ≤ ${formatMetricValue(metric.name as MetricName, METRIC_THRESHOLDS[metric.name as MetricName].good)}`}
            >
              <span>{metric.name}</span>
              <span className={metric.colorClass}>{metric.formattedValue}</span>
            </div>
          )
        )}
        {metrics.length === 0 ? (
          <div className="animate-pulse">{LOADING_TEXT}</div>
        ) : null}
      </div>
    </div>
  ) : null
}
