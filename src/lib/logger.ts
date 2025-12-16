import pino, { type Logger } from 'pino'

import { environment } from '@/environment'

export const logger: Logger = pino({
  browser: {
    asObject: true,
  },
  level: environment.LOG_LEVEL,
  timestamp: pino.stdTimeFunctions.isoTime,
})
