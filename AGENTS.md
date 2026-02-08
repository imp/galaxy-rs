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

**Strategy:**
1. Remove all `#[allow(dead_code)]` attributes
2. Run clippy to see what's actually unused
3. Add targeted `#[allow(dead_code)]` only where needed
4. When code becomes used, clippy will warn about unnecessary allows

Note: `#[expect()]` requires nightly Rust. Use `#[allow()]` on stable but
periodically audit and remove unnecessary ones.

### Code style
- **Encapsulation**: Use private fields with public getters/setters
- **No public struct fields**: Prefer accessor methods for better API control
- **Bevy Components**: Derive `Component` for entities, `Resource` for singletons
- **Import grouping**: rustfmt groups imports as `StdExternalCrate` with individual items


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
