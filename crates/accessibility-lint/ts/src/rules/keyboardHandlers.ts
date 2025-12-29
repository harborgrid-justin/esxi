/**
 * Rule: keyboard-handlers
 * Ensures interactive elements have appropriate keyboard event handlers
 * Critical for keyboard navigation and accessibility
 */

import { ESLintUtils, TSESTree } from '@typescript-eslint/utils';
import type { RuleContext } from '@typescript-eslint/utils/dist/ts-eslint';

type MessageIds =
  | 'missingKeyboardHandler'
  | 'clickWithoutKeyboard'
  | 'mouseOnlyInteraction'
  | 'addKeyDownHandler'
  | 'addKeyPressHandler'
  | 'nonInteractiveFocusable'
  | 'missingTabIndex';

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://meridian-gis.dev/lint-rules/${name}`
);

// Elements that are natively interactive
const INTERACTIVE_ELEMENTS = new Set([
  'a',
  'button',
  'input',
  'select',
  'textarea',
  'details',
  'summary',
  'area',
  'audio',
  'video',
]);

// Interactive ARIA roles
const INTERACTIVE_ROLES = new Set([
  'button',
  'link',
  'checkbox',
  'menuitem',
  'menuitemcheckbox',
  'menuitemradio',
  'option',
  'radio',
  'searchbox',
  'slider',
  'spinbutton',
  'switch',
  'tab',
  'textbox',
  'combobox',
]);

// Mouse event handlers that should have keyboard equivalents
const MOUSE_HANDLERS = new Set([
  'onClick',
  'onMouseDown',
  'onMouseUp',
  'onMouseOver',
  'onMouseOut',
  'onMouseEnter',
  'onMouseLeave',
]);

// Keyboard event handlers
const KEYBOARD_HANDLERS = new Set([
  'onKeyDown',
  'onKeyUp',
  'onKeyPress',
]);

export default createRule<[], MessageIds>({
  name: 'keyboard-handlers',
  meta: {
    type: 'problem',
    docs: {
      description: 'Enforce keyboard event handlers for interactive elements',
      recommended: 'error',
    },
    messages: {
      missingKeyboardHandler:
        'Element with {{mouseHandler}} should also have a keyboard handler (onKeyDown, onKeyUp, or onKeyPress).',
      clickWithoutKeyboard:
        'onClick handler should be accompanied by onKeyDown or onKeyPress for keyboard accessibility.',
      mouseOnlyInteraction:
        'Mouse-only event handler "{{handler}}" detected. Add keyboard alternative.',
      addKeyDownHandler:
        'Add onKeyDown handler to support keyboard interaction.',
      addKeyPressHandler:
        'Add onKeyPress handler for keyboard users.',
      nonInteractiveFocusable:
        'Non-interactive element with tabIndex should have keyboard handlers.',
      missingTabIndex:
        'Interactive element with mouse handlers should have tabIndex for keyboard focus.',
    },
    schema: [],
    hasSuggestions: true,
  },
  defaultOptions: [],
  create(context: RuleContext<MessageIds, []>) {
    function hasAttribute(
      openingElement: TSESTree.JSXOpeningElement,
      attributeName: string
    ): boolean {
      return openingElement.attributes.some(
        (attr) =>
          attr.type === 'JSXAttribute' &&
          attr.name.type === 'JSXIdentifier' &&
          attr.name.name === attributeName
      );
    }

    function getAttribute(
      openingElement: TSESTree.JSXOpeningElement,
      attributeName: string
    ): TSESTree.JSXAttribute | undefined {
      return openingElement.attributes.find(
        (attr) =>
          attr.type === 'JSXAttribute' &&
          attr.name.type === 'JSXIdentifier' &&
          attr.name.name === attributeName
      ) as TSESTree.JSXAttribute | undefined;
    }

    function getElementType(openingElement: TSESTree.JSXOpeningElement): string {
      if (openingElement.name.type === 'JSXIdentifier') {
        return openingElement.name.name.toLowerCase();
      }
      return '';
    }

    function getRoleValue(openingElement: TSESTree.JSXOpeningElement): string | null {
      const roleAttr = getAttribute(openingElement, 'role');
      if (roleAttr && roleAttr.value && roleAttr.value.type === 'Literal') {
        return String(roleAttr.value.value);
      }
      return null;
    }

    function isInteractiveElement(
      elementType: string,
      role: string | null
    ): boolean {
      if (INTERACTIVE_ELEMENTS.has(elementType)) {
        return true;
      }
      if (role && INTERACTIVE_ROLES.has(role)) {
        return true;
      }
      return false;
    }

    function getMouseHandlers(
      openingElement: TSESTree.JSXOpeningElement
    ): string[] {
      const handlers: string[] = [];
      for (const handler of MOUSE_HANDLERS) {
        if (hasAttribute(openingElement, handler)) {
          handlers.push(handler);
        }
      }
      return handlers;
    }

    function hasKeyboardHandler(
      openingElement: TSESTree.JSXOpeningElement
    ): boolean {
      for (const handler of KEYBOARD_HANDLERS) {
        if (hasAttribute(openingElement, handler)) {
          return true;
        }
      }
      return false;
    }

    return {
      JSXOpeningElement(node: TSESTree.JSXOpeningElement) {
        const elementType = getElementType(node);
        const role = getRoleValue(node);
        const isNativelyInteractive = isInteractiveElement(elementType, role);
        const mouseHandlers = getMouseHandlers(node);
        const hasKeyboard = hasKeyboardHandler(node);
        const hasTabIndex = hasAttribute(node, 'tabIndex');
        const hasOnClick = hasAttribute(node, 'onClick');

        // Check for onClick without keyboard handler
        if (hasOnClick && !isNativelyInteractive) {
          if (!hasKeyboard) {
            const onClickAttr = getAttribute(node, 'onClick');
            if (onClickAttr) {
              context.report({
                node: onClickAttr,
                messageId: 'clickWithoutKeyboard',
                suggest: [
                  {
                    messageId: 'addKeyDownHandler',
                    fix(fixer) {
                      // Suggest adding onKeyDown handler
                      return fixer.insertTextAfter(
                        onClickAttr,
                        ' onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { /* handle */ } }}'
                      );
                    },
                  },
                ],
              });
            }
          }

          // Non-interactive elements with onClick should have tabIndex
          if (!hasTabIndex && !role) {
            context.report({
              node,
              messageId: 'missingTabIndex',
            });
          }
        }

        // Check for other mouse handlers without keyboard equivalents
        for (const mouseHandler of mouseHandlers) {
          if (mouseHandler !== 'onClick' && !hasKeyboard && !isNativelyInteractive) {
            const handlerAttr = getAttribute(node, mouseHandler);
            if (handlerAttr) {
              context.report({
                node: handlerAttr,
                messageId: 'mouseOnlyInteraction',
                data: {
                  handler: mouseHandler,
                },
                suggest: [
                  {
                    messageId: 'addKeyDownHandler',
                    fix(fixer) {
                      return fixer.insertTextAfter(
                        handlerAttr,
                        ' onKeyDown={(e) => { /* handle keyboard event */ }}'
                      );
                    },
                  },
                ],
              });
            }
          }
        }

        // Check for tabIndex on non-interactive elements without keyboard handlers
        if (hasTabIndex && !isNativelyInteractive && !hasKeyboard && mouseHandlers.length === 0) {
          const tabIndexAttr = getAttribute(node, 'tabIndex');
          if (tabIndexAttr) {
            context.report({
              node: tabIndexAttr,
              messageId: 'nonInteractiveFocusable',
            });
          }
        }

        // Special case: onMouseOver/onMouseOut should have onFocus/onBlur
        const hasMouseOver = hasAttribute(node, 'onMouseOver') || hasAttribute(node, 'onMouseEnter');
        const hasMouseOut = hasAttribute(node, 'onMouseOut') || hasAttribute(node, 'onMouseLeave');
        const hasFocus = hasAttribute(node, 'onFocus');
        const hasBlur = hasAttribute(node, 'onBlur');

        if (hasMouseOver && !hasFocus) {
          const mouseOverAttr = getAttribute(node, 'onMouseOver') || getAttribute(node, 'onMouseEnter');
          if (mouseOverAttr) {
            context.report({
              node: mouseOverAttr,
              messageId: 'missingKeyboardHandler',
              data: {
                mouseHandler: mouseOverAttr.name.type === 'JSXIdentifier' ? mouseOverAttr.name.name : 'onMouseOver',
              },
              suggest: [
                {
                  messageId: 'addKeyDownHandler',
                  fix(fixer) {
                    return fixer.insertTextAfter(
                      mouseOverAttr,
                      ' onFocus={(e) => { /* handle focus */ }}'
                    );
                  },
                },
              ],
            });
          }
        }

        if (hasMouseOut && !hasBlur) {
          const mouseOutAttr = getAttribute(node, 'onMouseOut') || getAttribute(node, 'onMouseLeave');
          if (mouseOutAttr) {
            context.report({
              node: mouseOutAttr,
              messageId: 'missingKeyboardHandler',
              data: {
                mouseHandler: mouseOutAttr.name.type === 'JSXIdentifier' ? mouseOutAttr.name.name : 'onMouseOut',
              },
              suggest: [
                {
                  messageId: 'addKeyDownHandler',
                  fix(fixer) {
                    return fixer.insertTextAfter(
                      mouseOutAttr,
                      ' onBlur={(e) => { /* handle blur */ }}'
                    );
                  },
                },
              ],
            });
          }
        }
      },

      // Check class methods for mouse event handlers
      MethodDefinition(node: TSESTree.MethodDefinition) {
        if (node.key.type === 'Identifier') {
          const methodName = node.key.name;

          // Check for handle* methods that might be mouse handlers
          if (methodName.startsWith('handle') && methodName.includes('Click')) {
            // This is a heuristic check - could be enhanced
            const className = node.parent?.parent;
            if (className && className.type === 'ClassBody') {
              const hasKeyboardMethod = className.body.some(
                (member) =>
                  member.type === 'MethodDefinition' &&
                  member.key.type === 'Identifier' &&
                  (member.key.name.includes('KeyDown') ||
                    member.key.name.includes('KeyPress') ||
                    member.key.name.includes('Keyboard'))
              );

              if (!hasKeyboardMethod) {
                context.report({
                  node: node.key,
                  messageId: 'addKeyDownHandler',
                });
              }
            }
          }
        }
      },
    };
  },
});
