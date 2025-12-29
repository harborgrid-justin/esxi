/**
 * Strict ESLint configuration for accessibility linting
 * Maximum enforcement for enterprise-grade accessibility compliance
 * Suitable for projects requiring WCAG 2.1 Level AAA compliance
 */

export default {
  plugins: ['@meridian/accessibility-lint'],
  rules: {
    // Type safety rules - error level with no exceptions
    '@meridian/accessibility-lint/no-implicit-any': 'error',
    '@meridian/accessibility-lint/strict-null-checks': 'error',

    // Accessibility rules - all at error level for strict compliance
    '@meridian/accessibility-lint/accessible-components': [
      'error',
      {
        requireAriaLabel: true,
        checkCustomComponents: true,
      },
    ],
    '@meridian/accessibility-lint/aria-usage': 'error',
    '@meridian/accessibility-lint/keyboard-handlers': 'error',
    '@meridian/accessibility-lint/color-contrast': [
      'error',
      {
        wcagLevel: 'AAA',
        largeText: false,
      },
    ],
    '@meridian/accessibility-lint/semantic-html': 'error',
  },
  settings: {
    'meridian-accessibility': {
      version: '0.3.0',
      strictMode: true,
      wcagLevel: 'AAA',
      enforceAllRules: true,
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
  overrides: [
    {
      // TypeScript-specific overrides
      files: ['*.ts', '*.tsx'],
      parser: '@typescript-eslint/parser',
      parserOptions: {
        project: './tsconfig.json',
      },
    },
  ],
};
