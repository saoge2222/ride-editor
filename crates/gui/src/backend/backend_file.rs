#[derive(Clone, Debug)]
pub enum FileEvent {
    EntriesChanged { path: String },
    FileOpened { path: String },
    FileClosed { path: String },
    ContentUpdated { path: String },
}
