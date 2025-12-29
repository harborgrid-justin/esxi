/**
 * AST Utilities for ESLint Rules
 * Provides helper functions for analyzing and manipulating AST nodes
 */

import { TSESTree } from '@typescript-eslint/utils';
import type { RuleContext, RuleListener, RuleModule } from '@typescript-eslint/utils/dist/ts-eslint';

export type { RuleContext, RuleListener, RuleModule };

/**
 * Check if a node is a JSX element
 */
export function isJSXElement(node: TSESTree.Node): node is TSESTree.JSXElement {
  return node.type === 'JSXElement';
}

/**
 * Check if a node is a JSX opening element
 */
export function isJSXOpeningElement(
  node: TSESTree.Node
): node is TSESTree.JSXOpeningElement {
  return node.type === 'JSXOpeningElement';
}

/**
 * Get the name of a JSX element
 */
export function getJSXElementName(
  node: TSESTree.JSXElement | TSESTree.JSXOpeningElement
): string | null {
  const openingElement = 'openingElement' in node ? node.openingElement : node;

  if (openingElement.name.type === 'JSXIdentifier') {
    return openingElement.name.name;
  }

  if (openingElement.name.type === 'JSXMemberExpression') {
    return getJSXMemberExpressionName(openingElement.name);
  }

  return null;
}

/**
 * Get the full name of a JSX member expression (e.g., "Foo.Bar.Baz")
 */
function getJSXMemberExpressionName(
  node: TSESTree.JSXMemberExpression
): string {
  if (node.object.type === 'JSXIdentifier') {
    return `${node.object.name}.${node.property.name}`;
  }

  if (node.object.type === 'JSXMemberExpression') {
    return `${getJSXMemberExpressionName(node.object)}.${node.property.name}`;
  }

  return node.property.name;
}

/**
 * Get a JSX attribute by name
 */
export function getJSXAttribute(
  node: TSESTree.JSXOpeningElement,
  attributeName: string
): TSESTree.JSXAttribute | undefined {
  return node.attributes.find(
    (attr): attr is TSESTree.JSXAttribute =>
      attr.type === 'JSXAttribute' &&
      attr.name.type === 'JSXIdentifier' &&
      attr.name.name === attributeName
  );
}

/**
 * Check if a JSX element has a specific attribute
 */
export function hasJSXAttribute(
  node: TSESTree.JSXOpeningElement,
  attributeName: string
): boolean {
  return getJSXAttribute(node, attributeName) !== undefined;
}

/**
 * Get the value of a JSX attribute as a string
 */
export function getJSXAttributeValue(
  attr: TSESTree.JSXAttribute
): string | null {
  if (!attr.value) {
    return null;
  }

  if (attr.value.type === 'Literal') {
    return String(attr.value.value);
  }

  if (attr.value.type === 'JSXExpressionContainer') {
    const expr = attr.value.expression;
    if (expr.type === 'Literal') {
      return String(expr.value);
    }
    if (expr.type === 'TemplateLiteral' && expr.quasis.length === 1) {
      return expr.quasis[0].value.cooked || null;
    }
  }

  return null;
}

/**
 * Check if a JSX element has text content
 */
export function hasJSXTextContent(node: TSESTree.JSXElement): boolean {
  return node.children.some(
    (child) =>
      child.type === 'JSXText' ||
      child.type === 'JSXExpressionContainer' ||
      (child.type === 'JSXElement' && hasJSXTextContent(child))
  );
}

/**
 * Get all JSX attributes from an opening element
 */
export function getAllJSXAttributes(
  node: TSESTree.JSXOpeningElement
): Map<string, TSESTree.JSXAttribute> {
  const attrs = new Map<string, TSESTree.JSXAttribute>();

  for (const attr of node.attributes) {
    if (attr.type === 'JSXAttribute' && attr.name.type === 'JSXIdentifier') {
      attrs.set(attr.name.name, attr);
    }
  }

  return attrs;
}

/**
 * Check if a node is within a specific parent type
 */
export function isWithinNodeType(
  node: TSESTree.Node,
  parentType: TSESTree.AST_NODE_TYPES
): boolean {
  let current: TSESTree.Node | undefined = node.parent;

  while (current) {
    if (current.type === parentType) {
      return true;
    }
    current = current.parent;
  }

  return false;
}

/**
 * Find the nearest parent of a specific type
 */
export function findParentOfType<T extends TSESTree.Node>(
  node: TSESTree.Node,
  parentType: TSESTree.AST_NODE_TYPES
): T | null {
  let current: TSESTree.Node | undefined = node.parent;

  while (current) {
    if (current.type === parentType) {
      return current as T;
    }
    current = current.parent;
  }

  return null;
}

/**
 * Check if an identifier is a React component (starts with uppercase)
 */
export function isReactComponent(name: string): boolean {
  return /^[A-Z]/.test(name);
}

/**
 * Check if a node is a function component
 */
export function isFunctionComponent(
  node: TSESTree.Node
): node is TSESTree.FunctionDeclaration | TSESTree.ArrowFunctionExpression {
  if (node.type !== 'FunctionDeclaration' && node.type !== 'ArrowFunctionExpression') {
    return false;
  }

  // Check if function name starts with uppercase (for FunctionDeclaration)
  if (node.type === 'FunctionDeclaration' && node.id) {
    return isReactComponent(node.id.name);
  }

  // For arrow functions, check the variable declarator
  if (node.type === 'ArrowFunctionExpression' && node.parent) {
    const parent = node.parent;
    if (
      parent.type === 'VariableDeclarator' &&
      parent.id.type === 'Identifier'
    ) {
      return isReactComponent(parent.id.name);
    }
  }

  return false;
}

/**
 * Get the text content of a node
 */
export function getNodeText(
  node: TSESTree.Node,
  sourceCode: { getText: (node: TSESTree.Node) => string }
): string {
  return sourceCode.getText(node);
}

/**
 * Check if a value is a literal string
 */
export function isLiteralString(node: TSESTree.Node): node is TSESTree.Literal {
  return node.type === 'Literal' && typeof node.value === 'string';
}

/**
 * Check if a value is a template literal with no expressions
 */
export function isStaticTemplateLiteral(
  node: TSESTree.Node
): node is TSESTree.TemplateLiteral {
  return (
    node.type === 'TemplateLiteral' &&
    node.expressions.length === 0 &&
    node.quasis.length === 1
  );
}

/**
 * Get static string value from a node (literal or template)
 */
export function getStaticStringValue(node: TSESTree.Node): string | null {
  if (isLiteralString(node)) {
    return node.value;
  }

  if (isStaticTemplateLiteral(node)) {
    return node.quasis[0].value.cooked || null;
  }

  return null;
}

/**
 * Check if an object property exists and has a specific value
 */
export function hasObjectProperty(
  node: TSESTree.ObjectExpression,
  propertyName: string,
  expectedValue?: string
): boolean {
  for (const prop of node.properties) {
    if (
      prop.type === 'Property' &&
      !prop.computed &&
      prop.key.type === 'Identifier' &&
      prop.key.name === propertyName
    ) {
      if (expectedValue === undefined) {
        return true;
      }

      const value = getStaticStringValue(prop.value);
      return value === expectedValue;
    }
  }

  return false;
}

/**
 * Get object property value
 */
export function getObjectPropertyValue(
  node: TSESTree.ObjectExpression,
  propertyName: string
): TSESTree.Node | null {
  for (const prop of node.properties) {
    if (
      prop.type === 'Property' &&
      !prop.computed &&
      prop.key.type === 'Identifier' &&
      prop.key.name === propertyName
    ) {
      return prop.value;
    }
  }

  return null;
}

/**
 * Check if a node is a boolean literal
 */
export function isBooleanLiteral(
  node: TSESTree.Node,
  value?: boolean
): node is TSESTree.Literal {
  if (node.type !== 'Literal' || typeof node.value !== 'boolean') {
    return false;
  }

  if (value !== undefined) {
    return node.value === value;
  }

  return true;
}

/**
 * Check if a node represents a truthy value
 */
export function isTruthyValue(node: TSESTree.Node): boolean {
  if (node.type === 'Literal') {
    return Boolean(node.value);
  }

  if (node.type === 'Identifier' && node.name === 'undefined') {
    return false;
  }

  // For other nodes, we can't determine truthiness statically
  return true;
}
