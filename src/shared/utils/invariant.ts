export const panic: (message: string) => never = (message: string): never => {
  throw new Error(message)
}
