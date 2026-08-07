use crate::backend::backend_file::FileEvent;
use crate::backend::backend_git::GitEvent;
use crate::backend::backend_lsp::LspEvent;
use crate::backend::backend_syntax::SyntaxEvent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendSource {
    Lsp,
    Syntax,
    File,
    Git,
}

#[derive(Clone, Debug)]
pub enum BackendPayload {
    Lsp(LspEvent),
    Syntax(SyntaxEvent),
    File(FileEvent),
    Git(GitEvent),
}

#[derive(Clone, Debug)]
pub struct WorkerEvent {
    pub source: BackendSource,
    pub payload: BackendPayload,
}
