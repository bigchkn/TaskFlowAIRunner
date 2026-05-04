# TaskFlowAI Runner (`taskflow-runner`)

`taskflow-runner` is a Rust-based utility designed to orchestrate autonomous software development by scripting the execution of multiple AI CLIs (e.g., Gemini, Claude, Dirac, OpenCode). It leverages the [TaskFlowAI](https://github.com/bigchkn/TaskFlowAI) skill to drive project backlogs through a git-driven, worktree-isolated workflow.

## Overview

The runner transforms a static backlog (defined in `.taskflow/*.toml`) into an active development cycle. It dispatches tasks to configured AI providers, ensuring that each task is executed in a clean environment and its progress is tracked via Git.

### Prerequisites

- **TaskFlowAI Skill:** Must be installed and accessible to all configured AI providers.
- **AI CLIs:** One or more supported AI CLIs (e.g., `gemini`, `claude`) must be installed and authenticated.
- **Git:** Required for worktree management.

## Core Mechanics

### 1. Configuration & Initialization
- **Global Config:** User preferences are stored in `~/.taskflow/taskflow-runner.json`. This file manages enabled providers and their specific execution flags.
- **Initialization:** Running `taskflow-runner init` will guide the user through a setup wizard to enable supported providers and generate the initial configuration.

### 2. Git-Driven Execution (Worktrees)
To ensure isolation and safety, `taskflow-runner` utilizes Git worktrees:
- Every dispatched task is assigned a dedicated worktree located in `.taskflow/worktrees/<task-uid>`.
- This prevents execution side-effects from polluting the main working directory and allows for clean diffing/validation of AI-generated changes.
- Execution is currently sequential (no parallel dispatch).

### 3. Dispatch & Watch Mode
- **Command:** `taskflow-runner dispatch`
- **Behavior:** The runner enters a loop (watch mode), scanning the `.taskflow` roadmap for tasks ready for execution.
- **Execution Prompt:** For each task, the runner invokes the enabled AI CLI with a headless/skip-permission flag and the standard prompt: 
  > "Run /taskflow and execute the next available task."

### 4. Provider Driver System
The runner uses a config-driven driver system to interface with AI CLIs. Since most agents are invoked via a simple CLI command with a prompt, adding new providers is a matter of defining their execution patterns (command, headless flags, etc.) in the configuration.

## Roadmap & Progress

This project is managed using TaskFlowAI. See `.taskflow/roadmap/` for the current backlog and milestones.

## Design Goals for Initial Implementation
- **Robust Error Handling:** Detect when an AI agent fails or hangs.
- **Worktree Lifecycle Management:** Automatically create, prune, and cleanup worktrees.
- **TaskFlow State Sync:** Ensure the runner correctly interprets the status of tasks updated by the TaskFlowAI skill.
