/**
 * Rule: accessible-components
 * Ensures React/JSX components follow accessibility best practices
 * Validates component structure, naming, and required accessibility props
 */

import { ESLintUtils, TSESTree } from '@typescript-eslint/utils';
import type { RuleContext } from '@typescript-eslint/utils/dist/ts-eslint';

type MessageIds =
  | 'missingAccessibilityProps'
  | 'invalidComponentStructure'
  | 'missingAriaLabel'
  | 'interactiveElementMissingRole'
  | 'buttonMissingLabel'
  | 'formElementMissingLabel'
  | 'imagesMissingAlt';

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://meridian-gis.dev/lint-rules/${name}`
);

interface Options {
  requireAriaLabel?: boolean;
  checkCustomComponents?: boolean;
}

export default createRule<[Options?], MessageIds>({
  name: 'accessible-components',
  meta: {
    type: 'problem',
    docs: {
      description: 'Enforce accessibility requirements in React/JSX components',
      recommended: 'error',
    },
    messages: {
      missingAccessibilityProps: 'Component "{{name}}" is missing required accessibility props.',
      invalidComponentStructure: 'Component "{{name}}" has invalid accessibility structure.',
      missingAriaLabel: 'Interactive element requires aria-label or aria-labelledby attribute.',
      interactiveElementMissingRole: 'Interactive element "{{element}}" should have an appropriate ARIA role.',
      buttonMissingLabel: 'Button element must have accessible text content or aria-label.',
      formElementMissingLabel: 'Form element "{{element}}" must have an associated label.',
      imagesMissingAlt: 'Image element must have alt attribute for accessibility.',
    },
    schema: [
      {
        type: 'object',
        properties: {
          requireAriaLabel: {
            type: 'boolean',
            default: true,
          },
          checkCustomComponents: {
            type: 'boolean',
            default: true,
          },
        },
        additionalProperties: false,
      },
    ],
  },
  defaultOptions: [
    {
      requireAriaLabel: true,
      checkCustomComponents: true,
    },
  ],
  create(context: RuleContext<MessageIds, [Options?]>) {
    const options = context.options[0] || {};
    const requireAriaLabel = options.requireAriaLabel !== false;
    const checkCustomComponents = options.checkCustomComponents !== false;

    function hasAttribute(
      node: TSESTree.JSXElement | TSESTree.JSXOpeningElement,
      attributeName: string
    ): boolean {
      const openingElement = 'openingElement' in node ? node.openingElement : node;
      return openingElement.attributes.some(
        (attr) =>
          attr.type === 'JSXAttribute' &&
          attr.name.type === 'JSXIdentifier' &&
          attr.name.name === attributeName
      );
    }

    function getElementType(
      node: TSESTree.JSXElement | TSESTree.JSXOpeningElement
    ): string {
      const openingElement = 'openingElement' in node ? node.openingElement : node;
      if (openingElement.name.type === 'JSXIdentifier') {
        return openingElement.name.name;
      }
      return '';
    }

    function hasTextContent(node: TSESTree.JSXElement): boolean {
      return node.children.some(
        (child) =>
          child.type === 'JSXText' ||
          child.type === 'JSXExpressionContainer'
      );
    }

    function isInteractiveElement(elementType: string): boolean {
      const interactiveElements = [
        'button',
        'a',
        'input',
        'select',
        'textarea',
        'details',
        'summary',
      ];
      return interactiveElements.includes(elementType.toLowerCase());
    }

    return {
      JSXElement(node: TSESTree.JSXElement) {
        const elementType = getElementType(node);

        // Check button elements
        if (elementType === 'button') {
          if (!hasTextContent(node) && !hasAttribute(node, 'aria-label') && !hasAttribute(node, 'aria-labelledby')) {
            context.report({
              node: node.openingElement,
              messageId: 'buttonMissingLabel',
            });
          }
        }

        // Check image elements
        if (elementType === 'img') {
          if (!hasAttribute(node, 'alt')) {
            context.report({
              node: node.openingElement,
              messageId: 'imagesMissingAlt',
            });
          }
        }

        // Check form elements
        if (['input', 'select', 'textarea'].includes(elementType)) {
          const hasLabel = hasAttribute(node, 'aria-label') ||
                          hasAttribute(node, 'aria-labelledby') ||
                          hasAttribute(node, 'id'); // ID can be referenced by label element

          if (!hasLabel) {
            context.report({
              node: node.openingElement,
              messageId: 'formElementMissingLabel',
              data: {
                element: elementType,
              },
            });
          }
        }

        // Check interactive elements for ARIA attributes
        if (isInteractiveElement(elementType)) {
          const hasRole = hasAttribute(node, 'role');
          const hasAriaLabel = hasAttribute(node, 'aria-label');
          const hasAriaLabelledBy = hasAttribute(node, 'aria-labelledby');

          if (requireAriaLabel && !hasRole && !hasAriaLabel && !hasAriaLabelledBy && !hasTextContent(node)) {
            context.report({
              node: node.openingElement,
              messageId: 'missingAriaLabel',
            });
          }
        }

        // Check custom components (capitalized names)
        if (checkCustomComponents && /^[A-Z]/.test(elementType)) {
          const hasA11yProps =
            hasAttribute(node, 'aria-label') ||
            hasAttribute(node, 'role') ||
            hasAttribute(node, 'tabIndex') ||
            hasAttribute(node, 'aria-labelledby');

          // Only report if component seems interactive but lacks a11y props
          if (
            elementType.toLowerCase().includes('button') ||
            elementType.toLowerCase().includes('link') ||
            elementType.toLowerCase().includes('input') ||
            elementType.toLowerCase().includes('control')
          ) {
            if (!hasA11yProps && !hasTextContent(node)) {
              context.report({
                node: node.openingElement,
                messageId: 'missingAccessibilityProps',
                data: {
                  name: elementType,
                },
              });
            }
          }
        }
      },

      // Check onClick handlers without keyboard handlers
      'JSXOpeningElement[attributes]'(node: TSESTree.JSXOpeningElement) {
        const hasOnClick = hasAttribute(node, 'onClick');
        const hasOnKeyDown = hasAttribute(node, 'onKeyDown');
        const hasOnKeyPress = hasAttribute(node, 'onKeyPress');
        const hasRole = hasAttribute(node, 'role');
        const elementType = getElementType(node);

        // Non-interactive elements with onClick should have keyboard handlers
        if (hasOnClick && !isInteractiveElement(elementType)) {
          if (!hasOnKeyDown && !hasOnKeyPress) {
            context.report({
              node,
              messageId: 'interactiveElementMissingRole',
              data: {
                element: elementType,
              },
            });
          }

          if (!hasRole) {
            context.report({
              node,
              messageId: 'interactiveElementMissingRole',
              data: {
                element: elementType,
              },
            });
          }
        }
      },
    };
  },
});
