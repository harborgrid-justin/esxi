/**
 * Rule: color-contrast
 * Validates color contrast ratios meet WCAG 2.1 standards
 * Ensures text is readable against background colors
 */

import { ESLintUtils, TSESTree } from '@typescript-eslint/utils';
import type { RuleContext } from '@typescript-eslint/utils/dist/ts-eslint';

type MessageIds =
  | 'insufficientContrast'
  | 'contrastNotVerifiable'
  | 'improveContrast'
  | 'useContrastChecker'
  | 'wcagAAFailed'
  | 'wcagAAAFailed';

const createRule = ESLintUtils.RuleCreator(
  (name) => `https://meridian-gis.dev/lint-rules/${name}`
);

interface ContrastOptions {
  wcagLevel?: 'AA' | 'AAA';
  largeText?: boolean;
}

// WCAG 2.1 Contrast ratios
const WCAG_AA_NORMAL = 4.5;
const WCAG_AA_LARGE = 3.0;
const WCAG_AAA_NORMAL = 7.0;
const WCAG_AAA_LARGE = 4.5;

/**
 * Calculate relative luminance of an RGB color
 * Formula from WCAG 2.1: https://www.w3.org/TR/WCAG21/#dfn-relative-luminance
 */
function getLuminance(r: number, g: number, b: number): number {
  const [rs, gs, bs] = [r, g, b].map((c) => {
    const val = c / 255;
    return val <= 0.03928 ? val / 12.92 : Math.pow((val + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
}

/**
 * Calculate contrast ratio between two colors
 * Formula from WCAG 2.1: https://www.w3.org/TR/WCAG21/#dfn-contrast-ratio
 */
function getContrastRatio(
  rgb1: [number, number, number],
  rgb2: [number, number, number]
): number {
  const lum1 = getLuminance(...rgb1);
  const lum2 = getLuminance(...rgb2);
  const lighter = Math.max(lum1, lum2);
  const darker = Math.min(lum1, lum2);
  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * Parse CSS color value to RGB
 */
function parseColor(color: string): [number, number, number] | null {
  // Remove whitespace
  color = color.trim().toLowerCase();

  // Hex colors
  const hexMatch = color.match(/^#?([a-f0-9]{6}|[a-f0-9]{3})$/);
  if (hexMatch) {
    let hex = hexMatch[1];
    if (hex.length === 3) {
      hex = hex[0] + hex[0] + hex[1] + hex[1] + hex[2] + hex[2];
    }
    const r = parseInt(hex.substr(0, 2), 16);
    const g = parseInt(hex.substr(2, 2), 16);
    const b = parseInt(hex.substr(4, 2), 16);
    return [r, g, b];
  }

  // RGB/RGBA colors
  const rgbMatch = color.match(/^rgba?\((\d+),\s*(\d+),\s*(\d+)/);
  if (rgbMatch) {
    return [
      parseInt(rgbMatch[1], 10),
      parseInt(rgbMatch[2], 10),
      parseInt(rgbMatch[3], 10),
    ];
  }

  // Named colors (subset)
  const namedColors: Record<string, [number, number, number]> = {
    black: [0, 0, 0],
    white: [255, 255, 255],
    red: [255, 0, 0],
    green: [0, 128, 0],
    blue: [0, 0, 255],
    yellow: [255, 255, 0],
    gray: [128, 128, 128],
    grey: [128, 128, 128],
    orange: [255, 165, 0],
    purple: [128, 0, 128],
    pink: [255, 192, 203],
    brown: [165, 42, 42],
  };

  return namedColors[color] || null;
}

export default createRule<[ContrastOptions?], MessageIds>({
  name: 'color-contrast',
  meta: {
    type: 'problem',
    docs: {
      description: 'Enforce sufficient color contrast for accessibility',
      recommended: 'warn',
    },
    messages: {
      insufficientContrast:
        'Insufficient color contrast ({{ratio}}:1). Required: {{required}}:1 for {{level}}.',
      contrastNotVerifiable:
        'Cannot verify color contrast. Ensure colors are defined as literals.',
      improveContrast:
        'Color contrast ratio {{ratio}}:1 is below recommended {{required}}:1.',
      useContrastChecker:
        'Use a color contrast checker to verify accessibility compliance.',
      wcagAAFailed: 'Failed WCAG AA contrast requirements ({{ratio}}:1 < {{required}}:1).',
      wcagAAAFailed: 'Failed WCAG AAA contrast requirements ({{ratio}}:1 < {{required}}:1).',
    },
    schema: [
      {
        type: 'object',
        properties: {
          wcagLevel: {
            type: 'string',
            enum: ['AA', 'AAA'],
            default: 'AA',
          },
          largeText: {
            type: 'boolean',
            default: false,
          },
        },
        additionalProperties: false,
      },
    ],
  },
  defaultOptions: [
    {
      wcagLevel: 'AA',
      largeText: false,
    },
  ],
  create(context: RuleContext<MessageIds, [ContrastOptions?]>) {
    const options = context.options[0] || {};
    const wcagLevel = options.wcagLevel || 'AA';
    const largeText = options.largeText || false;

    // Determine required contrast ratio
    let requiredRatio: number;
    if (wcagLevel === 'AAA') {
      requiredRatio = largeText ? WCAG_AAA_LARGE : WCAG_AAA_NORMAL;
    } else {
      requiredRatio = largeText ? WCAG_AA_LARGE : WCAG_AA_NORMAL;
    }

    function checkContrast(
      node: TSESTree.Node,
      color: string | null,
      backgroundColor: string | null
    ): void {
      if (!color || !backgroundColor) {
        return;
      }

      const foreground = parseColor(color);
      const background = parseColor(backgroundColor);

      if (!foreground || !background) {
        context.report({
          node,
          messageId: 'contrastNotVerifiable',
        });
        return;
      }

      const ratio = getContrastRatio(foreground, background);
      const roundedRatio = Math.round(ratio * 100) / 100;

      if (ratio < requiredRatio) {
        const messageId = wcagLevel === 'AAA' ? 'wcagAAAFailed' : 'wcagAAFailed';
        context.report({
          node,
          messageId,
          data: {
            ratio: roundedRatio.toFixed(2),
            required: requiredRatio.toFixed(1),
            level: `WCAG ${wcagLevel}`,
          },
        });
      }
    }

    function extractStyleValue(
      properties: TSESTree.ObjectLiteralElement[],
      propertyName: string
    ): string | null {
      for (const prop of properties) {
        if (
          prop.type === 'Property' &&
          !prop.computed &&
          prop.key.type === 'Identifier' &&
          prop.key.name === propertyName
        ) {
          if (prop.value.type === 'Literal') {
            return String(prop.value.value);
          }
        }
      }
      return null;
    }

    return {
      // Check JSX style props
      'JSXAttribute[name.name="style"]'(node: TSESTree.JSXAttribute) {
        if (
          node.value &&
          node.value.type === 'JSXExpressionContainer' &&
          node.value.expression.type === 'ObjectExpression'
        ) {
          const properties = node.value.expression.properties;
          const color = extractStyleValue(properties, 'color');
          const backgroundColor = extractStyleValue(properties, 'backgroundColor') ||
                                 extractStyleValue(properties, 'background');

          checkContrast(node, color, backgroundColor);
        }
      },

      // Check CSS-in-JS objects
      'ObjectExpression > Property[key.name="color"]'(node: TSESTree.Property) {
        const parent = node.parent as TSESTree.ObjectExpression;
        const backgroundColor = extractStyleValue(parent.properties, 'backgroundColor') ||
                               extractStyleValue(parent.properties, 'background');

        if (node.value.type === 'Literal') {
          const color = String(node.value.value);
          checkContrast(node, color, backgroundColor);
        }
      },

      // Check className with color utilities (Tailwind-style)
      'JSXAttribute[name.name="className"]'(node: TSESTree.JSXAttribute) {
        if (node.value && node.value.type === 'Literal') {
          const classNames = String(node.value.value);

          // Simple heuristic: warn if both text and background color classes are present
          const hasTextColor = /text-(white|black|gray|red|blue|green|yellow|purple|pink|orange)/.test(classNames);
          const hasBgColor = /bg-(white|black|gray|red|blue|green|yellow|purple|pink|orange)/.test(classNames);

          if (hasTextColor && hasBgColor) {
            // Extract colors for checking (simplified)
            const textMatch = classNames.match(/text-(white|black|gray-\d+|red-\d+|blue-\d+|green-\d+)/);
            const bgMatch = classNames.match(/bg-(white|black|gray-\d+|red-\d+|blue-\d+|green-\d+)/);

            if (textMatch && bgMatch) {
              // Simplified check for obvious problematic combinations
              const text = textMatch[1];
              const bg = bgMatch[1];

              if (
                (text.includes('white') && bg.includes('white')) ||
                (text.includes('black') && bg.includes('black')) ||
                (text === bg)
              ) {
                context.report({
                  node,
                  messageId: 'useContrastChecker',
                });
              }
            }
          }
        }
      },
    };
  },
});
