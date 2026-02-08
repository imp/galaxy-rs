# Agent Instructions

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

## Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Code quality
`cargo fmt`
`cargo clippy --workspace --all-targets -- -D warnings`
`cargo test --workspace`

### Clippy attributes
Prefer `#[expect(...)]` over `#[allow(...)]`

**Why?** `#[expect()]` will warn you when the lint condition no longer applies,
making it easy to spot and remove unneeded attributes. This keeps the codebase clean.

### Dependencies
Prefer workspace dependencies over local dependencies.


**Strategy:**
1. Remove all `#[allow(dead_code)]` attributes
2. Run clippy to see what's actually unused
3. Add targeted `#[expect(dead_code)]` only where needed
4. When code becomes used, clippy will warn about unnecessary expects

Note: `#[expect()]` is available on stable Rust (stabilized in 1.81.0).
Use it instead of `#[allow()]` to get automatic cleanup warnings.

### Code style
- **Encapsulation**: Use private fields with public getters/setters
- **No public struct fields**: Prefer accessor methods for better API control
- **Bevy Components**: Derive `Component` for entities, `Resource` for singletons
- **Import grouping**: rustfmt groups imports as `StdExternalCrate` with individual items
  - Example: `use std::fmt;` and then `impl fmt::Display for Foo`
  - Example: `use std::io;` and then `fn foo() -> io::Result<()>`
- **Let-chains for nested conditions**: Use `if let Some(x) = ... && condition` instead of nested ifs
- **Prefer map_or over map().unwrap_or()**: clippy warns about the latter
- **Avoid .clone() on Copy types**: Use the value directly

### Borrow Checker Patterns
- **Separate read and write phases**: When analyzing state, use immutable borrows first,
  then apply changes with mutable borrows in a separate phase
- **Example**: Racebot analyzes game state (immutable), returns decisions struct,
  then GameState executes decisions (mutable)

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
