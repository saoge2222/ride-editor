use gpui::{
    App, Application, AppContext, AssetSource, Context, Entity, IntoElement, ParentElement,
    Render, Result, SharedString, Styled, Window, WindowBounds, WindowOptions, div, px, relative,
    rgb, size,
};
use gui_widgets::button::{Button, ButtonConfig, ButtonStyles};
use gui_widgets::data::{
    Breakpoint, DiagnosticSeverity, DocumentSymbol, GitLineState, GitStatus, LspDiagnostic,
    LspPosition, LspRange, SymbolKind, SyntaxToken, TokenKind,
};
use gui_widgets::editor::{Editor, EditorConfig, EditorStyles};
use std::borrow::Cow;

struct FsAssetSource;

impl AssetSource for FsAssetSource {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(Some(Cow::Owned(bytes))),
            Err(_) => Ok(None),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let _ = path;
        Ok(vec![])
    }
}

fn sample_source() -> &'static str {
    "fn main() {\n    let message = \"hello world\";\n    println!(\"{message}\");\n    if message.len() > 4 {\n        // too long\n        return;\n    }\n}\n"
}

fn line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (index, _) in text.char_indices() {
        if text[index..].starts_with('\n') {
            offsets.push(index + 1);
        }
    }
    offsets
}

fn sample_tokens(offsets: &[usize]) -> Vec<SyntaxToken> {
    let token = |line: usize, start: u32, end: u32, kind: TokenKind| SyntaxToken {
        start: (offsets[line] + start as usize) as u32,
        end: (offsets[line] + end as usize) as u32,
        kind,
    };
    vec![
        token(0, 0, 2, TokenKind::Keyword),
        token(0, 5, 9, TokenKind::Identifier),
        token(1, 4, 7, TokenKind::Keyword),
        token(1, 8, 15, TokenKind::Identifier),
        token(1, 18, 31, TokenKind::String),
        token(2, 4, 12, TokenKind::Identifier),
        token(2, 13, 24, TokenKind::String),
        token(3, 4, 6, TokenKind::Keyword),
        token(3, 8, 15, TokenKind::Identifier),
        token(3, 17, 22, TokenKind::Operator),
        token(3, 25, 26, TokenKind::Number),
        token(5, 8, 19, TokenKind::Comment),
        token(6, 4, 10, TokenKind::Keyword),
    ]
}

fn sample_diagnostics() -> Vec<LspDiagnostic> {
    vec![LspDiagnostic {
        range: LspRange {
            start: LspPosition { line: 3, character: 0 },
            end: LspPosition { line: 3, character: 26 },
        },
        message: "condition is always true".into(),
        severity: DiagnosticSeverity::Warning,
    }]
}

fn sample_git_status() -> Vec<GitLineState> {
    vec![
        GitLineState { line: 1, status: GitStatus::Modified },
        GitLineState { line: 5, status: GitStatus::Added },
        GitLineState { line: 6, status: GitStatus::Deleted },
    ]
}

fn sample_symbols() -> Vec<DocumentSymbol> {
    vec![DocumentSymbol {
        name: "fn main".into(),
        kind: SymbolKind::Function,
        range: LspRange {
            start: LspPosition { line: 0, character: 0 },
            end: LspPosition { line: 8, character: 8 },
        },
    }]
}

struct RootView {
    editor: Entity<Editor>,
    lsp_button: Entity<Button>,
    lsp_connected: bool,
}

impl RootView {
    fn new(cx: &mut Context<Self>) -> Self {
        let weak_lsp = cx.weak_entity();

        let styles = EditorStyles::new(px(880.), px(560.), rgb(0xe2e8f0), rgb(0x60a5fa))
            .font_family("Maple Mono")
            .background_color(rgb(0x0f172a));

        let editor = Editor::new(
            EditorConfig::new("main.rs", styles, sample_source())
                .on_write_file(|id, text, _window, _cx| {
                    println!("write_file: {id}\n{text}");
                })
                .on_text_changed(|id, text, _window, _cx| {
                    println!("text_changed: {id} len={}", text.len());
                })
                .on_mode_changed(|id, mode, _window, _cx| {
                    println!("mode_changed: {id} {mode}");
                })
                .on_toggle_breakpoint(|id, line, added, _window, _cx| {
                    println!("breakpoint: {id} line={line} added={added}");
                }),
            cx,
        );

        let offsets = line_offsets(sample_source());
        editor.update(cx, |editor, cx| {
            editor.set_tokens(sample_tokens(&offsets), cx);
            editor.set_diagnostics(sample_diagnostics(), cx);
            editor.set_git_status(sample_git_status(), cx);
            editor.set_breakpoints(vec![Breakpoint { line: 3 }], cx);
            editor.set_document_symbols(sample_symbols(), cx);
        });

        let lsp_button = Button::new(
            ButtonConfig::new(
                "lsp",
                ButtonStyles::new(px(160.), px(32.))
                    .border_color(rgb(0x3b82f6))
                    .fill_color(rgb(0x1e293b))
                    .text("Toggle LSP")
                    .text_color(rgb(0xf1f5f9))
                    .text_size(px(14.)),
            )
            .on_click(move |_id, _window, cx| {
                let _ = weak_lsp.update(cx, |root, cx| {
                    root.lsp_connected = !root.lsp_connected;
                    root.editor.update(cx, |editor, cx| {
                        editor.set_lsp_connected(root.lsp_connected, cx);
                    });
                    cx.notify();
                });
            }),
            cx,
        );

        RootView {
            editor,
            lsp_button,
            lsp_connected: false,
        }
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(relative(1.0))
            .h(relative(1.0))
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(0x111827))
            .child(self.editor.clone())
            .child(div().mt(px(16.)).child(self.lsp_button.clone()))
    }
}

fn main() {
    Application::new()
        .with_assets(FsAssetSource)
        .run(|cx: &mut App| {
            let font_path = format!(
                "{}/../gui-workbench/fonts/MapleMono-TTF/MapleMono-Regular.ttf",
                env!("CARGO_MANIFEST_DIR")
            );
            let font_bytes: std::borrow::Cow<'static, [u8]> =
                std::fs::read(font_path).expect("failed to read Maple Mono font").into();
            cx.text_system()
                .add_fonts(vec![font_bytes])
                .expect("failed to load Maple Mono font");
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::centered(size(px(1000.), px(700.)), cx)),
                ..Default::default()
            };
            let _ = cx.open_window(options, |_window, cx| cx.new(RootView::new));
            cx.activate(true);
        });
}
