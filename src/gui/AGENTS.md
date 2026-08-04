<!--
  @author saoge2010@163.com
  @date   2026-8-3
  @note   仅用于学习,非正式项目
-->

# AGENTS.md

## Entrypoint

- **Slint source**: `src/gui/src/main.slint`
- **Compilation**: `build.rs` calls `slint_build::compile("src/gui/src/main.slint")`
- **Generated code**: consumed in `src/main.rs` via `slint::include_modules!()`
  - Each exported component in `.slint` becomes an identically-named Rust struct (e.g. `MainEditorWindow`)

## Adding Slint Files

To add a new `.slint` file, either:
- `import` it into `main.slint`
- or pass it as an additional argument to `slint_build::compile()` in `build.rs`

## Component Modularization

Each UI component should reside in its own `.slint` file under `src/gui/src/`, exporting a single component per file.

In `main.slint`, import components via `import { ComponentName } from "file_name.slint";` and compose them within `MainEditorWindow`'s layout.

Example:

```slint
// main.slint
import { StatusBar } from "status_bar.slint";
import { EditorArea } from "editor_area.slint";

export component MainEditorWindow inherits Window {
    VerticalLayout {
        EditorArea { }
        StatusBar { }
    }
}
```

## Slint Conventions

- **Naming**: components use `UpperCamelCase` (e.g. `MainEditorWindow`, `StatusBar`)
- **Layout**: prefer `VerticalLayout` / `HorizontalLayout` over absolute positioning
- **Sizing**: avoid magic pixel values; use layout constraints or named constants where practical
- **Callback naming**: `foo-changed` / `foo_requested` for signals from UI to Rust
- **Property naming**: `snake_case` matching Rust side

## Available Imports

- `std-widgets.slint` — Button, LineEdit, TextEdit, ScrollView, etc.
- Custom widgets should be placed in `src/gui/src/` and imported in `main.slint`

## Rust ↔ Slint Binding

- Slint callbacks declared in `.slint` are callable from Rust via the generated struct
- Slint properties declared in `.slint` are get/set-able from Rust
- Use `component_handle.run()` to start the event loop (blocking call)
