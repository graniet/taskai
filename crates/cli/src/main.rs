/// Main entry point for the CLI application.
/// 
/// This module provides commands to generate a task backlog from a specification,
/// list tasks that are ready to work on, and mark tasks as done.
mod cmd_next;
mod cmd_done;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::{fs, process};
use taskai_core;

/// CLI argument parser structure.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Enum representing the available CLI commands.
#[derive(Subcommand)]
enum Commands {
    /// Generate a task backlog from a specification.
    Gen {
        /// Path to the specification file.
        spec_file: PathBuf,
        
        /// Language for prompts (en, fr).
        #[arg(long, default_value = "en")]
        lang: String,
        
        /// Style of the generated backlog.
        #[arg(long, default_value = "standard")]
        style: String,
    },
    
    /// List tasks that are ready to work on.
    Next {
        /// Path to the backlog file.
        backlog_file: PathBuf,
    },
    
    /// Mark a task as done.
    #[command(name = "mark-done")]
    MarkDone {
        /// Path to the backlog file.
        backlog_file: PathBuf,
        
        /// ID of the task to mark as done.
        #[arg(long)]
        task: String,
    },
}

/// Asynchronous main function for the CLI application.
/// Handles command parsing and dispatches to the appropriate command handler.
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Gen { spec_file, lang, style } => {
            // Read the specification file
            let spec = match fs::read_to_string(&spec_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading specification file: {}", err);
                    process::exit(1);
                }
            };

            // Generate the backlog
            let generator = taskai_core::BacklogGenerator::new()
                .with_language(&lang)
                .with_style(&style);

            match generator.generate(&spec).await {
                Ok(backlog) => {
                    // Output YAML to stdout
                    match serde_yaml::to_string(&backlog) {
                        Ok(yaml) => println!("{}", yaml),
                        Err(err) => {
                            eprintln!("Error serializing backlog to YAML: {}", err);
                            process::exit(1);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error generating backlog: {}", err);
                    process::exit(1);
                }
            }
        }
        
        Commands::Next { backlog_file } => {
            cmd_next::execute(&backlog_file);
        }
        
        Commands::MarkDone { backlog_file, task } => {
            cmd_done::execute(&backlog_file, &task);
        }
    }
}