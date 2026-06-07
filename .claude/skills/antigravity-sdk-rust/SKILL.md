```markdown
# antigravity-sdk-rust Development Patterns

> Auto-generated skill from repository analysis

## Overview
This skill teaches you the core development patterns, coding conventions, and workflows for contributing to the `antigravity-sdk-rust` repository. The SDK is written in Rust and is organized as a modular library, focusing on extensibility and clarity. You'll learn how to implement new features, wire them into the SDK, update documentation, and follow the project's conventions for code, commits, and testing.

## Coding Conventions

### File Naming
- Use **camelCase** for file names.
  - Example: `toolContext.rs`, `triggerHelpers.rs`

### Imports
- Use **relative imports** within modules.
  - Example:
    ```rust
    mod error;
    use crate::error::AntigravityError;
    ```

### Exports
- Use **named exports** for modules, traits, and structs.
  - Example:
    ```rust
    pub mod context;
    pub use context::Context;
    ```

### Commit Messages
- Use **conventional commit** prefixes:
  - `feat:` for new features
  - `docs:` for documentation changes
- Keep commit messages concise (average ~51 characters).
  - Example: `feat: add interactive tool context support`

## Workflows

### Feature Implementation and Wiring
**Trigger:** When adding a new core feature or module to the SDK  
**Command:** `/new-module`

1. **Create or update module files** in `src/` (e.g., `context.rs`, `error.rs`, `tool_context.rs`, `trigger_helpers.rs`, `interactive.rs`).
2. **Wire the new modules** into `src/lib.rs`:
    ```rust
    // In src/lib.rs
    pub mod toolContext;
    pub use toolContext::ToolContext;
    ```
3. **Optionally update or create related trait or struct files** (e.g., `hooks.rs`, `tools.rs`).
4. **Test the integration** by running or writing tests (see Testing Patterns).

#### Example
Suppose you're adding a new module `interactive.rs`:
```rust
// src/interactive.rs
pub struct Interactive {
    // fields
}
impl Interactive {
    pub fn new() -> Self { /* ... */ }
}
```
Update `src/lib.rs`:
```rust
pub mod interactive;
pub use interactive::Interactive;
```

---

### Documentation Update After Feature
**Trigger:** When documenting new features, modules, or API changes  
**Command:** `/update-docs`

1. **Update or create documentation files** in `docs/*.md` to cover new or changed components.
2. **Edit `README.md`** to add or update sections reflecting new features or changes.
3. **Add cross-links or summary sections** in `README.md` for better discoverability.

#### Example
If you add a new `Interactive` module:
- Update `docs/interactive.md` with usage and API details.
- Add a section in `README.md`:
    ```markdown
    ## Interactive Module
    Provides interactive SDK features for advanced workflows.
    See [docs/interactive.md](docs/interactive.md) for details.
    ```

## Testing Patterns

- **Test files** use the pattern `*.test.*` (e.g., `context.test.rs`).
- **Testing framework** is not explicitly specified; use Rust's built-in test framework.
- **Example test structure:**
    ```rust
    // src/context.test.rs
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_context_creation() {
            let ctx = Context::new();
            assert!(ctx.is_valid());
        }
    }
    ```

## Commands

| Command        | Purpose                                                        |
|----------------|----------------------------------------------------------------|
| /new-module    | Scaffold and wire a new core module into the SDK               |
| /update-docs   | Update documentation and README after a feature or API change  |
```
