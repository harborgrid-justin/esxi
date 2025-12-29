/**
 * @meridian/accessibility-lint
 * Enterprise-grade accessibility linting for TypeScript/JavaScript applications
 *
 * @module accessibility-lint
 * @version 0.3.0
 */

import noImplicitAny from './rules/noImplicitAny';
import strictNullChecks from './rules/strictNullChecks';
import accessibleComponents from './rules/accessibleComponents';
import ariaUsage from './rules/ariaUsage';
import colorContrast from './rules/colorContrast';
import keyboardHandlers from './rules/keyboardHandlers';
import semanticHTML from './rules/semanticHTML';
import recommended from './config/recommended';
import strict from './config/strict';

export const rules = {
  'no-implicit-any': noImplicitAny,
  'strict-null-checks': strictNullChecks,
  'accessible-components': accessibleComponents,
  'aria-usage': ariaUsage,
  'color-contrast': colorContrast,
  'keyboard-handlers': keyboardHandlers,
  'semantic-html': semanticHTML,
};

export const configs = {
  recommended,
  strict,
};

export default {
  rules,
  configs,
};

// Type exports for extensibility
export type { RuleModule, RuleContext, RuleListener } from './utils/astUtils';
