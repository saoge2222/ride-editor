#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VimMode {
    mode: Mode,
}

impl VimMode {
    pub fn new() -> Self {
        Self { mode: Mode::Normal }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn handle_key(&mut self, key: &str) -> bool {
        match (self.mode, key) {
            (Mode::Normal, "i") => {
                self.mode = Mode::Insert;
                true
            }
            (Mode::Insert, "escape") => {
                self.mode = Mode::Normal;
                true
            }
            _ => false,
        }
    }
}

impl Default for VimMode {
    fn default() -> Self {
        Self::new()
    }
}
