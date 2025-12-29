//! Many-to-many distance matrix computation

use crate::error::Result;
use crate::graph::{Graph, NodeId};
use crate::profile::RoutingProfile;
use geo_types::Point;
use rayon::prelude::*;

/// Calculate distance matrix between sources and targets
pub fn calculate_matrix(
    graph: &Graph,
    sources: &[Point],
    targets: &[Point],
    profile: &RoutingProfile,
) -> Result<Vec<Vec<f64>>> {
    // Map points to nodes
    let source_nodes: Vec<_> = sources
        .iter()
        .filter_map(|&p| graph.nearest_node(p))
        .collect();

    let target_nodes: Vec<_> = targets
        .iter()
        .filter_map(|&p| graph.nearest_node(p))
        .collect();

    // Parallel computation of rows
    let matrix: Vec<Vec<f64>> = source_nodes
        .par_iter()
        .map(|&source| {
            compute_row(graph, source, &target_nodes)
        })
        .collect();

    Ok(matrix)
}

/// Compute one row of distance matrix (from one source to all targets)
fn compute_row(graph: &Graph, source: NodeId, targets: &[NodeId]) -> Vec<f64> {
    use crate::algorithms::dijkstra::DijkstraRouter;

    let router = DijkstraRouter::new();
    let distances = router.compute_many_to_one(graph, source, targets);

    targets
        .iter()
        .map(|&target| distances.get(&target).copied().unwrap_or(f64::INFINITY))
        .collect()
}

/// Calculate symmetric distance matrix (when sources == targets)
pub fn calculate_symmetric_matrix(
    graph: &Graph,
    points: &[Point],
    profile: &RoutingProfile,
) -> Result<Vec<Vec<f64>>> {
    let nodes: Vec<_> = points
        .iter()
        .filter_map(|&p| graph.nearest_node(p))
        .collect();

    let n = nodes.len();
    let mut matrix = vec![vec![0.0; n]; n];

    // Compute upper triangle in parallel
    let rows: Vec<(usize, Vec<f64>)> = (0..n)
        .into_par_iter()
        .map(|i| {
            let source = nodes[i];
            let targets = &nodes[i..];
            let distances = compute_row(graph, source, targets);
            (i, distances)
        })
        .collect();

    // Fill matrix symmetrically
    for (i, row) in rows {
        for (j, &dist) in row.iter().enumerate() {
            let col = i + j;
            matrix[i][col] = dist;
            matrix[col][i] = dist;
        }
    }

    Ok(matrix)
}

/// Calculate distance matrix with custom cost function
pub fn calculate_matrix_with_cost<F>(
    graph: &Graph,
    sources: &[Point],
    targets: &[Point],
    cost_fn: F,
) -> Result<Vec<Vec<f64>>>
where
    F: Fn(NodeId, NodeId, &Graph) -> Option<f64> + Sync,
{
    let source_nodes: Vec<_> = sources
        .iter()
        .filter_map(|&p| graph.nearest_node(p))
        .collect();

    let target_nodes: Vec<_> = targets
        .iter()
        .filter_map(|&p| graph.nearest_node(p))
        .collect();

    let matrix: Vec<Vec<f64>> = source_nodes
        .par_iter()
        .map(|&source| {
            target_nodes
                .iter()
                .map(|&target| cost_fn(source, target, graph).unwrap_or(f64::INFINITY))
                .collect()
        })
        .collect();

    Ok(matrix)
}

/// Optimized many-to-many using Contraction Hierarchies (if available)
pub fn calculate_matrix_ch(
    graph: &Graph,
    ch: &crate::algorithms::ContractionHierarchies,
    sources: &[Point],
    targets: &[Point],
) -> Result<Vec<Vec<f64>>> {
    let source_nodes: Vec<_> = sources
        .iter()
        .filter_map(|&p| graph.nearest_node(p))
        .collect();

    let target_nodes: Vec<_> = targets
        .iter()
        .filter_map(|&p| graph.nearest_node(p))
        .collect();

    let matrix: Vec<Vec<f64>> = source_nodes
        .par_iter()
        .map(|&source| {
            target_nodes
                .iter()
                .map(|&target| {
                    ch.query(source, target)
                        .map(|r| r.distance)
                        .unwrap_or(f64::INFINITY)
                })
                .collect()
        })
        .collect();

    Ok(matrix)
}

/// Table routing: compute distances for all pairs efficiently
pub fn table_routing(
    graph: &Graph,
    points: &[Point],
) -> Result<Vec<Vec<f64>>> {
    let nodes: Vec<_> = points
        .iter()
        .filter_map(|&p| graph.nearest_node(p))
        .collect();

    let n = nodes.len();

    // For small sets, use symmetric matrix computation
    if n < 100 {
        return calculate_symmetric_matrix(graph, points, &RoutingProfile::driving());
    }

    // For larger sets, use parallel one-to-all computation
    let rows: Vec<Vec<f64>> = nodes
        .par_iter()
        .map(|&source| {
            let distances = compute_row(graph, source, &nodes);
            distances
        })
        .collect();

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    #[test]
    fn test_distance_matrix() {
        let graph = GraphBuilder::create_grid(5, 5, 0.01).unwrap();

        let sources = vec![
            Point::new(0.0, 0.0),
            Point::new(0.02, 0.02),
        ];

        let targets = vec![
            Point::new(0.04, 0.04),
            Point::new(0.01, 0.01),
        ];

        let profile = RoutingProfile::driving();
        let matrix = calculate_matrix(&graph, &sources, &targets, &profile).unwrap();

        assert_eq!(matrix.len(), 2);
        assert_eq!(matrix[0].len(), 2);
    }

    #[test]
    fn test_symmetric_matrix() {
        let graph = GraphBuilder::create_grid(5, 5, 0.01).unwrap();

        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(0.02, 0.02),
            Point::new(0.04, 0.04),
        ];

        let profile = RoutingProfile::driving();
        let matrix = calculate_symmetric_matrix(&graph, &points, &profile).unwrap();

        assert_eq!(matrix.len(), 3);
        // Check symmetry
        for i in 0..3 {
            for j in 0..3 {
                assert!((matrix[i][j] - matrix[j][i]).abs() < 1e-6);
            }
        }
    }
}
