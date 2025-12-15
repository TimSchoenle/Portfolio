'use client'

import {
  type Dispatch,
  type JSX,
  type SetStateAction,
  useCallback,
  useEffect,
  useState,
} from 'react'

import { type Locale, useLocale, useTranslations } from 'next-intl'

import {
  Briefcase,
  Code,
  CookieIcon,
  FileText,
  GitBranch,
  Home,
  // eslint-disable-next-line sonarjs/deprecation
  Linkedin,
  Mail,
  MessageSquare,
  User,
} from 'lucide-react'

import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import { usePathname, useRouter } from '@/i18n/routing'
import { siteConfig } from '@/lib/config'
import type { FCStrict } from '@/types/fc'
import type { Translations } from '@/types/i18n'

/* ──────────────────────────── types & aliases ─────────────────────────── */

type LocalizedRouter = ReturnType<typeof useRouter>
type SectionId = 'about' | 'contact' | 'projects' | 'skills'

/* ────────────────────────────── helpers ──────────────────────────────── */

const scrollToSection: (id: string) => void = (identifier: string): void => {
  const element: HTMLElement | null = document.querySelector<HTMLElement>(
    `[id="${identifier}"]`
  )
  if (element !== null) {
    element.scrollIntoView({ behavior: 'smooth' })
  }
}

interface GoToSectionParameters {
  readonly id: SectionId
  readonly locale: Locale
  readonly pathname: string // canonical (no locale prefix)
  readonly router: LocalizedRouter
}
const goToSection: (p: GoToSectionParameters) => void = (
  parameters: GoToSectionParameters
): void => {
  const isHome: boolean = parameters.pathname === '/'
  if (isHome) {
    scrollToSection(parameters.id)
    return
  }
  // push canonical href; next-intl router injects locale
  parameters.router.push(`/#${parameters.id}`, { locale: parameters.locale })
}

const openNewTab: (url: string) => void = (url: string): void => {
  window.open(url, '_blank', 'noopener,noreferrer')
}
const sendMailTo: (email: string) => void = (email: string): void => {
  window.location.href = `mailto:${email}`
}

/* factories to satisfy unicorn/consistent-function-scoping */

interface CreatePushHandlerParameters {
  readonly href: '/' | '/imprint' | '/privacy'
  readonly locale: Locale
  readonly router: LocalizedRouter
  readonly run: (function_: () => void) => void
}
const createPushHandler: (
  parameters: CreatePushHandlerParameters
) => (_value: string) => void = (
  parameters: CreatePushHandlerParameters
): ((_value: string) => void) => {
  return (_value: string): void => {
    parameters.run((): void => {
      parameters.router.push(parameters.href, { locale: parameters.locale })
    })
  }
}

interface CreateOnSelectSectionParameters {
  readonly id: SectionId
  readonly locale: Locale
  readonly pathname: string
  readonly router: LocalizedRouter
  readonly run: (function_: () => void) => void
}
const createOnSelectSection: (
  parameters: CreateOnSelectSectionParameters
) => (_value: string) => void = (
  parameters: CreateOnSelectSectionParameters
) => {
  return (_value: string): void => {
    parameters.run((): void => {
      goToSection({
        id: parameters.id,
        locale: parameters.locale,
        pathname: parameters.pathname,
        router: parameters.router,
      })
    })
  }
}

/* ───────────────────────────── sub-views ─────────────────────────────── */

interface NavGroupProperties {
  readonly locale: Locale
  readonly router: LocalizedRouter
  readonly run: (function_: () => void) => void
  readonly tAll: Translations<''>
  readonly tPalette: Translations<'commandPalette'>
}
const NavGroup: FCStrict<NavGroupProperties> = ({
  locale,
  router,
  run,
  tAll,
  tPalette,
}: NavGroupProperties): JSX.Element => {
  const goHome: (_value: string) => void = createPushHandler({
    href: '/',
    locale,
    router,
    run,
  })
  const goImprint: (_value: string) => void = createPushHandler({
    href: '/imprint',
    locale,
    router,
    run,
  })
  const goPrivacy: (_value: string) => void = createPushHandler({
    href: '/privacy',
    locale,
    router,
    run,
  })

  return (
    <CommandGroup heading={tPalette('navigation')}>
      <CommandItem onSelect={goHome}>
        <Home className="mr-2 h-4 w-4" />
        <span>{tPalette('home')}</span>
      </CommandItem>
      <CommandItem onSelect={goImprint}>
        <FileText className="mr-2 h-4 w-4" />
        <span>{tAll('imprint.title')}</span>
      </CommandItem>
      <CommandItem onSelect={goPrivacy}>
        <CookieIcon className="mr-2 h-4 w-4" />
        <span>{tAll('privacy.title')}</span>
      </CommandItem>
    </CommandGroup>
  )
}

interface SectionsGroupProperties {
  readonly locale: Locale
  readonly pathname: string
  readonly router: LocalizedRouter
  readonly run: (function_: () => void) => void
  readonly tAll: Translations<''>
  readonly tPalette: Translations<'commandPalette'>
}
const SectionsGroup: FCStrict<SectionsGroupProperties> = ({
  locale,
  pathname,
  router,
  run,
  tAll,
  tPalette,
}: SectionsGroupProperties): JSX.Element => {
  const toAbout: (_value: string) => void = createOnSelectSection({
    id: 'about',
    locale,
    pathname,
    router,
    run,
  })
  const toSkills: (_value: string) => void = createOnSelectSection({
    id: 'skills',
    locale,
    pathname,
    router,
    run,
  })
  const toProjects: (_value: string) => void = createOnSelectSection({
    id: 'projects',
    locale,
    pathname,
    router,
    run,
  })
  const toContact: (_value: string) => void = createOnSelectSection({
    id: 'contact',
    locale,
    pathname,
    router,
    run,
  })

  return (
    <CommandGroup heading={tPalette('sections')}>
      <CommandItem onSelect={toAbout}>
        <User className="mr-2 h-4 w-4" />
        <span>{tAll('about.title')}</span>
      </CommandItem>
      <CommandItem onSelect={toSkills}>
        <Code className="mr-2 h-4 w-4" />
        <span>{tAll('skills.title')}</span>
      </CommandItem>
      <CommandItem onSelect={toProjects}>
        <Briefcase className="mr-2 h-4 w-4" />
        <span>{tAll('projects.title')}</span>
      </CommandItem>
      <CommandItem onSelect={toContact}>
        <MessageSquare className="mr-2 h-4 w-4" />
        <span>{tAll('contact.title')}</span>
      </CommandItem>
    </CommandGroup>
  )
}

interface ActionsGroupProperties {
  readonly run: (function_: () => void) => void
  readonly tPalette: Translations<'commandPalette'>
}
const ActionsGroup: FCStrict<ActionsGroupProperties> = ({
  run,
  tPalette,
}: ActionsGroupProperties): JSX.Element => {
  const goGitHub: (_value: string) => void = (_value: string): void => {
    run((): void => {
      openNewTab(siteConfig.socials.github)
    })
  }
  const goLinkedIn: (_value: string) => void = (_value: string): void => {
    run((): void => {
      if (typeof siteConfig.socials.linkedin === 'string') {
        openNewTab(siteConfig.socials.linkedin)
      }
    })
  }
  const emailMe: (_value: string) => void = (_value: string): void => {
    run((): void => {
      sendMailTo(siteConfig.email)
    })
  }

  return (
    <CommandGroup heading={tPalette('actions')}>
      <CommandItem onSelect={goGitHub}>
        <GitBranch className="mr-2 h-4 w-4" />
        <span>{tPalette('github')}</span>
      </CommandItem>
      {typeof siteConfig.socials.linkedin === 'string' ? (
        <CommandItem onSelect={goLinkedIn}>
          {/* eslint-disable-next-line @typescript-eslint/no-deprecated, sonarjs/deprecation */}
          <Linkedin className="mr-2 h-4 w-4" />
          <span>{tPalette('linkedin')}</span>
        </CommandItem>
      ) : null}
      <CommandItem onSelect={emailMe}>
        <Mail className="mr-2 h-4 w-4" />
        <span>{tPalette('email')}</span>
      </CommandItem>
    </CommandGroup>
  )
}

/* ───────────────────────────── main component ─────────────────────────── */

// eslint-disable-next-line max-lines-per-function
export const CommandPalette: FCStrict = (): JSX.Element => {
  const tPalette: Translations<'commandPalette'> =
    useTranslations('commandPalette')
  const tAll: Translations<''> = useTranslations()
  const locale: Locale = useLocale()
  const pathname: string = usePathname() // canonical path (no locale)
  const router: LocalizedRouter = useRouter()

  const [open, setOpen]: [boolean, Dispatch<SetStateAction<boolean>>] =
    useState<boolean>(false)

  const runCommand: (function_: () => void) => void = useCallback(
    (function_: () => void): void => {
      setOpen(false)
      function_()
    },
    []
  )

  useEffect((): (() => void) => {
    const down: (error: KeyboardEvent) => void = (
      error: KeyboardEvent
    ): void => {
      if (error.key === 'k' && (error.metaKey || error.ctrlKey)) {
        error.preventDefault()
        setOpen((previous: boolean): boolean => !previous)
      }
    }
    document.addEventListener('keydown', down)
    return (): void => {
      document.removeEventListener('keydown', down)
    }
  }, [])

  return (
    <CommandDialog
      description={tPalette('description')}
      open={open}
      title={tPalette('title')}
      onOpenChange={setOpen}
    >
      <CommandInput placeholder={tPalette('placeholder')} />
      <CommandList>
        <CommandEmpty>{tPalette('noResults')}</CommandEmpty>
        <NavGroup
          locale={locale}
          router={router}
          run={runCommand}
          tAll={tAll}
          tPalette={tPalette}
        />
        <SectionsGroup
          locale={locale}
          pathname={pathname}
          router={router}
          run={runCommand}
          tAll={tAll}
          tPalette={tPalette}
        />
        <ActionsGroup run={runCommand} tPalette={tPalette} />
      </CommandList>
    </CommandDialog>
  )
}
