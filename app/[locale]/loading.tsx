export default function Loading() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background">
      <div className="space-y-4 text-center">
        <div className="relative h-16 w-16 mx-auto">
          <div className="absolute inset-0 rounded-full border-4 border-primary/20" />
          <div className="absolute inset-0 animate-spin rounded-full border-4 border-primary border-t-transparent" />
        </div>
        <p className="text-sm text-muted-foreground animate-pulse">
          Loading...
        </p>
      </div>
    </div>
  )
}

