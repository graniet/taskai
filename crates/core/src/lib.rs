mod validate;
mod next;

use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};
use taskai_schema::Backlog;
use std::path::Path;

/// BacklogGenerator is responsible for generating a project backlog from a specification using an LLM.
pub struct BacklogGenerator {
    model: String,
    language: String,
    style: String,
}

impl Default for BacklogGenerator {
    /// Returns a default BacklogGenerator with preset model, language, and style.
    fn default() -> Self {
        Self {
            model: "gpt-4.1-2025-04-14".to_string(),
            language: "en".to_string(),
            style: "standard".to_string(),
        }
    }
}

impl BacklogGenerator {
    /// Creates a new BacklogGenerator with default settings.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Sets the LLM model to use.
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }
    
    /// Sets the language for the prompts.
    pub fn with_language(mut self, language: &str) -> Self {
        self.language = language.to_string();
        self
    }
    
    /// Sets the style for the backlog generation.
    pub fn with_style(mut self, style: &str) -> Self {
        self.style = style.to_string();
        self
    }
    
    /// Returns the system prompt string based on the selected language.
    fn get_system_prompt(&self) -> String {
        match self.language.as_str() {
            "fr" => {
                if let Ok(content) = std::fs::read_to_string(Self::find_prompt_path("system_fr.txt")) {
                    content
                } else {
                    self.get_default_system_prompt()
                }
            }
            _ => self.get_default_system_prompt(),
        }
    }
    
    /// Returns the default system prompt in English, or a hardcoded fallback if the file is not found.
    fn get_default_system_prompt(&self) -> String {
        if let Ok(content) = std::fs::read_to_string(Self::find_prompt_path("system_en.txt")) {
            content
        } else {
            "You are a helpful assistant specialized in converting project specifications into structured task backlogs. Create a YAML backlog with tasks, dependencies, and deliverables.".to_string()
        }
    }
    
    /// Attempts to find the prompt file in several possible locations.
    fn find_prompt_path(filename: &str) -> String {
        let paths = vec![
            format!("prompts/{}", filename),
            format!("crates/core/prompts/{}", filename),
            format!("{}", filename),
        ];
        
        for path in paths {
            if Path::new(&path).exists() {
                return path;
            }
        }
        
        format!("crates/core/prompts/{}", filename)
    }
    
    /// Generates a backlog from the given specification using the configured LLM.
    pub async fn generate(&self, spec: &str) -> Result<Backlog, String> {
        #[cfg(test)]
        return self.generate_mock(spec);
        
        #[cfg(not(test))]
        {
            let system_prompt = self.get_system_prompt();
            let user_prompt = spec.to_string();
            
            let response = self.call_llm(&system_prompt, &user_prompt).await?;
            
            return validate::parse_and_validate_yaml(&response);
        }
        
        #[allow(unreachable_code)]
        {
            Err("Error: Unreachable code reached".to_string())
        }
    }
    
    /// Calls the LLM API with the given system and user prompts, returning the raw response.
    async fn call_llm(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY environment variable not set".to_string())?;

        let llm = LLMBuilder::new()
            .backend(LLMBackend::OpenAI)
            .api_key(api_key)
            .model(&self.model)
            .max_tokens(2048)
            .temperature(0.7)
            .stream(false)
            .build()
            .map_err(|e| format!("Failed to build LLM: {}", e))?;

        let formatted_prompt = format!("{}\n\n{}", system_prompt, user_prompt);

        let messages = vec![
            ChatMessage::user()
                .content(formatted_prompt)
                .build(),
        ];

        let completion = llm.chat(&messages)
            .await
            .map_err(|e| format!("LLM API error: {}", e))?;
        
        Ok(completion.to_string())
    }
    
    /// Determines if the input string is already a structured project specification.
    #[allow(dead_code)]
    fn is_structured_spec(input: &str) -> bool {
        input.contains("Project:") && 
        (input.contains("Language:") || input.contains("Goal:") || input.contains("Deliverables:"))
    }
    
    /// Generates a mock backlog for testing purposes.
    #[cfg(test)]
    fn generate_mock(&self, spec: &str) -> Result<Backlog, String> {
        let mock_yaml = format!(r#"
        project: mock-project
        rust_version: "1.77"
        tasks:
          - id: MOCK-1
            title: "Mock task from spec: {}"
            depends: []
            state: Todo
            deliverable: "src/main.rs"
            done_when:
              - "cargo test passes"
        "#, spec.trim());
        
        serde_yaml::from_str(&mock_yaml).map_err(|e| e.to_string())
    }
}

/// Returns a list of tasks that are ready to be worked on.
pub use next::get_ready_tasks;

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Tests the mock backlog generation.
    #[tokio::test]
    async fn gen_mock() {
        let generator = BacklogGenerator::new();
        let result = generator.generate("Test specification").await.unwrap();
        
        assert_eq!(result.project, "mock-project");
        assert_eq!(result.tasks[0].id, "MOCK-1");
    }
}