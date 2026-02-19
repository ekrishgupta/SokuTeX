#[allow(dead_code)]
pub struct CommandPalette {
    pub visible: bool,
    pub commands: Vec<String>,
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            visible: false,
            commands: vec!["Open".to_string(), "Save".to_string(), "About".to_string()],
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}
