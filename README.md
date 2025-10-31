# Portfolio - Tim (Timmi6790)

Modern, performant portfolio website built with Next.js 15, TypeScript, and Tailwind CSS.

## Features

- 🌐 **Internationalization (i18n)**: Full support for English and German
  Source_Serif_4,
- 🚀 **Performance**: Server-side rendering with optimized caching
- 🔄 **GitHub Integration**: Automatically fetches projects and contribution graphs
- 🎯 **Type-Safe**: Fully typed with TypeScript including i18n dictionaries
- 📱 **Responsive**: Mobile-first design that works on all devices
- 🐳 **Docker**: Production-ready Docker setup included

## Tech Stack

- **Framework**: Next.js 15 (App Router)
- **Language**: TypeScript
- **Styling**: Tailwind CSS
- **UI Components**: Radix UI
- **Icons**: Lucide React
- **Package Manager**: pnpm

## Getting Started

### Prerequisites

- Node.js 18+ or Docker
- pnpm (recommended) or npm
- GitHub token (optional, for fetching project data)

### Installation

```bash
# Install dependencies
pnpm install

# Run development server
pnpm dev

# Build for production
pnpm build

# Start production server
pnpm start
```

### Docker

```bash
# Build and run with Docker Compose
pnpm docker:prod

# Or build and run manually
pnpm docker:build
pnpm docker:run
```

### Environment Variables

Create a `.env.local` file:

```env
GITHUB_TOKEN=your_github_token_here
```

## Project Structure

```
portfolio/
├── app/              # Next.js app directory
│   ├── [locale]/    # Localized routes
│   └── api/         # API routes
├── components/       # React components
├── lib/             # Utility functions and configurations
├── messages/        # i18n translation files
└── public/          # Static assets
```

## Scripts

- `pnpm dev` - Start development server
- `pnpm build` - Build for production
- `pnpm start` - Start production server
- `pnpm lint` - Run ESLint
- `pnpm lint:fix` - Fix ESLint errors
- `pnpm format` - Format code with Prettier
- `pnpm type-check` - Run TypeScript type checking

## License

MIT License - feel free to use this as a template for your own portfolio!
