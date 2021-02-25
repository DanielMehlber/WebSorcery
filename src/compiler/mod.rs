
pub mod output;
mod model;
pub mod components;


use std::collections::LinkedList;

use crate::strings::{sourceview::SourcePosition, stringview::StringView};

pub struct CompilationWarnings {

}

pub struct CompilationError<'this> {
    name: String,
    desc: String,
    cause: String,
    suggestion: String,
    view: Option<StringView<'this>>,
    position: Option<SourcePosition>
}

impl<'this> output::ConsoleOutput for CompilationError<'this> {
    fn get_short(&self) -> String {
        self.name.to_string()
    }

    fn get_cutout(&self) -> String {
        match &self.view {
            Some(view) => view.show_slice(10),
            None => "(no source code available)".to_string()
        }
    }

    fn get_message(&self) -> String {
        format!("Description: {}\nCause: {}\nSuggestion: {}", self.desc, self.cause, self.suggestion)
    }

    fn get_location(&self) -> String {
        match &self.position{
            Some(pos) => format!("[{}]", pos),
            None => "[source]".to_string()
        }
    }

    fn get_type(&self) -> String {
        "\u{26D4} ERROR".to_string()
    }
}


enum CompilationStatus<'this> {
    OK,
    Error(CompilationError<'this>)
}

pub struct CompilationState<'t> {
    status: CompilationStatus<'t>,
    warnings: LinkedList<CompilationWarnings>
}

impl<'this> CompilationState<'this> {

    pub fn new() -> CompilationState<'this> {
        Self {
            status: CompilationStatus::OK,
            warnings: LinkedList::new()
        }
    }

    pub fn is_ok(&self) -> bool {
        match self.status {
            CompilationStatus::OK => true,
            CompilationStatus::Error(_) => false
        }
    }

    pub fn error(&mut self, err: CompilationError<'this>) {
        self.status = CompilationStatus::Error(err);
    }

    pub fn get_error(&self) -> Option<&CompilationError<'this>> {
        match &self.status {
            CompilationStatus::OK => None,
            CompilationStatus::Error(err) => Some(err)
        }
    }
}