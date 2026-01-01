/**
 * Spatial Index Service
 * R-tree based spatial indexing for fast queries
 */

import { Feature, Bounds, SpatialIndex, IndexedFeature } from '../types';
import { GeometryFactory } from '../geometry/GeometryFactory';

export class SpatialIndexService implements SpatialIndex {
  private tree: RTreeNode | null = null;
  private maxEntries = 9;
  private minEntries = 4;

  /**
   * Insert feature into index
   */
  insert(feature: Feature): void {
    const bounds = GeometryFactory.getBounds(feature.geometry);

    const indexed: IndexedFeature = {
      minX: bounds.minX,
      minY: bounds.minY,
      maxX: bounds.maxX,
      maxY: bounds.maxY,
      feature,
    };

    if (!this.tree) {
      this.tree = {
        children: [],
        leaf: true,
        bounds: { ...bounds },
      };
    }

    this.insertInternal(this.tree, indexed);
  }

  /**
   * Remove feature from index
   */
  remove(feature: Feature): void {
    if (!this.tree) return;

    const bounds = GeometryFactory.getBounds(feature.geometry);

    const indexed: IndexedFeature = {
      minX: bounds.minX,
      minY: bounds.minY,
      maxX: bounds.maxX,
      maxY: bounds.maxY,
      feature,
    };

    this.removeInternal(this.tree, indexed);
  }

  /**
   * Search for features intersecting bounds
   */
  search(bounds: Bounds): Feature[] {
    if (!this.tree) return [];

    const results: Feature[] = [];
    this.searchInternal(this.tree, bounds, results);

    return results;
  }

  /**
   * Clear the index
   */
  clear(): void {
    this.tree = null;
  }

  /**
   * Get all indexed features
   */
  all(): Feature[] {
    if (!this.tree) return [];

    const results: Feature[] = [];
    this.getAllInternal(this.tree, results);

    return results;
  }

  /**
   * Internal insert
   */
  private insertInternal(node: RTreeNode, item: IndexedFeature): void {
    if (node.leaf) {
      node.children.push(item);
      this.extendBounds(node.bounds, item);

      if (node.children.length > this.maxEntries) {
        this.splitNode(node);
      }
    } else {
      const bestChild = this.chooseSubtree(node, item);
      this.insertInternal(bestChild as RTreeNode, item);
      this.extendBounds(node.bounds, item);
    }
  }

  /**
   * Internal remove
   */
  private removeInternal(node: RTreeNode, item: IndexedFeature): boolean {
    if (node.leaf) {
      const index = node.children.findIndex(
        (child: any) => child.feature === item.feature
      );

      if (index !== -1) {
        node.children.splice(index, 1);
        return true;
      }

      return false;
    }

    for (let i = 0; i < node.children.length; i++) {
      const child = node.children[i] as RTreeNode;

      if (this.boundsIntersect(child.bounds, item)) {
        if (this.removeInternal(child, item)) {
          if (child.children.length < this.minEntries) {
            // Handle underflow
            node.children.splice(i, 1);
          }
          return true;
        }
      }
    }

    return false;
  }

  /**
   * Internal search
   */
  private searchInternal(
    node: RTreeNode,
    bounds: Bounds,
    results: Feature[]
  ): void {
    if (!this.boundsIntersect(node.bounds, bounds)) {
      return;
    }

    if (node.leaf) {
      for (const item of node.children as IndexedFeature[]) {
        if (this.boundsIntersect(item, bounds)) {
          results.push(item.feature);
        }
      }
    } else {
      for (const child of node.children as RTreeNode[]) {
        this.searchInternal(child, bounds, results);
      }
    }
  }

  /**
   * Get all features from tree
   */
  private getAllInternal(node: RTreeNode, results: Feature[]): void {
    if (node.leaf) {
      for (const item of node.children as IndexedFeature[]) {
        results.push(item.feature);
      }
    } else {
      for (const child of node.children as RTreeNode[]) {
        this.getAllInternal(child, results);
      }
    }
  }

  /**
   * Choose best subtree for insertion
   */
  private chooseSubtree(node: RTreeNode, item: IndexedFeature): RTreeNode {
    let minEnlargement = Infinity;
    let bestChild: RTreeNode = node.children[0] as RTreeNode;

    for (const child of node.children as RTreeNode[]) {
      const enlargement = this.calculateEnlargement(child.bounds, item);

      if (enlargement < minEnlargement) {
        minEnlargement = enlargement;
        bestChild = child;
      }
    }

    return bestChild;
  }

  /**
   * Split node when it overflows
   */
  private splitNode(node: RTreeNode): void {
    // Simplified split - production would use quadratic or R* split
    const midPoint = Math.floor(node.children.length / 2);
    const newNode: RTreeNode = {
      children: node.children.splice(midPoint),
      leaf: node.leaf,
      bounds: this.calculateBounds(node.children),
    };

    node.bounds = this.calculateBounds(node.children);
  }

  /**
   * Extend bounds to include item
   */
  private extendBounds(bounds: Bounds, item: Bounds): void {
    bounds.minX = Math.min(bounds.minX, item.minX);
    bounds.minY = Math.min(bounds.minY, item.minY);
    bounds.maxX = Math.max(bounds.maxX, item.maxX);
    bounds.maxY = Math.max(bounds.maxY, item.maxY);
  }

  /**
   * Calculate enlargement needed to include item
   */
  private calculateEnlargement(bounds: Bounds, item: Bounds): number {
    const currentArea =
      (bounds.maxX - bounds.minX) * (bounds.maxY - bounds.minY);

    const newMinX = Math.min(bounds.minX, item.minX);
    const newMinY = Math.min(bounds.minY, item.minY);
    const newMaxX = Math.max(bounds.maxX, item.maxX);
    const newMaxY = Math.max(bounds.maxY, item.maxY);

    const newArea = (newMaxX - newMinX) * (newMaxY - newMinY);

    return newArea - currentArea;
  }

  /**
   * Calculate bounds for a set of items
   */
  private calculateBounds(items: any[]): Bounds {
    if (items.length === 0) {
      return { minX: 0, minY: 0, maxX: 0, maxY: 0 };
    }

    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;

    for (const item of items) {
      const bounds = item.bounds || item;
      minX = Math.min(minX, bounds.minX);
      minY = Math.min(minY, bounds.minY);
      maxX = Math.max(maxX, bounds.maxX);
      maxY = Math.max(maxY, bounds.maxY);
    }

    return { minX, minY, maxX, maxY };
  }

  /**
   * Check if bounds intersect
   */
  private boundsIntersect(bounds1: Bounds, bounds2: Bounds): boolean {
    return !(
      bounds1.maxX < bounds2.minX ||
      bounds1.minX > bounds2.maxX ||
      bounds1.maxY < bounds2.minY ||
      bounds1.minY > bounds2.maxY
    );
  }

  /**
   * Get index statistics
   */
  getStats(): {
    nodeCount: number;
    featureCount: number;
    depth: number;
  } {
    if (!this.tree) {
      return { nodeCount: 0, featureCount: 0, depth: 0 };
    }

    return {
      nodeCount: this.countNodes(this.tree),
      featureCount: this.all().length,
      depth: this.calculateDepth(this.tree),
    };
  }

  /**
   * Count nodes in tree
   */
  private countNodes(node: RTreeNode): number {
    if (node.leaf) {
      return 1;
    }

    let count = 1;
    for (const child of node.children as RTreeNode[]) {
      count += this.countNodes(child);
    }

    return count;
  }

  /**
   * Calculate tree depth
   */
  private calculateDepth(node: RTreeNode): number {
    if (node.leaf) {
      return 1;
    }

    let maxDepth = 0;
    for (const child of node.children as RTreeNode[]) {
      maxDepth = Math.max(maxDepth, this.calculateDepth(child));
    }

    return maxDepth + 1;
  }
}

interface RTreeNode {
  children: (RTreeNode | IndexedFeature)[];
  leaf: boolean;
  bounds: Bounds;
}
