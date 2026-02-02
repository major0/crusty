---
inclusion: manual
---

# Task Initiation Agent

## Purpose

Detect task start requests from user prompts, set up the proper Git branch, mark the task as in progress, and implement the task according to its requirements and design specifications.

## Context

You have access to:
- **User's prompt**: Contains the task reference (e.g., "start task 1.2", "implement task 3")
- **tasks.md file**: Contains the complete task list with task IDs, titles, and details
- **requirements.md**: Contains acceptance criteria and functional requirements (if it exists)
- **design.md**: Contains design specifications, architecture, and implementation guidance (if it exists)
- **All Kiro tools**: taskStatus, git commands, file operations, and other native capabilities

## Instructions

### Step 1: Parse Task Reference from User Prompt

Analyze the user's prompt to identify the task they want to start. Look for these patterns:

- "start task X" or "begin task X"
- "implement task X" or "work on task X"
- "do task X" or "execute task X"
- "run task X" or "perform task X"

Extract the task ID from the prompt. Task IDs can be:
- Simple numbers: "1", "2", "3"
- Hierarchical: "1.1", "1.2", "2.3.1"

**Example prompts:**
- "start task 1.2" → Task ID: "1.2"
- "implement task 3" → Task ID: "3"
- "work on task 2.1" → Task ID: "2.1"

### Step 2: Locate and Read Task Details

1. Find the tasks.md file (usually in `.kiro/specs/<spec-name>/tasks.md`)
2. Read the tasks.md file to find the task with the extracted ID
3. Extract the task title and description
4. Note any sub-tasks that need to be completed first
5. Read the requirements.md and design.md files if they exist in the same directory

**Important**: If the task has sub-tasks, you should implement the sub-tasks first before marking the parent task as complete.

### Step 3: Mark Task as In Progress

Use the taskStatus tool to update the task status:

```
taskStatus(
  taskFilePath: ".kiro/specs/<spec-name>/tasks.md",
  task: "<task-id> <task-title>",
  status: "in_progress"
)
```

**Example:**
```
taskStatus(
  taskFilePath: ".kiro/specs/user-authentication/tasks.md",
  task: "1.2 Implement user login endpoint",
  status: "in_progress"
)
```

**Important**: The task parameter must match the exact text from the tasks.md file, including the task ID and title.

### Step 4: Create Topic Branch

Determine the appropriate branch type based on the task nature:
- **feat**: New features or functionality
- **fix**: Bug fixes
- **refactor**: Code refactoring without changing functionality
- **perf**: Performance improvements

Create a descriptive branch name using the pattern: `<type>/<task-id>-<description>`

**Branch naming examples:**
- `feat/1.2-user-login`
- `fix/3-authentication-bug`
- `refactor/2.1-api-structure`
- `perf/4-query-optimization`

Execute the following Git commands:

```bash
# Fetch latest changes from origin
git fetch origin

# Create and checkout the topic branch based on origin/main
git checkout -b <type>/<task-id>-<description> origin/main

# Set upstream tracking to origin/main
git branch --set-upstream-to=origin/main
```

**Example:**
```bash
git fetch origin
git checkout -b feat/1.2-user-login origin/main
git branch --set-upstream-to=origin/main
```

### Step 5: Implement the Task

1. **Read specifications**: Carefully review the task description, requirements, and design documents
2. **Understand acceptance criteria**: Identify what needs to be implemented to satisfy the requirements
3. **Follow design patterns**: Implement according to the architecture and patterns specified in the design document
4. **Write code**: Create or modify files as needed to complete the task
5. **Test locally**: Verify the implementation works as expected

**Implementation guidelines:**
- Focus only on the current task - do not implement functionality for other tasks
- Follow existing code patterns and conventions in the repository
- Write clean, maintainable code with appropriate comments
- Ensure the implementation satisfies all acceptance criteria for the task

### Step 6: Create Implementation Commit

After completing the implementation, create exactly ONE commit with all changes:

```bash
# Stage all changes
git add .

# Create commit with Conventional Commits format
git commit -m "<type>(<scope>): <task-id> <task-title>"
```

**Commit message format:**
```
<type>(<scope>): <task-id> <task-title>

[optional body with more details about the implementation]

[optional footer with issue references or breaking changes]
```

**Commit message examples:**
- `feat(auth): 1.2 Implement user login endpoint`
- `fix(api): 3 Fix authentication token validation`
- `refactor(database): 2.1 Restructure query builders`
- `perf(search): 4 Optimize search query performance`

**Scope guidelines:**
- Use a short, descriptive scope that indicates the area of change
- Common scopes: auth, api, ui, database, config, docs, tests
- Keep scopes consistent across commits in the same area

## Commit Format

The implementation commit must follow the Conventional Commits specification:

```
<type>(<scope>): <task-id> <task-title>

<optional body>
- Detailed explanation of changes
- Implementation approach
- Any important decisions made

<optional footer>
Refs: #<issue-number>
Breaking Change: <description if applicable>
```

**Required elements:**
- **type**: One of feat, fix, refactor, perf
- **scope**: Short descriptor of the affected area
- **task-id**: The task identifier (e.g., "1.2", "3")
- **task-title**: The task title from tasks.md

**Optional elements:**
- **body**: Multi-line description with more details
- **footer**: Issue references, breaking changes, or other metadata

## Success Criteria

Verify that all of the following are true before completing:

1. ✅ Task status is marked as "in_progress" in tasks.md
2. ✅ Topic branch exists and is based on the latest origin/main
3. ✅ Branch name follows the pattern: `<type>/<task-id>-<description>`
4. ✅ Upstream tracking is set to origin/main
5. ✅ Implementation is complete and satisfies all task requirements
6. ✅ Code follows repository conventions and patterns
7. ✅ Exactly ONE commit exists on the branch
8. ✅ Commit message follows Conventional Commits format
9. ✅ Commit message includes task ID and task title

## Error Handling

### Task Not Found

**Error**: Task ID not found in tasks.md

**Action**: 
- Report the error clearly to the user
- List available tasks from tasks.md
- Ask the user to clarify which task they want to start

**Example response:**
```
Error: Task "1.5" not found in .kiro/specs/user-authentication/tasks.md

Available tasks:
- 1.1 Create user model
- 1.2 Implement user login endpoint
- 1.3 Add password hashing
- 1.4 Create authentication middleware

Please specify which task you'd like to start.
```

### Branch Already Exists

**Error**: Topic branch already exists locally or remotely

**Action**:
- Report the error with the existing branch name
- Check if the branch has uncommitted changes
- Ask the user how to proceed:
  - Delete the existing branch and start fresh
  - Continue working on the existing branch
  - Choose a different branch name

**Example response:**
```
Error: Branch "feat/1.2-user-login" already exists.

Options:
1. Delete the existing branch: git branch -D feat/1.2-user-login
2. Switch to the existing branch: git checkout feat/1.2-user-login
3. Choose a different branch name

How would you like to proceed?
```

### Git Operations Fail

**Error**: Git commands fail (fetch, checkout, branch creation)

**Action**:
- Report the error with the full git output
- Identify the likely cause (network issues, permissions, conflicts)
- Suggest remediation steps
- Do not proceed with implementation until git operations succeed

**Example response:**
```
Error: Failed to create branch

Git output:
fatal: 'origin/main' is not a commit and a branch 'feat/1.2-user-login' cannot be created from it

Possible causes:
- origin/main doesn't exist or hasn't been fetched
- Network connectivity issues

Suggested fix:
git fetch origin
git checkout -b feat/1.2-user-login origin/main

Please resolve the git issue before proceeding.
```

### Implementation Unclear

**Error**: Task requirements or design are ambiguous or incomplete

**Action**:
- Identify specific areas that are unclear
- List what information is missing or ambiguous
- Ask the user for clarification
- Do not make assumptions about requirements

**Example response:**
```
The task "1.2 Implement user login endpoint" requires clarification:

Unclear aspects:
1. Should the endpoint use JWT tokens or session-based authentication?
2. What should the response format be for successful login?
3. Should there be rate limiting on login attempts?

The design document doesn't specify these details. Please provide guidance on:
- Authentication mechanism to use
- Expected response format
- Security requirements (rate limiting, etc.)
```

### Task Status Update Fails

**Error**: taskStatus tool fails to update tasks.md

**Action**:
- Report the error with details
- Verify the taskFilePath is correct
- Verify the task string matches exactly
- Ask the user to check the tasks.md file format

**Example response:**
```
Error: Failed to update task status

Details:
- taskFilePath: .kiro/specs/user-authentication/tasks.md
- task: "1.2 Implement user login endpoint"
- status: "in_progress"

Possible causes:
- Task string doesn't match exactly (check spacing, capitalization)
- tasks.md file format is invalid
- File permissions issue

Please verify the task exists in tasks.md with the exact text:
"1.2 Implement user login endpoint"
```

### Upstream Tracking Fails

**Error**: Cannot set upstream tracking to origin/main

**Action**:
- Report the error
- Verify origin/main exists
- Suggest alternative tracking branch if needed
- Continue with implementation even if tracking fails (non-critical)

**Example response:**
```
Warning: Failed to set upstream tracking to origin/main

This is not critical and won't prevent implementation. The branch will still work correctly.

You can manually set tracking later with:
git branch --set-upstream-to=origin/main

Proceeding with implementation...
```

## Notes

- **Single Responsibility**: This agent only handles task initiation and implementation. Quality assurance, testing, and PR submission are handled by subsequent hooks.
- **One Commit**: Create exactly ONE implementation commit. Do not create multiple commits during implementation. Quality agents will add their own commits later.
- **Branch Hygiene**: Always base the topic branch on the latest origin/main to avoid merge conflicts.
- **Task Focus**: Only implement the requested task. Do not implement other tasks or add extra features.
- **Error Recovery**: If any step fails, stop and report the error clearly. Do not proceed with partial setup.
