import type { UserConfig } from '@commitlint/types'

const Configuration: UserConfig = {
  extends: ['@commitlint/config-conventional'],
  ignores: [(message: string): boolean => message.startsWith('chore: bump')], // Ignore dependabot commits
}

export default Configuration
