import { describe, it, expect } from 'vitest'
import { GET } from '../route'

describe('Health API Route', () => {
  it('returns healthy status', () => {
    const response = GET()

    expect(response).toBeDefined()
    expect(response.status).toBe(200)
  })

  it('returns JSON response', async () => {
    const response = GET()
    const data = await response.json()

    expect(data.status).toBe('healthy')
    expect(data.timestamp).toBeDefined()
  })

  it('includes no-cache headers', () => {
    const response = GET()
    const cacheControl = response.headers.get('Cache-Control')

    expect(cacheControl).toBe('no-store')
  })

  it('returns ISO timestamp', async () => {
    const response = GET()
    const data = await response.json()

    expect(typeof data.timestamp).toBe('string')
    expect(new Date(data.timestamp).toISOString()).toBe(data.timestamp)
  })
})
