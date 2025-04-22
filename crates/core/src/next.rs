use taskai_schema::{Backlog, Task, TaskState};
use std::collections::HashMap;

/// Returns a vector of references to tasks that are ready to be worked on.
/// A task is considered ready if it is in the Todo state and all its dependencies are in the Done state.
/// Tasks are collected from both standalone tasks and tasks within epics.
pub fn get_ready_tasks(backlog: &Backlog) -> Vec<&Task> {
    let all_tasks = get_all_tasks(backlog);

    let task_map: HashMap<&str, &Task> = all_tasks
        .iter()
        .map(|task| (task.id.as_str(), *task))
        .collect();

    all_tasks
        .iter()
        .filter(|task| {
            if !matches!(task.state, TaskState::Todo) {
                return false;
            }

            task.depends.iter().all(|dep_id| {
                if let Some(dep_task) = task_map.get(dep_id.as_str()) {
                    matches!(dep_task.state, TaskState::Done)
                } else {
                    true
                }
            })
        })
        .copied()
        .collect()
}

/// Returns a vector of references to all tasks in the backlog, including both standalone tasks and tasks within epics.
fn get_all_tasks(backlog: &Backlog) -> Vec<&Task> {
    let mut all_tasks = Vec::new();

    for task in &backlog.tasks {
        all_tasks.push(task);
    }

    for epic in &backlog.epics {
        for task in &epic.tasks {
            all_tasks.push(task);
        }
    }

    all_tasks
}

#[cfg(test)]
mod tests {
    use super::*;
    use taskai_schema::TaskState;

    #[test]
    fn ready_simple() {
        let backlog = Backlog {
            project: "test".to_string(),
            rust_version: Some("1.77".to_string()),
            success_criteria: vec![],
            environment: HashMap::new(),
            epics: vec![],
            tasks: vec![
                Task {
                    id: "T-1".to_string(),
                    title: "Task 1".to_string(),
                    depends: vec![],
                    state: TaskState::Done,
                    description: None,
                    deliverable: None,
                    done_when: vec![],
                },
                Task {
                    id: "T-2".to_string(),
                    title: "Task 2".to_string(),
                    depends: vec!["T-1".to_string()],
                    state: TaskState::Todo,
                    description: None,
                    deliverable: None,
                    done_when: vec![],
                },
                Task {
                    id: "T-3".to_string(),
                    title: "Task 3".to_string(),
                    depends: vec!["T-1".to_string(), "T-2".to_string()],
                    state: TaskState::Todo,
                    description: None,
                    deliverable: None,
                    done_when: vec![],
                },
            ],
        };

        let ready_tasks = get_ready_tasks(&backlog);

        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, "T-2");
    }
}