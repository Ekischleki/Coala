use crate::compiler::{code_location::CodeLocation, diagnostic::DiagnosticPipelineLocation, settings::Settings};

use super::diagnostic::{Diagnostic, DiagnosticType};


pub struct Compilation {
    diagnostics: Vec<Diagnostic>,
    settings: Settings
}


impl Compilation {
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }


    pub fn add_error(&mut self, message: &str, location: Option<CodeLocation>) {
        self.add_diagnostic(Diagnostic::new(DiagnosticType::Error, message.to_owned(), location, DiagnosticPipelineLocation::Access));
    }
    pub fn add_warning(&mut self, message: &str, location: Option<CodeLocation>) {
        self.add_diagnostic(Diagnostic::new(DiagnosticType::Warning, message.to_owned(), location, DiagnosticPipelineLocation::Access));
    }
    pub fn add_info(&mut self, message: &str, location: Option<CodeLocation>) {
        self.add_diagnostic(Diagnostic::new(DiagnosticType::Info, message.to_owned(), location, DiagnosticPipelineLocation::Access));
    }
    pub fn is_error_free(&self) -> bool {
        self.diagnostics.iter().all(|d|  d.type_lower_than(DiagnosticType::Error))
    }


    pub fn new(settings: Settings) -> Self {
        Self {
            diagnostics: vec![],
            settings
        }
    }
    
    pub fn diagnostics(&self) -> &Vec<Diagnostic> {
        &self.diagnostics
    }
    
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}