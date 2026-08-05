mod filemng;
mod keyboard_monitor;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use slint::{ComponentHandle, SharedString, VecModel};

slint::include_modules!();

const CMD_INPUT_INDEX: u32 = 38;

fn focus_command_input(ui: &MainEditorWindow) {
    use i_slint_core::item_tree::ItemRc;
    use slint::private_unstable_api::re_exports::{ItemTreeVTable, VRc, WindowInner};
    use std::ptr;

    let window = ui.window();
    let window_inner: &WindowInner = unsafe { &*(window as *const _ as *const WindowInner) };

    let tree: VRc<ItemTreeVTable> = unsafe { ptr::read(ui as *const _ as *const VRc<ItemTreeVTable>) };
    let tree_clone = tree.clone();
    std::mem::forget(tree);

    let item = ItemRc::new(tree_clone, CMD_INPUT_INDEX);
    window_inner.set_focus_item(&item, true, Default::default());
}

fn sync_entries(ui: &MainEditorWindow, fm: &filemng::FileMngClient) {
    let entries: Vec<FileEntryModel> = fm
        .entries()
        .iter()
        .map(|e| FileEntryModel {
            name: e.name.clone().into(),
            path: e.path.clone().into(),
            is_dir: e.is_dir,
            depth: e.depth as i32,
        })
        .collect();
    ui.set_entries(Rc::new(VecModel::from(entries)).into());
}

fn sync_tabs(ui: &MainEditorWindow, fm: &filemng::FileMngClient) {
    let tabs: Vec<EditorTabItem> = fm
        .open_files()
        .iter()
        .map(|f| EditorTabItem {
            file_name: f.name.clone().into(),
            file_path: f.path.clone().into(),
        })
        .collect();
    ui.set_open_tabs(Rc::new(VecModel::from(tabs)).into());
}

fn sync_line_numbers(ui: &MainEditorWindow, text: &str) {
    let count = 1i32 + text.chars().filter(|&c| c == '\n').count() as i32;
    let nums: String = (1..=count)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    ui.set_line_numbers_text(nums.into());
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainEditorWindow::new().unwrap();
    let ui_weak = ui.as_weak();
    let monitor = Rc::new(RefCell::new(keyboard_monitor::KeyboardMonitor::new()));
    let file_mgr = Rc::new(RefCell::new(filemng::FileMngClient::spawn()));

    {
        let fm = file_mgr.borrow();
        ui.set_current_path(fm.current_dir().into());
        sync_entries(&ui, &fm);
        sync_line_numbers(&ui, fm.active_content());
    }

    {
        let ui_weak = ui_weak.clone();
        let monitor = monitor.clone();
        ui.on_key_down(move |text: SharedString| {
            let ui = ui_weak.unwrap();
            let mut m = monitor.borrow_mut();
            let handled = m.handle_key(&text);

            ui.set_current_mode(m.mode_str().into());
            ui.set_panel_open(m.panel_open());
            ui.set_command_text(m.command_text().into());
            ui.set_key_handled(handled);

            if m.take_needs_focus() {
                let ui_weak = ui_weak.clone();
                slint::Timer::single_shot(Duration::from_millis(200), move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        focus_command_input(&ui);
                    }
                });
            }
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let monitor = monitor.clone();
        let file_mgr = file_mgr.clone();
        ui.on_command_commit(move |text: SharedString| {
            let ui = ui_weak.unwrap();
            let mut m = monitor.borrow_mut();
            m.submit_command(&text);

            if text.trim() == ":w" {
                let editor_text = ui.get_editor_text();
                file_mgr.borrow_mut().update_active_content(&editor_text);
                // TODO: Full save-command integration — see ./temp/TODOS.md
                let _ = file_mgr.borrow_mut().save_file();
            }

            ui.set_current_mode(m.mode_str().into());
            ui.set_panel_open(m.panel_open());
            ui.set_command_text(m.command_text().into());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let monitor = monitor.clone();
        ui.on_command_cancel(move || {
            let ui = ui_weak.unwrap();
            let mut m = monitor.borrow_mut();
            m.cancel_command();

            ui.set_current_mode(m.mode_str().into());
            ui.set_panel_open(m.panel_open());
            ui.set_command_text(m.command_text().into());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let file_mgr = file_mgr.clone();
        ui.on_file_explorer_expand(move |path: SharedString| {
            let ui = ui_weak.unwrap();
            let mut fm = file_mgr.borrow_mut();
            fm.toggle_expand(&path);
            ui.set_current_path(fm.current_dir().into());
            sync_entries(&ui, &fm);
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let file_mgr = file_mgr.clone();
        ui.on_file_explorer_open(move |path: SharedString| {
            let ui = ui_weak.unwrap();
            let mut fm = file_mgr.borrow_mut();
            if let Some(idx) = fm.open_file(&path) {
                sync_tabs(&ui, &fm);
                ui.set_active_tab_index(idx as i32);
                ui.set_editor_text(fm.active_content().into());
                sync_line_numbers(&ui, fm.active_content());
            }
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let file_mgr = file_mgr.clone();
        ui.on_tab_clicked(move |idx: i32| {
            let ui = ui_weak.unwrap();
            let mut fm = file_mgr.borrow_mut();
            let current_text = ui.get_editor_text();
            fm.update_active_content(&current_text);
            fm.set_active_file(idx as usize);
            ui.set_active_tab_index(idx);
            ui.set_editor_text(fm.active_content().into());
            sync_line_numbers(&ui, fm.active_content());
        });
    }

    {
        let ui_weak = ui_weak.clone();
        let file_mgr = file_mgr.clone();
        ui.on_tab_close_requested(move |idx: i32| {
            let ui = ui_weak.unwrap();
            let mut fm = file_mgr.borrow_mut();
            fm.close_file(idx as usize);
            sync_tabs(&ui, &fm);
            let new_idx = fm.active_file_index() as i32;
            ui.set_active_tab_index(new_idx);
            if fm.open_files().is_empty() {
                ui.set_editor_text("".into());
                sync_line_numbers(&ui, "");
            } else {
                ui.set_editor_text(fm.active_content().into());
                sync_line_numbers(&ui, fm.active_content());
            }
        });
    }

    {
        let ui_weak = ui_weak.clone();
        ui.on_editor_text_changed(move || {
            let ui = ui_weak.unwrap();
            let text = ui.get_editor_text();
            sync_line_numbers(&ui, &text);
        });
    }

    ui.run().unwrap();
    Ok(())
}
