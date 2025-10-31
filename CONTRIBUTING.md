# Contributing to Portfolio

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to this project.

## Development Setup

### Prerequisites

- Node.js 22 or higher
- pnpm 9 or higher
- Docker (optional, for containerized development)

### Getting Started

1. Clone the repository:
   \`\`\`bash
   git clone https://github.com/Timmi6790/portfolio.git
   cd portfolio
   \`\`\`

2. Install dependencies:
   \`\`\`bash
   pnpm install
   \`\`\`

3. Run the development server:
   \`\`\`bash
   pnpm dev
   \`\`\`

4. Open [http://localhost:3000](http://localhost:3000) in your browser.

## Code Standards

### TypeScript

- Use TypeScript for all new files
- Avoid `any` types when possible
- Use proper type definitions

### Code Style

- We use Prettier for code formatting
- We use ESLint for code linting
- Run `pnpm format` before committing
- Run `pnpm lint` to check for issues

### Commit Messages

Follow conventional commits format:

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

Example: `feat: add dark mode toggle`

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Make your changes
4. Run tests and linting: `pnpm lint && pnpm type-check`
5. Commit your changes with a descriptive message
6. Push to your fork: `git push origin feat/my-feature`
7. Open a Pull Request

### PR Guidelines

- Provide a clear description of the changes
- Reference any related issues
- Ensure all CI checks pass
- Keep PRs focused and atomic
- Update documentation if needed

## Docker Development

### Build and run with Docker:

\`\`\`bash

# Development

pnpm docker:dev

# Production

pnpm docker:build
pnpm docker:run
\`\`\`

## Testing

Before submitting a PR, ensure:

- [ ] Code builds successfully: `pnpm build`
- [ ] No linting errors: `pnpm lint`
- [ ] No type errors: `pnpm type-check`
- [ ] Code is formatted: `pnpm format:check`

## Questions?

Feel free to open an issue for any questions or concerns.
