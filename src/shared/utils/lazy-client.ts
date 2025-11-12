import type { ComponentType, JSX } from 'react'

import dynamic from 'next/dynamic'

export interface LazyClientOptions {
  loading?: () => JSX.Element | null
}

/**
 * Lazily load a client-only component using a selector function.
 * - No bracket indexing (avoids object-injection lint)
 * - No unnecessary generics
 * - Explicit return types everywhere
 */
export function lazyClient<P, M>(
  importer: () => Promise<M>,
  select: (moduleNamespace: M) => ComponentType<P>,
  options?: LazyClientOptions
): ComponentType<P> {
  const loader: () => Promise<ComponentType<P>> = (): Promise<
    ComponentType<P>
  > =>
    importer().then(
      (moduleNamespace: M): ComponentType<P> => select(moduleNamespace)
    )

  return dynamic<P>(loader, {
    loading: options?.loading ?? ((): null => null),
    ssr: false,
  })
}

/** Convenience for default-exported components. */
export function lazyClientDefault<P, M extends { default: ComponentType<P> }>(
  importer: () => Promise<M>,
  options?: LazyClientOptions
): ComponentType<P> {
  const loader: () => Promise<ComponentType<P>> = (): Promise<
    ComponentType<P>
  > =>
    importer().then(
      (moduleNamespace: M): ComponentType<P> => moduleNamespace.default
    )

  return dynamic<P>(loader, {
    loading: options?.loading ?? ((): null => null),
    ssr: false,
  })
}
