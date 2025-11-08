import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'
import fs from 'node:fs'
import path from 'node:path'

const TAGS = [
  'wcag2a',
  'wcag2aa',
  'wcag21a',
  'wcag21aa',
  'wcag22aa',
  'best-practice',
] as const

function readCachedPaths(): string[] {
  const file = path.join(process.cwd(), 'tests/.cache/sitemap-paths.json')
  if (!fs.existsSync(file)) {
    throw new Error(
      'Missing tests/.cache/sitemap-paths.json. Did globalSetup run?'
    )
  }
  const data = JSON.parse(fs.readFileSync(file, 'utf8'))
  if (!Array.isArray(data))
    throw new Error('Cached sitemap paths is not an array')
  return data as string[]
}

const paths = readCachedPaths()

test.describe.parallel('A11y (axe) per sitemap URL', () => {
  for (const p of paths) {
    test(`♿ ${p}`, async ({ page }, testInfo) => {
      await page.goto(p, { waitUntil: 'networkidle' })

      const results = await new AxeBuilder({ page })
        .withTags(TAGS as unknown as string[])
        .analyze()

      if (results.violations.length) {
        await testInfo.attach(`axe-${p.replace(/\W+/g, '_')}.json`, {
          body: JSON.stringify(results, null, 2),
          contentType: 'application/json',
        })
      }

      expect(
        results.violations,
        `A11y violations on ${p}:\n${results.violations
          .map((v) => ` - ${v.id} (${v.impact ?? 'n/a'}): ${v.help}`)
          .join('\n')}`
      ).toEqual([])

      await expect(page.getByRole('main')).toBeVisible({ timeout: 2000 })
    })
  }
})
