//! Vim-style keyboard mode switcher.
//! Vim 风格键盘模式切换器。
//!
//! Maintains editor mode state and processes raw key events,
//! mapping them to mode transitions (Normal → Insert/Visual/Replace/Command).
//! Does not handle typing within modes — TextInput owns character input.

pub enum Mode {
    Normal,
    Insert,
    Visual,
    Replace,
    Command,
}

pub struct KeyboardMonitor {
    mode: Mode,
    panel_open: bool,
    command_text: String,
    needs_focus: bool,
}

impl KeyboardMonitor {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
            panel_open: false,
            command_text: String::new(),
            needs_focus: false,
        }
    }

    /// Process a raw key event text. Returns `true` if the key was consumed
    /// as a mode-switch or escape action.
    ///
    /// # Arguments
    /// * `text` - Key event text from Slint. Empty string or `\u{1b}` for Escape.
    pub fn handle_key(&mut self, text: &str) -> bool {
        // Escape always resets to normal and closes panel.
        if text.is_empty() || text == "\u{1b}" {
            self.mode = Mode::Normal;
            self.panel_open = false;
            self.command_text.clear();
            return true;
        }

        // ':' enters command mode.
        if text == ":" && !matches!(self.mode, Mode::Command) {
            self.mode = Mode::Command;
            self.panel_open = true;
            self.command_text.clear();
            self.needs_focus = true;
            return true;
        }

        // In command mode, let TextInput consume character keys.
        if matches!(self.mode, Mode::Command) {
            return false;
        }

        // Mode-switch keys.
        match text {
            "i" => {
                self.mode = Mode::Insert;
                true
            }
            "v" => {
                self.mode = Mode::Visual;
                true
            }
            "R" => {
                self.mode = Mode::Replace;
                true
            }
            _ => false,
        }
    }

    /// Returns the current mode as a display string (e.g. "NORMAL").
    pub fn mode_str(&self) -> &str {
        match self.mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
            Mode::Replace => "REPLACE",
            Mode::Command => "COMMAND",
        }
    }

    pub fn panel_open(&self) -> bool {
        self.panel_open
    }

    pub fn command_text(&self) -> &str {
        &self.command_text
    }

    /// Called when the command is submitted (Enter pressed in panel).
    pub fn submit_command(&mut self, _text: &str) {
        self.command_text.clear();
        self.panel_open = false;
        self.mode = Mode::Normal;
    }

    /// Called when the command is cancelled (Escape while panel open).
    pub fn cancel_command(&mut self) {
        self.command_text.clear();
        self.panel_open = false;
        self.mode = Mode::Normal;
    }

    /// Returns true if the TextInput needs auto-focus (panel just opened).
    /// Resets the flag after reading.
    pub fn take_needs_focus(&mut self) -> bool {
        let val = self.needs_focus;
        self.needs_focus = false;
        val
    }
}
