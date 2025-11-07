'use client'

import { type JSX } from 'react'

import { Moon, Sun } from 'lucide-react'

import { type Theme, useTheme } from '@/components/theme-provider'
import { Button } from '@/components/ui/button'
import type { FCStrict } from '@/types/fc'

export const ThemeToggle: FCStrict = (): JSX.Element => {
  const { setTheme, theme }: { theme: Theme; setTheme: (t: Theme) => void } =
    useTheme()

  const toggleTheme: () => void = (): void => {
    const next: Theme = theme === 'light' ? 'dark' : 'light'
    setTheme(next)
  }

  const srLabel: string = 'Toggle theme'

  return (
    <Button
      aria-label={srLabel}
      className="fixed top-4 right-20 z-50 bg-transparent"
      size="sm"
      variant="outline"
      onClick={toggleTheme}
    >
      <Sun className="h-4 w-4 scale-100 rotate-0 transition-all dark:scale-0 dark:-rotate-90" />
      <Moon className="absolute h-4 w-4 scale-0 rotate-90 transition-all dark:scale-100 dark:rotate-0" />
    </Button>
  )
}
