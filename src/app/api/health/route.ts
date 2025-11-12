import { NextResponse } from 'next/server'

export function GET(): NextResponse {
  return NextResponse.json(
    {
      status: 'healthy',
      timestamp: new Date().toISOString(),
    },
    { headers: { 'Cache-Control': 'no-store' }, status: 200 }
  )
}
