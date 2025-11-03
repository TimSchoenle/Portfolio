'use client'

import { useEffect, useState, useCallback } from 'react'
import { useRouter } from 'next/navigation'
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import {
  Home,
  User,
  Code,
  Briefcase,
  MessageSquare,
  Github,
  Mail,
  FileText,
} from 'lucide-react'
import { useLocale, useTranslations } from 'next-intl'
import { siteConfig } from '@/lib/config'

export function CommandPalette() {
  const paletteT = useTranslations('commandPalette')
  const allT = useTranslations()

  const locale = useLocale()

  const [open, setOpen] = useState(false)
  const router = useRouter()

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault()
        setOpen((open) => !open)
      }
    }

    document.addEventListener('keydown', down)
    return () => document.removeEventListener('keydown', down)
  }, [])

  const runCommand = useCallback((command: () => void) => {
    setOpen(false)
    command()
  }, [])

  const scrollToSection = (id: string) => {
    const element = document.getElementById(id)
    if (element) {
      element.scrollIntoView({ behavior: 'smooth' })
    }
  }

  return (
    <CommandDialog open={open} onOpenChange={setOpen}>
      <CommandInput placeholder={paletteT('placeholder')} />
      <CommandList>
        <CommandEmpty>{paletteT('noResults')}</CommandEmpty>

        <CommandGroup heading={paletteT('navigation')}>
          <CommandItem
            onSelect={() => runCommand(() => router.push(`/${locale}`))}
          >
            <Home className="mr-2 h-4 w-4" />
            <span>{paletteT('home')}</span>
          </CommandItem>
          <CommandItem
            onSelect={() => runCommand(() => router.push(`/${locale}/imprint`))}
          >
            <FileText className="mr-2 h-4 w-4" />
            <span>{allT('imprint.title')}</span>
          </CommandItem>
        </CommandGroup>

        <CommandGroup heading={paletteT('sections')}>
          <CommandItem
            onSelect={() => runCommand(() => scrollToSection('about'))}
          >
            <User className="mr-2 h-4 w-4" />
            <span>{allT('about.title')}</span>
          </CommandItem>
          <CommandItem
            onSelect={() => runCommand(() => scrollToSection('skills'))}
          >
            <Code className="mr-2 h-4 w-4" />
            <span>{allT('skills.title')}</span>
          </CommandItem>
          <CommandItem
            onSelect={() => runCommand(() => scrollToSection('projects'))}
          >
            <Briefcase className="mr-2 h-4 w-4" />
            <span>{allT('projects.title')}</span>
          </CommandItem>
          <CommandItem
            onSelect={() => runCommand(() => scrollToSection('contact'))}
          >
            <MessageSquare className="mr-2 h-4 w-4" />
            <span>{allT('contact.title')}</span>
          </CommandItem>
        </CommandGroup>

        <CommandGroup heading={paletteT('actions')}>
          <CommandItem
            onSelect={() =>
              runCommand(() => window.open(siteConfig.github, '_blank'))
            }
          >
            <Github className="mr-2 h-4 w-4" />
            <span>{paletteT('github')}</span>
          </CommandItem>
          <CommandItem
            onSelect={() =>
              runCommand(
                () => (window.location.href = `mailto:${siteConfig.email}`)
              )
            }
          >
            <Mail className="mr-2 h-4 w-4" />
            <span>{paletteT('email')}</span>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  )
}
