import {
  type createTranslator,
  type Locale,
  type Messages,
  type NamespaceKeys,
  type NestedKeyOf,
} from 'next-intl'

type Ns = NamespaceKeys<Messages, NestedKeyOf<Messages>>
type NormalizeAll<N> = [N] extends [''] ? never : N

/** Translator type. Requires a generic. `''` means “all messages”. */
export type Translations<N extends Ns> = ReturnType<
  typeof createTranslator<Messages, NormalizeAll<N> & Ns>
>

export interface UnparsedLocalePageProperties {
  // This needs to stay "local" to match the type of the `locale` prop
  readonly locale: string
}

export interface LocalePageProperties {
  readonly locale: Locale
}
