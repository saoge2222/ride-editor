<!--
  @author saoge2010@163.com
  @date   2026-8-3
  @note   仅用于学习,非正式项目
-->

# AGENTS.md

## Project

- **Package**: `ride-editor` (Rust edition 2024)
- **Purpose**: Vim-keybinding GUI text editor using the slint framework
- **Entrypoint**: `src/main.rs` — calls `slint::include_modules!()` to pull in generated slint code
- **Slint source**: `src/gui/src/main.slint` — compiled by `build.rs` via `slint_build::compile()`
- **Slint-specific notes**: see `src/gui/AGENTS.md`

## Build & Run

```bash
cargo build              # → target/debug/ride-editor
cargo build --release    # → target/release/ride-editor
cargo run                # build + run
cargo check              # fast compile check (no output binary)
```

There is no separate lint, format, or test command configured.

## Code Conventions

- **Naming**: UpperCamelCase for types, snake_case for values
- **No `unsafe`** anywhere
- **No magic values** — use named constants
- **Comments**:
  - Module-level: `//!` doc comments, bilingual (Chinese + English), covering purpose, params, returns, errors
  - Inline: `//` in English, only on non-obvious lines

## Restricted Directories

Do **not** modify or read as project source:
- `/target` — build output
- `/project_docs` — reference docs
- `/.vscode` — editor config
- `**/temp/` — test/scratch code (any temp folder under project root)

## Workflow Rules

1. **Show diffs before editing** — present the planned `-`/`+` changes and get confirmation
2. **Confirm before any shell command** — always ask
3. **Confirm before mass changes** — file restructuring, dependency changes, build config edits all require user approval
4. **No speculative "optimizations"** — only do what was asked

## Git Conventions

- Confirm before every commit
- Never commit: `/.vscode`, `/project_docs`, `/target`, `**/temp/`
- **Commit prefix tags**:
  - `feat-backend:` — Rust source changes
  - `feat-frontend:` — slint UI changes
  - `fix:` — bug fixes
  - `buildcfg:` — Cargo.toml / build.rs changes
  - `chores:` — docs, README, .gitignore
  - `agents:` — AGENTS.md changes
- **Branches**:
  - `main` — release code, tag with version on merge
  - `dev` — active development, merge into `main` at release milestones
  - Do **not** act on Pull Requests
