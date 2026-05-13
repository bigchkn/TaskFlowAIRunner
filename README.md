# TaskFlowAI Runner (`taskflow-runner`)

`taskflow-runner` is a Rust-based utility designed to orchestrate autonomous software development by scripting the execution of multiple AI CLIs (e.g., Gemini, Claude, Dirac, OpenCode). It leverages the [TaskFlowAI](https://github.com/bigchkn/TaskFlowAI) skill to drive project backlogs through a git-driven, worktree-isolated workflow.

## Overview

The runner transforms a static backlog (defined in `.taskflow/*.toml`) into an active development cycle. It dispatches tasks to configured AI providers, ensuring that each task is executed in a clean environment and its progress is tracked via Git.

### Prerequisites

- **TaskFlowAI Skill:** Must be installed and accessible to all configured AI providers.
- **AI CLIs:** One or more supported AI CLIs (e.g., `gemini`, `claude`) must be installed and authenticated.
- **Git:** Required for worktree management.

## Usage

Run all commands from the root of a Git repository that contains a TaskFlowAI project.

### Initialize the runner

Create or update the global runner config and add `.taskflow/worktrees` to the project's `.gitignore`:

```sh
taskflow-runner init
```

`init` detects supported providers in `PATH` (`gemini`, `claude`, `dirac`, and `opencode`) and writes the configuration to:

```text
~/.taskflow/taskflow-runner.json
```

If multiple providers are detected, the wizard asks which one should be the default enabled provider. If no supported provider is detected, create or edit the config manually.

### Configure providers

The config is JSON. A minimal config for a provider that reads the TaskFlow prompt from stdin looks like this:

```json
{
  "providers": {
    "gemini": {
      "command": "gemini",
      "args_before_prompt": [],
      "prompt_arg_mode": "stdin",
      "env": {},
      "timeout_seconds": null
    }
  },
  "enabled_provider": "gemini",
  "default_timeout_seconds": 3600,
  "taskflow_command": "taskflow-ai",
  "worktree_root": ".taskflow/worktrees"
}
```

Use `args_before_prompt` for static CLI flags, such as headless or permission flags. Set `prompt_arg_mode` to `positional` for CLIs that expect the prompt as a command-line argument instead of stdin:

```json
{
  "providers": {
    "claude": {
      "command": "claude",
      "args_before_prompt": ["--permission-mode", "bypassPermissions"],
      "prompt_arg_mode": "positional",
      "env": {},
      "timeout_seconds": 1800
    }
  },
  "enabled_provider": "claude",
  "default_timeout_seconds": 3600,
  "taskflow_command": "taskflow-ai",
  "worktree_root": ".taskflow/worktrees"
}
```

Validate the config after editing it:

```sh
taskflow-runner config validate
```

### Dispatch tasks

Run one task and exit:

```sh
taskflow-runner dispatch --once
```

Run continuously in watch mode:

```sh
taskflow-runner dispatch
```

For each available task, the runner creates a worktree under `.taskflow/worktrees/<task-id>`, marks the TaskFlowAI task as started, invokes the enabled provider with this prompt, validates the task, and records the result:

```text
Run /taskflow and execute the next available task.
```

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

### 4. Shell Completions
- **Command:** `taskflow-runner completions <shell>`
- **Supported shells:** Bash, Elvish, Fish, PowerShell, and Zsh.
- **Example:** `taskflow-runner completions zsh > _taskflow-runner`

### 5. Provider Driver System
The runner uses a config-driven driver system to interface with AI CLIs. Since most agents are invoked via a simple CLI command with a prompt, adding new providers is a matter of defining their execution patterns (command, headless flags, etc.) in the configuration.

## Roadmap & Progress

This project is managed using TaskFlowAI. See `.taskflow/roadmap/` for the current backlog and milestones.

## Design Goals for Initial Implementation
- **Robust Error Handling:** Detect when an AI agent fails or hangs.
- **Worktree Lifecycle Management:** Automatically create, prune, and cleanup worktrees.
- **TaskFlow State Sync:** Ensure the runner correctly interprets the status of tasks updated by the TaskFlowAI skill.
