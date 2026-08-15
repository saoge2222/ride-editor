use serde::{Deserialize, Serialize};

use crate::types::position::Range;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(from = "u32", into = "u32")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl From<u32> for DiagnosticSeverity {
    fn from(value: u32) -> Self {
        match value {
            1 => DiagnosticSeverity::Error,
            2 => DiagnosticSeverity::Warning,
            3 => DiagnosticSeverity::Information,
            _ => DiagnosticSeverity::Hint,
        }
    }
}

impl From<DiagnosticSeverity> for u32 {
    fn from(severity: DiagnosticSeverity) -> Self {
        match severity {
            DiagnosticSeverity::Error => 1,
            DiagnosticSeverity::Warning => 2,
            DiagnosticSeverity::Information => 3,
            DiagnosticSeverity::Hint => 4,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub range: Range,
    pub message: String,
    pub severity: DiagnosticSeverity,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PublishDiagnosticsParams {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>,
}
