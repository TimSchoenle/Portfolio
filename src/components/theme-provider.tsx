'use client'

import * as React from 'react'
import { type JSX } from 'react'

import type { FCWithChildren, WithChildren } from '@/types/fc'

export type Theme = 'dark' | 'light'

interface ThemeProviderProps extends WithChildren {
  readonly defaultTheme?: Theme
}

interface ThemeProviderState {
  readonly theme: Theme
  readonly setTheme: (theme: Theme) => void
}

const ThemeProviderContext: React.Context<ThemeProviderState | undefined> =
  React.createContext<ThemeProviderState | undefined>(undefined)

export const ThemeProvider: FCWithChildren<ThemeProviderProps> = ({
  children,
  defaultTheme = 'dark',
}: ThemeProviderProps): JSX.Element => {
  const [theme, setTheme]: [
    Theme,
    React.Dispatch<React.SetStateAction<Theme>>,
  ] = React.useState<Theme>(defaultTheme)

  React.useEffect((): void => {
    const root: HTMLElement = window.document.documentElement
    const savedTheme: Theme | null = localStorage.getItem(
      'theme'
    ) as Theme | null

    const next: Theme = savedTheme ?? defaultTheme
    setTheme(next)
    root.classList.remove('light', 'dark')
    root.classList.add(next)
  }, [defaultTheme])

  const value: ThemeProviderState = React.useMemo<ThemeProviderState>(
    (): ThemeProviderState => ({
      theme,
      setTheme: (newTheme: Theme): void => {
        localStorage.setItem('theme', newTheme)
        const root: HTMLElement = window.document.documentElement
        root.classList.remove('light', 'dark')
        root.classList.add(newTheme)
        setTheme(newTheme)
      },
    }),
    [theme]
  )

  return (
    <ThemeProviderContext.Provider value={value}>
      {children}
    </ThemeProviderContext.Provider>
  )
}

export const useTheme: () => ThemeProviderState = (): ThemeProviderState => {
  const context: ThemeProviderState | undefined =
    React.useContext(ThemeProviderContext)
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider')
  }
  return context
}
