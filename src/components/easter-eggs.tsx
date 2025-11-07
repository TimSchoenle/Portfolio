'use client'

import {
  type Dispatch,
  type JSX,
  type SetStateAction,
  useEffect,
  useState,
} from 'react'

import { toast } from 'sonner'

import type { FCNullable } from '@/types/fc'

/* ───────────────────────── constants & types ───────────────────────── */

const KONAMI_SEQUENCE: readonly [
  'ArrowUp',
  'ArrowUp',
  'ArrowDown',
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight',
  'ArrowLeft',
  'ArrowRight',
  'b',
  'a',
] = [
  'ArrowUp',
  'ArrowUp',
  'ArrowDown',
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight',
  'ArrowLeft',
  'ArrowRight',
  'b',
  'a',
]

const KONAMI_LEN: number = KONAMI_SEQUENCE.length
const TRIPLE_CLICK_WINDOW_MS: number = 500

const CLICK_MESSAGES: readonly string[] = [
  '👀 Why are you clicking so much?',
  '🎯 Triple click detected!',
  '🐛 Looking for bugs?',
  '💻 Console.log("Hello, curious developer!")',
  '🎨 Nice clicking skills!',
] as const

/* ───────────────────────── pure helpers ───────────────────────── */

const arraysEqual: (a: readonly string[], b: readonly string[]) => boolean = (
  one: readonly string[],
  two: readonly string[]
): boolean => {
  if (one.length !== two.length) {
    return false
  }
  for (let index: number = 0; index < one.length; index++) {
    const oneElement: string | undefined = one.at(index)
    const twoElement: string | undefined = two.at(index)
    if (oneElement !== twoElement) {
      return false
    }
  }
  return true
}

const activateKonamiEffect: () => void = (): void => {
  toast.success('🎮 Konami Code Activated!', {
    description: 'You found the secret! You are a true gamer.',
    duration: 5000,
  })
  document.body.style.animation = 'rainbow 2s linear infinite'
  window.setTimeout((): void => {
    document.body.style.animation = ''
  }, 5000)
}

/* Build typed handlers outside the component to satisfy
   unicorn/consistent-function-scoping and keep functions tiny. */

const buildKonamiHandler: (
  setKonamiCode: Dispatch<SetStateAction<readonly string[]>>
) => (event: KeyboardEvent) => void = (
  setKonamiCode: Dispatch<SetStateAction<readonly string[]>>
): ((event: KeyboardEvent) => void) => {
  return (event: KeyboardEvent): void => {
    setKonamiCode((previous: readonly string[]): readonly string[] => {
      const next: readonly string[] = [...previous, event.key].slice(
        -KONAMI_LEN
      )
      if (arraysEqual(next, KONAMI_SEQUENCE)) {
        activateKonamiEffect()
        return []
      }
      return next
    })
  }
}

const buildTripleClickHandler: (
  setClickCount: Dispatch<SetStateAction<number>>
) => (error: MouseEvent) => void = (
  setClickCount: Dispatch<SetStateAction<number>>
): ((error: MouseEvent) => void) => {
  return (_error: MouseEvent): void => {
    setClickCount((previous: number): number => {
      const newCount: number = previous + 1
      if (newCount === 3) {
        const messageIndex: number = Math.floor(
          Math.random() * CLICK_MESSAGES.length
        )
        const message: string =
          CLICK_MESSAGES.at(messageIndex) ?? CLICK_MESSAGES.at(0) ?? ''
        toast(message, { duration: 3000 })
        return 0
      }
      window.setTimeout((): void => {
        setClickCount(0)
      }, TRIPLE_CLICK_WINDOW_MS)
      return newCount
    })
  }
}

/* ───────────────────────── component ───────────────────────── */

// Keep the component tiny: hook + effect + fragment.
export const EasterEggs: FCNullable = (): JSX.Element | null => {
  const [_konamiCode, setKonamiCode]: [
    readonly string[],
    Dispatch<SetStateAction<readonly string[]>>,
  ] = useState<readonly string[]>([])
  const [_clickCount, setClickCount]: [
    number,
    Dispatch<SetStateAction<number>>,
  ] = useState<number>(0)

  useEffect((): (() => void) => {
    const handleKeyDown: (event: KeyboardEvent) => void =
      buildKonamiHandler(setKonamiCode)
    const handleClick: (event: MouseEvent) => void =
      buildTripleClickHandler(setClickCount)

    // fun console hints (your global rule forbids console;
    // enable only for these lines)
    // eslint-disable-next-line no-console
    console.log(
      '%c👋 Hello, curious developer!',
      'font-size: 20px; font-weight: bold; color: #2563eb;'
    )
    // eslint-disable-next-line no-console
    console.log(
      '%cTry the Konami Code: ⬆️⬆️⬇️⬇️⬅️➡️⬅️➡️BA',
      'font-size: 14px; color: #64748b;'
    )
    // eslint-disable-next-line no-console
    console.log(
      '%cCmd/Ctrl + K opens the command palette.',
      'font-size: 14px; color: #64748b;'
    )

    document.addEventListener('keydown', handleKeyDown)
    document.addEventListener('click', handleClick)
    return (): void => {
      document.removeEventListener('keydown', handleKeyDown)
      document.removeEventListener('click', handleClick)
    }
  }, [setKonamiCode, setClickCount])

  return null
}
