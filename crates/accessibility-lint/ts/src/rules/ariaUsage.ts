/**
 * Rule: aria-usage
 * Validates correct ARIA attribute usage according to WAI-ARIA specifications
 * Prevents misuse of ARIA roles, states, and properties
 */

import { ESLintUtils, TSESTree } from '@typescript-eslint/utils';
import type { RuleContext } from '@typescript-eslint/utils/dist/ts-eslint';
import { aria } from 'aria-query';

type MessageIds =
  | 'invalidAriaAttribute'
  | 'unsupportedAriaRole'
  | 'ariaAttributeTypeMismatch'
  | 'redundantAriaRole'
  | 'abstractAriaRole'
  | 'invalidAriaAttributeValue'
  | 'missingRequiredAriaAttribute';

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://meridian-gis.dev/lint-rules/${name}`
);

// Valid ARIA roles (subset of most common roles)
const VALID_ARIA_ROLES = new Set([
  'alert', 'alertdialog', 'application', 'article', 'banner', 'button',
  'checkbox', 'columnheader', 'combobox', 'complementary', 'contentinfo',
  'definition', 'dialog', 'directory', 'document', 'feed', 'figure', 'form',
  'grid', 'gridcell', 'group', 'heading', 'img', 'link', 'list', 'listbox',
  'listitem', 'log', 'main', 'marquee', 'math', 'menu', 'menubar', 'menuitem',
  'menuitemcheckbox', 'menuitemradio', 'navigation', 'none', 'note', 'option',
  'presentation', 'progressbar', 'radio', 'radiogroup', 'region', 'row',
  'rowgroup', 'rowheader', 'scrollbar', 'search', 'searchbox', 'separator',
  'slider', 'spinbutton', 'status', 'switch', 'tab', 'table', 'tablist',
  'tabpanel', 'term', 'textbox', 'timer', 'toolbar', 'tooltip', 'tree',
  'treegrid', 'treeitem',
]);

// Abstract ARIA roles that should not be used directly
const ABSTRACT_ARIA_ROLES = new Set([
  'command', 'composite', 'input', 'landmark', 'range', 'roletype',
  'section', 'sectionhead', 'select', 'structure', 'widget', 'window',
]);

// ARIA boolean attributes
const ARIA_BOOLEAN_ATTRIBUTES = new Set([
  'aria-atomic', 'aria-busy', 'aria-checked', 'aria-disabled', 'aria-expanded',
  'aria-grabbed', 'aria-haspopup', 'aria-hidden', 'aria-invalid', 'aria-modal',
  'aria-multiline', 'aria-multiselectable', 'aria-pressed', 'aria-readonly',
  'aria-required', 'aria-selected',
]);

// Elements with implicit ARIA roles
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

export default createRule<[], MessageIds>({
  name: 'aria-usage',
  meta: {
    type: 'problem',
    docs: {
      description: 'Enforce valid ARIA attributes and roles',
      recommended: 'error',
    },
    messages: {
      invalidAriaAttribute: 'Invalid ARIA attribute "{{attribute}}". Did you mean "{{suggestion}}"?',
      unsupportedAriaRole: 'ARIA role "{{role}}" is not a valid ARIA role.',
      ariaAttributeTypeMismatch: 'ARIA attribute "{{attribute}}" expects a {{expected}} value.',
      redundantAriaRole: 'Redundant ARIA role "{{role}}" on <{{element}}> element. This element has an implicit role.',
      abstractAriaRole: 'Abstract ARIA role "{{role}}" cannot be used directly. Use a concrete role instead.',
      invalidAriaAttributeValue: 'Invalid value for "{{attribute}}". Expected: {{expected}}.',
      missingRequiredAriaAttribute: 'Role "{{role}}" requires attribute "{{attribute}}".',
    },
    schema: [],
  },
  defaultOptions: [],
  create(context: RuleContext<MessageIds, []>) {
    function getAttributeValue(attr: TSESTree.JSXAttribute): string | null {
      if (!attr.value) return null;

      if (attr.value.type === 'Literal') {
        return String(attr.value.value);
      }

      if (attr.value.type === 'JSXExpressionContainer') {
        const expr = attr.value.expression;
        if (expr.type === 'Literal') {
          return String(expr.value);
        }
      }

      return null;
    }

    function checkAriaAttribute(
      node: TSESTree.JSXAttribute,
      attrName: string,
      attrValue: string | null
    ): void {
      // Check if attribute starts with aria-
      if (!attrName.startsWith('aria-')) {
        return;
      }

      // Check for valid ARIA attributes
      const validAriaAttrs = [
        'aria-activedescendant', 'aria-atomic', 'aria-autocomplete', 'aria-busy',
        'aria-checked', 'aria-colcount', 'aria-colindex', 'aria-colspan',
        'aria-controls', 'aria-current', 'aria-describedby', 'aria-details',
        'aria-disabled', 'aria-dropeffect', 'aria-errormessage', 'aria-expanded',
        'aria-flowto', 'aria-grabbed', 'aria-haspopup', 'aria-hidden', 'aria-invalid',
        'aria-keyshortcuts', 'aria-label', 'aria-labelledby', 'aria-level',
        'aria-live', 'aria-modal', 'aria-multiline', 'aria-multiselectable',
        'aria-orientation', 'aria-owns', 'aria-placeholder', 'aria-posinset',
        'aria-pressed', 'aria-readonly', 'aria-relevant', 'aria-required',
        'aria-roledescription', 'aria-rowcount', 'aria-rowindex', 'aria-rowspan',
        'aria-selected', 'aria-setsize', 'aria-sort', 'aria-valuemax',
        'aria-valuemin', 'aria-valuenow', 'aria-valuetext',
      ];

      if (!validAriaAttrs.includes(attrName)) {
        // Try to find closest match
        const suggestion = validAriaAttrs.find((valid) =>
          valid.includes(attrName.slice(5)) || attrName.includes(valid.slice(5))
        ) || 'aria-*';

        context.report({
          node,
          messageId: 'invalidAriaAttribute',
          data: {
            attribute: attrName,
            suggestion,
          },
        });
        return;
      }

      // Check boolean attributes
      if (ARIA_BOOLEAN_ATTRIBUTES.has(attrName) && attrValue) {
        if (attrValue !== 'true' && attrValue !== 'false') {
          context.report({
            node,
            messageId: 'ariaAttributeTypeMismatch',
            data: {
              attribute: attrName,
              expected: 'boolean (true/false)',
            },
          });
        }
      }

      // Check aria-live values
      if (attrName === 'aria-live' && attrValue) {
        const validLiveValues = ['off', 'polite', 'assertive'];
        if (!validLiveValues.includes(attrValue)) {
          context.report({
            node,
            messageId: 'invalidAriaAttributeValue',
            data: {
              attribute: attrName,
              expected: 'off, polite, or assertive',
            },
          });
        }
      }

      // Check aria-current values
      if (attrName === 'aria-current' && attrValue) {
        const validCurrentValues = ['page', 'step', 'location', 'date', 'time', 'true', 'false'];
        if (!validCurrentValues.includes(attrValue)) {
          context.report({
            node,
            messageId: 'invalidAriaAttributeValue',
            data: {
              attribute: attrName,
              expected: 'page, step, location, date, time, true, or false',
            },
          });
        }
      }
    }

    return {
      JSXAttribute(node: TSESTree.JSXAttribute) {
        if (node.name.type !== 'JSXIdentifier') {
          return;
        }

        const attrName = node.name.name;
        const attrValue = getAttributeValue(node);

        // Check ARIA attributes
        if (attrName.startsWith('aria-')) {
          checkAriaAttribute(node, attrName, attrValue);
        }

        // Check role attribute
        if (attrName === 'role' && attrValue) {
          const roles = attrValue.split(' ');

          for (const role of roles) {
            // Check for abstract roles
            if (ABSTRACT_ARIA_ROLES.has(role)) {
              context.report({
                node,
                messageId: 'abstractAriaRole',
                data: {
                  role,
                },
              });
              continue;
            }

            // Check for valid roles
            if (!VALID_ARIA_ROLES.has(role)) {
              context.report({
                node,
                messageId: 'unsupportedAriaRole',
                data: {
                  role,
                },
              });
            }
          }
        }
      },

      JSXOpeningElement(node: TSESTree.JSXOpeningElement) {
        if (node.name.type !== 'JSXIdentifier') {
          return;
        }

        const elementType = node.name.name.toLowerCase();
        const roleAttr = node.attributes.find(
          (attr) =>
            attr.type === 'JSXAttribute' &&
            attr.name.type === 'JSXIdentifier' &&
            attr.name.name === 'role'
        ) as TSESTree.JSXAttribute | undefined;

        // Check for redundant roles
        if (roleAttr && roleAttr.value) {
          const roleValue = getAttributeValue(roleAttr);
          const implicitRole = IMPLICIT_ROLES[elementType];

          if (implicitRole && roleValue === implicitRole) {
            context.report({
              node: roleAttr,
              messageId: 'redundantAriaRole',
              data: {
                role: roleValue,
                element: elementType,
              },
            });
          }
        }
      },
    };
  },
});
