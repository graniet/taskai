use taskai_schema::Backlog;

/// Parses a YAML string and validates it as a `Backlog`.
///
/// This function attempts to parse the provided YAML string into a `Backlog` object.
/// If the string is empty, it returns an error. It first tries to parse the string directly.
/// If parsing fails, it attempts to extract the YAML content from the string (in case it is embedded in markdown or other text).
/// If parsing still fails, it tries to fix common JSON formatting errors and parse again.
/// Returns a validated `Backlog` on success, or an error message on failure.
pub fn parse_and_validate_yaml(yaml_str: &str) -> Result<Backlog, String> {
    if yaml_str.trim().is_empty() {
        return Err("Empty response from LLM".to_string());
    }
    
    let parse_result = serde_yaml::from_str::<Backlog>(yaml_str);
    
    if let Ok(backlog) = parse_result {
        if backlog.validate().is_ok() {
            return Ok(backlog);
        }
    }
    
    let yaml_content = extract_yaml_content(yaml_str);
    
    match serde_yaml::from_str::<Backlog>(&yaml_content) {
        Ok(backlog) => {
            backlog.validate()?;
            Ok(backlog)
        }
        Err(e) => {
            eprintln!("Failed to parse YAML: {}", e);
            
            if let Some(fixed_yaml) = try_fix_json_errors(&yaml_content) {
                match serde_yaml::from_str::<Backlog>(&fixed_yaml) {
                    Ok(backlog) => {
                        backlog.validate()?;
                        Ok(backlog)
                    }
                    Err(e) => Err(format!("Failed to parse YAML after fixing JSON: {}", e)),
                }
            } else {
                eprintln!("Response sample: {}", &yaml_str[..std::cmp::min(yaml_str.len(), 200)]);
                Err(format!("Failed to parse YAML: {}", e))
            }
        }
    }
}

/// Extracts YAML content from a string that may contain additional text or formatting.
///
/// This function looks for YAML content in the input string, handling cases where the YAML
/// is embedded in markdown code blocks or surrounded by other text. It tries to find the
/// YAML section by looking for common markers and returns the extracted YAML as a string.
fn extract_yaml_content(text: &str) -> String {
    if text.trim().starts_with("project:") || (text.contains("project:") && text.contains("tasks:")) {
        return text.to_string();
    }

    if let Some(start) = text.find("```yaml") {
        let yaml_start = start + "```yaml".len();
        if let Some(end_marker) = text[yaml_start..].find("```") {
            let yaml_end = yaml_start + end_marker;
            if yaml_start < yaml_end {
                return text[yaml_start..yaml_end].trim().to_string();
            }
        }
    }
    
    if let Some(start) = text.find("```") {
        let yaml_start = start + "```".len();
        if let Some(end_marker) = text[yaml_start..].find("```") {
            let yaml_end = yaml_start + end_marker;
            if yaml_start < yaml_end {
                return text[yaml_start..yaml_end].trim().to_string();
            }
        }
    }
    
    if let Some(start) = text.find("project:") {
        let possible_ends = ["\n\n", "\r\n\r\n", "---", "###"];
        for end_marker in possible_ends.iter() {
            if let Some(end_pos) = text[start..].find(end_marker) {
                if start + end_pos > start {
                    return text[start..(start + end_pos)].trim().to_string();
                }
            }
        }
        return text[start..].trim().to_string();
    }
    
    text.to_string()
}

/// Attempts to fix common JSON formatting errors in a YAML string.
///
/// This function applies simple replacements to fix unquoted keys and other common issues
/// that may occur when JSON is embedded in YAML or vice versa. If the fixed string can be
/// parsed as a `Backlog`, it returns the fixed string; otherwise, it returns `None`.
fn try_fix_json_errors(yaml_str: &str) -> Option<String> {
    let mut fixed = yaml_str.to_string();
    
    fixed = fixed.replace("id:", "\"id\":");
    fixed = fixed.replace("title:", "\"title\":");
    fixed = fixed.replace("depends:", "\"depends\":");
    fixed = fixed.replace("deliverable:", "\"deliverable\":");
    fixed = fixed.replace("done_when:", "\"done_when\":");
    
    if serde_yaml::from_str::<Backlog>(&fixed).is_ok() {
        Some(fixed)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn retry_invalid_json() {
        let invalid_yaml = r#"
        project: test-project
        tasks:
          - id: T-1
            title: Test task
            depends: []
            deliverable: src/main.rs
            invalid_key without_colon
        "#;
        
        assert!(serde_yaml::from_str::<Backlog>(invalid_yaml).is_err());
        
        let valid_yaml = r#"
        project: test-project
        tasks:
          - id: T-1
            title: Test task
            depends: []
            deliverable: src/main.rs
        "#;
        
        let result = parse_and_validate_yaml(valid_yaml);
        assert!(result.is_ok());
    }
}