/**
 * React hook for accessibility tree management
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import type { AccessibilityNode } from '../types';
import { AccessibilityTreeBuilder } from '../analyzers/AccessibilityTreeBuilder';

export interface UseAccessibilityTreeOptions {
  root?: Element;
  autoUpdate?: boolean;
  updateDelay?: number;
}

export interface UseAccessibilityTreeReturn {
  tree: AccessibilityNode | null;
  isBuilding: boolean;
  rebuild: () => void;
  findNode: (element: Element) => AccessibilityNode | null;
  getFocusableNodes: () => AccessibilityNode[];
  getNodesByRole: (role: string) => AccessibilityNode[];
  getNodePath: (node: AccessibilityNode) => AccessibilityNode[];
  traverseTree: (callback: (node: AccessibilityNode) => void) => void;
}

export function useAccessibilityTree(
  options: UseAccessibilityTreeOptions = {}
): UseAccessibilityTreeReturn {
  const {
    root = document.body,
    autoUpdate = true,
    updateDelay = 500,
  } = options;

  const [tree, setTree] = useState<AccessibilityNode | null>(null);
  const [isBuilding, setIsBuilding] = useState(false);

  const treeBuilder = useRef(new AccessibilityTreeBuilder());
  const updateTimeoutRef = useRef<number>();

  /**
   * Rebuild the accessibility tree
   */
  const rebuild = useCallback(() => {
    setIsBuilding(true);

    // Use requestAnimationFrame to avoid blocking
    requestAnimationFrame(() => {
      try {
        const newTree = treeBuilder.current.buildTree(root);
        setTree(newTree);
      } catch (error) {
        console.error('Error building accessibility tree:', error);
      } finally {
        setIsBuilding(false);
      }
    });
  }, [root]);

  /**
   * Find node by element
   */
  const findNode = useCallback((element: Element): AccessibilityNode | null => {
    return treeBuilder.current.findNode(element);
  }, []);

  /**
   * Get all focusable nodes in tab order
   */
  const getFocusableNodes = useCallback((): AccessibilityNode[] => {
    if (!tree) return [];
    return treeBuilder.current.getFocusableNodes(tree);
  }, [tree]);

  /**
   * Get nodes by role
   */
  const getNodesByRole = useCallback((role: string): AccessibilityNode[] => {
    if (!tree) return [];

    const results: AccessibilityNode[] = [];

    const traverse = (node: AccessibilityNode) => {
      if (node.role === role && !node.hidden) {
        results.push(node);
      }
      node.children.forEach(traverse);
    };

    traverse(tree);
    return results;
  }, [tree]);

  /**
   * Get path from root to node
   */
  const getNodePath = useCallback((node: AccessibilityNode): AccessibilityNode[] => {
    const path: AccessibilityNode[] = [];
    let current: AccessibilityNode | null = node;

    while (current) {
      path.unshift(current);
      current = current.parent;
    }

    return path;
  }, []);

  /**
   * Traverse tree with callback
   */
  const traverseTree = useCallback((callback: (node: AccessibilityNode) => void) => {
    if (!tree) return;

    const traverse = (node: AccessibilityNode) => {
      callback(node);
      node.children.forEach(traverse);
    };

    traverse(tree);
  }, [tree]);

  /**
   * Initial build
   */
  useEffect(() => {
    rebuild();
  }, [rebuild]);

  /**
   * Watch for DOM changes
   */
  useEffect(() => {
    if (!autoUpdate) return;

    const observer = new MutationObserver(() => {
      // Clear existing timeout
      if (updateTimeoutRef.current) {
        window.clearTimeout(updateTimeoutRef.current);
      }

      // Debounce rebuild
      updateTimeoutRef.current = window.setTimeout(() => {
        rebuild();
      }, updateDelay);
    });

    observer.observe(root, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: [
        'aria-label',
        'aria-labelledby',
        'aria-describedby',
        'aria-hidden',
        'aria-disabled',
        'aria-expanded',
        'aria-selected',
        'aria-checked',
        'aria-pressed',
        'aria-current',
        'aria-live',
        'aria-atomic',
        'aria-relevant',
        'aria-busy',
        'role',
        'alt',
        'title',
        'tabindex',
        'disabled',
        'readonly',
        'required',
        'hidden',
      ],
    });

    return () => {
      observer.disconnect();
      if (updateTimeoutRef.current) {
        window.clearTimeout(updateTimeoutRef.current);
      }
    };
  }, [root, autoUpdate, updateDelay, rebuild]);

  return {
    tree,
    isBuilding,
    rebuild,
    findNode,
    getFocusableNodes,
    getNodesByRole,
    getNodePath,
    traverseTree,
  };
}
