//! Plugin dependency resolution and management.

use std::collections::{HashMap, HashSet, VecDeque};

use semver::Version;

use crate::error::{PluginError, PluginResult};
use crate::traits::{PluginDependency, PluginMetadata};

/// Dependency resolver for plugins.
#[derive(Debug)]
pub struct DependencyResolver {
    /// Map of plugin ID to metadata.
    plugins: HashMap<String, PluginMetadata>,

    /// Map of plugin ID to its resolved dependencies.
    dependency_graph: HashMap<String, Vec<String>>,
}

impl DependencyResolver {
    /// Create a new dependency resolver.
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            dependency_graph: HashMap::new(),
        }
    }

    /// Register a plugin's metadata.
    pub fn register(&mut self, metadata: PluginMetadata) {
        self.plugins.insert(metadata.id.clone(), metadata);
    }

    /// Unregister a plugin.
    pub fn unregister(&mut self, plugin_id: &str) {
        self.plugins.remove(plugin_id);
        self.dependency_graph.remove(plugin_id);
    }

    /// Resolve dependencies for a plugin.
    pub fn resolve(&mut self, plugin_id: &str) -> PluginResult<Vec<String>> {
        let metadata = self.plugins.get(plugin_id).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_id.to_string(),
            }
        })?;

        // Check for circular dependencies
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        self.detect_cycles(plugin_id, &mut visited, &mut stack)?;

        // Build dependency list
        let mut dependencies = Vec::new();
        let mut to_process = VecDeque::new();
        let mut processed = HashSet::new();

        to_process.push_back(plugin_id.to_string());

        while let Some(current_id) = to_process.pop_front() {
            if processed.contains(&current_id) {
                continue;
            }

            let current_metadata = self.plugins.get(&current_id).ok_or_else(|| {
                PluginError::DependencyError {
                    id: plugin_id.to_string(),
                    reason: format!("Dependency '{}' not found", current_id),
                }
            })?;

            for dep in &current_metadata.dependencies {
                // Check if dependency exists
                let dep_metadata = self.plugins.get(&dep.id).ok_or_else(|| {
                    if dep.optional {
                        // Optional dependency missing is okay
                        return PluginError::DependencyError {
                            id: plugin_id.to_string(),
                            reason: format!(
                                "Optional dependency '{}' not found (skipping)",
                                dep.id
                            ),
                        };
                    } else {
                        PluginError::DependencyError {
                            id: plugin_id.to_string(),
                            reason: format!("Required dependency '{}' not found", dep.id),
                        }
                    }
                });

                if let Ok(dep_metadata) = dep_metadata {
                    // Check version compatibility
                    if !dep.version.matches(&dep_metadata.version) {
                        return Err(PluginError::DependencyError {
                            id: plugin_id.to_string(),
                            reason: format!(
                                "Dependency '{}' version {} does not satisfy requirement {}",
                                dep.id, dep_metadata.version, dep.version
                            ),
                        });
                    }

                    if !processed.contains(&dep.id) {
                        to_process.push_back(dep.id.clone());
                        dependencies.push(dep.id.clone());
                    }
                }
            }

            processed.insert(current_id);
        }

        // Store the resolved dependencies
        self.dependency_graph
            .insert(plugin_id.to_string(), dependencies.clone());

        Ok(dependencies)
    }

    /// Get the load order for plugins (topological sort).
    pub fn get_load_order(&self, plugin_ids: &[String]) -> PluginResult<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize
        for plugin_id in plugin_ids {
            in_degree.insert(plugin_id.clone(), 0);
            adj_list.insert(plugin_id.clone(), Vec::new());
        }

        // Build adjacency list and in-degree count
        for plugin_id in plugin_ids {
            if let Some(metadata) = self.plugins.get(plugin_id) {
                for dep in &metadata.dependencies {
                    if plugin_ids.contains(&dep.id) {
                        adj_list
                            .entry(dep.id.clone())
                            .or_default()
                            .push(plugin_id.clone());
                        *in_degree.entry(plugin_id.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        // Kahn's algorithm for topological sort
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut load_order = Vec::new();

        while let Some(plugin_id) = queue.pop_front() {
            load_order.push(plugin_id.clone());

            if let Some(dependents) = adj_list.get(&plugin_id) {
                for dependent in dependents {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }

        // Check if all plugins were processed (no cycles)
        if load_order.len() != plugin_ids.len() {
            return Err(PluginError::DependencyError {
                id: "multiple".to_string(),
                reason: "Circular dependency detected in plugin set".to_string(),
            });
        }

        Ok(load_order)
    }

    /// Detect circular dependencies using DFS.
    fn detect_cycles(
        &self,
        plugin_id: &str,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
    ) -> PluginResult<()> {
        visited.insert(plugin_id.to_string());
        stack.insert(plugin_id.to_string());

        if let Some(metadata) = self.plugins.get(plugin_id) {
            for dep in &metadata.dependencies {
                if !visited.contains(&dep.id) {
                    self.detect_cycles(&dep.id, visited, stack)?;
                } else if stack.contains(&dep.id) {
                    return Err(PluginError::DependencyError {
                        id: plugin_id.to_string(),
                        reason: format!("Circular dependency detected with '{}'", dep.id),
                    });
                }
            }
        }

        stack.remove(plugin_id);
        Ok(())
    }

    /// Get all plugins that depend on a given plugin.
    pub fn get_dependents(&self, plugin_id: &str) -> Vec<String> {
        let mut dependents = Vec::new();

        for (id, metadata) in &self.plugins {
            if metadata
                .dependencies
                .iter()
                .any(|dep| dep.id == plugin_id)
            {
                dependents.push(id.clone());
            }
        }

        dependents
    }

    /// Check if a plugin can be safely unloaded.
    pub fn can_unload(&self, plugin_id: &str, loaded_plugins: &HashSet<String>) -> bool {
        let dependents = self.get_dependents(plugin_id);

        // Can unload if no loaded plugins depend on it
        !dependents.iter().any(|dep| loaded_plugins.contains(dep))
    }

    /// Get dependency tree for a plugin.
    pub fn get_dependency_tree(&self, plugin_id: &str) -> PluginResult<DependencyTree> {
        let metadata = self.plugins.get(plugin_id).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_id.to_string(),
            }
        })?;

        let mut children = Vec::new();
        for dep in &metadata.dependencies {
            if let Ok(dep_tree) = self.get_dependency_tree(&dep.id) {
                children.push(dep_tree);
            }
        }

        Ok(DependencyTree {
            plugin_id: plugin_id.to_string(),
            version: metadata.version.clone(),
            children,
        })
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Dependency tree representation.
#[derive(Debug, Clone)]
pub struct DependencyTree {
    pub plugin_id: String,
    pub version: Version,
    pub children: Vec<DependencyTree>,
}

impl DependencyTree {
    /// Pretty-print the dependency tree.
    pub fn format(&self, indent: usize) -> String {
        let mut result = format!(
            "{}{} v{}\n",
            "  ".repeat(indent),
            self.plugin_id,
            self.version
        );

        for child in &self.children {
            result.push_str(&child.format(indent + 1));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::VersionReq;

    fn create_test_metadata(
        id: &str,
        version: &str,
        deps: Vec<(&str, &str)>,
    ) -> PluginMetadata {
        PluginMetadata {
            id: id.to_string(),
            name: id.to_string(),
            version: Version::parse(version).unwrap(),
            description: String::new(),
            authors: vec![],
            license: None,
            homepage: None,
            dependencies: deps
                .into_iter()
                .map(|(id, version)| PluginDependency {
                    id: id.to_string(),
                    version: VersionReq::parse(version).unwrap(),
                    optional: false,
                })
                .collect(),
            min_platform_version: Version::new(0, 1, 0),
            max_platform_version: None,
            capabilities: vec![],
            tags: vec![],
        }
    }

    #[test]
    fn test_simple_dependency_resolution() {
        let mut resolver = DependencyResolver::new();

        let plugin_a = create_test_metadata("a", "1.0.0", vec![]);
        let plugin_b = create_test_metadata("b", "1.0.0", vec![("a", "1.0.0")]);

        resolver.register(plugin_a);
        resolver.register(plugin_b);

        let deps = resolver.resolve("b").unwrap();
        assert_eq!(deps, vec!["a"]);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut resolver = DependencyResolver::new();

        let plugin_a = create_test_metadata("a", "1.0.0", vec![("b", "1.0.0")]);
        let plugin_b = create_test_metadata("b", "1.0.0", vec![("a", "1.0.0")]);

        resolver.register(plugin_a);
        resolver.register(plugin_b);

        assert!(resolver.resolve("a").is_err());
    }

    #[test]
    fn test_load_order() {
        let mut resolver = DependencyResolver::new();

        let plugin_a = create_test_metadata("a", "1.0.0", vec![]);
        let plugin_b = create_test_metadata("b", "1.0.0", vec![("a", "1.0.0")]);
        let plugin_c = create_test_metadata("c", "1.0.0", vec![("b", "1.0.0")]);

        resolver.register(plugin_a);
        resolver.register(plugin_b);
        resolver.register(plugin_c);

        let load_order = resolver
            .get_load_order(&["a".to_string(), "b".to_string(), "c".to_string()])
            .unwrap();

        assert_eq!(load_order, vec!["a", "b", "c"]);
    }
}
