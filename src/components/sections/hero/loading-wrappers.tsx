'use client'

import { type ComponentType } from 'react'

import dynamic from 'next/dynamic'

import type * as CommandPaletteModule from '@/components/features/command-palette/command-palette-hint'
import type * as ScrollSnapModule from '@/components/features/scroll-snap/scroll-snap-pair-controller'

export const DynamicCommandPaletteHint: ComponentType = dynamic(
  async (): Promise<ComponentType> =>
    // Delay loading by 500ms to prioritize LCP
    new Promise((resolve: (value: ComponentType) => void): void => {
      const requestIdleCallbackUnknown: unknown =
        typeof window === 'object'
          ? (window as { requestIdleCallback?: unknown }).requestIdleCallback
          : undefined

      if (typeof requestIdleCallbackUnknown === 'function') {
        const requestIdleCallbackFunction: (callback: () => void) => number =
          requestIdleCallbackUnknown as (callback: () => void) => number
        requestIdleCallbackFunction((): void => {
          void import('@/components/features/command-palette/command-palette-hint').then(
            (module_: typeof CommandPaletteModule): void => {
              resolve(module_.CommandPaletteHint)
            }
          )
        })
      } else {
        // Fallback to setTimeout
        setTimeout((): void => {
          void import('@/components/features/command-palette/command-palette-hint').then(
            (module_: typeof CommandPaletteModule): void => {
              resolve(module_.CommandPaletteHint)
            }
          )
        }, 500)
      }
    }),
  { ssr: false }
)

export const DynamicScrollSnapPairController: ComponentType<ScrollSnapModule.ScrollSnapPairControllerProperties> =
  dynamic(
    async (): Promise<
      ComponentType<ScrollSnapModule.ScrollSnapPairControllerProperties>
    > =>
      import('@/components/features/scroll-snap/scroll-snap-pair-controller').then(
        (
          module_: typeof ScrollSnapModule
        ): ComponentType<ScrollSnapModule.ScrollSnapPairControllerProperties> =>
          module_.ScrollSnapPairController
      ),
    { ssr: false }
  )
