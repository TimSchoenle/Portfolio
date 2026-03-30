import React, { type JSX } from 'react'

import { Code2, Mail, Network } from 'lucide-react'

import {
  CommandGroup,
  CommandItem,
  CommandShortcut,
} from '@/components/ui/command'
import { siteConfig } from '@/data/config'
import type { Translations } from '@/types/i18n'

import { openNewTab, sendMailTo } from '../utils/actions'

const SHORTCUT_E: string = 'E'
const SHORTCUT_G: string = 'G'
const SHORTCUT_L: string = 'L'

interface ActionsGroupProperties {
  readonly run: (function_: () => void) => void
  readonly tPalette: Translations<'commandPalette'>
}

export const ActionsGroup: React.FC<ActionsGroupProperties> = ({
  run,
  tPalette,
}: ActionsGroupProperties): JSX.Element => {
  const goGitHub: (_value: string) => void = (): void => {
    run((): void => {
      openNewTab(siteConfig.socials.github)
    })
  }

  const goLinkedIn: (_value: string) => void = (): void => {
    run((): void => {
      // eslint-disable-next-line eqeqeq
      if (siteConfig.socials.linkedin != null) {
        openNewTab(siteConfig.socials.linkedin)
      }
    })
  }

  const emailMe: (_value: string) => void = (): void => {
    run((): void => {
      sendMailTo(siteConfig.email)
    })
  }

  return (
    <CommandGroup heading={tPalette('actions.heading')}>
      <CommandItem
        keywords={['email', 'contact', 'mail']}
        value="email"
        onSelect={emailMe}
      >
        <Mail className="mr-2 h-4 w-4" />
        <span>{tPalette('actions.email')}</span>
        <CommandShortcut>{SHORTCUT_E}</CommandShortcut>
      </CommandItem>
      {Boolean(siteConfig.socials.linkedin) && (
        <CommandItem
          keywords={['linkedin', 'social', 'profile']}
          value="linkedin"
          onSelect={goLinkedIn}
        >
          <Network className="mr-2 h-4 w-4" />
          <span>{tPalette('actions.linkedin')}</span>
          <CommandShortcut>{SHORTCUT_L}</CommandShortcut>
        </CommandItem>
      )}
      <CommandItem
        keywords={['github', 'code', 'source']}
        value="github"
        onSelect={goGitHub}
      >
        <Code2 className="mr-2 h-4 w-4" />
        <span>{tPalette('actions.github')}</span>
        <CommandShortcut>{SHORTCUT_G}</CommandShortcut>
      </CommandItem>
    </CommandGroup>
  )
}
