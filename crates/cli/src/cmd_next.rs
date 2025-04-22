use std::fs;
use std::path::Path;
use std::process;
use taskai_schema::Backlog;

/// Executes the "next" command: reads the backlog file, parses it, and prints the list of tasks that are ready to be worked on.
/// A task is considered ready if it is in the Todo state and all its dependencies are in the Done state.
/// For each ready task, prints its ID, title, description (if any), and deliverables (if any).
pub fn execute(backlog_file: &Path) {
    let content = match fs::read_to_string(backlog_file) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Error reading backlog file: {}", err);
            process::exit(1);
        }
    };

    let backlog: Backlog = match serde_yaml::from_str(&content) {
        Ok(b) => b,
        Err(err) => {
            eprintln!("Error parsing backlog file: {}", err);
            process::exit(1);
        }
    };

    let ready_tasks = taskai_core::get_ready_tasks(&backlog);

    if ready_tasks.is_empty() {
        println!("No tasks are ready to work on.");
        return;
    }

    println!("Tasks ready to work on:");
    for task in ready_tasks {
        println!("{}: {}", task.id, task.title);

        if let Some(desc) = &task.description {
            for line in desc.lines() {
                println!("  {}", line);
            }
        }

        if let Some(deliverable) = &task.deliverable {
            match deliverable {
                taskai_schema::DeliverableSpec::Single(path) => {
                    println!("  Deliverable: {}", path);
                },
                taskai_schema::DeliverableSpec::Multiple(paths) => {
                    println!("  Deliverables:");
                    for path in paths {
                        println!("    - {}", path);
                    }
                }
            }
        }

        println!();
    }
}