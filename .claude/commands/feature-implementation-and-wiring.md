---
name: feature-implementation-and-wiring
description: Workflow command scaffold for feature-implementation-and-wiring in antigravity-sdk-rust.
allowed_tools: ["Bash", "Read", "Write", "Grep", "Glob"]
---

# /feature-implementation-and-wiring

Use this workflow when working on **feature-implementation-and-wiring** in `antigravity-sdk-rust`.

## Goal

Implements a new core feature or module, then wires it into the main library entry point and supporting files.

## Common Files

- `src/*.rs`
- `src/lib.rs`

## Suggested Sequence

1. Understand the current state and failure mode before editing.
2. Make the smallest coherent change that satisfies the workflow goal.
3. Run the most relevant verification for touched files.
4. Summarize what changed and what still needs review.

## Typical Commit Signals

- Create or update one or more module files in src/ (e.g., context.rs, error.rs, tool_context.rs, trigger_helpers.rs, interactive.rs)
- Update src/lib.rs to wire in the new modules
- Optionally, update or create related trait or struct files (e.g., hooks.rs, tools.rs)

## Notes

- Treat this as a scaffold, not a hard-coded script.
- Update the command if the workflow evolves materially.