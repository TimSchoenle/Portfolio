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

export type NoProps = Readonly<Record<never, never>>

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

export type FCStrict<P = NoProps> = (
  props: Readonly<P & NoChildren>
) => JSX.Element

export type FCNullable<P = NoProps> = (
  props: Readonly<P & NoChildren>
) => JSX.Element | null

export type FCWithChildren<P = NoProps> = (
  props: Readonly<P & WithChildren>
) => JSX.Element

export type FCWithRequiredChildren<P = NoProps> = (
  props: Readonly<P & WithRequiredChildren>
) => JSX.Element

/** Async server components (e.g., Next.js Server Components) */
export type FCAsync<P = NoProps> = (
  props: Readonly<P & NoChildren>
) => Promise<JSX.Element>

export type FCAsyncWithChildren<P = NoProps> = (
  props: Readonly<P & WithChildren>
) => Promise<JSX.Element>

/** ------------------------------------------------------------------------
 *  forwardRef variants
 *  --------------------------------------------------------------------- */

/** Preferred: component *result* type (no callback params → no param linting at all). */
export type ForwardRefComponent<P, R> = ForwardRefExoticComponent<
  Readonly<P & NoChildren> & RefAttributes<R>
>

export type ForwardRefComponentWithChildren<P, R> = ForwardRefExoticComponent<
  Readonly<P & WithChildren> & RefAttributes<R>
>

/**
 * Optional: callback *renderer* type (when you want to annotate the render fn).
 * React's ForwardedRef<R> is a union that can include a function; the readonly rule
 * becomes noisy here. We keep props readonly and suppress the rule for `ref` only.
 */

export type ForwardRefRender<P, R> = (
  props: Readonly<P & NoChildren>,
  ref: ForwardedRef<R>
) => JSX.Element

export type ForwardRefRenderWithChildren<P, R> = (
  props: Readonly<P & WithChildren>,
  ref: ForwardedRef<R>
) => JSX.Element

/** ------------------------------------------------------------------------
 *  Polymorphic "as" pattern
 *  --------------------------------------------------------------------- */

export interface AsProp<E extends ElementType> {
  readonly as?: E
}

export type PolymorphicComponentProps<E extends ElementType, P> = Readonly<
  P & AsProp<E>
> &
  Omit<ComponentPropsWithRef<E>, keyof P | 'as'>

export type PolymorphicFCStrict<E extends ElementType, P = NoProps> = <
  EE extends ElementType = E,
>(
  props: PolymorphicComponentProps<EE, P & NoChildren>
) => JSX.Element

export type PolymorphicFCWithChildren<E extends ElementType, P = NoProps> = <
  EE extends ElementType = E,
>(
  props: PolymorphicComponentProps<EE, P & WithChildren>
) => JSX.Element

export type PolymorphicFCWithRequiredChildren<
  E extends ElementType,
  P = NoProps,
> = <EE extends ElementType = E>(
  props: PolymorphicComponentProps<EE, P & WithRequiredChildren>
) => JSX.Element

/** ------------------------------------------------------------------------
 *  Next.js convenience aliases
 *  --------------------------------------------------------------------- */

export type LayoutFC = FCWithRequiredChildren

export type PageFC<P = NoProps> = FCStrict<P>
export type AsyncPageFC<P = NoProps> = FCAsync<P>
