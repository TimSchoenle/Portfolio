/* eslint-disable no-console */
'use client'

import { useEffect, useState } from 'react'
import { toast } from 'sonner'

export const EasterEggs = () => {
  const [_konamiCode, setKonamiCode] = useState<string[]>([])
  const [_clickCount, setClickCount] = useState(0)

  useEffect(() => {
    const konamiSequence = [
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

    const handleKeyDown = (e: KeyboardEvent) => {
      setKonamiCode((prev) => {
        const newCode = [...prev, e.key].slice(-10)

        if (JSON.stringify(newCode) === JSON.stringify(konamiSequence)) {
          toast.success('🎮 Konami Code Activated!', {
            description: 'You found the secret! You are a true gamer.',
            duration: 5000,
          })

          // Add fun animation
          document.body.style.animation = 'rainbow 2s linear infinite'
          setTimeout(() => {
            document.body.style.animation = ''
          }, 5000)

          return []
        }

        return newCode
      })
    }

    // Triple click easter egg
    const handleClick = () => {
      setClickCount((prev) => {
        const newCount = prev + 1

        if (newCount === 3) {
          const messages = [
            '👀 Why are you clicking so much?',
            '🎯 Triple click detected!',
            '🐛 Looking for bugs?',
            '💻 Console.log("Hello, curious developer!")',
            '🎨 Nice clicking skills!',
          ]
          const randomMessage =
            messages[Math.floor(Math.random() * messages.length)]
          toast(randomMessage, { duration: 3000 })
          return 0
        }

        setTimeout(() => setClickCount(0), 500)
        return newCount
      })
    }

    // Console message
    console.log(
      '%c👋 Hello, curious developer!',
      'font-size: 20px; font-weight: bold; color: #2563eb;'
    )
    console.log(
      '%cLooking for secrets? Try the Konami Code! ⬆️⬆️⬇️⬇️⬅️➡️⬅️➡️BA',
      'font-size: 14px; color: #64748b;'
    )
    console.log(
      '%cOr press Cmd/Ctrl + K for the command palette!',
      'font-size: 14px; color: #64748b;'
    )

    document.addEventListener('keydown', handleKeyDown)
    document.addEventListener('click', handleClick)

    return () => {
      document.removeEventListener('keydown', handleKeyDown)
      document.removeEventListener('click', handleClick)
    }
  }, [])

  return null
}
