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
          { minScore: isDesktop ? 0.9 : 0.8 },
        ],

        'first-contentful-paint': [
          'error',
          {
            maxNumericValue: isDesktop ? 1300 : 1500,
            aggregationMethod: 'median',
          },
        ],
        'largest-contentful-paint': [
          'error',
          {
            maxNumericValue: isDesktop ? 3500 : 4000,
            aggregationMethod: 'median',
          },
        ],
        'total-blocking-time': [
          'error',
          {
            maxNumericValue: isDesktop ? 350 : 400,
            aggregationMethod: 'median',
          },
        ],
        'cumulative-layout-shift': [
          'error',
          { maxNumericValue: 0.03, aggregationMethod: 'median' },
        ],
        'speed-index': [
          'error',
          {
            maxNumericValue: isDesktop ? 2200 : 3500,
            aggregationMethod: 'median',
          },
        ],

        'render-blocking-resources': ['error', { maxLength: 0 }],
        'errors-in-console': ['error', { maxLength: 0 }],

        'legacy-javascript': ['warn', { maxLength: 2 }],
        'legacy-javascript-insight': 'warn',
        'unused-javascript': 'warn',

        'uses-responsive-images': 'off',
        'image-size-responsive': 'off',
        'preload-lcp-image': 'off',
      },
    },
  },
}
