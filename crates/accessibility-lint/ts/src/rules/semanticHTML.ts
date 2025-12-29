/**
 * Rule: semantic-html
 * Enforces use of semantic HTML elements for better accessibility
 * Prevents misuse of div/span when semantic elements are more appropriate
 */

import { ESLintUtils, TSESTree } from '@typescript-eslint/utils';
import type { RuleContext } from '@typescript-eslint/utils/dist/ts-eslint';

type MessageIds =
  | 'useSemanticElement'
  | 'divWithRole'
  | 'spanWithRole'
  | 'useButton'
  | 'useNav'
  | 'useMain'
  | 'useHeader'
  | 'useFooter'
  | 'useArticle'
  | 'useSection'
  | 'useAside'
  | 'nonSemanticHeading'
  | 'divAsButton'
  | 'tableForLayout';

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://meridian-gis.dev/lint-rules/${name}`
);

// Mapping of ARIA roles to semantic HTML elements
const ROLE_TO_SEMANTIC: Record<string, string> = {
  button: 'button',
  navigation: 'nav',
  main: 'main',
  banner: 'header',
  contentinfo: 'footer',
  article: 'article',
  region: 'section',
  complementary: 'aside',
  list: 'ul or ol',
  listitem: 'li',
  heading: 'h1-h6',
  link: 'a',
  form: 'form',
  search: 'form with role="search" or search element',
  table: 'table',
  row: 'tr',
  cell: 'td',
  columnheader: 'th',
  rowheader: 'th',
};

// Roles that should trigger semantic element suggestions
const INTERACTIVE_ROLES_NEEDING_SEMANTIC = new Set([
  'button',
  'link',
  'checkbox',
  'radio',
  'textbox',
  'searchbox',
]);

// Structural roles that have semantic equivalents
const STRUCTURAL_ROLES_NEEDING_SEMANTIC = new Set([
  'navigation',
  'main',
  'banner',
  'contentinfo',
  'article',
  'region',
  'complementary',
  'list',
  'listitem',
  'heading',
]);

export default createRule<[], MessageIds>({
  name: 'semantic-html',
  meta: {
    type: 'suggestion',
    docs: {
      description: 'Enforce use of semantic HTML elements over generic div/span',
      recommended: 'warn',
    },
    messages: {
      useSemanticElement:
        'Use semantic <{{semantic}}> element instead of <{{element}}> with role="{{role}}".',
      divWithRole:
        'Avoid using <div> with role="{{role}}". Use <{{semantic}}> instead.',
      spanWithRole:
        'Avoid using <span> with role="{{role}}". Use <{{semantic}}> instead.',
      useButton:
        'Use <button> element instead of <{{element}}> for clickable controls.',
      useNav:
        'Use <nav> element for navigation sections.',
      useMain:
        'Use <main> element for main content area.',
      useHeader:
        'Use <header> element for page or section headers.',
      useFooter:
        'Use <footer> element for page or section footers.',
      useArticle:
        'Use <article> element for self-contained content.',
      useSection:
        'Use <section> element for thematic grouping.',
      useAside:
        'Use <aside> element for complementary content.',
      nonSemanticHeading:
        'Use heading elements (h1-h6) instead of styled text for headings.',
      divAsButton:
        'Interactive <div> detected. Use <button> element for clickable controls.',
      tableForLayout:
        'Avoid using tables for layout. Use CSS Grid or Flexbox instead.',
    },
    schema: [],
    fixable: 'code',
  },
  defaultOptions: [],
  create(context: RuleContext<MessageIds, []>) {
    const sourceCode = context.getSourceCode();

    function hasAttribute(
      node: TSESTree.JSXOpeningElement,
      attributeName: string
    ): boolean {
      return node.attributes.some(
        (attr) =>
          attr.type === 'JSXAttribute' &&
          attr.name.type === 'JSXIdentifier' &&
          attr.name.name === attributeName
      );
    }

    function getAttributeValue(
      node: TSESTree.JSXOpeningElement,
      attributeName: string
    ): string | null {
      const attr = node.attributes.find(
        (a) =>
          a.type === 'JSXAttribute' &&
          a.name.type === 'JSXIdentifier' &&
          a.name.name === attributeName
      ) as TSESTree.JSXAttribute | undefined;

      if (attr && attr.value && attr.value.type === 'Literal') {
        return String(attr.value.value);
      }
      return null;
    }

    function getElementType(node: TSESTree.JSXOpeningElement): string {
      if (node.name.type === 'JSXIdentifier') {
        return node.name.name.toLowerCase();
      }
      return '';
    }

    return {
      JSXOpeningElement(node: TSESTree.JSXOpeningElement) {
        const elementType = getElementType(node);
        const role = getAttributeValue(node, 'role');
        const hasOnClick = hasAttribute(node, 'onClick');
        const hasOnKeyDown = hasAttribute(node, 'onKeyDown');

        // Check div/span with semantic roles
        if ((elementType === 'div' || elementType === 'span') && role) {
          const semanticElement = ROLE_TO_SEMANTIC[role];

          if (semanticElement) {
            const messageId = elementType === 'div' ? 'divWithRole' : 'spanWithRole';
            context.report({
              node,
              messageId,
              data: {
                role,
                semantic: semanticElement,
              },
            });
          }

          // Specific checks for interactive roles
          if (role === 'button') {
            context.report({
              node,
              messageId: 'useButton',
              data: {
                element: elementType,
              },
              fix(fixer) {
                if (node.name.type === 'JSXIdentifier') {
                  return fixer.replaceText(node.name, 'button');
                }
                return null;
              },
            });
          }

          // Specific checks for structural roles
          if (role === 'navigation') {
            context.report({
              node,
              messageId: 'useNav',
              fix(fixer) {
                if (node.name.type === 'JSXIdentifier') {
                  return fixer.replaceText(node.name, 'nav');
                }
                return null;
              },
            });
          }

          if (role === 'main') {
            context.report({
              node,
              messageId: 'useMain',
              fix(fixer) {
                if (node.name.type === 'JSXIdentifier') {
                  return fixer.replaceText(node.name, 'main');
                }
                return null;
              },
            });
          }

          if (role === 'banner') {
            context.report({
              node,
              messageId: 'useHeader',
              fix(fixer) {
                if (node.name.type === 'JSXIdentifier') {
                  return fixer.replaceText(node.name, 'header');
                }
                return null;
              },
            });
          }

          if (role === 'contentinfo') {
            context.report({
              node,
              messageId: 'useFooter',
              fix(fixer) {
                if (node.name.type === 'JSXIdentifier') {
                  return fixer.replaceText(node.name, 'footer');
                }
                return null;
              },
            });
          }

          if (role === 'article') {
            context.report({
              node,
              messageId: 'useArticle',
              fix(fixer) {
                if (node.name.type === 'JSXIdentifier') {
                  return fixer.replaceText(node.name, 'article');
                }
                return null;
              },
            });
          }

          if (role === 'complementary') {
            context.report({
              node,
              messageId: 'useAside',
              fix(fixer) {
                if (node.name.type === 'JSXIdentifier') {
                  return fixer.replaceText(node.name, 'aside');
                }
                return null;
              },
            });
          }
        }

        // Check for div with onClick (should be button)
        if (elementType === 'div' && hasOnClick) {
          context.report({
            node,
            messageId: 'divAsButton',
            fix(fixer) {
              if (node.name.type === 'JSXIdentifier') {
                return fixer.replaceText(node.name, 'button');
              }
              return null;
            },
          });
        }

        // Check for div with role="heading" (should use h1-h6)
        if (elementType === 'div' && role === 'heading') {
          const ariaLevel = getAttributeValue(node, 'aria-level');
          const level = ariaLevel || '2'; // Default to h2
          context.report({
            node,
            messageId: 'nonSemanticHeading',
            fix(fixer) {
              if (node.name.type === 'JSXIdentifier') {
                return fixer.replaceText(node.name, `h${level}`);
              }
              return null;
            },
          });
        }

        // Check for tables used for layout (heuristic)
        if (elementType === 'table') {
          const hasRolePresentation = role === 'presentation' || role === 'none';
          const hasBorder = getAttributeValue(node, 'border') === '0';

          // If table has role="presentation" or border="0", likely used for layout
          if (hasRolePresentation || hasBorder) {
            context.report({
              node,
              messageId: 'tableForLayout',
            });
          }
        }

        // Check for text styled to look like headings
        if (elementType === 'div' || elementType === 'p' || elementType === 'span') {
          const className = getAttributeValue(node, 'className');
          if (className) {
            // Check for heading-like class names
            const headingLikePatterns = [
              /\bheading\b/i,
              /\btitle\b/i,
              /\bh[1-6]\b/i,
              /\btext-[2-5]?xl\b/, // Tailwind large text
            ];

            const looksLikeHeading = headingLikePatterns.some((pattern) =>
              pattern.test(className)
            );

            if (looksLikeHeading && !role) {
              context.report({
                node,
                messageId: 'nonSemanticHeading',
              });
            }
          }
        }
      },
    };
  },
});
