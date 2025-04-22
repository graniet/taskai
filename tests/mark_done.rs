use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_mark_done_integration() {
    // Create a temporary YAML file
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
    
    // Run the CLI command
    let output = Command::new(env!("CARGO_BIN_EXE_taskai"))
        .args(["mark-done", file.path().to_str().unwrap(), "--task", "TEST-1"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    
    // Read the file back and check if the task state was updated
    let content = fs::read_to_string(file.path()).unwrap();
    
    // The state should now be "Done" instead of "Todo"
    assert!(content.contains("state: Done"));
}