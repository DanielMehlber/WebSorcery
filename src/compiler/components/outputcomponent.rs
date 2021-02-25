use string_builder::Builder;
use crate::compiler::model::CoreModel;

pub struct OutputComponent {
    lines: Builder
} 

impl OutputComponent {
    pub fn new() -> Self {
        OutputComponent {
            lines: Builder::default()
        }
    }
}

impl OutputComponent {
    pub fn add_line(&mut self, s: String) {
        self.lines.append(s);
        self.lines.append('\n');
    }

    /**
     * Writes C-Code 
     */
    pub fn generateOutput(&mut self, model: &CoreModel) {
        // first declare all functions (templates)
        for temp in &model.templates {
            self.add_line(format!("char* {}();", temp.id));
        }

        // 
    }

    /**
     * OuputComponent is consumed and then destroyed.
     */
    pub fn finish(self) -> String {
        self.lines.string().unwrap()
    }
}