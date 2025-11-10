// eslint-disable-next-line no-restricted-syntax
const isDesktop = (process.env.LHCI_FORM_FACTOR || 'mobile') === 'desktop'

module.exports = {
  ci: {
    collect: {
      numberOfRuns: isDesktop ? 3 : 5,
      settings: {
        emulatedFormFactor: isDesktop ? 'desktop' : 'mobile',
        throttlingMethod: 'simulate',
        chromeFlags: ['--disable-gpu', '--no-sandbox', '--no-zygote'],
      },
      budgetsPath: '.github/lighthouse/budgets.json',
    },
    assert: {
      preset: 'lighthouse:recommended',
      assertions: {
        'categories:performance': [
          'error',
          { minScore: isDesktop ? 0.99 : 0.95 },
        ],
        'categories:accessibility': ['error', { minScore: 0.98 }],
        'categories:best-practices': ['error', { minScore: 1 }],
        'categories:seo': ['error', { minScore: 0.98 }],

        'first-contentful-paint': [
          'error',
          { maxNumericValue: 1200, aggregationMethod: 'median' },
        ],
        'largest-contentful-paint': [
          'error',
          { maxNumericValue: 1800, aggregationMethod: 'median' },
        ],
        'total-blocking-time': [
          'error',
          { maxNumericValue: 100, aggregationMethod: 'median' },
        ],
        'cumulative-layout-shift': [
          'error',
          { maxNumericValue: 0.03, aggregationMethod: 'median' },
        ],
        'speed-index': [
          'error',
          { maxNumericValue: 2000, aggregationMethod: 'median' },
        ],

        'render-blocking-resources': ['error', { maxLength: 0 }],
        'legacy-javascript': ['error', { maxLength: 0 }],
        'uses-responsive-images': ['error', { maxNumericValue: 0 }],
        'image-size-responsive': ['error', { maxNumericValue: 0 }],
        'preload-lcp-image': ['error', { maxLength: 0 }],
        'errors-in-console': ['error', { maxLength: 0 }],
      },
    },
  },
}
