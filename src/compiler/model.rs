use std::collections::LinkedList;
use super::CompilationError;

/**
 * Nested reference to JSON element
 */
struct JsonElement<'t> {
    sub: Option<&'t JsonElement<'t>>
}

/**
 * Reference to a variables value
 */
pub struct VariableReference {
    id: String
}

impl Clone for VariableReference {
    fn clone(&self) -> Self {
        VariableReference {
            id: self.id.clone()
        }
    }
}


/**
 * Data/Values can be represented in multiple ways:
 * - it can come inside a json element (=path)
 * - reference to a variable containing the value
 * - the pure value as a String
 */
enum Data<'t> {
    Json(JsonElement<'t>),
    Variable(VariableReference),
    Value(String)
}

pub struct PrintOperation<'t> {
    content: Data<'t>
}


pub struct FetchOperation<'t> {
    from: Data<'t>,
    arguments: LinkedList<Data<'t>>
} 

pub struct InsertOperation<'t> {
    content: Data<'t>
}

pub struct CallOperation<'t> {
    template: Data<'t>,
    params: LinkedList<Data<'t>>
}

pub enum TemplateOperation<'t> {
    Print(PrintOperation<'t>),
    Fetch(FetchOperation<'t>),
    Insert(InsertOperation<'t>),
    Call(CallOperation<'t>)
}


pub struct Template<'t>{
    pub operations: LinkedList<TemplateOperation<'t>>,
    pub variables: LinkedList<VariableReference>,
    pub id: String
}

impl<'t> Template<'t> {
    pub fn new(id: String) -> Template<'t> {
        Self {
            operations: LinkedList::new(),
            variables: LinkedList::new(),
            id
        }
    }

    pub fn get_variable_by_id(&self, id: String) -> Result<VariableReference, CompilationError> {
        for var in self.variables.iter() {
            if var.id.eq(&id) {
                return Ok(var.clone())
            }
        }

        Err(
            CompilationError {
                name: "ERR_UNDEFINED_VARIABLE".to_string(), 
                cause: format!("variable '{}' has not been defined in template '{}'", id, self.id), 
                suggestion: format!("define variable '{}' before its use", id), 
                desc: format!("cannot use variable '{}'", id),
                view: None,
                position: None
            }
        )
    }
}

pub struct CoreModel<'t> {
    pub templates: LinkedList<Template<'t>>,
}

impl<'t> CoreModel<'t> {
    pub fn new() -> CoreModel<'t> {
        CoreModel {
            templates: LinkedList::new()
        }
    }
}