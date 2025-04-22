use std::fs;
use std::path::Path;
use std::process;
use taskai_schema::{Backlog, TaskState};

/// Marks a task as done in the backlog file given its ID.
///
/// This function reads the backlog YAML file, searches for the task with the specified `task_id`
/// (either as a standalone task or within an epic), marks it as done, and writes the updated
/// backlog back to the file. If the task is not found or if any file operation fails, the process exits with an error.
pub fn execute(backlog_file: &Path, task_id: &str) {
    let content = match fs::read_to_string(backlog_file) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Error reading backlog file: {}", err);
            process::exit(1);
        }
    };

    let mut backlog: Backlog = match serde_yaml::from_str(&content) {
        Ok(b) => b,
        Err(err) => {
            eprintln!("Error parsing backlog file: {}", err);
            process::exit(1);
        }
    };

    let mut found = false;

    for task in &mut backlog.tasks {
        if task.id == task_id {
            task.state = TaskState::Done;
            found = true;
            break;
        }
    }

    if !found {
        for epic in &mut backlog.epics {
            for task in &mut epic.tasks {
                if task.id == task_id {
                    task.state = TaskState::Done;
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
    }

    if !found {
        eprintln!("Task with ID '{}' not found in the backlog.", task_id);
        process::exit(1);
    }

    match serde_yaml::to_string(&backlog) {
        Ok(yaml) => {
            if let Err(err) = fs::write(backlog_file, yaml) {
                eprintln!("Error writing to backlog file: {}", err);
                process::exit(1);
            }
            println!("Task {} marked as done.", task_id);
        }
        Err(err) => {
            eprintln!("Error serializing backlog to YAML: {}", err);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use taskai_schema::{Backlog};

    /// Tests that a task can be marked as done in the backlog file.
    #[test]
    fn test_mark_done() {
        let mut file = NamedTempFile::new().unwrap();
        
        let test_yaml = r#"
project: test-project
tasks:
  - id: TEST-1
    title: Test Task
    depends: []
    state: Todo
"#;
        
        file.write_all(test_yaml.as_bytes()).unwrap();
        file.flush().unwrap();
        
        execute(file.path(), "TEST-1");
        
        let content = fs::read_to_string(file.path()).unwrap();
        let backlog: Backlog = serde_yaml::from_str(&content).unwrap();
        
        assert_eq!(backlog.tasks.len(), 1);
        match backlog.tasks[0].state {
            TaskState::Done => {},
            _ => panic!("Task was not marked as done"),
        }
    }
}