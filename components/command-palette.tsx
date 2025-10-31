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
  BookOpen,
  Github,
  Mail,
  FileText,
} from 'lucide-react'
import { CommandPaletteDictionary, Dictionary } from '@/lib/dictionary'

interface CommandPaletteProps {
  locale: string
  dict: Dictionary
}

export function CommandPalette({ locale, dict }: CommandPaletteProps) {
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
      <CommandInput
        placeholder={
          dict.commandPalette?.placeholder || 'Type a command or search...'
        }
      />
      <CommandList>
        <CommandEmpty>
          {dict.commandPalette?.noResults || 'No results found.'}
        </CommandEmpty>

        <CommandGroup heading={dict.commandPalette?.navigation || 'Navigation'}>
          <CommandItem
            onSelect={() => runCommand(() => router.push(`/${locale}`))}
          >
            <Home className="mr-2 h-4 w-4" />
            <span>{dict.commandPalette?.home || 'Home'}</span>
          </CommandItem>
          <CommandItem
            onSelect={() => runCommand(() => router.push(`/${locale}/imprint`))}
          >
            <FileText className="mr-2 h-4 w-4" />
            <span>{dict.commandPalette?.imprint || 'Imprint'}</span>
          </CommandItem>
        </CommandGroup>

        <CommandGroup heading={dict.commandPalette?.sections || 'Sections'}>
          <CommandItem
            onSelect={() => runCommand(() => scrollToSection('about'))}
          >
            <User className="mr-2 h-4 w-4" />
            <span>{dict.about?.title || 'About'}</span>
          </CommandItem>
          <CommandItem
            onSelect={() => runCommand(() => scrollToSection('skills'))}
          >
            <Code className="mr-2 h-4 w-4" />
            <span>{dict.skills?.title || 'Skills'}</span>
          </CommandItem>
          <CommandItem
            onSelect={() => runCommand(() => scrollToSection('projects'))}
          >
            <Briefcase className="mr-2 h-4 w-4" />
            <span>{dict.projects?.title || 'Projects'}</span>
          </CommandItem>
          <CommandItem
            onSelect={() => runCommand(() => scrollToSection('contact'))}
          >
            <MessageSquare className="mr-2 h-4 w-4" />
            <span>{dict.contact?.title || 'Contact'}</span>
          </CommandItem>
        </CommandGroup>

        <CommandGroup heading={dict.commandPalette?.actions || 'Actions'}>
          <CommandItem
            onSelect={() =>
              runCommand(() =>
                window.open('https://github.com/Timmi6790', '_blank')
              )
            }
          >
            <Github className="mr-2 h-4 w-4" />
            <span>{dict.commandPalette?.github || 'Open GitHub Profile'}</span>
          </CommandItem>
          <CommandItem
            onSelect={() =>
              runCommand(
                () => (window.location.href = 'mailto:contact@timmi6790.de')
              )
            }
          >
            <Mail className="mr-2 h-4 w-4" />
            <span>{dict.commandPalette?.email || 'Send Email'}</span>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  )
}
