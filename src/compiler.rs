use std::error::Error;

pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self
    }

    pub fn compile(latex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        // Stub: Tectonic removed due to build issues.
        // In real impl, this would call Tectonic.
        println!("Compiling: {}", latex);
        Err("Tectonic compilation disabled".into())
    }
}
