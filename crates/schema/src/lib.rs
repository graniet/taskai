use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the state of a task.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum TaskState {
    /// The task is yet to be completed.
    Todo,
    /// The task has been completed.
    Done,
}

impl Default for TaskState {
    fn default() -> Self {
        TaskState::Todo
    }
}

/// Represents a single task in the backlog.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Task {
    /// Unique identifier for the task.
    pub id: String,
    /// Title of the task.
    pub title: String,
    /// List of task IDs that this task depends on.
    #[serde(default)]
    pub depends: Vec<String>,
    /// Current state of the task.
    #[serde(default)]
    pub state: TaskState,
    /// Optional description of the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional deliverable specification for the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deliverable: Option<DeliverableSpec>,
    /// List of criteria that define when the task is considered done.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub done_when: Vec<String>,
}

/// Represents the deliverable(s) for a task.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum DeliverableSpec {
    /// A single deliverable as a string.
    Single(String),
    /// Multiple deliverables as a list of strings.
    Multiple(Vec<String>),
}

/// Represents an epic, which is a collection of related tasks.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Epic {
    /// Unique identifier for the epic.
    pub id: String,
    /// Title of the epic.
    pub title: String,
    /// List of tasks associated with the epic.
    #[serde(default)]
    pub tasks: Vec<Task>,
}

/// Represents the entire project backlog, including tasks, epics, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Backlog {
    /// Name of the project.
    pub project: String,
    /// Optional Rust version for the project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_version: Option<String>,
    /// List of success criteria for the project.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub success_criteria: Vec<String>,
    /// Environment variables or settings for the project.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub environment: HashMap<String, serde_json::Value>,
    /// List of epics in the backlog.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub epics: Vec<Epic>,
    /// List of standalone tasks in the backlog.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<Task>,
}

impl Backlog {
    /// Validates the backlog for missing dependencies and cycles.
    ///
    /// Returns `Ok(())` if the backlog is valid, or an error message otherwise.
    pub fn validate(&self) -> Result<(), String> {
        let task_ids = self.all_task_ids();
        
        for task in self.all_tasks() {
            for dep_id in &task.depends {
                if !task_ids.contains(dep_id) {
                    return Err(format!("Task {} depends on non-existent task {}", task.id, dep_id));
                }
            }
        }
        
        if let Err(cycle) = self.check_cycles() {
            return Err(format!("Dependency cycle detected: {}", cycle));
        }
        
        Ok(())
    }
    
    /// Returns a vector of references to all tasks, including those in epics.
    fn all_tasks(&self) -> Vec<&Task> {
        let mut all_tasks = Vec::new();
        
        for task in &self.tasks {
            all_tasks.push(task);
        }
        
        for epic in &self.epics {
            for task in &epic.tasks {
                all_tasks.push(task);
            }
        }
        
        all_tasks
    }
    
    /// Returns a vector of all task IDs in the backlog.
    fn all_task_ids(&self) -> Vec<String> {
        self.all_tasks().iter().map(|t| t.id.clone()).collect()
    }
    
    /// Checks for cycles in the task dependency graph.
    ///
    /// Returns `Ok(())` if no cycles are found, or an error message with the cycle path.
    fn check_cycles(&self) -> Result<(), String> {
        let all_tasks = self.all_tasks();
        let task_map: HashMap<String, &Task> = all_tasks.into_iter()
            .map(|t| (t.id.clone(), t))
            .collect();
        
        for task in task_map.values() {
            let mut visited = HashMap::new();
            let mut path = Vec::new();
            
            if self.has_cycle(task, &task_map, &mut visited, &mut path) {
                return Err(path.join(" -> "));
            }
        }
        
        Ok(())
    }
    
    /// Helper function to detect cycles starting from a given task.
    ///
    /// Returns `true` if a cycle is found, otherwise `false`.
    fn has_cycle(
        &self,
        task: &Task,
        task_map: &HashMap<String, &Task>,
        visited: &mut HashMap<String, bool>,
        path: &mut Vec<String>,
    ) -> bool {
        let task_id = &task.id;
        
        if let Some(in_path) = visited.get(task_id) {
            if *in_path {
                path.push(task_id.clone());
                return true;
            }
            return false;
        }
        
        visited.insert(task_id.clone(), true);
        path.push(task_id.clone());
        
        for dep_id in &task.depends {
            if let Some(dep_task) = task_map.get(dep_id) {
                if self.has_cycle(dep_task, task_map, visited, path) {
                    return true;
                }
            }
        }
        
        visited.insert(task_id.clone(), false);
        path.pop();
        
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;
    
    /// Tests serialization and deserialization of the Backlog struct.
    #[test]
    fn roundtrip() {
        let yaml = r#"
        project: test-project
        rust_version: "1.77"
        tasks:
          - id: T-1
            title: "Test task"
            depends: []
            deliverable: "src/main.rs"
            done_when:
              - "cargo test passes"
        "#;
        
        let backlog: Backlog = serde_yaml::from_str(yaml).unwrap();
        let serialized = serde_yaml::to_string(&backlog).unwrap();
        let deserialized: Backlog = serde_yaml::from_str(&serialized).unwrap();
        
        assert_eq!(backlog.project, deserialized.project);
        assert_eq!(backlog.tasks[0].id, deserialized.tasks[0].id);
    }
    
    /// Tests roundtrip serialization and deserialization of task states.
    #[test]
    fn state_roundtrip() {
        let yaml = r#"
        project: test-project
        tasks:
          - id: T-1
            title: "Test task"
            state: Done
            depends: []
          - id: T-2
            title: "Another task"
            state: Todo
            depends: ["T-1"]
        "#;
        
        let backlog: Backlog = serde_yaml::from_str(yaml).unwrap();
        
        match backlog.tasks[0].state {
            TaskState::Done => {},
            _ => panic!("Expected task T-1 to be Done"),
        }
        
        match backlog.tasks[1].state {
            TaskState::Todo => {},
            _ => panic!("Expected task T-2 to be Todo"),
        }
        
        let serialized = serde_yaml::to_string(&backlog).unwrap();
        let deserialized: Backlog = serde_yaml::from_str(&serialized).unwrap();
        
        match deserialized.tasks[0].state {
            TaskState::Done => {},
            _ => panic!("Expected task T-1 to be Done after roundtrip"),
        }
        
        match deserialized.tasks[1].state {
            TaskState::Todo => {},
            _ => panic!("Expected task T-2 to be Todo after roundtrip"),
        }
    }
}