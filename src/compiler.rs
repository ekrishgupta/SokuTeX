use std::error::Error;

pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Self
    }

    pub fn compile(latex: &str, _vfs: &crate::vfs::Vfs) -> Result<Vec<u8>, Box<dyn Error>> {
        // Stub: Tectonic removed due to build issues.
        // In real impl, this would use Tectonic with VFS.
        println!("Compiling from VFS: {}", latex);
        Err("Tectonic compilation disabled".into())
    }
}
