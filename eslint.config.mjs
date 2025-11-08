import tseslint from 'typescript-eslint'
import globals from 'globals'
import react from 'eslint-plugin-react'
import reactHooks from 'eslint-plugin-react-hooks'
import nextPlugin from '@next/eslint-plugin-next'
import importPlugin from 'eslint-plugin-import'
import securityPlugin from 'eslint-plugin-security'
import sonarjs from 'eslint-plugin-sonarjs'
import noSecrets from 'eslint-plugin-no-secrets'
import perfectionist from 'eslint-plugin-perfectionist'
import eslintPluginUnicorn from 'eslint-plugin-unicorn'
import reactCompiler from 'eslint-plugin-react-compiler'
import jsxA11y from 'eslint-plugin-jsx-a11y'

const TSLINT_FILES = ['**/*.{ts,tsx}']

const forTS = (...groups) =>
  groups
    .flat()
    .filter(Boolean)
    .map((c) => ({ ...c, files: TSLINT_FILES }))

const sonarRecommended = (sonarjs.configs &&
  sonarjs.configs['flat/recommended']) || {
  plugins: { sonarjs },
  rules: sonarjs.configs?.recommended?.rules ?? {},
}

export default tseslint.config(
  // Ignore build/vendor outputs
  {
    ignores: [
      '**/node_modules/',
      '**/.next/',
      '**/out/',
      '**/public/',
      '**next-env.d.ts',
      '**/playwright-report/',
      '**/test-results/',
      '**/tests/',
      '**/test/',
    ],
  },

  // Disallow JS/CJS/MJS everywhere...
  {
    files: ['**/*.{js,jsx,cjs,mjs}'],
    ignores: ['eslint.config.mjs', 'postcss.config.mjs'], // ...except these two
    rules: {
      'no-restricted-syntax': [
        'error',
        {
          selector: 'Program',
          message:
            'JavaScript files are not allowed in this repo. Use TypeScript (.ts/.tsx).',
        },
      ],
    },
  },

  // Allow + lightly lint our two Node-side config files
  {
    files: ['eslint.config.mjs', 'postcss.config.mjs'],
    languageOptions: {
      ecmaVersion: 2024,
      sourceType: 'module',
      globals: { ...globals.node },
    },
    rules: {
      'unicorn/no-null': 'off',
      'no-duplicate-imports': 'error',
      'no-eval': 'error',
      'no-implied-eval': 'error',
    },
  },

  // TS
  ...forTS(
    tseslint.configs.recommendedTypeChecked,
    tseslint.configs.strictTypeChecked,
    tseslint.configs.stylisticTypeChecked,
    eslintPluginUnicorn.configs.recommended,
    jsxA11y.flatConfigs.strict,
    sonarRecommended,
    securityPlugin.configs.recommended,
    react.configs.flat.recommended,
    reactCompiler.configs.recommended
  ),

  // Project rules
  {
    files: TSLINT_FILES,
    languageOptions: {
      // Make the presets type-aware using your tsconfig
      parserOptions: {
        project: './tsconfig.json',
        tsconfigRootDir: import.meta.dirname,
      },
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
    plugins: {
      react,
      'react-hooks': reactHooks,
      next: nextPlugin,
      import: importPlugin,
      security: securityPlugin,
      sonarjs,
      'no-secrets': noSecrets,
      perfectionist,
    },
    settings: {
      react: { version: 'detect' },
      'import/resolver': { typescript: { project: './tsconfig.json' } },
    },
    rules: {
      // Strict TS safety
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          args: 'all',
          ignoreRestSiblings: false,
          caughtErrors: 'all',
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_',
          destructuredArrayIgnorePattern: '^_',
        },
      ],
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/await-thenable': 'error',
      '@typescript-eslint/no-misused-promises': 'error',
      '@typescript-eslint/strict-boolean-expressions': 'error',
      '@typescript-eslint/consistent-type-imports': [
        'error',
        { prefer: 'type-imports', fixStyle: 'inline-type-imports' },
      ],
      '@typescript-eslint/no-unsafe-assignment': 'error',
      '@typescript-eslint/no-unsafe-call': 'error',
      '@typescript-eslint/no-unsafe-member-access': 'error',
      '@typescript-eslint/no-unsafe-return': 'error',
      '@typescript-eslint/explicit-module-boundary-types': 'error',
      '@typescript-eslint/explicit-function-return-type': [
        'error',
        { allowExpressions: false, allowTypedFunctionExpressions: false },
      ],
      '@typescript-eslint/explicit-member-accessibility': [
        'error',
        { accessibility: 'explicit' },
      ],
      '@typescript-eslint/no-unnecessary-condition': 'off',
      '@typescript-eslint/prefer-nullish-coalescing': 'error',
      '@typescript-eslint/prefer-optional-chain': 'error',
      '@typescript-eslint/no-inferrable-types': 'off',
      '@typescript-eslint/typedef': [
        'error',
        {
          arrayDestructuring: true,
          arrowParameter: true,
          objectDestructuring: true,
          variableDeclaration: true,
          memberVariableDeclaration: true,
        },
      ],
      '@typescript-eslint/ban-ts-comment': [
        'error',
        { 'ts-ignore': 'allow-with-description' },
      ],
      '@typescript-eslint/consistent-type-exports': 'error',
      '@typescript-eslint/consistent-indexed-object-style': ['error', 'record'],
      '@typescript-eslint/prefer-readonly': 'error',
      '@typescript-eslint/sort-type-constituents': [
        'error',
        { checkIntersections: true, checkUnions: true },
      ],

      // Naming + files (with hook exception)
      '@typescript-eslint/naming-convention': [
        'error',

        // Types, interfaces, enums, type params
        { selector: 'typeLike', format: ['PascalCase'] },

        // Variables:
        // - PascalCase allowed so const React components pass
        // - camelCase for regular vars
        // - UPPER_CASE for true constants
        {
          selector: 'variable',
          format: ['camelCase', 'PascalCase', 'UPPER_CASE'],
          custom: {
            regex: '^(?:_+)?[A-Za-z][A-Za-z0-9_]{2,}(?:_+)?$',
            match: true,
          }, // min length 3
          leadingUnderscore: 'allow',
          trailingUnderscore: 'allow',
        },

        // Functions:
        // - PascalCase allowed for function React components
        // - camelCase for everything else (incl. Next.js `generateMetadata`)
        { selector: 'function', format: ['camelCase', 'PascalCase'] },

        // Hooks must start with "use" + capital next letter
        {
          selector: 'function',
          filter: { regex: '^use', match: true },
          format: ['camelCase'],
          custom: { regex: '^use[A-Z].*', match: true },
        },

        // Don’t police property names (API payloads, CSS-in-JS, etc.)
        { selector: 'property', format: null },
        { selector: 'objectLiteralProperty', format: null },

        // Parameters (allow leading underscore to mark intentionally unused)
        {
          selector: 'parameter',
          format: ['camelCase'],
          custom: { regex: '^[A-Za-z][A-Za-z0-9]{2,}$', match: true },
          leadingUnderscore: 'allow',
          trailingUnderscore: 'allow',
        },

        // Destructured variables: allow any case.
        { selector: 'variable', modifiers: ['destructured'], format: null },
      ],

      'unicorn/prevent-abbreviations': [
        'warn',
        {
          allowList: {
            i18n: true,
          },
          // exact identifiers we never want flagged (Next.js & friends)
          ignore: [
            'generateStaticParams',
            'generateMetadata',
            'generateViewport',
            'generateImageMetadata',
            'getStaticProps',
            'getStaticPaths',
            'getServerSideProps',
          ],
        },
      ],
      'unicorn/filename-case': [
        'error',
        {
          cases: { kebabCase: true, camelCase: true, pascalCase: true },
          ignore: ['^\\[.*\\]$'], // dynamic routes: [locale], [id], etc.
        },
      ],

      // Class & interface member ordering (no 3rd-party conflicts)
      '@typescript-eslint/member-ordering': [
        'error',
        {
          default: {
            memberTypes: [
              'signature',
              // fields
              'public-static-field',
              'protected-static-field',
              'private-static-field',
              'public-decorated-field',
              'protected-decorated-field',
              'private-decorated-field',
              'public-instance-field',
              'protected-instance-field',
              'private-instance-field',
              // constructor
              'public-constructor',
              'protected-constructor',
              'private-constructor',
              // methods
              'public-static-method',
              'protected-static-method',
              'private-static-method',
              'public-instance-method',
              'protected-instance-method',
              'private-instance-method',
            ],
            order: 'as-written',
          },
        },
      ],

      // Perfectionist: keep what doesn’t clash with TS/import rules
      'perfectionist/sort-objects': [
        'error',
        { type: 'natural', order: 'asc' },
      ],
      // Disable union sorting here (TS rule already handles it)
      'perfectionist/sort-union-types': 'off',
      'perfectionist/sort-enums': ['error', { type: 'natural', order: 'asc' }],
      'perfectionist/sort-interfaces': [
        'error',
        { type: 'natural', order: 'asc' },
      ],
      'perfectionist/sort-named-imports': [
        'error',
        { type: 'natural', order: 'asc', ignoreCase: true },
      ],
      'perfectionist/sort-named-exports': [
        'error',
        { type: 'natural', order: 'asc', ignoreCase: true },
      ],
      // Keep using import/order for module grouping (don’t enable perfectionist import sorter)

      // Maintainability
      'no-var': 'error',
      'prefer-const': 'error',
      'no-duplicate-imports': 'error',
      'no-unreachable': 'error',
      eqeqeq: ['error', 'always'],
      'no-console': 'error',
      complexity: ['error', 8],
      'max-lines-per-function': ['error', 60],
      'max-lines': ['error', 400],
      'max-params': ['error', 3],
      'max-depth': ['error', 3],
      'max-nested-callbacks': ['error', 3],
      'no-else-return': 'error',
      'no-return-await': 'error',
      curly: ['error', 'all'],
      'no-param-reassign': 'error',

      // React / JSX
      'react/react-in-jsx-scope': 'off',
      'react/prop-types': 'off',
      'react/jsx-no-target-blank': 'error',
      'react/jsx-key': 'error',
      'react/no-unescaped-entities': 'error',
      'react/self-closing-comp': 'error',
      'react/jsx-pascal-case': 'error',
      'react/jsx-boolean-value': ['error', 'always'],
      'react/jsx-curly-brace-presence': [
        'error',
        { props: 'never', children: 'never' },
      ],
      'react/jsx-no-useless-fragment': 'error',
      'react/jsx-sort-props': [
        'error',
        { callbacksLast: true, shorthandFirst: true, ignoreCase: true },
      ],
      'react/function-component-definition': [
        'error',
        {
          namedComponents: 'arrow-function',
          unnamedComponents: 'arrow-function',
        },
      ],
      'react/no-array-index-key': 'error',
      'react/jsx-no-literals': ['error', { allowedStrings: [] }],
      'react/no-danger': 'error',
      'react/no-danger-with-children': 'error',

      // Hooks
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'error',

      // Next.js
      'next/no-html-link-for-pages': ['error', 'src/app'],
      'next/no-img-element': 'error',
      'next/no-head-element': 'error',
      'next/no-sync-scripts': 'error',
      'next/no-document-import-in-page': 'error',
      'next/no-before-interactive-script-outside-document': 'error',
      'next/next-script-for-ga': 'error',
      'next/no-css-tags': 'error',

      // Imports (grouping + line order)
      'import/order': [
        'error',
        {
          groups: [
            'builtin',
            'external',
            'internal',
            ['parent', 'sibling', 'index'],
            'object',
            'type',
          ],
          pathGroupsExcludedImportTypes: ['builtin'],
          alphabetize: { order: 'asc', caseInsensitive: true },
          'newlines-between': 'always',
          pathGroups: [
            { pattern: 'react', group: 'external', position: 'before' },
            { pattern: 'next**', group: 'external', position: 'before' },
            { pattern: '@/**', group: 'internal', position: 'after' },
          ],
        },
      ],
      'import/no-duplicates': 'error',
      'import/no-cycle': 'error',

      // Security
      'no-eval': 'error',
      'no-implied-eval': 'error',
      'no-new-func': 'error',
      'no-script-url': 'error',
      '@typescript-eslint/no-implied-eval': 'error',
      'security/detect-object-injection': 'error',
      'security/detect-non-literal-fs-filename': 'error',
      'security/detect-non-literal-regexp': 'error',
      'no-secrets/no-secrets': 'error',

      // SonarJS
      'sonarjs/cognitive-complexity': ['error', 15],
      'sonarjs/no-duplicate-string': [
        'error',
        { threshold: 3, ignoreStrings: 'application/json' },
      ],
      // We handle that somewhere else
      'sonarjs/no-unused-vars': 'off',

      // Unicorn -- Disable rules which are causing issues
      'unicorn/no-null': 'off',
      'unicorn/prefer-string-raw': 'off',
      'unicorn/prefer-global-this': 'off',
    },
  }
)
