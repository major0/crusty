# Task Orchestration Workflow - Requirements

## Overview

A hook-based orchestration system that automates the complete task execution lifecycle through Kiro's native capabilities. The workflow is initiated by user prompt, handles branch creation, implementation, comprehensive quality assurance, PR submission with clean commit history, and CI/CD monitoring with automatic fixes.

## Key Principle

**Zero Custom Code**: The entire workflow is implemented using:
- **Hooks** (`.kiro/hooks/*.json`) - Trigger agents based on promptSubmit and agentStop events
- **Steering Files** (`.kiro/steering/*.md`) - Guide agent behavior
- **Existing Kiro Tools** - invokeSubAgent, taskStatus, git, gh cli

## Workflow Architecture

The workflow consists of 4 sequential hooks:

1. **Hook 1 (promptSubmit)**: Task initiation, branch creation, implementation
2. **Hook 2 (agentStop)**: Quality assurance with 10 parallel sub-agents
3. **Hook 3 (agentStop)**: Task completion, commit consolidation, PR submission
4. **Hook 4 (agentStop)**: CI/CD monitoring and automatic fixes

## User Stories

### 1. As a developer, I want to start a task by simply asking the agent, and have it automatically set up the proper branch and begin implementation

**Acceptance Criteria:**
- 1.1 Hook triggers on promptSubmit event
- 1.2 Hook detects if prompt is requesting to start a task from tasks.md
- 1.3 Hook marks task as "in_progress" using taskStatus tool
- 1.4 Hook creates topic branch: `git checkout -b <type>/<task-id>-<description> origin/main`
- 1.5 Hook ensures branch is up-to-date with origin/main before starting
- 1.6 Hook sets upstream tracking to origin/main
- 1.7 Agent implements task according to requirements and design
- 1.8 Agent creates ONE implementation commit following Conventional Commits

**Implementation**: Hook `.kiro/hooks/on-task-start.kiro.hook` with promptSubmit trigger

### 2. As a developer, I want comprehensive quality checks to run automatically after implementation with maximum parallelization

**Acceptance Criteria:**
- 2.1 Hook triggers on agentStop event after implementation
- 2.2 Hook detects if previous work was task implementation (checks for feat/fix commit)
- 2.3 Hook launches 10 sub-agents in parallel (maximum allowed):
  - 2.3.1 Documentation agent - Update inline comments, README, API docs
  - 2.3.2 Property testing agent - Create property-based tests (100+ iterations)
  - 2.3.3 Unit testing agent - Create unit tests for specific cases
  - 2.3.4 Coverage testing agent - Verify 90%+ coverage (statements, branches, functions, lines)
  - 2.3.5 Linting agent - Run ESLint/TSLint with auto-fix
  - 2.3.6 Formatting agent - Run Prettier with auto-fix
  - 2.3.7 Pre-commit validation agent - Run all pre-commit hooks
  - 2.3.8 Security audit agent - Check for vulnerabilities, secrets, injection risks
  - 2.3.9 Type checking agent - Verify TypeScript compilation
  - 2.3.10 Build verification agent - Verify project builds successfully
- 2.4 Each agent commits changes independently with appropriate commit type
- 2.5 Each agent references appropriate steering document for detailed instructions
- 2.6 Hook waits for all 10 agents to complete before proceeding

**Implementation**: Hook `.kiro/hooks/after-implementation.kiro.hook` with agentStop trigger

### 3. As a developer, I want the PR to have a clean commit history with all changes consolidated into logical commits

**Acceptance Criteria:**
- 3.1 Hook triggers on agentStop event after all quality agents complete
- 3.2 Hook detects that quality agents have finished (checks for docs/test/chore commits)
- 3.3 Hook validates all quality agents completed successfully
- 3.4 Hook marks task as "completed" using taskStatus tool
- 3.5 Hook performs soft reset against origin/main: `git reset --soft origin/main`
- 3.6 Hook re-commits all changes with clean, consolidated commits:
  - 3.6.1 Implementation commit: `<type>(<scope>): <task-id> <task-title>`
  - 3.6.2 Testing commit: `test(<scope>): add comprehensive tests for <task-context>`
  - 3.6.3 Documentation commit: `docs(<scope>): update documentation for <task-context>`
  - 3.6.4 Quality commit: `chore(<scope>): apply code quality fixes for <task-context>`
- 3.7 Hook pushes branch to origin
- 3.8 Hook creates PR using gh cli with generated title and body
- 3.9 Hook adds appropriate labels based on commit types

**Implementation**: Hook `.kiro/hooks/after-quality-agents.kiro.hook` with agentStop trigger

### 4. As a developer, I want CI/CD failures to be automatically detected and fixed so that PRs pass all checks

**Acceptance Criteria:**
- 4.1 Hook triggers on agentStop event after PR creation/update
- 4.2 Hook detects if previous work was PR submission (checks for gh pr create/push)
- 4.3 Hook monitors CI/CD status: `gh pr checks <pr-number> --watch`
- 4.4 If all checks pass, hook reports success and exits
- 4.5 If checks fail, hook analyzes failure logs
- 4.6 Hook attempts to fix failures:
  - 4.6.1 Pull latest changes from origin/main if needed
  - 4.6.2 Fix identified issues (test failures, linting, coverage, etc.)
  - 4.6.3 Soft reset against origin/main: `git reset --soft origin/main`
  - 4.6.4 Re-commit all changes with clean commits (same structure as 3.6)
  - 4.6.5 Force push to PR branch: `git push --force-with-lease`
- 4.7 Hook repeats monitoring after push (max 3 fix attempts)
- 4.8 If fixes fail after 3 attempts, hook reports to user with detailed error info

**Implementation**: Hook `.kiro/hooks/after-pr-submission.kiro.hook` with agentStop trigger

## Non-Functional Requirements

### Performance

- **NFR-1**: Complete workflow should execute in 10-25 minutes for typical tasks
- **NFR-2**: Quality agents must execute in parallel (10 simultaneous agents)
- **NFR-3**: Parallel execution should achieve 60-70% time savings vs sequential
- **NFR-4**: CI/CD monitoring should timeout after 30 minutes

### Reliability

- **NFR-5**: If individual quality agent fails, others must continue
- **NFR-6**: Workflow must be idempotent (safe to re-run)
- **NFR-7**: Workflow must be resumable from any point
- **NFR-8**: Soft reset must preserve all changes while cleaning history

### Usability

- **NFR-9**: Steering files must provide clear, actionable instructions
- **NFR-10**: Workflow must be customizable via hook/steering file edits
- **NFR-11**: All configuration files must be valid JSON/Markdown
- **NFR-12**: Error messages must be clear and actionable

### Maintainability

- **NFR-13**: Zero custom code (only JSON hooks and Markdown steering)
- **NFR-14**: Hooks must be self-documenting with clear prompts
- **NFR-15**: Steering files must be modular and focused

## Configuration Files

### Hooks (4 files in `.kiro/hooks/`)

1. **on-task-start.kiro.hook** - Triggers on promptSubmit, initiates task workflow
2. **after-implementation.kiro.hook** - Triggers on agentStop, launches 10 quality agents
3. **after-quality-agents.kiro.hook** - Triggers on agentStop, consolidates commits and creates PR
4. **after-pr-submission.kiro.hook** - Triggers on agentStop, monitors CI/CD and fixes failures

### Steering Files (11 files in `.kiro/steering/`)

1. **task-initiation-agent.md** - Guidance for task detection and branch setup
2. **documentation-agent.md** - Guidance for documentation updates
3. **property-testing-agent.md** - Guidance for property-based tests
4. **unit-testing-agent.md** - Guidance for unit tests
5. **coverage-testing-agent.md** - Guidance for coverage verification
6. **linting-agent.md** - Guidance for linting
7. **formatting-agent.md** - Guidance for code formatting
8. **pre-commit-agent.md** - Guidance for pre-commit validation
9. **security-agent.md** - Guidance for security auditing
10. **type-checking-agent.md** - Guidance for TypeScript type checking
11. **pr-submission-agent.md** - Guidance for PR creation with clean commit history

## Constraints

- **C-1**: Must use only Kiro native capabilities (hooks, steering, tools)
- **C-2**: Must not require custom TypeScript/JavaScript code
- **C-3**: Must not require build process or compilation
- **C-4**: Must work with existing Kiro CLI and tools
- **C-5**: Must follow Conventional Commits format
- **C-6**: Must maintain clean PR commit history (no iterative fix commits)
- **C-7**: Must use soft reset to consolidate commits before pushing

## Success Metrics

- **SM-1**: Zero lines of custom code required
- **SM-2**: 4 hook files + 11 steering files = 15 configuration files total
- **SM-3**: 10 quality agents execute in parallel
- **SM-4**: Complete workflow executes in 10-25 minutes
- **SM-5**: 90%+ test coverage achieved automatically
- **SM-6**: PRs have clean, consolidated commit history (4 commits max)
- **SM-7**: CI/CD failures fixed automatically in 80%+ of cases
- **SM-8**: Workflow is customizable by editing configuration files
- **SM-9**: All commits follow Conventional Commits format
- **SM-10**: No "WIP", "fix-up", or iterative commits in PR history
