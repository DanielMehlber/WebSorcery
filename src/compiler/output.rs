
pub trait ConsoleOutput {
    fn get_short(&self) -> String;
    fn get_message(&self) -> String;
    fn get_location(&self) -> String;
    fn get_cutout(&self) -> String; 
    fn get_type(&self) -> String;

    fn get_header(&self) -> String {
        format!("{}: '{}' @ {}", self.get_type(), self.get_short(), self.get_location())
    }

    fn get_body(&self) -> String {
        format!("{}\n{}", self.get_cutout(), self.get_message())
    }
}