---
name: developing-features
description: A comprehensive, structured workflow for feature development. Guides you through discovery, codebase exploration, architecture design, implementation, and quality review to build high-quality features. Use when the user asks to "build a feature", "add functionality", or uses /feature-dev.
---

# Developing Features

## When to use this skill
- When the user asks to implement a new feature (e.g., "Add user authentication", "Create a new API endpoint").
- When the user uses the slash command `/feature-dev`.
- When the task touches multiple files or requires architectural decisions.
- **Do not use** for simple bug fixes or trivial changes.

## Workflow
This skill follows a strict 7-phase process to ensure quality and maintainability.

### Phase 1: Discovery
**Goal**: Understand what needs to be built.
1.  **Analyze the Request**: Identify the core problem and requirements.
2.  **Ask Clarifying Questions**: If the request is vague, ask:
    - What is the specific problem you are solving?
    - What are the constraints?
    - Are there specific performance or security requirements?
3.  **Confirm Understanding**: Summarize your understanding back to the user before proceeding.

### Phase 2: Codebase Exploration
**Goal**: Deeply understand existing code and patterns.
1.  **Adopt the [Code Explorer](resources/code-explorer.md) Persona**:
    - Use `codebase_search` and `grep_search` to find relevant entry points.
    - Trace execution paths from entry to database/storage.
    - Identify design patterns and similar features to emulate.
2.  **Identify Key Files**: List the 5-10 most critical files to read.
3.  **Read Files**: Use `view_file` to read these files fully.
4.  **Synthesize**: Present a comprehensive summary of how the current system works and how the new feature will fit in.

### Phase 3: Clarifying Questions (Critical)
**Goal**: Resolve all ambiguities before design.
1.  **Review Findings**: based on Phase 2, identify gaps.
2.  **Ask Specific Questions**:
    - Edge cases?
    - Error handling strategies?
    - Integration points?
    - Backward compatibility?
3.  **Wait for User Input**: Do not proceed to architecture until these are resolved.

### Phase 4: Architecture Design
**Goal**: Create a solid implementation plan.
1.  **Adopt the [Code Architect](resources/code-architect.md) Persona**:
    - Design 2-3 approaches (e.g., "Minimal Change", "Clean Architecture", "Pragmatic").
    - **Draft the Blueprint**: Select the best approach and detail:
        - Files to create/modify.
        - Data flow.
        - Component responsibilities.
        - Build sequence.
2.  **Present & Approve**: Show the plan to the user and ask for approval or preference.

### Phase 5: Implementation
**Goal**: Build the feature.
1.  **Wait for Approval**: Do not start coding until the design is approved.
2.  **Execute the Plan**:
    - Create/modify files as specified.
    - Follow project conventions strictly.
    - Use `write_to_file` and `run_command` (for tests/builds).
3.  **Track Progress**: Keep a checklist of the build sequence.

### Phase 6: Quality Review
**Goal**: Ensure code quality before finishing.
1.  **Adopt the [Code Reviewer](resources/code-reviewer.md) Persona**:
    - Review your own changes (or use `git diff`).
    - Check for:
        - Bugs/Logic errors.
        - Style/Convention violations.
        - Security issues.
2.  **Refine**: Fix any "High Confidence" issues immediately.

### Phase 7: Summary
**Goal**: Wrap up.
1.  **Verify**: Run tests if available.
2.  **Report**: Summarize what was built, files modified, and suggest next steps (e.g., "Add unit tests", "Deploy").

## Resources
- [Code Explorer Instructions](resources/code-explorer.md)
- [Code Architect Instructions](resources/code-architect.md)
- [Code Reviewer Instructions](resources/code-reviewer.md)
