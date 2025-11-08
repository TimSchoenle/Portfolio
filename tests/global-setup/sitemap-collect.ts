import fs from 'node:fs'
import fsp from 'node:fs/promises'
import path from 'node:path'
import { gunzipSync } from 'node:zlib'

export interface CrawlOptions {
  cacheFile?: string // absolute path for cache JSON
  exclude?: RegExp[]
  maxPages?: number
  origin: string
  roots?: string[] // absolute sitemap root URLs (xml or index)
}

const DEFAULT_EXCLUDE = [/^\/api\//, /^\/_next\//]
const DEFAULT_MAX_PAGES = 1000
const DEFAULT_CACHE_FILE = path.join(
  process.cwd(),
  'tests/.cache/sitemap-paths.json'
)

function normalizePath(p: string): string {
  const noTrail = p.replace(/\/+$/, '')
  return noTrail === '' ? '/' : noTrail
}

function toLocalPath(
  u: string,
  origin: string,
  exclude: RegExp[]
): string | null {
  try {
    const url = new URL(u, origin)
    const p = normalizePath(url.pathname)
    if (!p.startsWith('/')) {
      return null
    }
    if (exclude.some((rx) => rx.test(p))) {
      return null
    }
    return p
  } catch {
    return null
  }
}

async function fetchText(url: string): Promise<string> {
  const res = await fetch(url, { headers: { 'accept-encoding': 'gzip' } })
  if (!res.ok) {
    throw new Error(`${url} -> ${res.status}`)
  }
  const ct = (res.headers.get('content-type') ?? '').toLowerCase()
  if (url.endsWith('.gz') || ct.includes('application/gzip')) {
    const buf = Buffer.from(await res.arrayBuffer())
    return gunzipSync(buf).toString('utf8')
  }
  return await res.text()
}

function extract(xml: string, re: RegExp): string[] {
  return [...xml.matchAll(re)].map((m) => m[1].trim())
}

function extractLocs(xml: string): string[] {
  return extract(xml, /<loc>\s*([^<]+?)\s*<\/loc>/gis)
}

function extractAlternateHrefs(xml: string): string[] {
  return extract(
    xml,
    /<(?:\w+:)?link\b[^>]*\brel=['"]alternate['"][^>]*\bhref=['"]([^'"]+)['"][^>]*>/gis
  )
}

export async function crawlSitemap(options: CrawlOptions): Promise<string[]> {
  const exclude = options.exclude ?? DEFAULT_EXCLUDE
  const maxPages = options.maxPages ?? DEFAULT_MAX_PAGES
  const roots = options.roots ?? [
    `${options.origin}/sitemap.xml`,
    `${options.origin}/sitemap_index.xml`,
    `${options.origin}/sitemap-index.xml`,
  ]

  const visited = new Set<string>()
  const urls = new Set<string>()

  async function walk(u: string) {
    if (visited.has(u)) {
      return
    }
    visited.add(u)

    const xml = await fetchText(u)
    const isIndex = /<sitemapindex[\s>]/i.test(xml)
    if (isIndex) {
      for (const child of extractLocs(xml)) {
        await walk(child)
        if (urls.size >= maxPages) {
          break
        }
      }
      return
    }

    for (const href of [...extractLocs(xml), ...extractAlternateHrefs(xml)]) {
      const p = toLocalPath(href, options.origin, exclude)
      if (p) {
        urls.add(p)
        if (urls.size >= maxPages) {
          break
        }
      }
    }
  }

  for (const r of roots) {
    try {
      await walk(r)
      if (urls.size > 0) {
        break
      }
    } catch {
      // try next root
    }
  }

  return [...urls]
}

export async function ensureSitemapPaths(
  options: Partial<CrawlOptions> & { origin: string }
): Promise<string[]> {
  const cacheFile = options.cacheFile ?? DEFAULT_CACHE_FILE

  if (fs.existsSync(cacheFile)) {
    try {
      const array = JSON.parse(fs.readFileSync(cacheFile, 'utf8'))
      if (Array.isArray(array)) {
        return array as string[]
      }
    } catch {
      // fall through to rebuild
    }
  }

  const paths = await crawlSitemap(options as CrawlOptions)
  if (paths.length === 0) {
    throw new Error(
      `No URLs discovered from sitemaps at origin ${options.origin}`
    )
  }

  await fsp.mkdir(path.dirname(cacheFile), { recursive: true })
  await fsp.writeFile(cacheFile, JSON.stringify(paths, null, 2), 'utf8')
  return paths
}
