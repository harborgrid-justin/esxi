/**
 * Cluster Analysis
 * Spatial clustering algorithms (DBSCAN, K-means, Hierarchical)
 */

import {
  Position,
  Feature,
  ClusterOptions,
} from '../types';
import { GeometryFactory } from '../geometry/GeometryFactory';

export interface Cluster {
  id: number;
  points: Position[];
  centroid: Position;
  features?: Feature[];
}

export class ClusterAnalysis {
  /**
   * Perform clustering based on algorithm
   */
  static cluster(
    points: Position[],
    options: ClusterOptions
  ): Cluster[] {
    switch (options.algorithm) {
      case 'dbscan':
        return this.dbscan(
          points,
          options.epsilon || 100,
          options.minPoints || 3,
          options.distanceMetric || 'euclidean'
        );
      case 'kmeans':
        return this.kmeans(
          points,
          options.k || 5,
          options.distanceMetric || 'euclidean'
        );
      case 'hierarchical':
        return this.hierarchical(
          points,
          options.k || 5,
          options.distanceMetric || 'euclidean'
        );
      default:
        throw new Error(`Unknown clustering algorithm: ${options.algorithm}`);
    }
  }

  /**
   * DBSCAN (Density-Based Spatial Clustering of Applications with Noise)
   */
  static dbscan(
    points: Position[],
    epsilon: number,
    minPoints: number,
    metric: 'euclidean' | 'manhattan' | 'haversine' = 'euclidean'
  ): Cluster[] {
    const n = points.length;
    const labels = new Array(n).fill(-1); // -1 = noise
    let clusterId = 0;

    const distance = (p1: Position, p2: Position): number => {
      return this.calculateDistance(p1, p2, metric);
    };

    const rangeQuery = (pointIdx: number): number[] => {
      const neighbors: number[] = [];
      for (let i = 0; i < n; i++) {
        if (distance(points[pointIdx], points[i]) <= epsilon) {
          neighbors.push(i);
        }
      }
      return neighbors;
    };

    for (let i = 0; i < n; i++) {
      if (labels[i] !== -1) continue;

      const neighbors = rangeQuery(i);

      if (neighbors.length < minPoints) {
        labels[i] = -2; // Noise
        continue;
      }

      labels[i] = clusterId;
      const seeds = [...neighbors];

      while (seeds.length > 0) {
        const current = seeds.shift()!;

        if (labels[current] === -2) {
          labels[current] = clusterId;
        }

        if (labels[current] !== -1) continue;

        labels[current] = clusterId;
        const currentNeighbors = rangeQuery(current);

        if (currentNeighbors.length >= minPoints) {
          seeds.push(...currentNeighbors);
        }
      }

      clusterId++;
    }

    // Convert labels to clusters
    const clusters: Cluster[] = [];
    for (let i = 0; i < clusterId; i++) {
      const clusterPoints = points.filter((_, idx) => labels[idx] === i);
      if (clusterPoints.length > 0) {
        clusters.push({
          id: i,
          points: clusterPoints,
          centroid: this.calculateCentroid(clusterPoints),
        });
      }
    }

    return clusters;
  }

  /**
   * K-means clustering
   */
  static kmeans(
    points: Position[],
    k: number,
    metric: 'euclidean' | 'manhattan' | 'haversine' = 'euclidean',
    maxIterations = 100
  ): Cluster[] {
    const n = points.length;

    // Initialize centroids randomly
    let centroids = this.initializeCentroids(points, k);
    let labels = new Array(n).fill(0);
    let converged = false;
    let iteration = 0;

    const distance = (p1: Position, p2: Position): number => {
      return this.calculateDistance(p1, p2, metric);
    };

    while (!converged && iteration < maxIterations) {
      // Assignment step: assign each point to nearest centroid
      const newLabels = points.map((point) => {
        let minDist = Infinity;
        let nearestCluster = 0;

        for (let i = 0; i < k; i++) {
          const dist = distance(point, centroids[i]);
          if (dist < minDist) {
            minDist = dist;
            nearestCluster = i;
          }
        }

        return nearestCluster;
      });

      // Update step: recalculate centroids
      const newCentroids: Position[] = [];
      for (let i = 0; i < k; i++) {
        const clusterPoints = points.filter((_, idx) => newLabels[idx] === i);
        if (clusterPoints.length > 0) {
          newCentroids.push(this.calculateCentroid(clusterPoints));
        } else {
          newCentroids.push(centroids[i]); // Keep old centroid if empty
        }
      }

      // Check convergence
      converged = centroids.every((centroid, i) =>
        this.positionsEqual(centroid, newCentroids[i])
      );

      centroids = newCentroids;
      labels = newLabels;
      iteration++;
    }

    // Convert to clusters
    const clusters: Cluster[] = [];
    for (let i = 0; i < k; i++) {
      const clusterPoints = points.filter((_, idx) => labels[idx] === i);
      if (clusterPoints.length > 0) {
        clusters.push({
          id: i,
          points: clusterPoints,
          centroid: centroids[i],
        });
      }
    }

    return clusters;
  }

  /**
   * Hierarchical clustering (agglomerative)
   */
  static hierarchical(
    points: Position[],
    k: number,
    metric: 'euclidean' | 'manhattan' | 'haversine' = 'euclidean'
  ): Cluster[] {
    const n = points.length;

    // Start with each point as its own cluster
    let clusters: Position[][] = points.map((p) => [p]);

    const distance = (p1: Position, p2: Position): number => {
      return this.calculateDistance(p1, p2, metric);
    };

    // Merge clusters until we have k clusters
    while (clusters.length > k) {
      let minDist = Infinity;
      let mergeI = 0;
      let mergeJ = 1;

      // Find closest pair of clusters
      for (let i = 0; i < clusters.length; i++) {
        for (let j = i + 1; j < clusters.length; j++) {
          const dist = this.clusterDistance(clusters[i], clusters[j], distance);
          if (dist < minDist) {
            minDist = dist;
            mergeI = i;
            mergeJ = j;
          }
        }
      }

      // Merge the closest clusters
      const merged = [...clusters[mergeI], ...clusters[mergeJ]];
      clusters = clusters.filter((_, idx) => idx !== mergeI && idx !== mergeJ);
      clusters.push(merged);
    }

    // Convert to Cluster objects
    return clusters.map((clusterPoints, i) => ({
      id: i,
      points: clusterPoints,
      centroid: this.calculateCentroid(clusterPoints),
    }));
  }

  /**
   * Calculate distance between two clusters (average linkage)
   */
  private static clusterDistance(
    cluster1: Position[],
    cluster2: Position[],
    distanceFn: (p1: Position, p2: Position) => number
  ): number {
    let sum = 0;
    let count = 0;

    for (const p1 of cluster1) {
      for (const p2 of cluster2) {
        sum += distanceFn(p1, p2);
        count++;
      }
    }

    return sum / count;
  }

  /**
   * Initialize K-means centroids using k-means++
   */
  private static initializeCentroids(points: Position[], k: number): Position[] {
    const centroids: Position[] = [];

    // Choose first centroid randomly
    const firstIdx = Math.floor(Math.random() * points.length);
    centroids.push(points[firstIdx]);

    // Choose remaining centroids
    for (let i = 1; i < k; i++) {
      const distances = points.map((point) => {
        let minDist = Infinity;
        for (const centroid of centroids) {
          const dist = GeometryFactory.distance(point, centroid);
          minDist = Math.min(minDist, dist);
        }
        return minDist;
      });

      // Choose point with probability proportional to distance squared
      const sumDistances = distances.reduce((a, b) => a + b * b, 0);
      let rand = Math.random() * sumDistances;

      for (let j = 0; j < points.length; j++) {
        rand -= distances[j] * distances[j];
        if (rand <= 0) {
          centroids.push(points[j]);
          break;
        }
      }
    }

    return centroids;
  }

  /**
   * Calculate centroid of points
   */
  private static calculateCentroid(points: Position[]): Position {
    const n = points.length;
    const sum = points.reduce(
      (acc, p) => [acc[0] + p[0], acc[1] + p[1]],
      [0, 0]
    );
    return [sum[0] / n, sum[1] / n];
  }

  /**
   * Calculate distance based on metric
   */
  private static calculateDistance(
    p1: Position,
    p2: Position,
    metric: 'euclidean' | 'manhattan' | 'haversine'
  ): number {
    switch (metric) {
      case 'euclidean':
        return GeometryFactory.distance(p1, p2);
      case 'manhattan':
        return Math.abs(p1[0] - p2[0]) + Math.abs(p1[1] - p2[1]);
      case 'haversine':
        return GeometryFactory.haversineDistance(p1, p2);
      default:
        return GeometryFactory.distance(p1, p2);
    }
  }

  /**
   * Check if positions are equal
   */
  private static positionsEqual(p1: Position, p2: Position, tolerance = 1e-6): boolean {
    return (
      Math.abs(p1[0] - p2[0]) < tolerance && Math.abs(p1[1] - p2[1]) < tolerance
    );
  }

  /**
   * Calculate silhouette score for cluster quality
   */
  static silhouetteScore(
    points: Position[],
    clusters: Cluster[],
    metric: 'euclidean' | 'manhattan' | 'haversine' = 'euclidean'
  ): number {
    const distance = (p1: Position, p2: Position): number => {
      return this.calculateDistance(p1, p2, metric);
    };

    const pointToCluster = new Map<Position, number>();
    clusters.forEach((cluster, i) => {
      cluster.points.forEach((point) => pointToCluster.set(point, i));
    });

    let totalScore = 0;

    for (const point of points) {
      const clusterIdx = pointToCluster.get(point)!;
      const cluster = clusters[clusterIdx];

      // Calculate a: average distance to points in same cluster
      let a = 0;
      if (cluster.points.length > 1) {
        for (const other of cluster.points) {
          if (other !== point) {
            a += distance(point, other);
          }
        }
        a /= cluster.points.length - 1;
      }

      // Calculate b: min average distance to points in other clusters
      let b = Infinity;
      for (let i = 0; i < clusters.length; i++) {
        if (i === clusterIdx) continue;

        let avgDist = 0;
        for (const other of clusters[i].points) {
          avgDist += distance(point, other);
        }
        avgDist /= clusters[i].points.length;

        b = Math.min(b, avgDist);
      }

      const s = (b - a) / Math.max(a, b);
      totalScore += s;
    }

    return totalScore / points.length;
  }

  /**
   * Cluster features based on attributes
   */
  static clusterFeatures(
    features: Feature[],
    options: ClusterOptions & { attributes?: string[] }
  ): Cluster[] {
    // Extract positions from features
    const points = features.map((f) =>
      GeometryFactory.getCentroid(f.geometry)
    );

    // Perform clustering
    const clusters = this.cluster(points, options);

    // Assign features to clusters
    clusters.forEach((cluster) => {
      cluster.features = [];
      for (let i = 0; i < features.length; i++) {
        const point = points[i];
        if (cluster.points.some((p) => this.positionsEqual(p, point))) {
          cluster.features.push(features[i]);
        }
      }
    });

    return clusters;
  }
}
