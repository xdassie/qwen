# Planning Skill: Micro-Iteration Breakdown with Confidence Metrics

## Overview
This skill teaches you to break down complex software development tasks into small, testable micro-iterations with confidence scoring and automated verification steps.

---

## Core Principles

### 1. Micro-Iterations
Break any task into smallest possible units that:
- Can be completed in one sitting (5-30 minutes)
- Have a single clear goal
- Include verification step (cargo check, test, lint)
- Are reversible if they fail

### 2. Confidence Metrics
Assign confidence level (1-10) based on:
- **Low (1-4)**: Complex algorithms, unknown APIs, high bug risk
- **Medium (5-7)**: Moderate complexity, some unknowns
- **High (8-10)**: Simple, straightforward, low risk

### 3. Verification Steps
Every micro-iteration MUST include:
- Specific action(s)
- `cargo check` (or appropriate verification)
- Failure recovery steps

---

## Planning Workflow

### Step 1: Read Existing Code
```
- Understand current structure
- Identify entry points
- Find existing patterns
- Note dependencies
```

### Step 2: Define Requirements
```
- What needs to be built?
- User specifications
- Constraints and preferences
- Success criteria
```

### Step 3: Create Phases
Group related micro-iterations into phases:
- Phase 1: Foundation (data structures, enums)
- Phase 2: Core functionality
- Phase 3: Integration
- Phase 4: Polish/Testing

### Step 4: Break into Micro-Iterations
For each task:
1. Identify atomic action
2. Assign confidence level
3. Add verification step
4. Add failure recovery
5. Order by confidence (low to high)

### Step 5: Add Documentation
Include in plan:
- File structure
- Data structures
- Testing strategy
- Recovery procedures

---

## Template for Micro-Iteration

```markdown
**Micro-Iteration X.Y**: [Action verb] [what]
- **Confidence**: [N/10]
- [Brief description of risk/complexity]
- 1. [Action 1]
- 2. [Action 2]
- 3. Run `cargo check`
- 4. If fail: [recovery steps]
```

---

## Failure Recovery Patterns

### Missing Import
1. Check error message for exact type name
2. Add `use crate::...` or `use bevy::...`
3. Verify it's exported from parent module

### Type Mismatch
1. Check function signature
2. Verify parameter/return types
3. Ensure trait implementations exist

### API Change
1. Run `cargo doc` to check latest API
2. Search for function in docs
3. Match exact signature

### Unknown Behavior
1. Isolate the failing code
2. Comment out recent changes
3. Binary search for culprit
4. Revert and re-add incrementally

---

## Confidence Scoring Guide

| Score | Description | Examples |
|-------|-------------|----------|
| 8-10 | Simple, well-understood | Adding enum, updating import |
| 5-7 | Moderate, some unknowns | API integration, config parsing |
| 1-4 | Complex, high risk | Algorithm implementation, new file |

---

## Example Plan Structure

```markdown
# Project Title

## Implementation Plan

### Phase 1: [Lowest Confidence Tasks]

**Micro-Iteration 1.1**: [Task]
- **Confidence**: Low (3/10)
- [Risks]
- 1. [Action]
- 2. [Action]
- 3. Run `cargo check`
- 4. If fail: [recovery]

**Micro-Iteration 1.2**: [Task]
- **Confidence**: Medium (5/10)
- ...

### Phase 2: [Medium Confidence Tasks]

### Phase 3: [High Confidence Tasks]

## Verification Strategy
- [Testing approach]

## Recovery Procedures
- [Common failure patterns]
```

---

## Best Practices

### Do
- Start with lowest confidence items first
- Keep micro-iterations small and focused
- Always include verification step
- Document failure recovery
- Order by confidence (ascending)

### Don't
- Don't combine unrelated tasks
- Don't skip verification steps
- Don't make assumptions about APIs
- Don't bundle multiple changes in one iteration

---

## Quick Reference

| When to Use | Strategy |
|-------------|----------|
| New feature | Phase by confidence |
| Bug fix | Isolate, fix, verify |
| Refactoring | Small chunks, test each |
| Integration | One interface at a time |
| Algorithm | Skeleton first, then details |

---

## Notes
- Always run `cargo check` after each micro-iteration
- Document confidence in comments
- Use consistent naming (Micro-Iteration X.Y)
- Include file names clearly
- Add context to each action
