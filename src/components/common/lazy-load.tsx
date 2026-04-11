'use client'

import React, {
  type Dispatch,
  type JSX,
  type SetStateAction,
  useEffect,
  useRef,
  useState,
} from 'react'

import {
  type FCWithRequiredChildren,
  type WithRequiredChildren,
} from '@/types/fc'

interface LazyLoadProperties extends WithRequiredChildren {
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
}: LazyLoadProperties): JSX.Element => {
  const [isVisible, setIsVisible]: [
    boolean,
    Dispatch<SetStateAction<boolean>>,
  ] = useState(false)
  const containerReference: React.RefObject<HTMLDivElement | null> =
    useRef<HTMLDivElement>(null)

  useEffect((): (() => void) => {
    let observer: IntersectionObserver | null = null

    if (typeof IntersectionObserver !== 'undefined') {
      observer = new IntersectionObserver(
        (entries: IntersectionObserverEntry[]): void => {
          const entry: IntersectionObserverEntry | undefined = entries[0]
          if (entry?.isIntersecting === true) {
            setIsVisible(true)
            observer?.disconnect()
          }
        },
        { rootMargin }
      )
    }

    if (observer === null) {
      // If IntersectionObserver is not supported (e.g. server or old browser), load immediately.
      setIsVisible(true)
    } else if (containerReference.current) {
      observer.observe(containerReference.current)
    }

    return (): void => {
      if (observer !== null) {
        observer.disconnect()
      }
    }
  }, [rootMargin])

  return (
    <div className={className} ref={containerReference}>
      {isVisible ? children : null}
    </div>
  )
}
