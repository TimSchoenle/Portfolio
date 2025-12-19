'use client'

import {
  type Dispatch,
  type JSX,
  type ReactNode,
  type SetStateAction,
  useEffect,
  useRef,
  useState,
} from 'react'

import { type FCWithRequiredChildren } from '@/types/fc'

interface LazyLoadProperties {
  readonly className?: string
  /**
   * Distance in pixels before the element is visible to trigger loading.
   * Default: 200px
   */
  readonly rootMargin?: string
}

export const LazyLoad: FCWithRequiredChildren<LazyLoadProperties> = ({
  children,
  className,
  rootMargin = '200px',
}: LazyLoadProperties & {
  readonly children: ReactNode
}): JSX.Element => {
  const [isVisible, setIsVisible]: [
    boolean,
    Dispatch<SetStateAction<boolean>>,
  ] = useState(false)
  const containerReference: React.RefObject<HTMLDivElement | null> =
    useRef<HTMLDivElement>(null)

  useEffect((): (() => void) | undefined => {
    // If IntersectionObserver is not supported (e.g. server or old browser), load immediately
    if (typeof IntersectionObserver === 'undefined') {
      setIsVisible(true)
      return undefined
    }

    const observer: IntersectionObserver = new IntersectionObserver(
      (entries: IntersectionObserverEntry[]): void => {
        const entry: IntersectionObserverEntry | undefined = entries[0]
        if (entry?.isIntersecting === true) {
          setIsVisible(true)
          observer.disconnect()
        }
      },
      { rootMargin }
    )

    if (containerReference.current) {
      observer.observe(containerReference.current)
    }

    return (): void => {
      observer.disconnect()
    }
  }, [rootMargin])

  return (
    <div className={className} ref={containerReference}>
      {isVisible ? children : null}
    </div>
  )
}
