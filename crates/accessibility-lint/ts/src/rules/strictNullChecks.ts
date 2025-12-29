/**
 * Rule: strict-null-checks
 * Enforces strict null checking patterns to prevent runtime null/undefined errors
 * Essential for accessibility applications where null-safety is critical
 */

import { ESLintUtils, TSESTree } from '@typescript-eslint/utils';
import type { RuleContext } from '@typescript-eslint/utils/dist/ts-eslint';

type MessageIds =
  | 'nullableAccess'
  | 'undefinedAccess'
  | 'missingNullCheck'
  | 'useOptionalChaining'
  | 'useNullishCoalescing';

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://meridian-gis.dev/lint-rules/${name}`
);

export default createRule<[], MessageIds>({
  name: 'strict-null-checks',
  meta: {
    type: 'problem',
    docs: {
      description: 'Enforce strict null checking patterns and safe property access',
      recommended: 'error',
    },
    messages: {
      nullableAccess: 'Unsafe access of potentially nullable value "{{name}}".',
      undefinedAccess: 'Unsafe access of potentially undefined value "{{name}}".',
      missingNullCheck: 'Add null/undefined check before accessing "{{name}}".',
      useOptionalChaining: 'Use optional chaining (?.) for safer access of "{{name}}".',
      useNullishCoalescing: 'Consider using nullish coalescing (??) for "{{name}}".',
    },
    schema: [],
    hasSuggestions: true,
  },
  defaultOptions: [],
  create(context: RuleContext<MessageIds, []>) {
    const sourceCode = context.getSourceCode();

    function isNullableType(node: TSESTree.Node): boolean {
      // Check if type annotation includes null or undefined
      if (node.type === 'Identifier' && 'typeAnnotation' in node) {
        const typeAnnotation = (node as TSESTree.Identifier).typeAnnotation;
        if (typeAnnotation) {
          const typeNode = typeAnnotation.typeAnnotation;
          if (typeNode.type === 'TSUnionType') {
            return typeNode.types.some(
              (t) =>
                t.type === 'TSNullKeyword' ||
                t.type === 'TSUndefinedKeyword'
            );
          }
        }
      }
      return false;
    }

    function isWithinNullCheck(node: TSESTree.Node): boolean {
      let current: TSESTree.Node | undefined = node.parent;

      while (current) {
        // Check for if statement with null check
        if (current.type === 'IfStatement') {
          const test = current.test;
          if (
            test.type === 'BinaryExpression' &&
            (test.operator === '!==' || test.operator === '!=')
          ) {
            return true;
          }
        }

        // Check for optional chaining
        if (current.type === 'ChainExpression') {
          return true;
        }

        current = current.parent;
      }

      return false;
    }

    return {
      MemberExpression(node: TSESTree.MemberExpression) {
        if (node.object.type === 'Identifier') {
          const objectName = node.object.name;

          // Check if this is potentially nullable access
          if (isNullableType(node.object) && !isWithinNullCheck(node)) {
            const propertyName = node.property.type === 'Identifier'
              ? node.property.name
              : '<computed>';

            context.report({
              node,
              messageId: 'useOptionalChaining',
              data: {
                name: `${objectName}.${propertyName}`,
              },
              suggest: [
                {
                  messageId: 'useOptionalChaining',
                  data: {
                    name: objectName,
                  },
                  fix(fixer) {
                    const token = sourceCode.getFirstToken(node.object);
                    if (token) {
                      return fixer.insertTextAfter(node.object, '?');
                    }
                    return null;
                  },
                },
              ],
            });
          }
        }
      },

      // Check for logical OR that should use nullish coalescing
      LogicalExpression(node: TSESTree.LogicalExpression) {
        if (node.operator === '||' && node.left.type === 'Identifier') {
          if (isNullableType(node.left)) {
            context.report({
              node,
              messageId: 'useNullishCoalescing',
              data: {
                name: node.left.name,
              },
              suggest: [
                {
                  messageId: 'useNullishCoalescing',
                  data: {
                    name: node.left.name,
                  },
                  fix(fixer) {
                    const operatorToken = sourceCode.getTokenAfter(
                      node.left,
                      (token) => token.value === '||'
                    );
                    if (operatorToken) {
                      return fixer.replaceText(operatorToken, '??');
                    }
                    return null;
                  },
                },
              ],
            });
          }
        }
      },

      // Check call expressions on potentially null/undefined values
      CallExpression(node: TSESTree.CallExpression) {
        if (node.callee.type === 'MemberExpression') {
          const object = node.callee.object;
          if (object.type === 'Identifier' && isNullableType(object)) {
            if (!isWithinNullCheck(node)) {
              context.report({
                node: node.callee,
                messageId: 'missingNullCheck',
                data: {
                  name: object.name,
                },
              });
            }
          }
        }
      },
    };
  },
});
