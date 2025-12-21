'use client'

import {
  type ComponentType,
  type Dispatch,
  type JSX,
  type SetStateAction,
  useEffect,
  useState,
} from 'react'

import { lazyClient } from '@/lib/lazy-client'
import type { FCNullable } from '@/types/fc'

/* ---------- local types ---------- */
type NoProperties = Record<string, never>

interface CommandPaletteModule {
  CommandPalette: ComponentType<NoProperties>
}

/* ---------- typed selectors (no bracket indexing) ---------- */
const pickCommandPalette: (
  moduleNamespace: CommandPaletteModule
) => ComponentType<NoProperties> = (
  moduleNamespace: CommandPaletteModule
): ComponentType<NoProperties> => moduleNamespace.CommandPalette

/* ---------- typed importers (explicit return types) ---------- */
const importCommandPalette: () => Promise<CommandPaletteModule> =
  (): Promise<CommandPaletteModule> =>
    import('@/components/features/command-palette/command-palette')

/* ---------- lazy client widgets ---------- */
const CommandPalette: ComponentType<NoProperties> = lazyClient<
  NoProperties,
  CommandPaletteModule
>(importCommandPalette, pickCommandPalette)

const useIdleFlag: () => boolean = (): boolean => {
  const [ready, setReady]: [boolean, Dispatch<SetStateAction<boolean>>] =
    useState<boolean>(false)

  useEffect((): (() => void) => {
    const onIdle: () => void = (): void => {
      setReady(true)
    }

    const ricUnknown: unknown = (window as { requestIdleCallback?: unknown })
      .requestIdleCallback
    if (typeof ricUnknown === 'function') {
      const requestIdleCallbackFunction: (
        callback: () => void,
        options?: { timeout?: number }
      ) => number = ricUnknown as (
        callback: () => void,
        options?: { timeout?: number }
      ) => number

      const handle: number = requestIdleCallbackFunction(onIdle)

      return (): void => {
        const cicUnknown: unknown = (window as { cancelIdleCallback?: unknown })
          .cancelIdleCallback
        if (typeof cicUnknown === 'function') {
          const cancelIdleCallbackFunction: (id: number) => void =
            cicUnknown as (id: number) => void
          cancelIdleCallbackFunction(handle)
        }
      }
    }

    const timeoutId: number = window.setTimeout(onIdle, 0)
    return (): void => {
      clearTimeout(timeoutId)
    }
  }, [])

  return ready
}

/* ---------- component ---------- */
const DeferredClientUi: FCNullable = (): JSX.Element | null => {
  const show: boolean = useIdleFlag()
  if (!show) {
    return null
  }

  return <CommandPalette />
}

export default DeferredClientUi
