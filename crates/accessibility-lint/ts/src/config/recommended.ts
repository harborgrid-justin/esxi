/**
 * Recommended ESLint configuration for accessibility linting
 * Provides balanced enforcement suitable for most projects
 */

export default {
  plugins: ['@meridian/accessibility-lint'],
  rules: {
    // Type safety rules - error level
    '@meridian/accessibility-lint/no-implicit-any': 'error',
    '@meridian/accessibility-lint/strict-null-checks': 'error',

    // Accessibility rules - error level for critical issues
    '@meridian/accessibility-lint/accessible-components': [
      'error',
      {
        requireAriaLabel: true,
        checkCustomComponents: true,
      },
    ],
    '@meridian/accessibility-lint/aria-usage': 'error',
    '@meridian/accessibility-lint/keyboard-handlers': 'error',

    // Accessibility rules - warning level for best practices
    '@meridian/accessibility-lint/color-contrast': [
      'warn',
      {
        wcagLevel: 'AA',
        largeText: false,
      },
    ],
    '@meridian/accessibility-lint/semantic-html': 'warn',
  },
  settings: {
    'meridian-accessibility': {
      version: '0.3.0',
      strictMode: false,
    },
  },
  env: {
    browser: true,
    es2022: true,
  },
  parserOptions: {
    ecmaVersion: 2022,
    sourceType: 'module',
    ecmaFeatures: {
      jsx: true,
    },
  },
};
