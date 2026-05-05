# High-Level Design: TaskFlow Runner Initial Architecture

Type: hld
Status: approved
Milestone: M1
Task: TF-1

## 1. Introduction

`taskflow-runner` is a Rust CLI that turns a TaskFlowAI roadmap into an automated development loop. It selects ready tasks, creates an isolated Git worktree for each task, invokes a configured AI coding CLI with the TaskFlow prompt, and records the result through Git and TaskFlowAI state.

The initial implementation is intentionally sequential. That keeps task ordering, worktree ownership, and provider failure handling simple while establishing the core contracts needed for later parallel dispatch.

## 2. Goals

- Provide `taskflow-runner init` to create a global user configuration at `~/.taskflow/taskflow-runner.json`.
  - **Environment Check**: The command must bail with an error if not executed within a directory containing a `.git/` folder.
  - **Ignore Management**: Automatically add `.taskflow/worktrees` to the project's `.gitignore` file. If `.gitignore` does not exist, create it.
- Provide `taskflow-runner dispatch` to run a watch loop that repeatedly asks TaskFlowAI for the next task and sends it to an enabled provider.
- Execute each task in a dedicated Git worktree under `.taskflow/worktrees/<task-uid>`.
- Support config-driven provider definitions for CLIs such as Claude, Gemini, Dirac, and OpenCode.
- Detect provider failures, timeouts, and missing prerequisites with actionable errors.
- Keep TaskFlowAI and Git as the source of truth for task state and code changes.

### Non-Goals

- Parallel task execution.
- A daemon, web UI, or remote execution service.
- Provider-specific deep integrations beyond process execution and exit status handling.
- Automatic merge, review, or conflict resolution after a provider finishes.
- Replacing TaskFlowAI roadmap files or metadata semantics.

## 3. Architecture

The runner is organized as a small orchestration pipeline around external systems. TaskFlowAI owns roadmap state, Git owns code isolation and review state, and provider CLIs own autonomous code execution. The runner coordinates those systems without introducing a second project database.

The sequential dispatch flow is:

1. Load and validate configuration.
2. Ensure the current directory is a Git repository with a TaskFlowAI project.
3. Ask TaskFlowAI for the next available task.
4. If no task is available, sleep and continue watching.
5. Create or reuse the task worktree.
6. Mark task execution as started.
7. Invoke the configured provider with the standard prompt.
8. Run TaskFlow validation.
9. Mark execution success or failure with a concise log.
10. Sync roadmap output.
11. Continue the loop unless `--once` is provided.

TaskFlowAI remains the authoritative task state store. Git remains the authoritative code state store. The runner may keep only derived operational data:

- Runtime logs emitted to stdout/stderr.
- Worktree directories under `.taskflow/worktrees`.
- Provider process metadata included in TaskFlow execution logs.

The global config is user preference, not project state.

### Error Handling

The runner should distinguish these failure classes:

- Configuration errors: invalid JSON, no enabled provider, missing command.
- Environment errors: not in a Git repository, missing `.taskflow`, missing `taskflow-ai`.
- Worktree errors: branch already exists with incompatible state, path collision, Git command failure.
- Provider errors: executable not found, non-zero exit, timeout.
- TaskFlow errors: unable to parse next task, validation failure, completion update failure.

Failures should be logged with enough context to continue manual recovery. A provider or validation failure should leave the worktree in place and mark the TaskFlow execution as failed when possible.

## 4. Components

### CLI Layer

The CLI exposes the initial command surface:

- `taskflow-runner init`
- `taskflow-runner dispatch`
- `taskflow-runner config validate`

The CLI layer parses flags, loads configuration, validates the repository context, and delegates to the dispatcher. It should keep command behavior explicit and avoid hidden state changes outside the config file and `.taskflow/worktrees`.

**`taskflow-runner init` Details**:
The `init` command acts as a setup wizard. It should:
1. Verify the presence of a `.git/` directory.
2. Update or create `.gitignore` to include `.taskflow/worktrees`.
3. Configure providers by either:
   - Prompting the user to select from a list of known providers.
   - Automatically configuring all detected providers, with the default enabled provider falling to alphabetical order.

### Configuration

Global configuration lives at `~/.taskflow/taskflow-runner.json`.

The initial schema should include:

- `providers`: named provider definitions.
- `enabled_provider`: the provider selected for sequential dispatch.
- `default_timeout_seconds`: maximum provider execution time.
- `taskflow_command`: default `taskflow-ai` executable name or path.
- `worktree_root`: default `.taskflow/worktrees`.

Each provider definition should include:

- `command`: executable name or absolute path.
- `args_before_prompt`: static arguments such as headless flags.
- `prompt_arg_mode`: whether the prompt is passed as a positional argument or through stdin.
- `env`: optional environment overrides.
- `timeout_seconds`: optional provider-specific timeout.

The default prompt is:

```text
Run /taskflow and execute the next available task.
```

### TaskFlow Adapter

The TaskFlow adapter shells out to `taskflow-ai` and parses its command results. In the first version, it only needs narrow operations:

- Find the next task via `taskflow-ai next`.
- Start execution via `taskflow-ai execute start <task-id> --agent <provider>`.
- Validate after provider completion via `taskflow-ai validate <task-id>`.
- Complete or fail execution via `taskflow-ai execute complete <task-id> --outcome <success|failure> --log <message>`.
- Sync generated roadmap output via `taskflow-ai sync`.

The adapter should isolate command execution and parsing so future versions can replace text parsing with structured output if TaskFlowAI exposes it.

### Git Worktree Manager

The worktree manager owns the lifecycle for per-task directories:

1. Resolve the current repository root.
2. Derive a stable worktree path from the TaskFlow task UID when available; otherwise use the task ID.
3. Create a branch such as `taskflow/<task-id>` from the current base branch.
4. Create the worktree at `.taskflow/worktrees/<task-uid>`.
5. Run provider commands inside that worktree.
6. Leave the worktree intact after execution for review and follow-up.

Cleanup should be explicit in the initial release. Automatic pruning is limited to stale or failed setup artifacts that are known to be empty.

### Provider Driver

The provider driver builds a process invocation from configuration and runs it in the task worktree. It captures stdout, stderr, exit status, start time, end time, and timeout state.

The first version treats provider behavior uniformly:

- Exit code `0` means the provider process completed.
- Non-zero exit code means provider failure.
- Timeout means provider failure with a distinct diagnostic.
- Successful provider exit still requires TaskFlow validation before the runner marks the task successful.

## Initial Task Breakdown

1. Create the Rust CLI crate and command parsing.
2. Implement config loading, defaults, and `init` wizard output.
3. Implement process execution utilities with timeout support.
4. Implement the TaskFlow adapter for next/start/validate/complete/sync.
5. Implement Git repository and worktree management.
6. Implement provider driver construction from config.
7. Implement `dispatch --once` and the sequential watch loop.
8. Add tests around config parsing, provider command construction, TaskFlow output parsing, and worktree path derivation.
9. Add README usage examples for init, config, and dispatch.

## Open Questions

- Should the initial branch name include task UID to avoid collisions when task IDs are reused across archived roadmaps?
- Should provider prompts be configurable per project, or only globally?
- Should successful provider runs automatically create commits in the task worktree, or leave commits to the provider and reviewer?
- Should `dispatch` default to watch mode, or require `--watch` with `--once` as the default for early releases?

## Acceptance Criteria

- The design describes the initial command surface, config model, TaskFlow integration, worktree lifecycle, provider execution, and failure modes.
- The design explicitly scopes out parallel execution and automatic merge behavior.
- The implementation roadmap is split into concrete follow-up tasks.
- `taskflow-ai validate TF-1` succeeds.
