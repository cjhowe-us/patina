# Patina Language - ADK Agents

This directory contains the implementation of the AI agents for Patina language development using Google's Agent Development Kit (ADK).

## Structure

- `agents/`: Agent implementations
- `tools/`: Tool implementations for agents
- `config/`: Configuration for ADK
- `prompts/`: Agent prompts (extracted from `docs/llm/prompts.md`)

## Setup

1. Make sure you have pipenv installed and the ADK dependencies are installed:

   ```bash
   pipenv install
   ```

2. Source environment variables:

   ```bash
   source .env
   ```

## Agent Roles

- **Coordinator Agent**: Oversees development, decomposes tasks, monitors progress, handles cross-agent decisions, initiates human intervention.
- **Syntax Agent**: Defines and updates the formal grammar for Patina using PEG, generates and maintains parser code.
- **Semantic Analysis Agent**: Performs semantic analysis, type checking, and validation of Patina code.
- **Memory Safety Agent**: Implements mutable value semantics for borrow checking within the 'system' model.
- **IR Agent**: Designs and generates Intermediate Representation(s) for Patina code.
- **Testing Agent**: Creates test cases for language features, memory models, and safety rules.
- **QA/Validation Agent**: Tests the compiler from an external perspective, providing user feedback.
- **Infrastructure Agent**: Sets up and maintains GCP infrastructure, GitHub repository, and CI/CD pipelines.

## Development

To add new tools or modify existing ones, edit the files in the `tools/` directory. Each tool follows the ADK Tool specification with a name, description, parameters, and an async function implementation.
