# Task Orchestration Workflow

A hook-based automation system that manages the complete lifecycle of task execution in Kiro, from branch creation through CI/CD monitoring—all without writing a single line of custom code.

## Overview

The Task Orchestration Workflow automates the entire task execution process using only Kiro's native capabilities:

- **Hooks** (`.kiro/hooks/*.kiro.hook`) - JSON configuration files that trigger agents based on events
- **Steering Files** (`.kiro/steering/*.md`) - Markdown documents that guide agent behavior
- **Existing Kiro Tools** - Built-in tools like `invokeSubAgent`, `taskStatus`, `git`, and `gh cli`

### What It Does

When you ask Kiro to "start task 1.2", the workflow automatically:

1. ✅ Creates a properly named topic branch based on `origin/main`
2. ✅ Marks the task as "in progress" in your tasks.md
3. ✅ Implements the task according to requirements and design
4. ✅ Creates one clean implementation commit
5. ✅ Runs 10 quality assurance agents in parallel:
   - Documentation updates
   - Property-based tests (100+ iterations)
   - Unit tests for edge cases
   - Coverage verification (90%+ threshold)
   - Linting with auto-fix
   - Code formatting
   - Pre-commit validation
   - Security auditing
   - Type checking
   - Build verification
6. ✅ Consolidates all commits into clean, logical history (4 commits max)
7. ✅ Creates a pull request with proper metadata and labels
8. ✅ Monitors CI/CD checks and automatically fixes failures

All of this happens automatically, with no custom code required.

## Architecture

### Hook Chain

The workflow consists of 4 sequential hooks that trigger based on events:

```
User: "start task 1.2"
         ↓
    [promptSubmit event]
         ↓
┌────────────────────────────┐
│ Hook 1: on-task-start      │
│ - Parse task reference     │
│ - Create topic branch      │
│ - Implement task           │
│ - Create 1 commit          │
└────────────────────────────┘
         ↓
    [agentStop event]
         ↓
┌────────────────────────────┐
│ Hook 2: after-impl         │
│ - Launch 10 QA agents      │
│   (parallel execution)     │
│ - Each creates 1 commit    │
└────────────────────────────┘
         ↓
    [agentStop event]
         ↓
┌────────────────────────────┐
│ Hook 3: after-quality      │
│ - Soft reset to origin/main│
│ - Consolidate to 4 commits │
│ - Push branch              │
│ - Create PR with metadata  │
└────────────────────────────┘
         ↓
    [agentStop event]
         ↓
┌────────────────────────────┐
│ Hook 4: after-pr           │
│ - Monitor CI/CD checks     │
│ - Auto-fix failures        │
│ - Re-consolidate & push    │
│ - Retry up to 3 times      │
└────────────────────────────┘
         ↓
    PR ready for review!
```

### Component Structure

```
.kiro/
├── hooks/                          # Hook configuration files
│   ├── on-task-start.kiro.hook     # Hook 1: Task initiation
│   ├── after-implementation.kiro.hook  # Hook 2: Quality agents
│   ├── after-quality-agents.kiro.hook  # Hook 3: PR submission
│   └── after-pr-submission.kiro.hook   # Hook 4: CI/CD monitoring
│
└── steering/                       # Agent guidance documents
    ├── task-initiation-agent.md    # Task detection & branch setup
    ├── documentation-agent.md      # Documentation updates
    ├── property-testing-agent.md   # Property-based tests
    ├── unit-testing-agent.md       # Unit tests
    ├── coverage-testing-agent.md   # Coverage verification
    ├── linting-agent.md            # Linting with auto-fix
    ├── formatting-agent.md         # Code formatting
    ├── pre-commit-agent.md         # Pre-commit validation
    ├── security-agent.md           # Security auditing
    ├── type-checking-agent.md      # Type checking
    ├── build-verification-agent.md # Build verification
    ├── pr-submission-agent.md      # PR creation
    └── ci-monitoring-agent.md      # CI/CD monitoring
```

## Getting Started

### Prerequisites

- Kiro CLI installed and configured
- Git repository with `origin/main` branch
- GitHub CLI (`gh`) authenticated (for PR creation)
- Task specification files in `.kiro/specs/<spec-name>/`:
  - `tasks.md` - Task list
  - `requirements.md` - Requirements (optional)
  - `design.md` - Design document (optional)

### Installation

The workflow is already installed if you have the hook and steering files in your `.kiro/` directory. No additional setup required!

### Basic Usage

1. **Start a task:**
   ```
   You: "start task 1.2"
   ```

2. **Watch the workflow execute:**
   - Branch creation
   - Implementation
   - Quality assurance (10 agents in parallel)
   - PR creation
   - CI/CD monitoring

3. **Review the PR:**
   - Check the PR on GitHub
   - Review the clean commit history (4 commits)
   - Verify all CI/CD checks pass

That's it! The workflow handles everything automatically.

## Workflow Execution Flow

### Phase 1: Task Initiation (Hook 1)

**Trigger:** User prompt containing "start task X"

**What Happens:**
1. Agent parses the prompt to extract task ID (e.g., "1.2")
2. Agent reads task details from `.kiro/specs/<spec-name>/tasks.md`
3. Agent marks task as "in_progress" using `taskStatus` tool
4. Agent creates topic branch:
   ```bash
   git checkout -b feat/1.2-user-login origin/main
   git branch --set-upstream-to=origin/main
   ```
5. Agent implements the task according to requirements and design
6. Agent creates ONE implementation commit:
   ```
   feat(auth): 1.2 Implement user login endpoint
   ```

**Output:** Topic branch with one implementation commit

### Phase 2: Quality Assurance (Hook 2)

**Trigger:** Agent stops after implementation (detects feat/fix/refactor/perf commit)

**What Happens:**
1. Hook detects implementation is complete
2. Hook launches 10 quality agents **in parallel**:

   | Agent | Purpose | Commit Type |
   |-------|---------|-------------|
   | Documentation | Update comments, README, API docs | `docs` |
   | Property Testing | Create property tests (100+ iterations) | `test` |
   | Unit Testing | Create unit tests for edge cases | `test` |
   | Coverage Testing | Verify 90%+ coverage | `test` |
   | Linting | Run ESLint/TSLint with auto-fix | `chore` |
   | Formatting | Run Prettier with auto-fix | `chore` |
   | Pre-commit | Run all pre-commit hooks | `chore` |
   | Security | Check for vulnerabilities, secrets | `chore` |
   | Type Checking | Verify TypeScript compilation | `chore` |
   | Build Verification | Verify project builds | `chore` |

3. Each agent commits changes independently
4. Hook waits for all 10 agents to complete

**Output:** 10 additional commits (docs, test, chore types)

**Time Savings:** Parallel execution achieves 60-70% time savings vs sequential execution

### Phase 3: PR Submission (Hook 3)

**Trigger:** Agent stops after quality agents (detects docs/test/chore commits)

**What Happens:**
1. Hook validates all quality agents completed
2. Hook marks task as "completed" using `taskStatus` tool
3. Hook performs **soft reset** to consolidate commits:
   ```bash
   git reset --soft origin/main
   ```
   *(This preserves all changes but removes commit history)*

4. Hook re-commits with clean structure (4 commits):
   ```
   Commit 1: feat(auth): 1.2 Implement user login endpoint
   Commit 2: test(auth): add comprehensive tests for user authentication
   Commit 3: docs(auth): update documentation for authentication system
   Commit 4: chore(auth): apply code quality fixes for authentication
   ```

5. Hook pushes branch:
   ```bash
   git push -u origin HEAD
   ```

6. Hook creates PR using GitHub CLI:
   ```bash
   gh pr create --title "feat(auth): Implement user login endpoint" --body "..."
   ```

7. Hook adds labels based on commit types:
   - `enhancement` (for feat)
   - `testing` (for test commits)
   - `documentation` (for docs commits)
   - `maintenance` (for chore commits)

**Output:** PR created with clean commit history and proper metadata

### Phase 4: CI/CD Monitoring (Hook 4)

**Trigger:** Agent stops after PR creation (detects `git push` or `gh pr create`)

**What Happens:**
1. Hook identifies PR number for current branch
2. Hook monitors CI/CD checks:
   ```bash
   gh pr checks <pr-number> --watch
   ```

3. **If all checks pass:**
   - Report success and exit

4. **If checks fail:**
   - Retrieve failure logs
   - Analyze failures (test failures, linting, coverage, etc.)
   - Pull latest from `origin/main` if needed
   - Apply fixes
   - Soft reset to `origin/main`
   - Re-create clean commit structure (4 commits)
   - Force push with lease:
     ```bash
     git push --force-with-lease origin HEAD
     ```
   - Restart monitoring

5. **Retry logic:**
   - Maximum 3 fix attempts
   - If fixes fail after 3 attempts, report to user with detailed logs

6. **Timeout:**
   - If checks don't complete in 30 minutes, timeout and report status

**Output:** All CI/CD checks passing, or detailed error report if unfixable

## Customization

### Adding a New Quality Agent

To add a new quality agent to the workflow:

1. **Create a steering file** in `.kiro/steering/`:
   ```markdown
   # My Custom Agent
   
   ## Purpose
   What this agent does
   
   ## Context
   What information it has access to
   
   ## Instructions
   Step-by-step guidance
   
   ## Commit Format
   Expected commit message format
   
   ## Success Criteria
   How to know it succeeded
   
   ## Error Handling
   What to do if things go wrong
   ```

2. **Update Hook 2** (`.kiro/hooks/after-implementation.kiro.hook`):
   - Add your agent to the list in the prompt
   - Reference your steering file
   - **Note:** Kiro supports maximum 10 parallel agents, so you may need to replace an existing agent

3. **Test the agent:**
   - Start a test task
   - Verify your agent executes correctly
   - Verify it creates the expected commit

### Modifying Hook Behavior

Hooks are JSON configuration files that you can edit directly:

**Example: Change when Hook 1 activates**

Edit `.kiro/hooks/on-task-start.kiro.hook`:
```json
{
  "when": {
    "type": "promptSubmit"
  }
}
```

Change to activate on a different event:
```json
{
  "when": {
    "type": "userTriggered"
  }
}
```

**Example: Modify agent instructions**

Edit the `prompt` field in any hook to change what the agent does:
```json
{
  "then": {
    "type": "askAgent",
    "prompt": "Your custom instructions here..."
  }
}
```

### Disabling Quality Agents

To disable a specific quality agent:

1. **Edit Hook 2** (`.kiro/hooks/after-implementation.kiro.hook`)
2. **Remove the agent** from the prompt list
3. **Save the file**

Example - Remove the security agent:
```json
{
  "prompt": "Launch 9 quality agents:\n1. Documentation\n2. Property Testing\n...\n(remove Security Audit line)\n..."
}
```

### Customizing Commit Messages

To change commit message format:

1. **Edit the relevant steering file** (e.g., `.kiro/steering/linting-agent.md`)
2. **Update the "Commit Format" section**
3. **Save the file**

Example:
```markdown
## Commit Format

```
chore(<scope>): [LINT] apply linting fixes for <context>
```
```

### Changing Branch Naming

To change branch naming pattern:

1. **Edit** `.kiro/steering/task-initiation-agent.md`
2. **Update Step 3** with your preferred pattern
3. **Save the file**

Example - Use different pattern:
```markdown
### Step 3: Create Topic Branch

Create the branch:
```bash
git checkout -b task-<task-id>-<description> origin/main
```
```

### Adjusting CI/CD Monitoring

To change CI/CD monitoring behavior:

1. **Edit** `.kiro/steering/ci-monitoring-agent.md`
2. **Modify timeout, retry count, or monitoring strategy**
3. **Save the file**

Example - Change timeout to 60 minutes:
```markdown
**Timeout:**
- If checks don't complete in 60 minutes, timeout and report status
```

## Common Customization Examples

### Example 1: Skip Documentation Agent

**Use Case:** Your project doesn't require documentation updates for every task

**Solution:**
1. Edit `.kiro/hooks/after-implementation.kiro.hook`
2. Remove "Documentation Agent" from the list
3. Update count from "10 quality agents" to "9 quality agents"

### Example 2: Add Custom Test Agent

**Use Case:** You want to run integration tests as a separate agent

**Solution:**
1. Create `.kiro/steering/integration-testing-agent.md`:
   ```markdown
   # Integration Testing Agent
   
   ## Purpose
   Run integration tests to verify system components work together
   
   ## Instructions
   1. Run integration test suite: `npm run test:integration`
   2. If tests fail, analyze failures and fix
   3. Commit changes: `test(<scope>): add integration tests for <context>`
   ```

2. Edit `.kiro/hooks/after-implementation.kiro.hook`
3. Replace one of the existing agents with your integration testing agent

### Example 3: Change PR Title Format

**Use Case:** You want PR titles to include the task ID in brackets

**Solution:**
1. Edit `.kiro/steering/pr-submission-agent.md`
2. Update the "PR Title Format" section:
   ```markdown
   **PR Title Format**:
   ```
   [<task-id>] <type>(<scope>): <task-title>
   ```
   
   **Example:** `[1.2] feat(auth): Implement user login endpoint`
   ```

### Example 4: Disable CI/CD Auto-Fix

**Use Case:** You want to manually fix CI/CD failures

**Solution:**
1. Edit `.kiro/hooks/after-pr-submission.kiro.hook`
2. Change the prompt to only monitor, not fix:
   ```json
   {
     "prompt": "Monitor CI/CD checks and report status. Do not attempt automatic fixes. Report all failures to the user."
   }
   ```

### Example 5: Add Slack Notifications

**Use Case:** You want to receive Slack notifications when workflow completes

**Solution:**
1. Create `.kiro/steering/notification-agent.md`:
   ```markdown
   # Notification Agent
   
   ## Purpose
   Send Slack notification when workflow completes
   
   ## Instructions
   1. Check if PR was created successfully
   2. Send Slack message: `curl -X POST -H 'Content-type: application/json' --data '{"text":"PR created: <pr-url>"}' <webhook-url>`
   ```

2. Edit `.kiro/hooks/after-pr-submission.kiro.hook`
3. Add notification step to the prompt

## Troubleshooting

### Issue: Hook doesn't trigger

**Symptoms:**
- You say "start task 1.2" but nothing happens
- Hook should activate but doesn't

**Possible Causes:**
1. Hook is disabled in configuration
2. Event type doesn't match
3. Context detection logic doesn't match current state

**Solutions:**
1. Check hook is enabled:
   ```json
   {
     "enabled": true
   }
   ```

2. Verify event type matches:
   - Hook 1: `"type": "promptSubmit"`
   - Hooks 2-4: `"type": "agentStop"`

3. Check context detection:
   - Hook 1: Prompt must contain "start task" or similar
   - Hook 2: Must have feat/fix/refactor/perf commit
   - Hook 3: Must have docs/test/chore commits
   - Hook 4: Must have recent `git push` or `gh pr create`

### Issue: Quality agents fail

**Symptoms:**
- One or more quality agents report errors
- Commits are missing from expected agents

**Possible Causes:**
1. Tool not installed (ESLint, Prettier, etc.)
2. Configuration file missing (.eslintrc, .prettierrc, etc.)
3. Code has errors that can't be auto-fixed

**Solutions:**
1. Install missing tools:
   ```bash
   npm install --save-dev eslint prettier
   ```

2. Create configuration files if missing

3. Review agent error messages and fix manually:
   ```bash
   git log --oneline -20  # Check which agents succeeded
   ```

4. If agent fails, it won't block others - remaining agents continue

### Issue: Commit consolidation fails

**Symptoms:**
- Soft reset fails
- Consolidated commits are incorrect
- PR has wrong number of commits

**Possible Causes:**
1. Branch is not based on `origin/main`
2. Merge conflicts with `origin/main`
3. Git state is corrupted

**Solutions:**
1. Verify branch base:
   ```bash
   git merge-base HEAD origin/main
   ```

2. Pull latest from `origin/main`:
   ```bash
   git fetch origin
   git merge origin/main
   ```

3. If consolidation fails, reset manually:
   ```bash
   git reset --soft origin/main
   git status  # Verify all changes are staged
   ```

### Issue: PR creation fails

**Symptoms:**
- Branch is pushed but no PR created
- `gh pr create` fails with error

**Possible Causes:**
1. GitHub CLI not authenticated
2. PR already exists for branch
3. Branch protection rules prevent PR creation

**Solutions:**
1. Authenticate GitHub CLI:
   ```bash
   gh auth login
   ```

2. Check if PR exists:
   ```bash
   gh pr list --head $(git branch --show-current)
   ```

3. Check branch protection rules in GitHub settings

4. Create PR manually:
   ```bash
   gh pr create --title "..." --body "..."
   ```

### Issue: CI/CD monitoring times out

**Symptoms:**
- Monitoring runs for 30 minutes then stops
- Checks are still running

**Possible Causes:**
1. CI/CD pipeline is very slow
2. Checks are stuck or hanging
3. External service is down

**Solutions:**
1. Check CI/CD status on GitHub:
   ```bash
   gh pr checks <pr-number>
   ```

2. Increase timeout in `.kiro/steering/ci-monitoring-agent.md`:
   ```markdown
   **Timeout:**
   - If checks don't complete in 60 minutes, timeout
   ```

3. Cancel stuck checks and re-run:
   ```bash
   gh run cancel <run-id>
   gh run rerun <run-id>
   ```

### Issue: CI/CD auto-fix fails

**Symptoms:**
- Fixes are applied but checks still fail
- Maximum retry limit (3) reached

**Possible Causes:**
1. Fixes are incorrect or incomplete
2. New failures introduced by fixes
3. Failures require manual intervention

**Solutions:**
1. Review failure logs:
   ```bash
   gh pr checks <pr-number> --json name,conclusion,detailsUrl
   ```

2. Fix issues manually:
   ```bash
   # Make fixes
   git add .
   git commit -m "fix: address CI/CD failures"
   git push
   ```

3. Disable auto-fix if needed (see Customization section)

### Issue: Branch has wrong commit history

**Symptoms:**
- PR shows more than 4 commits
- Commits have wrong messages
- "WIP" or "fixup" commits visible

**Possible Causes:**
1. Soft reset didn't work correctly
2. Manual commits were added
3. Consolidation was skipped

**Solutions:**
1. Manually consolidate commits:
   ```bash
   git reset --soft origin/main
   git status  # Verify changes are staged
   
   # Re-create clean commits
   git commit -m "feat(scope): task-id task-title"
   git commit -m "test(scope): add comprehensive tests"
   git commit -m "docs(scope): update documentation"
   git commit -m "chore(scope): apply quality fixes"
   
   git push --force-with-lease
   ```

2. Use interactive rebase if needed:
   ```bash
   git rebase -i origin/main
   ```

### Issue: Task status not updating

**Symptoms:**
- Task remains "not_started" after starting
- Task not marked "completed" after PR creation

**Possible Causes:**
1. `taskStatus` tool failed
2. Task ID doesn't match exactly
3. tasks.md file format is incorrect

**Solutions:**
1. Check task ID matches exactly:
   ```bash
   cat .kiro/specs/<spec-name>/tasks.md | grep "1.2"
   ```

2. Update status manually:
   ```bash
   # Edit tasks.md and change:
   - [ ] 1.2 Task title
   # to:
   - [x] 1.2 Task title
   ```

3. Verify tasks.md format follows spec

### Issue: Parallel execution is slow

**Symptoms:**
- Quality agents take longer than expected
- Agents appear to run sequentially

**Possible Causes:**
1. System resource constraints
2. Agents are waiting for shared resources
3. Network latency for external tools

**Solutions:**
1. Check system resources:
   ```bash
   top  # Check CPU and memory usage
   ```

2. Reduce number of parallel agents if needed

3. Optimize agent execution:
   - Limit linting/formatting to changed files only
   - Use local caches for dependencies
   - Skip slow checks if not critical

### Getting Help

If you encounter issues not covered here:

1. **Check agent logs:**
   ```bash
   git log --oneline -20  # See what agents completed
   ```

2. **Review steering files:**
   - Read the relevant steering file in `.kiro/steering/`
   - Check if instructions are clear

3. **Inspect hook configuration:**
   - Open the relevant hook file in `.kiro/hooks/`
   - Verify JSON is valid
   - Check prompt is correct

4. **Test manually:**
   - Try running the agent's commands manually
   - Verify tools are installed and working

5. **Ask for help:**
   - Provide error messages
   - Share relevant logs
   - Describe what you expected vs what happened

## Performance

### Execution Time

Typical workflow execution times:

| Phase | Time | Notes |
|-------|------|-------|
| Task Initiation | 2-5 min | Depends on implementation complexity |
| Quality Assurance | 5-10 min | 10 agents in parallel |
| PR Submission | 1-2 min | Commit consolidation and PR creation |
| CI/CD Monitoring | 5-15 min | Depends on CI/CD pipeline speed |
| **Total** | **10-25 min** | For typical tasks |

### Optimization Tips

1. **Limit scope of quality agents:**
   - Run linting only on changed files
   - Run tests only for affected modules
   - Skip optional checks if not needed

2. **Use caching:**
   - Enable dependency caching in CI/CD
   - Use local caches for npm, pip, cargo, etc.

3. **Parallelize CI/CD checks:**
   - Configure CI/CD to run checks in parallel
   - Use matrix builds for multiple environments

4. **Reduce retry attempts:**
   - If CI/CD is reliable, reduce max retries from 3 to 1
   - Edit `.kiro/steering/ci-monitoring-agent.md`

## Best Practices

### For Task Implementation

1. **Keep tasks focused:**
   - One task = one feature or fix
   - Avoid combining multiple unrelated changes

2. **Follow requirements:**
   - Read requirements.md and design.md before implementing
   - Ensure implementation satisfies all acceptance criteria

3. **Write clean code:**
   - Follow project conventions
   - Add comments for complex logic
   - Keep functions small and focused

### For Quality Assurance

1. **Trust the agents:**
   - Let quality agents do their job
   - Don't manually fix issues that agents can handle

2. **Review agent commits:**
   - Check what each agent changed
   - Verify changes are correct

3. **Fix critical issues first:**
   - If agents report errors, fix critical ones first
   - Non-critical issues can be addressed later

### For PR Submission

1. **Review before merging:**
   - Check the consolidated commit history
   - Verify PR description is accurate
   - Ensure all CI/CD checks pass

2. **Keep PRs small:**
   - Smaller PRs are easier to review
   - Break large tasks into smaller sub-tasks

3. **Respond to feedback:**
   - Address review comments promptly
   - Push fixes to the same branch (workflow will re-consolidate)

### For CI/CD Monitoring

1. **Monitor actively:**
   - Watch CI/CD checks as they run
   - Be ready to intervene if auto-fix fails

2. **Understand failures:**
   - Read failure logs carefully
   - Understand root cause before fixing

3. **Improve CI/CD:**
   - If checks fail frequently, improve CI/CD configuration
   - Add better error messages
   - Make checks more reliable

## FAQ

### Q: Can I use this workflow without GitHub?

**A:** Partially. The workflow uses GitHub CLI (`gh`) for PR creation and CI/CD monitoring (Hooks 3 and 4). If you're not using GitHub:
- Hooks 1 and 2 will work (task initiation and quality assurance)
- Hook 3 will fail at PR creation (you can create PRs manually)
- Hook 4 won't work (no CI/CD monitoring)

You can modify Hook 3 to skip PR creation and Hook 4 to use a different CI/CD monitoring tool.

### Q: Can I run this workflow on multiple tasks simultaneously?

**A:** No. The workflow is designed for one task at a time. Running multiple workflows simultaneously could cause:
- Branch conflicts
- Commit history corruption
- Hook activation conflicts

If you need to work on multiple tasks, complete one workflow before starting another.

### Q: What if I need to make changes after PR is created?

**A:** Just push changes to the same branch:
```bash
# Make changes
git add .
git commit -m "fix: address review feedback"
git push
```

The CI/CD monitoring hook (Hook 4) will detect the push and restart monitoring. The workflow will automatically consolidate commits again if needed.

### Q: Can I customize the 4-commit structure?

**A:** Yes. Edit `.kiro/steering/pr-submission-agent.md` and modify the commit consolidation instructions. For example, you could consolidate to 2 commits (implementation + quality) or 6 commits (separate test types).

### Q: What if a quality agent is not applicable to my project?

**A:** Disable it by removing it from Hook 2. For example, if your project doesn't use TypeScript, remove the Type Checking Agent from the list.

### Q: Can I add custom quality checks?

**A:** Yes, but you're limited to 10 parallel agents (Kiro's maximum). To add a custom agent:
1. Create a steering file for your agent
2. Replace one of the existing agents in Hook 2
3. Test your agent

### Q: What happens if my implementation takes multiple commits?

**A:** The workflow expects ONE implementation commit. If you make multiple commits during implementation:
- Hook 2 may trigger prematurely
- Commit consolidation will include all commits
- Final PR will still have clean history (4 commits)

Best practice: Make one implementation commit, or squash commits before quality agents run.

### Q: Can I skip the quality assurance phase?

**A:** Not recommended, but yes. You can disable Hook 2 by setting `"enabled": false` in `.kiro/hooks/after-implementation.kiro.hook`. However, this defeats the purpose of the workflow.

### Q: What if CI/CD checks take longer than 30 minutes?

**A:** The workflow will timeout and report the current status. You can:
1. Increase the timeout in `.kiro/steering/ci-monitoring-agent.md`
2. Continue monitoring manually: `gh pr checks <pr-number> --watch`
3. Optimize your CI/CD pipeline to run faster

### Q: Can I use this workflow with GitLab or Bitbucket?

**A:** Not directly. The workflow uses GitHub-specific tools (`gh cli`). To adapt for GitLab/Bitbucket:
1. Replace `gh pr create` with GitLab/Bitbucket CLI commands
2. Replace `gh pr checks` with appropriate CI/CD monitoring commands
3. Update Hook 3 and Hook 4 accordingly

### Q: What if I want different commit messages?

**A:** Edit the relevant steering file and update the "Commit Format" section. The agents will follow the new format.

### Q: Can I run quality agents sequentially instead of parallel?

**A:** Yes, but it will be much slower (60-70% slower). Edit Hook 2 and change the prompt to invoke agents sequentially using `invokeSubAgent` one at a time.

### Q: What if I need to customize the workflow for different task types?

**A:** You can create multiple hook files with different configurations:
- `on-task-start-feature.kiro.hook` - For feature tasks
- `on-task-start-bugfix.kiro.hook` - For bug fixes
- `on-task-start-refactor.kiro.hook` - For refactoring

Enable/disable hooks based on task type.

## License

This workflow is part of the Kiro project. See the main project license for details.

## Acknowledgments

This workflow was designed to leverage Kiro's native capabilities to their fullest, demonstrating that powerful automation doesn't require custom code—just thoughtful configuration.

Special thanks to the Kiro team for building a system that makes this level of automation possible through hooks and steering files alone.

---

**Need help?** Check the [Troubleshooting](#troubleshooting) section or review the [steering files](.kiro/steering/) for detailed agent instructions.

**Want to customize?** See the [Customization](#customization) section for examples and guidance.
