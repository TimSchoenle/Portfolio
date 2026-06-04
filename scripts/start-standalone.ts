import { type ChildProcess, spawn } from 'node:child_process'
import fs, { promises as fsp } from 'node:fs'
import path from 'node:path'

interface Paths {
  destinationDotNext: string
  destinationPublic: string
  destinationRoot: string
  destinationStatic: string
  server: string
  srcPublic: string
  srcStatic: string
}

const projectRoot: string = process.cwd()
const isWindows: boolean = process.platform === 'win32'

const runtimeEnvironment: NodeJS.ProcessEnv = {
  ...process.env,
  HOSTNAME: process.env['HOSTNAME'] ?? '0.0.0.0',
  NODE_ENV: process.env.NODE_ENV ?? 'production',
  PORT: process.env['PORT'] ?? '3000',
}

const PATHS: Paths = {
  destinationDotNext: joinInsideRoot('.next', 'standalone', '.next'),
  destinationPublic: joinInsideRoot('.next', 'standalone', 'public'),
  destinationRoot: joinInsideRoot('.next', 'standalone'),
  destinationStatic: joinInsideRoot('.next', 'standalone', '.next', 'static'),
  server: joinInsideRoot('.next', 'standalone', 'server.js'),
  srcPublic: joinInsideRoot('public'),
  srcStatic: joinInsideRoot('.next', 'static'),
}

function joinInsideRoot(...segments: string[]): string {
  const absolute: string = path.resolve(projectRoot, path.join(...segments))
  if (!absolute.startsWith(projectRoot)) {
    throw new Error(`Refusing to operate outside project root: ${absolute}`)
  }
  return absolute
}

async function exists(filePath: string): Promise<boolean> {
  try {
    // eslint-disable-next-line security/detect-non-literal-fs-filename
    await fsp.lstat(filePath)
    return true
  } catch {
    return false
  }
}

async function removeFileOrDirectory(targetPath: string): Promise<void> {
  if (!(await exists(targetPath))) {
    return
  }
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  const stats: fs.Stats = await fsp.lstat(targetPath)

  await (stats.isSymbolicLink()
    ? // eslint-disable-next-line security/detect-non-literal-fs-filename
      fsp.unlink(targetPath)
    : fsp.rm(targetPath, { force: true, recursive: true }))
}

async function ensureDirectory(directoryPath: string): Promise<void> {
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  await fsp.mkdir(directoryPath, { recursive: true })
}

async function copyDirectory(
  sourceDirectory: string,
  destinationDirectory: string
): Promise<void> {
  await ensureDirectory(destinationDirectory)
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  const entries: fs.Dirent[] = await fsp.readdir(sourceDirectory, {
    withFileTypes: true,
  })
  for (const entry of entries) {
    const sourcePath: string = path.join(sourceDirectory, entry.name)
    const destinationPath: string = path.join(destinationDirectory, entry.name)

    if (entry.isDirectory()) {
      await copyDirectory(sourcePath, destinationPath)
      continue
    }

    if (entry.isSymbolicLink()) {
      // eslint-disable-next-line security/detect-non-literal-fs-filename
      const linkTarget: string | null = await fsp
        .readlink(sourcePath)
        .catch((): null => null)
      if (linkTarget === null) {
        await fsp.copyFile(sourcePath, destinationPath)
      } else {
        const resolved: string = path.resolve(
          path.dirname(sourcePath),
          linkTarget
        )
        // eslint-disable-next-line security/detect-non-literal-fs-filename
        const linkStats: fs.Stats = await fsp.lstat(resolved)
        await (linkStats.isDirectory()
          ? copyDirectory(resolved, destinationPath)
          : fsp.copyFile(resolved, destinationPath))
      }
      continue
    }

    await fsp.copyFile(sourcePath, destinationPath)
  }
}

async function linkOrCopyDirectory(
  sourceDirectory: string,
  destinationDirectory: string
): Promise<void> {
  await removeFileOrDirectory(destinationDirectory)
  try {
    // On Windows: 'junction' works for directories without admin
    // eslint-disable-next-line security/detect-non-literal-fs-filename
    fs.symlinkSync(
      sourceDirectory,
      destinationDirectory,
      isWindows ? 'junction' : 'dir'
    )
    log(
      `linked ${shortPath(sourceDirectory)} -> ${shortPath(destinationDirectory)}`
    )
  } catch (error) {
    warn(`symlink failed (${(error as Error).message}); copying instead…`)
    await copyDirectory(sourceDirectory, destinationDirectory)
    log(
      `copied ${shortPath(sourceDirectory)} -> ${shortPath(destinationDirectory)}`
    )
  }
}

function shortPath(inputPath: string): string {
  const relativePath: string = path.relative(projectRoot, inputPath)
  return relativePath === '' ? '.' : relativePath
}

function runChildProcess(
  bin: string,
  arguments_: string[],
  options: { env?: NodeJS.ProcessEnv } = {}
): Promise<number> {
  return new Promise<number>(
    (
      resolve: (value: number) => void,
      reject: (reason?: unknown) => void
    ): void => {
      const child: ChildProcess = spawn(bin, arguments_, {
        cwd: projectRoot,
        env: options.env ?? process.env,
        stdio: 'inherit',
        windowsHide: false,
      })
      child.on(
        'exit',
        (code: number | null, signal: NodeJS.Signals | null): void => {
          if (signal) {
            reject(new Error(`${bin} terminated by signal ${signal}`))
            return
          }
          resolve(code ?? 0)
        }
      )
      child.on('error', reject)
    }
  )
}

function getNextBinPath(): string {
  const binName: string = isWindows ? 'next.cmd' : 'next'
  const candidate: string = path.join(
    projectRoot,
    'node_modules',
    '.bin',
    binName
  )
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  return fs.existsSync(candidate) ? candidate : binName
}

interface BunProcessVersions extends NodeJS.ProcessVersions {
  bun?: string
}

function getRuntimeBin(): string {
  const versions: BunProcessVersions = process.versions

  if (versions.bun !== undefined && versions.bun !== '') {
    return process.execPath
  }

  const binName: string = isWindows ? 'node.exe' : 'node'
  return process.execPath.includes(binName) ? process.execPath : 'node'
}

function log(message: string): void {
  // eslint-disable-next-line no-console
  console.log(`[start-standalone] ${message}`)
}

function warn(message: string): void {
  // eslint-disable-next-line no-console
  console.warn(`[start-standalone] WARN: ${message}`)
}

function errorOut(message: string): void {
  // eslint-disable-next-line no-console
  console.error(`[start-standalone] ERROR: ${message}`)
}

async function ensureStandaloneBuild(): Promise<void> {
  if (await exists(PATHS.server)) {
    return
  }

  log('No standalone server found. Building…')
  const nextBin: string = getNextBinPath()

  const buildEnvironment: NodeJS.ProcessEnv = {
    ...process.env,
    NEXT_STANDALONE: 'true',
    NODE_ENV: 'production',
  }

  const code: number = await runChildProcess(nextBin, ['build'], {
    env: buildEnvironment,
  })
  if (code !== 0) {
    throw new Error(`next build failed with code ${String(code)}`)
  }

  if (!(await exists(PATHS.server))) {
    throw new Error(
      'Build finished but .next/standalone/server.js is missing. ' +
        "Ensure next.config sets `output: 'standalone'` when NEXT_STANDALONE=true."
    )
  }
}

async function prepareAssets(): Promise<void> {
  await ensureDirectory(PATHS.destinationDotNext)

  if (await exists(PATHS.srcStatic)) {
    await linkOrCopyDirectory(PATHS.srcStatic, PATHS.destinationStatic)
  } else {
    warn(
      `${shortPath(PATHS.srcStatic)} not found; proceeding without static assets.`
    )
  }

  if (await exists(PATHS.srcPublic)) {
    await linkOrCopyDirectory(PATHS.srcPublic, PATHS.destinationPublic)
  } else {
    log('No public directory detected.')
  }
}

async function startServer(): Promise<number> {
  const hostEnvironment: unknown = runtimeEnvironment['HOSTNAME']
  const portEnvironment: unknown = runtimeEnvironment['PORT']
  const nodeEnvironment_: unknown = runtimeEnvironment.NODE_ENV

  const host: string =
    typeof hostEnvironment === 'string' && hostEnvironment.length > 0
      ? hostEnvironment
      : '0.0.0.0'
  const port: string =
    typeof portEnvironment === 'string' && portEnvironment.length > 0
      ? portEnvironment
      : '3000'
  const nodeEnvironment: string =
    typeof nodeEnvironment_ === 'string' && nodeEnvironment_.length > 0
      ? nodeEnvironment_
      : 'production'

  log(`Starting server on ${host}:${port} (NODE_ENV=${nodeEnvironment})…`)

  return runChildProcess(getRuntimeBin(), [PATHS.server], {
    env: runtimeEnvironment,
  })
}

async function main(): Promise<void> {
  await ensureStandaloneBuild()
  await prepareAssets()
  process.exitCode = await startServer()
}

// eslint-disable-next-line unicorn/prefer-top-level-await
void main().catch((error: unknown): void => {
  errorOut((error as Error).message)

  process.exitCode = 1
})
