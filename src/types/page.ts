import type { Metadata } from 'next'

import type { FCAsync, FCAsyncWithChildren, WithChildren } from '@/types/fc'

export interface PageParameters<P extends object> {
  readonly params: Promise<P>
}

export interface PageParametersWithChildren<P extends object>
  extends PageParameters<P>, WithChildren {}

export type RoutePageFC<P extends object> = FCAsync<PageParameters<P>>

export type RoutePageWithChildrenFC<P extends object> = FCAsyncWithChildren<
  PageParameters<P>
>

export type GenerateMetadataFC<P extends object> = (
  arguments_: PageParameters<P>
) => Promise<Metadata>
