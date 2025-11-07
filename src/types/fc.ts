/**
 * Types-only utilities for React components in a strict TS/ESLint setup.
 *
 * Linting approach:
 * - This file only exports types. Some linters running without full type info
 *   can flag exported type parameters/aliases as "unused". We disable that here,
 *   *only* for this file, to avoid false positives without weakening app code.
 */

import type {
  ComponentPropsWithRef,
  ElementType,
  ForwardedRef,
  ForwardRefExoticComponent,
  JSX,
  ReactNode,
  RefAttributes,
} from 'react'

/** ------------------------------------------------------------------------
 *  Helpers
 *  --------------------------------------------------------------------- */

export type NoProperties = Readonly<Record<never, never>>

export interface NoChildren {
  readonly children?: never
}
export interface WithChildren {
  readonly children?: Readonly<ReactNode>
}
export interface WithRequiredChildren {
  readonly children: ReactNode
}

/** ------------------------------------------------------------------------
 *  Core FC shapes (no React.FC)
 *  - Forbid `null` by returning JSX.Element (not ReactElement | null).
 *  - Parameters are Readonly<…> to satisfy prefer-readonly-parameter-types.
 *  --------------------------------------------------------------------- */

export type FCStrict<P = NoProperties> = (
  properties: Readonly<NoChildren & P>
) => JSX.Element

export type FCNullable<P = NoProperties> = (
  properties: Readonly<NoChildren & P>
) => JSX.Element | null

export type FCWithChildren<P = NoProperties> = (
  properties: Readonly<P & WithChildren>
) => JSX.Element

export type FCWithRequiredChildren<P = NoProperties> = (
  properties: Readonly<P & WithRequiredChildren>
) => JSX.Element

/** Async server components (e.g., Next.js Server Components) */
export type FCAsync<P = NoProperties> = (
  properties: Readonly<NoChildren & P>
) => Promise<JSX.Element>

export type FCAsyncWithChildren<P = NoProperties> = (
  properties: Readonly<P & WithChildren>
) => Promise<JSX.Element>

/** ------------------------------------------------------------------------
 *  forwardRef variants
 *  --------------------------------------------------------------------- */

/** Preferred: component *result* type (no callback params → no param linting at all). */
export type ForwardReferenceComponent<P, R> = ForwardRefExoticComponent<
  Readonly<NoChildren & P> & RefAttributes<R>
>

export type ForwardReferenceComponentWithChildren<P, R> =
  ForwardRefExoticComponent<Readonly<P & WithChildren> & RefAttributes<R>>

/**
 * Optional: callback *renderer* type (when you want to annotate the render fn).
 * React's ForwardedRef<R> is a union that can include a function; the readonly rule
 * becomes noisy here. We keep props readonly and suppress the rule for `ref` only.
 */

export type ForwardReferenceRender<P, R> = (
  properties: Readonly<NoChildren & P>,
  reference: ForwardedRef<R>
) => JSX.Element

export type ForwardReferenceRenderWithChildren<P, R> = (
  properties: Readonly<P & WithChildren>,
  reference: ForwardedRef<R>
) => JSX.Element

/** ------------------------------------------------------------------------
 *  Polymorphic "as" pattern
 *  --------------------------------------------------------------------- */

export interface AsProperty<E extends ElementType> {
  readonly as?: E
}

export type PolymorphicComponentProperties<E extends ElementType, P> = Omit<
  ComponentPropsWithRef<E>,
  keyof P | 'as'
> &
  Readonly<AsProperty<E> & P>

export type PolymorphicFCStrict<E extends ElementType, P = NoProperties> = <
  EE extends ElementType = E,
>(
  properties: PolymorphicComponentProperties<EE, NoChildren & P>
) => JSX.Element

export type PolymorphicFCWithChildren<
  E extends ElementType,
  P = NoProperties,
> = <EE extends ElementType = E>(
  properties: PolymorphicComponentProperties<EE, P & WithChildren>
) => JSX.Element

export type PolymorphicFCWithRequiredChildren<
  E extends ElementType,
  P = NoProperties,
> = <EE extends ElementType = E>(
  properties: PolymorphicComponentProperties<EE, P & WithRequiredChildren>
) => JSX.Element

/** ------------------------------------------------------------------------
 *  Next.js convenience aliases
 *  --------------------------------------------------------------------- */

export type LayoutFC = FCWithRequiredChildren

export type PageFC<P = NoProperties> = FCStrict<P>
export type AsyncPageFC<P = NoProperties> = FCAsync<P>
