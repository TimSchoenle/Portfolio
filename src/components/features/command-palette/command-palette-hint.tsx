'use client'

import React, { type JSX, useEffect, useState } from 'react'

import { BlueprintLabel } from '@/components/blueprint/blueprint-label'
import { COMMAND_PALETTE_SHORTCUT } from '@/lib/constants/keyboard'
import type { FCNullable } from '@/types/fc'

/**
 * Client component that detects the user's actual keyboard layout
 * and displays the correct keyboard shortcut for the command palette.
 * Uses the Keyboard Layout Map API to get real key labels.
 * Returns null if the API is not supported.
 */
const getModifierLabel: (
  layoutMap: Map<string, string>,
  keys: readonly string[]
) => string | undefined = (
  layoutMap: Map<string, string>,
  keys: readonly string[]
): string | undefined =>
  keys
    .map((key: string): string | undefined => layoutMap.get(key))
    .find((label: string | undefined): boolean => label !== undefined)

// Helper to get keyboard shortcut label
const buildShortcutLabel: (layoutMap: Map<string, string>) => string = (
  layoutMap: Map<string, string>
): string => {
  // Get the appropriate modifier for the current OS
  const userAgent: string = navigator.userAgent
  const isMac: boolean = /Mac|iPhone|iPod|iPad/i.test(userAgent)

  const modifierName: string = isMac
    ? COMMAND_PALETTE_SHORTCUT.modifiers.mac
    : COMMAND_PALETTE_SHORTCUT.modifiers.default

  // Get layout map keys for this modifier
  const layoutKeys: readonly string[] = ((): readonly string[] => {
    if (modifierName === 'Ctrl') {
      return ['ControlLeft', 'ControlRight']
    }
    if (modifierName === 'Cmd') {
      return ['MetaLeft', 'MetaRight']
    }
    if (modifierName === 'Shift') {
      return ['ShiftLeft', 'ShiftRight']
    }
    return ['AltLeft', 'AltRight']
  })()

  const modifierLabel: string | undefined = getModifierLabel(
    layoutMap,
    layoutKeys
  )
  const modifierKey: string = modifierLabel ?? modifierName

  const key: string = COMMAND_PALETTE_SHORTCUT.key

  return `${modifierKey}+${key}`
}

export const CommandPaletteHint: FCNullable = (): JSX.Element | null => {
  const [shortcutLabel, setShortcutLabel]: [
    string | null,
    React.Dispatch<React.SetStateAction<string | null>>,
  ] = useState<string | null>(null)

  useEffect((): void => {
    const getKeyboardShortcut: () => Promise<void> =
      async (): Promise<void> => {
        if (!('keyboard' in navigator)) {
          setShortcutLabel(null)
          return
        }

        try {
          const keyboard: { getLayoutMap: () => Promise<Map<string, string>> } =
            (
              navigator as Navigator & {
                keyboard: { getLayoutMap: () => Promise<Map<string, string>> }
              }
            ).keyboard
          const layoutMap: Map<string, string> = await keyboard.getLayoutMap()

          setShortcutLabel(buildShortcutLabel(layoutMap))
        } catch {
          setShortcutLabel(null)
        }
      }

    void getKeyboardShortcut()
  }, [])

  const shouldRenderHint: boolean =
    shortcutLabel !== null && shortcutLabel !== ''

  return shouldRenderHint ? (
    <BlueprintLabel className="writing-vertical-rl absolute right-20 bottom-20 font-mono text-[10px] text-brand/40 select-none">
      {shortcutLabel}
    </BlueprintLabel>
  ) : null
}
