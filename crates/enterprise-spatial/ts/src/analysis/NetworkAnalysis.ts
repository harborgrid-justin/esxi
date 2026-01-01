/**
 * Network Analysis
 * Routing, shortest path, and network analysis operations
 */

import {
  Position,
  NetworkAnalysisOptions,
  Geometry,
} from '../types';
import { GeometryFactory } from '../geometry/GeometryFactory';

export interface Network {
  nodes: NetworkNode[];
  edges: NetworkEdge[];
}

export interface NetworkNode {
  id: string;
  position: Position;
  properties?: Record<string, any>;
}

export interface NetworkEdge {
  id: string;
  from: string;
  to: string;
  cost: number;
  length?: number;
  geometry?: Position[];
  properties?: Record<string, any>;
}

export interface Route {
  path: string[]; // Node IDs
  cost: number;
  distance: number;
  geometry: Position[];
}

export class NetworkAnalysis {
  /**
   * Find shortest path between two nodes
   */
  static shortestPath(
    network: Network,
    startNodeId: string,
    endNodeId: string,
    options: NetworkAnalysisOptions = { algorithm: 'dijkstra' }
  ): Route | null {
    switch (options.algorithm) {
      case 'dijkstra':
        return this.dijkstra(network, startNodeId, endNodeId, options);
      case 'astar':
        return this.astar(network, startNodeId, endNodeId, options);
      case 'bellman-ford':
        return this.bellmanFord(network, startNodeId, endNodeId, options);
      default:
        return this.dijkstra(network, startNodeId, endNodeId, options);
    }
  }

  /**
   * Dijkstra's algorithm
   */
  private static dijkstra(
    network: Network,
    startId: string,
    endId: string,
    options: NetworkAnalysisOptions
  ): Route | null {
    const distances = new Map<string, number>();
    const previous = new Map<string, string>();
    const unvisited = new Set<string>();

    // Initialize
    network.nodes.forEach((node) => {
      distances.set(node.id, node.id === startId ? 0 : Infinity);
      unvisited.add(node.id);
    });

    while (unvisited.size > 0) {
      // Find node with minimum distance
      let current: string | null = null;
      let minDist = Infinity;

      for (const nodeId of unvisited) {
        const dist = distances.get(nodeId)!;
        if (dist < minDist) {
          minDist = dist;
          current = nodeId;
        }
      }

      if (current === null || minDist === Infinity) break;

      unvisited.delete(current);

      if (current === endId) break;

      // Update distances to neighbors
      const neighbors = this.getNeighbors(network, current);

      for (const { nodeId, edge } of neighbors) {
        if (!unvisited.has(nodeId)) continue;

        const alt = distances.get(current)! + this.getEdgeCost(edge, options);

        if (alt < distances.get(nodeId)!) {
          distances.set(nodeId, alt);
          previous.set(nodeId, current);
        }
      }
    }

    // Reconstruct path
    return this.reconstructPath(network, previous, startId, endId, distances);
  }

  /**
   * A* algorithm
   */
  private static astar(
    network: Network,
    startId: string,
    endId: string,
    options: NetworkAnalysisOptions
  ): Route | null {
    const endNode = network.nodes.find((n) => n.id === endId);
    if (!endNode) return null;

    const gScore = new Map<string, number>();
    const fScore = new Map<string, number>();
    const previous = new Map<string, string>();
    const openSet = new Set<string>([startId]);

    network.nodes.forEach((node) => {
      gScore.set(node.id, node.id === startId ? 0 : Infinity);
      fScore.set(
        node.id,
        node.id === startId
          ? this.heuristic(node.position, endNode.position)
          : Infinity
      );
    });

    while (openSet.size > 0) {
      // Find node with lowest fScore
      let current: string | null = null;
      let minF = Infinity;

      for (const nodeId of openSet) {
        const f = fScore.get(nodeId)!;
        if (f < minF) {
          minF = f;
          current = nodeId;
        }
      }

      if (current === null) break;

      if (current === endId) {
        return this.reconstructPath(network, previous, startId, endId, gScore);
      }

      openSet.delete(current);

      const neighbors = this.getNeighbors(network, current);

      for (const { nodeId, edge } of neighbors) {
        const tentativeG = gScore.get(current)! + this.getEdgeCost(edge, options);

        if (tentativeG < gScore.get(nodeId)!) {
          previous.set(nodeId, current);
          gScore.set(nodeId, tentativeG);

          const node = network.nodes.find((n) => n.id === nodeId)!;
          fScore.set(
            nodeId,
            tentativeG + this.heuristic(node.position, endNode.position)
          );

          openSet.add(nodeId);
        }
      }
    }

    return null;
  }

  /**
   * Bellman-Ford algorithm (handles negative weights)
   */
  private static bellmanFord(
    network: Network,
    startId: string,
    endId: string,
    options: NetworkAnalysisOptions
  ): Route | null {
    const distances = new Map<string, number>();
    const previous = new Map<string, string>();

    // Initialize
    network.nodes.forEach((node) => {
      distances.set(node.id, node.id === startId ? 0 : Infinity);
    });

    // Relax edges repeatedly
    for (let i = 0; i < network.nodes.length - 1; i++) {
      for (const edge of network.edges) {
        const dist = distances.get(edge.from)!;
        if (dist === Infinity) continue;

        const alt = dist + this.getEdgeCost(edge, options);

        if (alt < distances.get(edge.to)!) {
          distances.set(edge.to, alt);
          previous.set(edge.to, edge.from);
        }
      }
    }

    // Check for negative cycles
    for (const edge of network.edges) {
      const dist = distances.get(edge.from)!;
      if (dist === Infinity) continue;

      const alt = dist + this.getEdgeCost(edge, options);

      if (alt < distances.get(edge.to)!) {
        throw new Error('Network contains a negative cycle');
      }
    }

    return this.reconstructPath(network, previous, startId, endId, distances);
  }

  /**
   * Get neighbors of a node
   */
  private static getNeighbors(
    network: Network,
    nodeId: string
  ): Array<{ nodeId: string; edge: NetworkEdge }> {
    const neighbors: Array<{ nodeId: string; edge: NetworkEdge }> = [];

    for (const edge of network.edges) {
      if (edge.from === nodeId) {
        neighbors.push({ nodeId: edge.to, edge });
      }
    }

    return neighbors;
  }

  /**
   * Get edge cost considering impedance field
   */
  private static getEdgeCost(
    edge: NetworkEdge,
    options: NetworkAnalysisOptions
  ): number {
    if (options.impedance && edge.properties) {
      return edge.properties[options.impedance] || edge.cost;
    }
    return edge.cost;
  }

  /**
   * Heuristic function for A* (Euclidean distance)
   */
  private static heuristic(pos1: Position, pos2: Position): number {
    return GeometryFactory.distance(pos1, pos2);
  }

  /**
   * Reconstruct path from previous map
   */
  private static reconstructPath(
    network: Network,
    previous: Map<string, string>,
    startId: string,
    endId: string,
    distances: Map<string, number>
  ): Route | null {
    if (distances.get(endId) === Infinity) {
      return null;
    }

    const path: string[] = [];
    let current = endId;

    while (current !== startId) {
      path.unshift(current);
      const prev = previous.get(current);
      if (!prev) return null;
      current = prev;
    }

    path.unshift(startId);

    // Build geometry
    const geometry: Position[] = [];
    let totalDistance = 0;

    for (let i = 0; i < path.length - 1; i++) {
      const edge = network.edges.find(
        (e) => e.from === path[i] && e.to === path[i + 1]
      );

      if (edge?.geometry) {
        geometry.push(...edge.geometry);
        totalDistance += edge.length || 0;
      } else {
        const fromNode = network.nodes.find((n) => n.id === path[i])!;
        const toNode = network.nodes.find((n) => n.id === path[i + 1])!;
        geometry.push(fromNode.position, toNode.position);
        totalDistance += GeometryFactory.distance(
          fromNode.position,
          toNode.position
        );
      }
    }

    return {
      path,
      cost: distances.get(endId)!,
      distance: totalDistance,
      geometry,
    };
  }

  /**
   * Calculate service area (all nodes reachable within cost threshold)
   */
  static serviceArea(
    network: Network,
    startNodeId: string,
    maxCost: number,
    options: NetworkAnalysisOptions = { algorithm: 'dijkstra' }
  ): string[] {
    const distances = new Map<string, number>();
    const unvisited = new Set<string>();

    network.nodes.forEach((node) => {
      distances.set(node.id, node.id === startNodeId ? 0 : Infinity);
      unvisited.add(node.id);
    });

    while (unvisited.size > 0) {
      let current: string | null = null;
      let minDist = Infinity;

      for (const nodeId of unvisited) {
        const dist = distances.get(nodeId)!;
        if (dist < minDist) {
          minDist = dist;
          current = nodeId;
        }
      }

      if (current === null || minDist > maxCost) break;

      unvisited.delete(current);

      const neighbors = this.getNeighbors(network, current);

      for (const { nodeId, edge } of neighbors) {
        if (!unvisited.has(nodeId)) continue;

        const alt = distances.get(current)! + this.getEdgeCost(edge, options);

        if (alt < distances.get(nodeId)!) {
          distances.set(nodeId, alt);
        }
      }
    }

    // Return all nodes within maxCost
    return Array.from(distances.entries())
      .filter(([_, dist]) => dist <= maxCost)
      .map(([nodeId, _]) => nodeId);
  }

  /**
   * Find all paths between two nodes
   */
  static allPaths(
    network: Network,
    startId: string,
    endId: string,
    maxPaths = 10
  ): Route[] {
    const paths: Route[] = [];
    const visited = new Set<string>();

    const dfs = (current: string, path: string[], cost: number): void => {
      if (paths.length >= maxPaths) return;

      if (current === endId) {
        const distances = new Map<string, number>();
        path.forEach((id, i) => distances.set(id, i === 0 ? 0 : cost));

        const route = this.reconstructPath(
          network,
          new Map(path.slice(0, -1).map((id, i) => [path[i + 1], id])),
          startId,
          endId,
          distances
        );

        if (route) paths.push(route);
        return;
      }

      visited.add(current);

      const neighbors = this.getNeighbors(network, current);

      for (const { nodeId, edge } of neighbors) {
        if (!visited.has(nodeId)) {
          dfs(nodeId, [...path, nodeId], cost + edge.cost);
        }
      }

      visited.delete(current);
    };

    dfs(startId, [startId], 0);

    return paths.sort((a, b) => a.cost - b.cost);
  }

  /**
   * Traveling Salesman Problem (nearest neighbor heuristic)
   */
  static tsp(network: Network, nodeIds: string[]): Route | null {
    if (nodeIds.length < 2) return null;

    const visited = new Set<string>();
    const path: string[] = [nodeIds[0]];
    visited.add(nodeIds[0]);

    let totalCost = 0;

    while (visited.size < nodeIds.length) {
      const current = path[path.length - 1];
      let nearest: string | null = null;
      let minCost = Infinity;

      for (const nodeId of nodeIds) {
        if (visited.has(nodeId)) continue;

        const route = this.shortestPath(network, current, nodeId);
        if (route && route.cost < minCost) {
          minCost = route.cost;
          nearest = nodeId;
        }
      }

      if (nearest === null) break;

      path.push(nearest);
      visited.add(nearest);
      totalCost += minCost;
    }

    // Return to start
    const returnRoute = this.shortestPath(
      network,
      path[path.length - 1],
      path[0]
    );
    if (returnRoute) {
      path.push(path[0]);
      totalCost += returnRoute.cost;
    }

    const distances = new Map<string, number>();
    distances.set(path[path.length - 1], totalCost);

    return this.reconstructPath(
      network,
      new Map(path.slice(0, -1).map((id, i) => [path[i + 1], id])),
      path[0],
      path[path.length - 1],
      distances
    );
  }
}
