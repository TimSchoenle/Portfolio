import {
  type createTranslator,
  type Locale,
  type Messages,
  type NamespaceKeys,
  type NestedKeyOf,
} from 'next-intl'

type Nested = NamespaceKeys<Messages, NestedKeyOf<Messages>>

export type Translations<N extends Nested> = ReturnType<
  typeof createTranslator<Messages, N>
>

export interface UnparsedLocalePageProps {
  // This needs to stay "local" to match the type of the `locale` prop
  readonly locale: string
}

export interface LocalePageProps {
  readonly locale: Locale
}
