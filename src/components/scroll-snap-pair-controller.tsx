'use client'

import { type EffectCallback, type FC, type JSX, useEffect } from 'react'

export interface ScrollSnapPairOptions {
  /**
   * Minimum absolute wheel delta to react to (filters out tiny jitters).
   * Default: 12
   */
  readonly minDelta?: number
  /**
   * Pixel tolerance when deciding a smooth scroll is "finished".
   * Default: 2
   */
  readonly snapTolerance?: number
  /**
   * Pixel band around the top of the viewport where a section is considered "at the top".
   * Default: 64
   */
  readonly topZone?: number
}

export interface ScrollSnapPairControllerProperties {
  /**
   * The DOM id of the "bottom" section (e.g. about).
   */
  readonly bottomSectionId: string
  /**
   * Optional tuning parameters.
   */
  readonly options?: ScrollSnapPairOptions
  /**
   * The DOM id of the "top" section (e.g. hero).
   */
  readonly topSectionId: string
}

/* ---------- internal types ---------- */

interface ResolvedOptions {
  readonly minDelta: number
  readonly snapTolerance: number
  readonly topZone: number
}

interface ScrollSnapState {
  bottomElement: HTMLElement
  isSnapping: boolean
  options: ResolvedOptions
  topElement: HTMLElement
}

interface GeometrySnapshot {
  bottomRect: DOMRect
  bottomSectionTopY: number
  topRect: DOMRect
  topSectionTopY: number
}

/* ---------- small helpers ---------- */

function resolveOptions(options?: ScrollSnapPairOptions): ResolvedOptions {
  const minDelta: number = options?.minDelta ?? 12
  const topZone: number = options?.topZone ?? 64
  const snapTolerance: number = options?.snapTolerance ?? 2

  return { minDelta, snapTolerance, topZone }
}

function takeGeometrySnapshot(state: ScrollSnapState): GeometrySnapshot {
  const topRect: DOMRect = state.topElement.getBoundingClientRect()
  const bottomRect: DOMRect = state.bottomElement.getBoundingClientRect()
  const scrollY: number = window.scrollY

  const topSectionTopY: number = scrollY + topRect.top
  const bottomSectionTopY: number = scrollY + bottomRect.top

  return {
    bottomRect,
    bottomSectionTopY,
    topRect,
    topSectionTopY,
  }
}

function startSmoothScroll(targetY: number, state: ScrollSnapState): void {
  state.isSnapping = true

  const timeoutId: ReturnType<typeof setTimeout> = setTimeout((): void => {
    state.isSnapping = false
    window.removeEventListener('scroll', handleScroll)
  }, 2000) // Cleanup after reasonable timeout

  function handleScroll(): void {
    const distance: number = Math.abs(window.scrollY - targetY)

    if (distance <= state.options.snapTolerance) {
      clearTimeout(timeoutId)
      state.isSnapping = false
      window.removeEventListener('scroll', handleScroll)
    }
  }

  window.addEventListener('scroll', handleScroll)
  window.scrollTo({ behavior: 'smooth', top: targetY })
}

function shouldSnapDown(
  deltaY: number,
  snapshot: GeometrySnapshot,
  options: ResolvedOptions
): boolean {
  if (deltaY <= 0) {
    return false
  }

  const topNearTop: boolean =
    snapshot.topRect.top >= -options.topZone &&
    snapshot.topRect.top <= options.topZone

  const bottomBelowViewportTop: boolean =
    snapshot.bottomRect.top > options.topZone

  return topNearTop && bottomBelowViewportTop
}

function shouldSnapUp(
  deltaY: number,
  snapshot: GeometrySnapshot,
  options: ResolvedOptions
): boolean {
  if (deltaY >= 0) {
    return false
  }

  const bottomNearTop: boolean =
    snapshot.bottomRect.top >= -options.topZone &&
    snapshot.bottomRect.top <= options.topZone

  const topAboveViewport: boolean = snapshot.topRect.bottom <= 0

  return bottomNearTop && topAboveViewport
}

function createWheelHandler(
  state: ScrollSnapState
): (event: WheelEvent) => void {
  function handleWheel(event: WheelEvent): void {
    if (state.isSnapping) {
      return
    }

    const deltaY: number = event.deltaY
    const minDelta: number = state.options.minDelta

    // Ignore tiny trackpad / wheel noise
    if (Math.abs(deltaY) < minDelta) {
      return
    }

    const snapshot: GeometrySnapshot = takeGeometrySnapshot(state)

    // DOWN: top section -> bottom section
    if (shouldSnapDown(deltaY, snapshot, state.options)) {
      event.preventDefault()
      startSmoothScroll(snapshot.bottomSectionTopY, state)
      return
    }

    // UP: bottom section -> top section
    if (shouldSnapUp(deltaY, snapshot, state.options)) {
      event.preventDefault()
      startSmoothScroll(snapshot.topSectionTopY, state)
    }
  }

  return handleWheel
}

/* ---------- component ---------- */

export const ScrollSnapPairController: FC<
  ScrollSnapPairControllerProperties
> = (properties: ScrollSnapPairControllerProperties): JSX.Element | null => {
  const effectCallback: EffectCallback = (): (() => void) | undefined => {
    const topSelector: string = `#${properties.topSectionId}`
    const bottomSelector: string = `#${properties.bottomSectionId}`

    const topElement: HTMLElement | null =
      document.querySelector<HTMLElement>(topSelector)
    const bottomElement: HTMLElement | null =
      document.querySelector<HTMLElement>(bottomSelector)

    if (topElement === null || bottomElement === null) {
      return
    }

    const resolvedOptions: ResolvedOptions = resolveOptions(properties.options)

    const state: ScrollSnapState = {
      bottomElement,
      isSnapping: false,
      options: resolvedOptions,
      topElement,
    }

    const wheelHandler: (event: WheelEvent) => void = createWheelHandler(state)

    window.addEventListener('wheel', wheelHandler, { passive: false })

    return (): void => {
      window.removeEventListener('wheel', wheelHandler)
    }
  }

  useEffect(effectCallback, [
    properties.topSectionId,
    properties.bottomSectionId,
    properties.options,
  ])

  return null
}
