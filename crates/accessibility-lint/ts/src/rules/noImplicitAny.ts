/**
 * Rule: no-implicit-any
 * Enforces explicit type annotations to prevent implicit 'any' types
 * Critical for type safety in enterprise accessibility applications
 */

import { ESLintUtils, TSESTree } from '@typescript-eslint/utils';
import type { RuleContext } from '@typescript-eslint/utils/dist/ts-eslint';

type MessageIds = 'noImplicitAny' | 'addTypeAnnotation' | 'functionParameter';

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://meridian-gis.dev/lint-rules/${name}`
);

export default createRule<[], MessageIds>({
  name: 'no-implicit-any',
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow implicit any types in function parameters and variables',
      recommended: 'error',
    },
    messages: {
      noImplicitAny: 'Parameter "{{name}}" implicitly has an "any" type.',
      addTypeAnnotation: 'Add an explicit type annotation to "{{name}}".',
      functionParameter: 'Function parameter "{{name}}" must have an explicit type annotation.',
    },
    schema: [],
    fixable: undefined,
  },
  defaultOptions: [],
  create(context: RuleContext<MessageIds, []>) {
    return {
      // Check function parameters
      'FunctionDeclaration > Identifier.params'(node: TSESTree.Identifier) {
        const parent = node.parent as TSESTree.FunctionDeclaration;
        const param = parent.params.find(
          (p) => p.type === 'Identifier' && p.name === node.name
        ) as TSESTree.Identifier | undefined;

        if (param && !param.typeAnnotation) {
          context.report({
            node,
            messageId: 'functionParameter',
            data: {
              name: node.name,
            },
          });
        }
      },

      // Check arrow function parameters
      'ArrowFunctionExpression > Identifier.params'(node: TSESTree.Identifier) {
        if (!node.typeAnnotation) {
          context.report({
            node,
            messageId: 'noImplicitAny',
            data: {
              name: node.name,
            },
          });
        }
      },

      // Check variable declarations without initializers
      VariableDeclarator(node: TSESTree.VariableDeclarator) {
        if (
          node.id.type === 'Identifier' &&
          !node.id.typeAnnotation &&
          !node.init
        ) {
          context.report({
            node: node.id,
            messageId: 'addTypeAnnotation',
            data: {
              name: node.id.name,
            },
          });
        }
      },

      // Check class property declarations
      PropertyDefinition(node: TSESTree.PropertyDefinition) {
        if (
          node.key.type === 'Identifier' &&
          !node.typeAnnotation &&
          !node.value &&
          !node.computed
        ) {
          context.report({
            node: node.key,
            messageId: 'addTypeAnnotation',
            data: {
              name: node.key.name,
            },
          });
        }
      },
    };
  },
});
