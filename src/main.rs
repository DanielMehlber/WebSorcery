mod strings;
mod compiler;

use compiler::components::scanner::InputComponent;
use strings::{sourceview::SourceView};
use compiler::output::ConsoleOutput;

fn main() {
    println!("Welcome to WebSorcery \u{1F468}");
    let source = SourceView::from_string("{}");

    let mut input_comp = InputComponent::new("hallo".to_string());
    input_comp.scan(source);


    let status = input_comp.is_ok();
    println!("State of inputcomponent: {}", status);

    if !status {
        
        match input_comp.get_error() {
            Some(err) => {
                println!("{}\n\n{}", err.get_header(), err.get_body());
            },
            None => {}
        }
    } else {
        println!("{:#?}", input_comp.tmodel);
    }

}
