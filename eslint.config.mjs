import typescriptEslint from '@typescript-eslint/eslint-plugin'
import tsParser from '@typescript-eslint/parser'
import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import react from 'eslint-plugin-react'
import nextPlugin from '@next/eslint-plugin-next'
import importPlugin from 'eslint-plugin-import'
import securityPlugin from 'eslint-plugin-security'
import sonarjs from 'eslint-plugin-sonarjs'
import unicorn from 'eslint-plugin-unicorn'
import jsxA11y from 'eslint-plugin-jsx-a11y'
import noSecrets from 'eslint-plugin-no-secrets'

export default [
  {
    ignores: [
      '**/node_modules/',
      '**/.next/',
      '**/out/',
      '**/public/',
      '**/*.config.js',
      '**/*.config.mjs',
    ],
  },

  js.configs.recommended,

  {
    files: ['**/*.{ts,tsx,js,jsx}'],
    plugins: {
      '@typescript-eslint': typescriptEslint,
      'react-hooks': reactHooks,
      react,
      next: nextPlugin,
      import: importPlugin,
      security: securityPlugin,
      sonarjs,
      unicorn,
      'jsx-a11y': jsxA11y,
      'no-secrets': noSecrets,
    },
    languageOptions: {
      parser: tsParser,
      ecmaVersion: 2024,
      sourceType: 'module',
      globals: {
        ...globals.browser,
        ...globals.node,
      },
      parserOptions: {
        ecmaFeatures: { jsx: true },
        project: './tsconfig.json',
        tsconfigRootDir: import.meta.dirname,
      },
    },
    settings: {
      react: { version: 'detect' },
    },
    rules: {
      /**
       * TypeScript
       */
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          ignoreRestSiblings: false,
          args: 'all',
          caughtErrors: 'all',
        },
      ],
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/strict-boolean-expressions': 'error',
      '@typescript-eslint/await-thenable': 'error',
      '@typescript-eslint/no-misused-promises': 'error',
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
      '@typescript-eslint/no-unnecessary-condition': 'error',
      '@typescript-eslint/prefer-nullish-coalescing': 'error',
      '@typescript-eslint/prefer-optional-chain': 'error',
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
      '@typescript-eslint/prefer-readonly-parameter-types': 'error',

      /**
       * JavaScript / maintainability
       */
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

      /**
       * React / JSX
       */
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

      /**
       * Hooks
       */
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'error',

      /**
       * Next.js
       */
      'next/no-html-link-for-pages': ['error', 'src/app'],
      'next/no-img-element': 'error',
      'next/no-head-element': 'error',
      'next/no-sync-scripts': 'error',
      'next/no-document-import-in-page': 'error',
      'next/no-before-interactive-script-outside-document': 'error',
      'next/next-script-for-ga': 'error',
      'next/no-css-tags': 'error',

      /**
       * Imports
       */
      'import/order': [
        'error',
        { alphabetize: { order: 'asc' }, 'newlines-between': 'always' },
      ],
      'import/no-duplicates': 'error',
      'import/no-cycle': 'error',

      /**
       * Security
       */
      'no-eval': 'error',
      'no-implied-eval': 'error',
      'no-new-func': 'error',
      'no-script-url': 'error',
      '@typescript-eslint/no-implied-eval': 'error',
      'security/detect-object-injection': 'error',
      'security/detect-non-literal-fs-filename': 'error',
      'security/detect-non-literal-regexp': 'error',
      'no-secrets/no-secrets': 'error',

      /**
       * SonarJS
       */
      'sonarjs/cognitive-complexity': ['error', 15],
      'sonarjs/no-duplicate-string': [
        'error',
        { threshold: 3, ignoreStrings: 'application/json' },
      ],
      'sonarjs/no-ignored-return': 'error',
      'sonarjs/no-useless-catch': 'error',
      'sonarjs/prefer-immediate-return': 'error',
      'sonarjs/prefer-object-literal': 'error',

      /**
       * Unicorn
       */
      'unicorn/consistent-function-scoping': 'error',
      'unicorn/prefer-optional-catch-binding': 'error',
      'unicorn/throw-new-error': 'error',
      'unicorn/no-array-reduce': 'warn',
      'unicorn/prefer-includes': 'error',
      'unicorn/prefer-query-selector': 'error',
      'unicorn/prefer-string-starts-ends-with': 'error',

      /**
       * Accessibility
       */
      'jsx-a11y/alt-text': 'error',
      'jsx-a11y/anchor-is-valid': 'error',
      'jsx-a11y/click-events-have-key-events': 'error',
      'jsx-a11y/no-noninteractive-element-interactions': 'error',
      'jsx-a11y/no-static-element-interactions': 'error',
      'jsx-a11y/aria-props': 'error',
      'jsx-a11y/aria-proptypes': 'error',
      'jsx-a11y/aria-role': 'error',
      'jsx-a11y/aria-unsupported-elements': 'error',
    },
  },
]
