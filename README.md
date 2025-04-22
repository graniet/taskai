# TaskAI

<div align="center">

<img src="docs/logo.png" alt="TaskAI Logo" width="150px" />

**Generate structured task backlogs for AI agents and automation workflows**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Rust Version](https://img.shields.io/badge/rust-1.77%2B-orange.svg)

</div>

## 📖 Overview

TaskAI bridges natural language and structured task definitions, generating well-organized backlogs that both humans and AI agents can understand and execute. Convert informal descriptions into actionable tasks with dependencies, deliverables, and completion criteria - perfect for driving autonomous agents or organizing human-AI collaboration.

## ✨ Features

- 🤖 **AI-Agent Ready**: Generate structured YAML task definitions optimized for AI agent consumption
- 🗣️ **Natural Language Input**: Convert simple text descriptions into comprehensive task breakdowns
- 📋 **State Tracking**: Monitor task progress with Todo/Done states
- 🔄 **Dependency Resolution**: Automatically identify tasks ready for execution based on dependencies
- ✅ **Progress Tracking**: Mark tasks as complete and manage the workflow lifecycle
- 🌐 **Multilingual**: Support for inputs in both English and French
- 🐳 **Containerized**: Run in Docker for CI/CD integration and consistent environments

## 🚀 Installation

```bash
# Install directly from crates.io
cargo install taskai

# Or build from source
git clone https://github.com/graniet/taskai.git
cd taskai
cargo install --path crates/cli
```

## 🎮 Usage

### 1. Create a task backlog from natural language

Create a simple text file with your requirements:

```
# simple_request.txt
Create a Python script that fetches weather data from an API.
```

Generate a structured task backlog:

```bash
taskai gen simple_request.txt > weather_tasks.yml
```

The output will be a structured YAML backlog:

```yaml
project: weather-api-client
tasks:
  - id: W-1
    title: "Setup project structure"
    depends: []
    state: Todo
    deliverable: ["README.md", "requirements.txt"]
    done_when: ["Project directory structure is created"]
  
  - id: W-2
    title: "Create API client for weather data"
    depends: ["W-1"]
    state: Todo
    deliverable: ["weather_client.py"]
    done_when: ["Client can connect to weather API", "Basic error handling implemented"]
  
  - id: W-3
    title: "Implement data parsing and formatting"
    depends: ["W-2"]
    state: Todo
    deliverable: ["weather_client.py"]
    done_when: ["Weather data is properly parsed and formatted"]
```

### 2. Query Tasks Ready for Execution

Identify tasks that are ready to be worked on (all dependencies satisfied):

```bash
taskai next weather_tasks.yml
```

Output:
```
Tasks ready to work on:
W-1: Setup project structure
  Deliverables:
    - README.md
    - requirements.txt
```

### Using Claude with TaskAI - Simple Workflow

With TaskAI, you can supercharge Claude's coding capabilities by giving it structured tasks to work on:

```bash
# Generate tasks from a simple description
echo "Build a weather dashboard with React" > idea.txt
taskai gen idea.txt > tasks.yml
```

Now, simply tell Claude to read and implement the task:

```
Read tasks.yml and implement the next ready task. 
Use 'taskai next tasks.yml' to see what task to work on 
and 'taskai mark-done tasks.yml --task TASK-ID' to mark it complete when done.
```

That's it! With just this simple prompt, Claude will:
- Read the tasks.yml file
- Find the next ready task 
- Implement it according to the specifications
- Provide the command to mark the task as complete

When Claude finishes, simply mark the task as done:

```bash
taskai mark-done tasks.yml --task TASK-ID
```

Then you can ask Claude to work on the next task with the same basic prompt. This creates a continuous loop where Claude methodically works through the entire project, one task at a time, with minimal input from you.


## 📊 Architecture

TaskAI is organized into three Rust crates:

- **schema**: Defines the data structures for tasks, dependencies and completion criteria
- **core**: Implements the LLM communication and YAML generation/validation
- **cli**: Provides the command-line interface

## 🧪 Environment Variables

- `OPENAI_API_KEY`: Required for LLM functionality

## 🤝 Contributing

Contributions welcome! Please feel free to submit a Pull Request.

## 📜 License

This project is licensed under the MIT License - see the LICENSE file for details.