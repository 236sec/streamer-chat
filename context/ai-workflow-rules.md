# AI Workflow Rules

## Development Pipeline

Every feature follows this pipeline. No step is skipped.

```
Engineer → Plan Agent → Engineer Review → Build Agent → Coding Verification → Review Agent → Engineer Review
```

### Agent Isolation

Every agent (Plan, Build, Review) runs as a **separate subagent** with its own context. Agents do not share context with each other — each one starts fresh. Use the subagent or orchestration features to spawn each agent. This ensures:

- No context pollution between pipeline stages
- Each agent reasons independently from its own reading of the project
- Failures in one agent do not corrupt another agent's state

### Context Loading

Before doing any work, every agent **except the Build Agent** must read all relevant context files:

1. `context/project-overview.md`
2. `context/architecture.md`
3. `context/ui-tokens.md`
4. `context/ui-rules.md`
5. `context/ui-registry.md`
6. `context/code-standards.md`
7. `context/library-docs.md`
8. `context/build-plan.md`
9. `context/progress-tracker.md`

The **Build Agent** is the exception — it receives the approved implementation plan and feature spec as input, plus `code-standards.md` and `architecture.md` for conventions and boundaries. It does not need to re-explore the full project context.

---

### Step 1 — Engineer (Me)

Describe what to build. Reference `build-plan.md` for the current frontier tickets and `progress-tracker.md` for what is next. Be specific about the ticket, its acceptance criteria, and any constraints.

### Step 2 — Plan Agent

**Spawn a subagent.** Invoke `/architect` in an isolated subagent. The Plan Agent:

1. Reads **all context files** (see Context Loading above) to understand the project
2. Aligns on language — defines ambiguous terms and confirms with me
3. Surfaces decisions that would change the implementation direction
4. Sketches testing seams — prefer existing seams over new ones, use the highest seam possible
5. Produces a feature spec in `context/feature-specs/` using the **spec template** below

The Plan Agent does not write code. It thinks, decides, and documents.

**Output:** Feature spec file following this structure:

```
## Problem Statement
The problem from the user's perspective.

## Solution
The solution from the user's perspective.

## User Stories
Extensive numbered list. Each: "As an <actor>, I want a <feature>, so that <benefit>."

## Implementation Decisions
Modules, interfaces, architectural decisions, schema changes, API contracts.
No file paths or code snippets — they go stale fast.
Exception: prototype snippets that encode a decision more precisely than prose can.

## Testing Decisions
What makes a good test, which modules to test, prior art in the codebase.

## Out of Scope
What this spec does not cover.

## Further Notes
Anything else relevant.
```

### Step 3 — Engineer Review (Gate)

I review the plan and feature spec. Three outcomes:

- **Approve** — proceed to Build Agent
- **Revise** — send back to Plan Agent with feedback
- **Reject** — discard and re-scope

Do not proceed to building until the plan is explicitly approved.

### Step 4 — Build Agent

**Spawn a subagent.** The Build Agent runs in an isolated subagent. It receives:

- The approved implementation plan and feature spec
- `context/code-standards.md` for conventions
- `context/architecture.md` for boundaries

The Build Agent does **not** read all context files — it works from the approved plan.

It implements the approved plan:

- Follows the implementation steps from the feature spec exactly
- [If TDD: Follows the red-green-refactor cycle — write a failing test, make it pass, refactor. Never write implementation before a failing test.]
- [If not TDD: Writes implementation and tests. Tests must cover the behaviors defined in the feature spec.]
- Stays within the approved scope — does not add unplanned features

The Build Agent does not proceed past this step until Coding Verification passes.

### Step 5 — Coding Verification (Automated Gate)

All verification commands must pass before code moves to review. Run each command in order:

1. `npm run lint`
2. `npm run build`
3. `cargo fmt -- --check`
4. `cargo clippy -- -D warnings`
5. `cargo test`

**On failure:** The Build Agent fixes the issue and re-runs verification. This loop continues until all commands pass. The Build Agent does not escalate to the Engineer for verification failures — it owns the fix.

### Step 6 — Review Agent

**Spawn a subagent.** Invoke `/review` in an isolated subagent. The Review Agent:

1. Reads **all context files** (see Context Loading above) to understand the full project
2. Reads the approved implementation plan and feature spec
3. Inspects the built code across three layers:

- **Plan alignment** — does the code match the approved implementation plan?
- **System integrity** — does it respect architecture boundaries, design tokens, and code standards?
- **Production readiness** — error handling, edge cases, missing states?

**Output:** A review report with issues categorized by severity (Critical / Important / Minor). For each issue, the Review Agent suggests a specific code change.

The Review Agent does not apply fixes. It reports and suggests.

### Step 7 — Engineer Review (Final Gate)

I review the Review Agent's report. For each suggested edit:

- **Accept** — apply the fix
- **Reject** — explain why and move on
- **Defer** — acknowledge but fix later

If critical issues were found, loop back to the Build Agent with the accepted fixes. Otherwise, the feature is complete.

After final approval:

1. Update `progress-tracker.md` with completed work
2. Update `ui-registry.md` if UI components were built
3. Move to the next feature

---

## Scoping Rules

- Work on one tracer-bullet ticket at a time as defined in `build-plan.md`.
- Pick from the frontier — tickets whose blockers are all done.
- Each ticket is a vertical slice: narrow but complete through every layer, independently demoable.
- Do not combine unrelated system boundaries in a single ticket.

## When to Split a Ticket

Split a ticket if it combines:

- Frontend UI setup and complex Rust backend integrations that can be built independently
- Multiple unrelated API routes or use cases
- UI changes spanning multiple pages with no shared dependency
- Behavior not clearly defined in the context files

If a ticket cannot be verified end to end quickly, the slice is too wide — split it.

## Handling Missing Requirements

- Do not invent product behavior not defined in the context files.
- If a requirement is ambiguous, resolve it in the relevant context file before implementing. Propose the resolution and wait for confirmation.
- If a requirement is missing, add it as an open question in `progress-tracker.md` before continuing.

## Protected Files

Do not modify the following unless explicitly instructed:

- `frontend/components/ui/*` — generated library components, add via CLI only
- Generated mocks — regenerate, do not hand-edit
- Any third-party library internals

## Keeping Docs in Sync

Update the relevant context file whenever implementation changes:

- System architecture or boundaries → `architecture.md`
- Storage model decisions → `architecture.md`
- Code conventions or standards → `code-standards.md`
- Feature scope, goals, or user flow → `project-overview.md`
- Theme, colors, typography → `ui-tokens.md`
- Layout patterns, component conventions → `ui-rules.md`

`progress-tracker.md` must be updated after every meaningful implementation change.

## TDD Discipline (If Applicable)

When the project uses TDD:

- Every feature or bugfix begins with a failing test. No exceptions.
- Tests must be written at the appropriate level:
  - **Backend**: Unit tests for use cases and domain logic. Integration tests for repositories against a test database.
  - **Frontend**: Component tests for UI behavior. Integration tests for page-level flows.
- Tests must cover both happy paths and error/edge cases.
- Mocks are used only at system boundaries (external APIs, databases in unit tests). Do not mock business logic.
- A test that doesn't fail before implementation is invalid — prove the red phase before going green.

## Before Moving to the Next Ticket

1. The full pipeline was followed — Plan → Build → Verify → Review → Approve.
2. All verification commands pass.
3. No critical issues from the Review Agent remain unresolved.
4. No invariant defined in `architecture.md` was violated.
5. `progress-tracker.md` reflects the completed ticket and the frontier is updated.
6. Feature spec in `feature-specs/` is up to date with what was actually built.
