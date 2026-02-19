pub struct CommandPalette {
    pub visible: bool,
}

impl CommandPalette {
    pub fn new() -> Self {
        Self {
            visible: false,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
}
