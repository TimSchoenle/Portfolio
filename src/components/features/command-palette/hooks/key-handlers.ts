import { type Locale } from 'next-intl'

import { siteConfig } from '@/data/config'

import {
  createOnSelectSection,
  createPushHandler,
  type LocalizedRouter,
  openNewTab,
  type SectionId,
  sendMailTo,
} from '../utils/actions'

interface KeyHandlerDependencies {
  locale: Locale
  pathname: string
  router: LocalizedRouter
  run: (function_: () => void) => void
}

export const handleNavigationKey: (
  key: string,
  deps: KeyHandlerDependencies,
  handleRequest: (action: (value: string) => void) => void
) => boolean = (
  key: string,
  deps: KeyHandlerDependencies,
  handleRequest: (action: (value: string) => void) => void
): boolean => {
  const { locale, router, run }: KeyHandlerDependencies = deps
  switch (key) {
    case 'H': {
      handleRequest(createPushHandler({ href: '/', locale, router, run }))
      return true
    }
    case 'I': {
      handleRequest(
        createPushHandler({ href: '/imprint', locale, router, run })
      )
      return true
    }
    case 'D': {
      handleRequest(
        createPushHandler({ href: '/privacy', locale, router, run })
      )
      return true
    }
  }
  return false
}

export const handleSectionKey: (
  key: string,
  deps: KeyHandlerDependencies,
  handleRequest: (action: (value: string) => void) => void
) => boolean = (
  key: string,
  deps: KeyHandlerDependencies,
  handleRequest: (action: (value: string) => void) => void
): boolean => {
  const { locale, pathname, router, run }: KeyHandlerDependencies = deps
  const sectionMap: Map<string, SectionId> = new Map<string, SectionId>([
    ['A', 'about'],
    ['C', 'contact'],
    ['K', 'skills'],
    ['P', 'projects'],
    ['X', 'experience'],
  ])

  const sectionId: SectionId | undefined = sectionMap.get(key)

  if (sectionId !== undefined) {
    handleRequest(
      createOnSelectSection({
        locale,
        pathname,
        router,
        run,
        sectionId,
      })
    )
    return true
  }
  return false
}

export const handleActionKey: (
  key: string,
  deps: KeyHandlerDependencies,
  handleRequest: (action: (value: string) => void) => void
) => boolean = (
  key: string,
  deps: KeyHandlerDependencies,
  handleRequest: (action: (value: string) => void) => void
): boolean => {
  const { run }: KeyHandlerDependencies = deps
  switch (key) {
    case 'E': {
      handleRequest((): void => {
        run((): void => {
          sendMailTo(siteConfig.email)
        })
      })
      return true
    }
    case 'G': {
      handleRequest((): void => {
        run((): void => {
          openNewTab(siteConfig.socials.github)
        })
      })
      return true
    }
    case 'L': {
      handleRequest((): void => {
        run((): void => {
          if (
            siteConfig.socials.linkedin !== undefined &&
            siteConfig.socials.linkedin.length > 0
          ) {
            openNewTab(siteConfig.socials.linkedin)
          }
        })
      })
      return true
    }
  }
  return false
}

export const handlePaletteKey: (
  key: string,
  deps: KeyHandlerDependencies,
  handleRequest: (action: (value: string) => void) => void
) => void = (
  key: string,
  deps: KeyHandlerDependencies,
  handleRequest: (action: (value: string) => void) => void
): void => {
  if (handleNavigationKey(key, deps, handleRequest)) {
    return
  }
  if (handleSectionKey(key, deps, handleRequest)) {
    return
  }
  handleActionKey(key, deps, handleRequest)
}
