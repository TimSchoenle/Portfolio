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
type UnknownProperties = Record<string, unknown>

interface CommandPaletteModule {
  CommandPalette: ComponentType<NoProperties>
}
interface EasterEggsModule {
  EasterEggs: ComponentType<NoProperties>
}
interface SonnerModule {
  Toaster: ComponentType<UnknownProperties>
}

/* ---------- typed selectors (no bracket indexing) ---------- */
const pickCommandPalette: (
  moduleNamespace: CommandPaletteModule
) => ComponentType<NoProperties> = (
  moduleNamespace: CommandPaletteModule
): ComponentType<NoProperties> => moduleNamespace.CommandPalette

const pickEasterEggs: (
  moduleNamespace: EasterEggsModule
) => ComponentType<NoProperties> = (
  moduleNamespace: EasterEggsModule
): ComponentType<NoProperties> => moduleNamespace.EasterEggs

const pickToaster: (
  moduleNamespace: SonnerModule
) => ComponentType<UnknownProperties> = (
  moduleNamespace: SonnerModule
): ComponentType<UnknownProperties> => moduleNamespace.Toaster

/* ---------- typed importers (explicit return types) ---------- */
const importCommandPalette: () => Promise<CommandPaletteModule> =
  (): Promise<CommandPaletteModule> =>
    import('@/components/features/command-palette/command-palette')

const importEasterEggs: () => Promise<EasterEggsModule> =
  (): Promise<EasterEggsModule> => import('@/components/features/easter-eggs')

const importSonner: () => Promise<SonnerModule> = (): Promise<SonnerModule> =>
  import('sonner')

/* ---------- lazy client widgets ---------- */
const CommandPalette: ComponentType<NoProperties> = lazyClient<
  NoProperties,
  CommandPaletteModule
>(importCommandPalette, pickCommandPalette)

const EasterEggs: ComponentType<NoProperties> = lazyClient<
  NoProperties,
  EasterEggsModule
>(importEasterEggs, pickEasterEggs)

const Toaster: ComponentType<UnknownProperties> = lazyClient<
  UnknownProperties,
  SonnerModule
>(importSonner, pickToaster)

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

  return (
    <>
      <CommandPalette />
      <EasterEggs />
      <Toaster position="bottom-right" />
    </>
  )
}

export default DeferredClientUi
