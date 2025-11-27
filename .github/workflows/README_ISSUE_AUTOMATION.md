# Automated Issue Creation for CI Failures

## Overview

This workflow automatically creates GitHub issues when the CI pipeline fails, making it easy to track and fix problems.

## How It Works

**Trigger**: Runs when the "CI" workflow completes with a failure status

**What It Does**:
1. Checks if an issue already exists for this workflow run
2. Gathers detailed failure information:
   - Failed job names
   - Failed step names
   - Duration of each job
   - Branch, commit, and triggerer info
3. Creates a new issue (or comments on existing one) with:
   - Intuitive title: `CI Failure: <job-name> on <branch>`
   - Detailed failure breakdown
   - Quick links to logs, commits, and branches
   - Suggested debugging actions
4. Auto-assigns to the person who triggered the workflow
5. Labels with `ci-failure` and `automated`

## Issue Format

**Title Examples**:
- `CI Failure: Test Suite on main`
- `CI Failure: Clippy (Lints) on feature/new-renderer`
- `CI Failure: 3 jobs failed on develop`

**Body Includes**:
- 🚨 Workflow details (name, run number, branch, commit)
- ❌ Failed jobs with duration and failed steps
- 📋 Quick links to logs and commits
- 🔧 Suggested debugging actions

## Features

✅ **No Duplicates**: Checks for existing open issues before creating new ones  
✅ **Smart Titles**: Context-aware titles based on failure type  
✅ **Auto-Assignment**: Assigns to workflow triggerer  
✅ **Follow-ups**: Comments on existing issues if failure repeats  
✅ **Detailed Context**: Shows exactly which jobs/steps failed  

## Permissions Required

```yaml
permissions:
  issues: write    # Create and update issues
  actions: read    # Read workflow run details
```

## Customization

To monitor additional workflows, update the `workflows` array:

```yaml
on:
  workflow_run:
    workflows: ["CI", "Build", "Deploy"]  # Add more workflows here
    types:
      - completed
```

## Example Issue

```markdown
## 🚨 Workflow Failed

**Workflow**: CI ([#42](link))
**Branch**: `main`
**Commit**: `abc1234`
**Triggered by**: @wilkerHop

### ❌ Test Suite
- **Duration**: 45s
- **Failed Steps**:
  - `Run tests`

### 📋 Quick Links
- [View Workflow Run](link)
- [View Commit](link)
- [View Branch](link)

### 🔧 Suggested Actions
1. Review the failed job logs above
2. Check if this is a flaky test or infrastructure issue
3. Review recent changes in commit `abc1234`
4. Re-run the workflow if it's a transient failure
```

## Labels Used

- `ci-failure`: Indicates this is a CI/CD failure issue
- `automated`: Indicates this issue was auto-generated

---

**Benefits**:
- Never miss a CI failure
- Automatic tracking and assignment
- Detailed context for faster debugging
- Prevents duplicate issue clutter
- Keeps team informed of build health
