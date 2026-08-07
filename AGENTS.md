<!--
  @author saoge2010@163.com
  @date   2026-8-3
  @note   仅用于学习,非正式项目
-->

# AGENTS.md

## Project

- **Package**: `ride-editor` (Rust edition 2024)
- **Purpose**: Vim-keybinding GUI text editor using Vulkan-based rendering
- **Entrypoint**: `crates/ride-editor/main.rs` — initializes Vulkan window and rendering pipeline
- **GUI frontend**: `crates/gui-workbench/` — new Vulkan-based GUI (contains `fonts/`)

## Build & Run

```bash
cargo build              # → target/debug/ride-editor
cargo build --release    # → target/release/ride-editor
cargo run                # build + run
cargo check              # fast compile check (no output binary)
```

There is no separate lint, format, or test command configured.

## Dependencies

### Vulkan SDK

#### Debian/Ubuntu

```bash
sudo apt install vulkan-tools libvulkan-dev vulkan-validationlayers-dev
```

#### Fedora

```bash
sudo dnf install vulkan-tools vulkan-loader-devel vulkan-validation-layers-devel
```

#### Arch

```bash
sudo pacman -S vulkan-tools vulkan-validation-layers
```

Verify installation:

```bash
vulkaninfo | head -20
```

### Vulkano (Rust Vulkan bindings)

Add to `Cargo.toml`:

```toml
[dependencies]
vulkano = "0.34"
vulkano-shaders = "0.34"
winit = "0.29"
```

Vulkano requires the Vulkan SDK (see above) to be installed before building.

## Code Conventions

- **Naming**: UpperCamelCase for types, snake_case for values
- **No `unsafe`** anywhere
- **No magic values** — use named constants
- **Clarity & safety over speed** — prioritize readability and safety; avoid sacrificing code clarity or safety for runtime performance or lower overhead
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
  - `feat-frontend:` — Vulkan GUI changes
  - `fix:` — bug fixes
  - `buildcfg:` — Cargo.toml / build.rs changes
  - `chores:` — docs, README, .gitignore
  - `agents:` — AGENTS.md changes
- **Branches**:
  - `main` — release code, tag with version on merge
  - `dev` — active development, merge into `main` at release milestones
  - Do **not** act on Pull Requests
