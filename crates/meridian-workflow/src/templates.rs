//! Workflow templates and reusable components.

use crate::dag::{DependencyType, Task, WorkflowDag};
use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// Template parameter definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    /// Parameter name.
    pub name: String,

    /// Parameter type (string, number, boolean, object, array).
    pub param_type: String,

    /// Parameter description.
    pub description: Option<String>,

    /// Default value.
    pub default: Option<serde_json::Value>,

    /// Whether the parameter is required.
    pub required: bool,

    /// Validation rules.
    pub validation: Option<ParameterValidation>,
}

impl TemplateParameter {
    /// Creates a new template parameter.
    pub fn new(name: impl Into<String>, param_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: param_type.into(),
            description: None,
            default: None,
            required: true,
            validation: None,
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the default value.
    pub fn with_default(mut self, default: serde_json::Value) -> Self {
        self.default = Some(default);
        self.required = false;
        self
    }

    /// Sets whether the parameter is required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Validates a value against this parameter definition.
    pub fn validate(&self, value: &serde_json::Value) -> WorkflowResult<()> {
        // Type checking
        match (self.param_type.as_str(), value) {
            ("string", serde_json::Value::String(_)) => {}
            ("number", serde_json::Value::Number(_)) => {}
            ("boolean", serde_json::Value::Bool(_)) => {}
            ("object", serde_json::Value::Object(_)) => {}
            ("array", serde_json::Value::Array(_)) => {}
            ("any", _) => {}
            _ => {
                return Err(WorkflowError::TemplateRenderError(format!(
                    "Parameter '{}' has invalid type, expected {}",
                    self.name, self.param_type
                )));
            }
        }

        // Additional validation
        if let Some(ref validation) = self.validation {
            validation.validate(value)?;
        }

        Ok(())
    }
}

/// Parameter validation rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValidation {
    /// Minimum value (for numbers).
    pub min: Option<f64>,

    /// Maximum value (for numbers).
    pub max: Option<f64>,

    /// Minimum length (for strings/arrays).
    pub min_length: Option<usize>,

    /// Maximum length (for strings/arrays).
    pub max_length: Option<usize>,

    /// Pattern (regex for strings).
    pub pattern: Option<String>,

    /// Allowed values (enum).
    pub allowed_values: Option<Vec<serde_json::Value>>,
}

impl ParameterValidation {
    /// Validates a value against these rules.
    pub fn validate(&self, value: &serde_json::Value) -> WorkflowResult<()> {
        match value {
            serde_json::Value::Number(n) => {
                let num = n.as_f64().unwrap_or(0.0);
                if let Some(min) = self.min {
                    if num < min {
                        return Err(WorkflowError::TemplateRenderError(format!(
                            "Value {} is less than minimum {}",
                            num, min
                        )));
                    }
                }
                if let Some(max) = self.max {
                    if num > max {
                        return Err(WorkflowError::TemplateRenderError(format!(
                            "Value {} is greater than maximum {}",
                            num, max
                        )));
                    }
                }
            }
            serde_json::Value::String(s) => {
                if let Some(min_len) = self.min_length {
                    if s.len() < min_len {
                        return Err(WorkflowError::TemplateRenderError(format!(
                            "String length {} is less than minimum {}",
                            s.len(),
                            min_len
                        )));
                    }
                }
                if let Some(max_len) = self.max_length {
                    if s.len() > max_len {
                        return Err(WorkflowError::TemplateRenderError(format!(
                            "String length {} is greater than maximum {}",
                            s.len(),
                            max_len
                        )));
                    }
                }
            }
            serde_json::Value::Array(arr) => {
                if let Some(min_len) = self.min_length {
                    if arr.len() < min_len {
                        return Err(WorkflowError::TemplateRenderError(format!(
                            "Array length {} is less than minimum {}",
                            arr.len(),
                            min_len
                        )));
                    }
                }
                if let Some(max_len) = self.max_length {
                    if arr.len() > max_len {
                        return Err(WorkflowError::TemplateRenderError(format!(
                            "Array length {} is greater than maximum {}",
                            arr.len(),
                            max_len
                        )));
                    }
                }
            }
            _ => {}
        }

        // Check allowed values
        if let Some(ref allowed) = self.allowed_values {
            if !allowed.contains(value) {
                return Err(WorkflowError::TemplateRenderError(
                    "Value is not in allowed values list".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// A reusable workflow template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    /// Template ID.
    pub id: Uuid,

    /// Template name.
    pub name: String,

    /// Template description.
    pub description: Option<String>,

    /// Template version.
    pub version: String,

    /// Template author.
    pub author: Option<String>,

    /// Template parameters.
    pub parameters: Vec<TemplateParameter>,

    /// Template workflow definition (may contain placeholders).
    pub template_workflow: WorkflowDag,

    /// Tags for categorization.
    pub tags: Vec<String>,

    /// Examples of parameter values.
    pub examples: Vec<HashMap<String, serde_json::Value>>,
}

impl WorkflowTemplate {
    /// Creates a new workflow template.
    pub fn new(name: impl Into<String>, template_workflow: WorkflowDag) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            version: "1.0.0".to_string(),
            author: None,
            parameters: Vec::new(),
            template_workflow,
            tags: Vec::new(),
            examples: Vec::new(),
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a parameter.
    pub fn with_parameter(mut self, parameter: TemplateParameter) -> Self {
        self.parameters.push(parameter);
        self
    }

    /// Adds multiple parameters.
    pub fn with_parameters(mut self, parameters: Vec<TemplateParameter>) -> Self {
        self.parameters.extend(parameters);
        self
    }

    /// Adds tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Validates parameter values.
    pub fn validate_parameters(
        &self,
        values: &HashMap<String, serde_json::Value>,
    ) -> WorkflowResult<()> {
        // Check required parameters
        for param in &self.parameters {
            if param.required && !values.contains_key(&param.name) {
                return Err(WorkflowError::TemplateRenderError(format!(
                    "Required parameter '{}' is missing",
                    param.name
                )));
            }

            // Validate provided values
            if let Some(value) = values.get(&param.name) {
                param.validate(value)?;
            }
        }

        Ok(())
    }

    /// Renders a workflow from this template with the given parameter values.
    pub fn render(
        &self,
        values: HashMap<String, serde_json::Value>,
    ) -> WorkflowResult<WorkflowDag> {
        // Validate parameters
        self.validate_parameters(&values)?;

        // Add default values for missing parameters
        let mut final_values = values.clone();
        for param in &self.parameters {
            if !final_values.contains_key(&param.name) {
                if let Some(ref default) = param.default {
                    final_values.insert(param.name.clone(), default.clone());
                }
            }
        }

        // Clone the template workflow
        let mut workflow = self.template_workflow.clone();

        // Generate new workflow ID
        workflow.id = Uuid::new_v4();

        // Render task configurations with parameter substitution
        for task in workflow.tasks_mut() {
            task.config = self.substitute_parameters(&task.config, &final_values)?;
        }

        Ok(workflow)
    }

    /// Substitutes template parameters in a JSON value.
    fn substitute_parameters(
        &self,
        value: &serde_json::Value,
        params: &HashMap<String, serde_json::Value>,
    ) -> WorkflowResult<serde_json::Value> {
        match value {
            serde_json::Value::String(s) => {
                // Simple template substitution: {{param_name}}
                let mut result = s.clone();
                for (key, val) in params {
                    let placeholder = format!("{{{{{}}}}}", key);
                    if let Some(val_str) = val.as_str() {
                        result = result.replace(&placeholder, val_str);
                    } else {
                        result = result.replace(&placeholder, &val.to_string());
                    }
                }
                Ok(serde_json::Value::String(result))
            }
            serde_json::Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    new_map.insert(k.clone(), self.substitute_parameters(v, params)?);
                }
                Ok(serde_json::Value::Object(new_map))
            }
            serde_json::Value::Array(arr) => {
                let new_arr: Result<Vec<_>, _> = arr
                    .iter()
                    .map(|v| self.substitute_parameters(v, params))
                    .collect();
                Ok(serde_json::Value::Array(new_arr?))
            }
            _ => Ok(value.clone()),
        }
    }
}

/// Template registry for managing workflow templates.
pub struct TemplateRegistry {
    /// Templates by ID.
    templates: Arc<RwLock<HashMap<Uuid, WorkflowTemplate>>>,

    /// Templates by name.
    templates_by_name: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
}

impl TemplateRegistry {
    /// Creates a new template registry.
    pub fn new() -> Self {
        Self {
            templates: Arc::new(RwLock::new(HashMap::new())),
            templates_by_name: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registers a template.
    pub async fn register_template(&self, template: WorkflowTemplate) -> WorkflowResult<Uuid> {
        let template_id = template.id;
        let template_name = template.name.clone();

        // Validate the template workflow
        template.template_workflow.validate()?;

        info!("Registering template: {} ({})", template_name, template_id);

        let mut templates = self.templates.write().await;
        templates.insert(template_id, template);

        let mut templates_by_name = self.templates_by_name.write().await;
        templates_by_name
            .entry(template_name)
            .or_insert_with(Vec::new)
            .push(template_id);

        Ok(template_id)
    }

    /// Gets a template by ID.
    pub async fn get_template(&self, template_id: Uuid) -> WorkflowResult<WorkflowTemplate> {
        let templates = self.templates.read().await;
        templates
            .get(&template_id)
            .cloned()
            .ok_or_else(|| WorkflowError::TemplateNotFound(template_id.to_string()))
    }

    /// Gets templates by name.
    pub async fn get_templates_by_name(&self, name: &str) -> Vec<WorkflowTemplate> {
        let templates_by_name = self.templates_by_name.read().await;
        if let Some(ids) = templates_by_name.get(name) {
            let templates = self.templates.read().await;
            ids.iter()
                .filter_map(|id| templates.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Lists all templates.
    pub async fn list_templates(&self) -> Vec<WorkflowTemplate> {
        let templates = self.templates.read().await;
        templates.values().cloned().collect()
    }

    /// Finds templates by tag.
    pub async fn find_by_tag(&self, tag: &str) -> Vec<WorkflowTemplate> {
        let templates = self.templates.read().await;
        templates
            .values()
            .filter(|t| t.tags.iter().any(|t_tag| t_tag == tag))
            .cloned()
            .collect()
    }

    /// Removes a template.
    pub async fn remove_template(&self, template_id: Uuid) -> WorkflowResult<()> {
        let mut templates = self.templates.write().await;
        let template = templates.remove(&template_id).ok_or_else(|| {
            WorkflowError::TemplateNotFound(template_id.to_string())
        })?;

        let mut templates_by_name = self.templates_by_name.write().await;
        if let Some(ids) = templates_by_name.get_mut(&template.name) {
            ids.retain(|&id| id != template_id);
        }

        info!("Removed template: {} ({})", template.name, template_id);
        Ok(())
    }

    /// Renders a workflow from a template.
    pub async fn render_workflow(
        &self,
        template_id: Uuid,
        values: HashMap<String, serde_json::Value>,
    ) -> WorkflowResult<WorkflowDag> {
        let template = self.get_template(template_id).await?;
        template.render(values)
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Pre-defined workflow templates.
pub struct StandardTemplates;

impl StandardTemplates {
    /// Creates a simple sequential workflow template.
    pub fn sequential_pipeline() -> WorkflowTemplate {
        let mut workflow = WorkflowDag::new("sequential_pipeline");

        let task1 = Task::new("step_1", "{{task_type_1}}")
            .with_config(serde_json::json!({"config": "{{config_1}}"}));
        let task2 = Task::new("step_2", "{{task_type_2}}")
            .with_config(serde_json::json!({"config": "{{config_2}}"}));
        let task3 = Task::new("step_3", "{{task_type_3}}")
            .with_config(serde_json::json!({"config": "{{config_3}}"}));

        let id1 = workflow.add_task(task1);
        let id2 = workflow.add_task(task2);
        let id3 = workflow.add_task(task3);

        workflow
            .add_dependency(id1, id2, DependencyType::Sequential)
            .ok();
        workflow
            .add_dependency(id2, id3, DependencyType::Sequential)
            .ok();

        WorkflowTemplate::new("Sequential Pipeline", workflow)
            .with_description("A simple sequential 3-step pipeline")
            .with_parameters(vec![
                TemplateParameter::new("task_type_1", "string"),
                TemplateParameter::new("task_type_2", "string"),
                TemplateParameter::new("task_type_3", "string"),
                TemplateParameter::new("config_1", "string").with_default(serde_json::json!("{}")),
                TemplateParameter::new("config_2", "string").with_default(serde_json::json!("{}")),
                TemplateParameter::new("config_3", "string").with_default(serde_json::json!("{}")),
            ])
            .with_tags(vec!["pipeline".to_string(), "sequential".to_string()])
    }

    /// Creates a parallel fan-out/fan-in workflow template.
    pub fn fan_out_fan_in() -> WorkflowTemplate {
        let mut workflow = WorkflowDag::new("fan_out_fan_in");

        let start = Task::new("start", "{{start_task_type}}")
            .with_config(serde_json::json!({"config": "{{start_config}}"}));

        let parallel1 = Task::new("parallel_1", "{{parallel_task_type}}")
            .with_config(serde_json::json!({"config": "{{parallel_config_1}}"}));
        let parallel2 = Task::new("parallel_2", "{{parallel_task_type}}")
            .with_config(serde_json::json!({"config": "{{parallel_config_2}}"}));
        let parallel3 = Task::new("parallel_3", "{{parallel_task_type}}")
            .with_config(serde_json::json!({"config": "{{parallel_config_3}}"}));

        let end = Task::new("end", "{{end_task_type}}")
            .with_config(serde_json::json!({"config": "{{end_config}}"}));

        let start_id = workflow.add_task(start);
        let p1_id = workflow.add_task(parallel1);
        let p2_id = workflow.add_task(parallel2);
        let p3_id = workflow.add_task(parallel3);
        let end_id = workflow.add_task(end);

        // Fan out
        workflow
            .add_dependency(start_id, p1_id, DependencyType::Sequential)
            .ok();
        workflow
            .add_dependency(start_id, p2_id, DependencyType::Sequential)
            .ok();
        workflow
            .add_dependency(start_id, p3_id, DependencyType::Sequential)
            .ok();

        // Fan in
        workflow
            .add_dependency(p1_id, end_id, DependencyType::Sequential)
            .ok();
        workflow
            .add_dependency(p2_id, end_id, DependencyType::Sequential)
            .ok();
        workflow
            .add_dependency(p3_id, end_id, DependencyType::Sequential)
            .ok();

        WorkflowTemplate::new("Fan-Out Fan-In", workflow)
            .with_description("Parallel execution pattern with start and end tasks")
            .with_parameters(vec![
                TemplateParameter::new("start_task_type", "string"),
                TemplateParameter::new("parallel_task_type", "string"),
                TemplateParameter::new("end_task_type", "string"),
                TemplateParameter::new("start_config", "string")
                    .with_default(serde_json::json!("{}")),
                TemplateParameter::new("parallel_config_1", "string")
                    .with_default(serde_json::json!("{}")),
                TemplateParameter::new("parallel_config_2", "string")
                    .with_default(serde_json::json!("{}")),
                TemplateParameter::new("parallel_config_3", "string")
                    .with_default(serde_json::json!("{}")),
                TemplateParameter::new("end_config", "string").with_default(serde_json::json!("{}")),
            ])
            .with_tags(vec!["pipeline".to_string(), "parallel".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_validation() {
        let param = TemplateParameter::new("test", "string").with_description("Test parameter");

        assert!(param.validate(&serde_json::json!("hello")).is_ok());
        assert!(param.validate(&serde_json::json!(123)).is_err());
    }

    #[test]
    fn test_template_rendering() {
        let template = StandardTemplates::sequential_pipeline();

        let mut values = HashMap::new();
        values.insert("task_type_1".to_string(), serde_json::json!("process"));
        values.insert("task_type_2".to_string(), serde_json::json!("analyze"));
        values.insert("task_type_3".to_string(), serde_json::json!("finalize"));

        let workflow = template.render(values).unwrap();
        assert_eq!(workflow.tasks().len(), 3);
    }

    #[tokio::test]
    async fn test_template_registry() {
        let registry = TemplateRegistry::new();

        let template = StandardTemplates::sequential_pipeline();
        let template_id = registry.register_template(template).await.unwrap();

        let retrieved = registry.get_template(template_id).await.unwrap();
        assert_eq!(retrieved.name, "Sequential Pipeline");
    }
}
