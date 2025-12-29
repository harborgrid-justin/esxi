/**
 * Custom Jest/Vitest matcher: toHaveValidARIA
 * Validates that an element has valid ARIA attributes and roles
 */

import { roles as ariaRoles } from 'aria-query';

interface ARIAValidationError {
  type: 'invalid-role' | 'invalid-attribute' | 'missing-required' | 'invalid-value' | 'redundant-role';
  message: string;
  element: string;
  attribute?: string;
  role?: string;
}

/**
 * Valid ARIA roles
 */
const VALID_ROLES = new Set(Array.from(ariaRoles.keys()));

/**
 * Abstract ARIA roles that should not be used directly
 */
const ABSTRACT_ROLES = new Set([
  'command',
  'composite',
  'input',
  'landmark',
  'range',
  'roletype',
  'section',
  'sectionhead',
  'select',
  'structure',
  'widget',
  'window',
]);

/**
 * Elements with implicit ARIA roles
 */
const IMPLICIT_ROLES: Record<string, string> = {
  a: 'link',
  button: 'button',
  h1: 'heading',
  h2: 'heading',
  h3: 'heading',
  h4: 'heading',
  h5: 'heading',
  h6: 'heading',
  img: 'img',
  input: 'textbox',
  nav: 'navigation',
  main: 'main',
  aside: 'complementary',
  footer: 'contentinfo',
  header: 'banner',
  form: 'form',
  section: 'region',
  article: 'article',
};

/**
 * Valid ARIA attributes (comprehensive list)
 */
const VALID_ARIA_ATTRIBUTES = new Set([
  'aria-activedescendant',
  'aria-atomic',
  'aria-autocomplete',
  'aria-busy',
  'aria-checked',
  'aria-colcount',
  'aria-colindex',
  'aria-colspan',
  'aria-controls',
  'aria-current',
  'aria-describedby',
  'aria-details',
  'aria-disabled',
  'aria-dropeffect',
  'aria-errormessage',
  'aria-expanded',
  'aria-flowto',
  'aria-grabbed',
  'aria-haspopup',
  'aria-hidden',
  'aria-invalid',
  'aria-keyshortcuts',
  'aria-label',
  'aria-labelledby',
  'aria-level',
  'aria-live',
  'aria-modal',
  'aria-multiline',
  'aria-multiselectable',
  'aria-orientation',
  'aria-owns',
  'aria-placeholder',
  'aria-posinset',
  'aria-pressed',
  'aria-readonly',
  'aria-relevant',
  'aria-required',
  'aria-roledescription',
  'aria-rowcount',
  'aria-rowindex',
  'aria-rowspan',
  'aria-selected',
  'aria-setsize',
  'aria-sort',
  'aria-valuemax',
  'aria-valuemin',
  'aria-valuenow',
  'aria-valuetext',
]);

/**
 * ARIA boolean attributes
 */
const BOOLEAN_ARIA_ATTRIBUTES = new Set([
  'aria-atomic',
  'aria-busy',
  'aria-disabled',
  'aria-grabbed',
  'aria-hidden',
  'aria-modal',
  'aria-multiline',
  'aria-multiselectable',
  'aria-readonly',
  'aria-required',
]);

/**
 * Validate ARIA attributes on an element
 */
function validateARIAAttributes(element: Element): ARIAValidationError[] {
  const errors: ARIAValidationError[] = [];
  const attributes = element.attributes;
  const tagName = element.tagName.toLowerCase();

  for (let i = 0; i < attributes.length; i++) {
    const attr = attributes[i];
    if (!attr) continue;

    const attrName = attr.name;
    const attrValue = attr.value;

    // Check ARIA attributes
    if (attrName.startsWith('aria-')) {
      // Validate attribute name
      if (!VALID_ARIA_ATTRIBUTES.has(attrName)) {
        errors.push({
          type: 'invalid-attribute',
          message: `Invalid ARIA attribute: ${attrName}`,
          element: getElementSelector(element),
          attribute: attrName,
        });
        continue;
      }

      // Validate boolean attributes
      if (BOOLEAN_ARIA_ATTRIBUTES.has(attrName)) {
        if (attrValue !== 'true' && attrValue !== 'false') {
          errors.push({
            type: 'invalid-value',
            message: `ARIA attribute ${attrName} must be "true" or "false", got "${attrValue}"`,
            element: getElementSelector(element),
            attribute: attrName,
          });
        }
      }

      // Validate specific attributes
      if (attrName === 'aria-live') {
        const validValues = ['off', 'polite', 'assertive'];
        if (!validValues.includes(attrValue)) {
          errors.push({
            type: 'invalid-value',
            message: `aria-live must be one of: ${validValues.join(', ')}. Got "${attrValue}"`,
            element: getElementSelector(element),
            attribute: attrName,
          });
        }
      }

      if (attrName === 'aria-current') {
        const validValues = ['page', 'step', 'location', 'date', 'time', 'true', 'false'];
        if (!validValues.includes(attrValue)) {
          errors.push({
            type: 'invalid-value',
            message: `aria-current must be one of: ${validValues.join(', ')}. Got "${attrValue}"`,
            element: getElementSelector(element),
            attribute: attrName,
          });
        }
      }

      if (attrName === 'aria-autocomplete') {
        const validValues = ['inline', 'list', 'both', 'none'];
        if (!validValues.includes(attrValue)) {
          errors.push({
            type: 'invalid-value',
            message: `aria-autocomplete must be one of: ${validValues.join(', ')}. Got "${attrValue}"`,
            element: getElementSelector(element),
            attribute: attrName,
          });
        }
      }
    }

    // Check role attribute
    if (attrName === 'role') {
      const roles = attrValue.split(' ');

      for (const role of roles) {
        // Check for abstract roles
        if (ABSTRACT_ROLES.has(role)) {
          errors.push({
            type: 'invalid-role',
            message: `Abstract role "${role}" cannot be used directly`,
            element: getElementSelector(element),
            role,
          });
          continue;
        }

        // Check for valid roles
        if (!VALID_ROLES.has(role)) {
          errors.push({
            type: 'invalid-role',
            message: `Invalid ARIA role: "${role}"`,
            element: getElementSelector(element),
            role,
          });
          continue;
        }

        // Check for redundant roles
        const implicitRole = IMPLICIT_ROLES[tagName];
        if (implicitRole === role) {
          errors.push({
            type: 'redundant-role',
            message: `Redundant role="${role}" on <${tagName}>. This element has an implicit role.`,
            element: getElementSelector(element),
            role,
          });
        }
      }
    }
  }

  return errors;
}

/**
 * Get a CSS selector for an element
 */
function getElementSelector(element: Element): string {
  const tagName = element.tagName.toLowerCase();
  const id = element.id ? `#${element.id}` : '';
  const classes = element.className
    ? `.${element.className.split(' ').filter(Boolean).join('.')}`
    : '';

  return `${tagName}${id}${classes}` || 'unknown';
}

/**
 * Get element HTML snippet
 */
function getElementHTML(element: Element): string {
  const html = element.outerHTML;
  if (html.length <= 100) {
    return html;
  }
  return html.substring(0, 100) + '...';
}

/**
 * Recursively validate ARIA on element and descendants
 */
function validateARIARecursive(element: Element): ARIAValidationError[] {
  const errors: ARIAValidationError[] = [];

  // Validate current element
  errors.push(...validateARIAAttributes(element));

  // Validate children
  const children = element.children;
  for (let i = 0; i < children.length; i++) {
    const child = children[i];
    if (child) {
      errors.push(...validateARIARecursive(child));
    }
  }

  return errors;
}

/**
 * Format errors for output
 */
function formatErrors(errors: ARIAValidationError[]): string {
  if (errors.length === 0) {
    return 'No ARIA validation errors found.';
  }

  const byType = {
    'invalid-role': errors.filter((e) => e.type === 'invalid-role'),
    'invalid-attribute': errors.filter((e) => e.type === 'invalid-attribute'),
    'invalid-value': errors.filter((e) => e.type === 'invalid-value'),
    'missing-required': errors.filter((e) => e.type === 'missing-required'),
    'redundant-role': errors.filter((e) => e.type === 'redundant-role'),
  };

  const summary = `
ARIA Validation Summary:
  Invalid Roles: ${byType['invalid-role'].length}
  Invalid Attributes: ${byType['invalid-attribute'].length}
  Invalid Values: ${byType['invalid-value'].length}
  Missing Required: ${byType['missing-required'].length}
  Redundant Roles: ${byType['redundant-role'].length}
  Total: ${errors.length}
`;

  const formatted = errors.map((error, index) => {
    return `
${index + 1}. [${error.type.toUpperCase()}] ${error.message}
   Element: ${error.element}
   ${error.role ? `Role: ${error.role}` : ''}
   ${error.attribute ? `Attribute: ${error.attribute}` : ''}`;
  });

  return summary + '\nDetailed Errors:\n' + formatted.join('\n');
}

/**
 * Custom matcher implementation
 */
export function toHaveValidARIA(
  this: jest.MatcherContext,
  received: HTMLElement | Element | Document
): jest.CustomMatcherResult {
  let element: Element;

  if (received instanceof Document) {
    element = received.documentElement;
  } else if (received instanceof Element) {
    element = received;
  } else {
    return {
      pass: false,
      message: () =>
        'toHaveValidARIA expects an HTMLElement, Element, or Document.\n' +
        `Received: ${typeof received}`,
    };
  }

  const errors = validateARIARecursive(element);
  const pass = errors.length === 0;

  if (pass) {
    return {
      pass: true,
      message: () =>
        'Expected element to have invalid ARIA attributes, but all ARIA usage is valid.\n' +
        'All ARIA roles and attributes conform to WAI-ARIA specifications.',
    };
  }

  return {
    pass: false,
    message: () =>
      `Expected element to have valid ARIA attributes, but found ${errors.length} error(s):\n\n` +
      formatErrors(errors) +
      '\n\nResources:\n' +
      '  • WAI-ARIA: https://www.w3.org/TR/wai-aria-1.2/\n' +
      '  • ARIA Authoring Practices: https://www.w3.org/WAI/ARIA/apg/\n' +
      '  • ARIA Roles: https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Roles',
  };
}
