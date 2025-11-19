import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { Button } from '../ui/button'
import { Card } from '../ui/card'
import { Heading } from '../ui/heading'

describe('UI Components', () => {
  describe('Button', () => {
    it('renders children', () => {
      render(<Button>Click me</Button>)
      expect(screen.getByRole('button', { name: 'Click me' })).toBeDefined()
    })
  })

  describe('Card', () => {
    it('renders content', () => {
      render(<Card>Card Content</Card>)
      expect(screen.getByText('Card Content')).toBeDefined()
    })
  })

  describe('Heading', () => {
    it('renders as h2 by default', () => {
      render(<Heading>Title</Heading>)
      const heading = screen.getByRole('heading', { level: 2 })
      expect(heading.textContent).toBe('Title')
    })

    it('renders as h1 when specified', () => {
      render(<Heading as="h1">Title</Heading>)
      const heading = screen.getByRole('heading', { level: 1 })
      expect(heading.textContent).toBe('Title')
    })
  })
})
