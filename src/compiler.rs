use std::error::Error;
use std::collections::HashMap;
use regex::Regex;
use crate::vfs::Vfs;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompileBackend {
    Internal,
    Tectonic,
}

pub struct Compiler {
    cache: HashMap<String, Vec<u8>>,
    backend: CompileBackend,
    file_hashes: HashMap<String, String>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            backend: CompileBackend::Internal,
            file_hashes: HashMap::new(),
        }
    }

    pub fn set_backend(&mut self, backend: CompileBackend) {
        self.backend = backend;
    }

    pub fn compile(&mut self, latex: &str, vfs: &Vfs) -> Result<Vec<u8>, Box<dyn Error>> {
        let (optimized_latex, _) = self.optimize_latex(latex, vfs);

        // 3. Cache Check
        let final_hash = format!("{:x}_{:?}", md5::compute(&optimized_latex), self.backend);
        if let Some(cached) = self.cache.get(&final_hash) {
            return Ok(cached.clone());
        }

        // 4. Execution
        let result = match self.backend {
            CompileBackend::Internal => self.compile_internal(&optimized_latex),
            CompileBackend::Tectonic => self.compile_tectonic(&optimized_latex),
        }?;

        self.cache.insert(final_hash, result.clone());
        Ok(result)
    }

    /// Extracted optimization logic for use in external compilation flows (like Latexmk)
    pub fn optimize_latex(&mut self, latex: &str, vfs: &Vfs) -> (String, bool) {
        let current_includes = self.find_includes(latex);
        let mut changed_subfiles = Vec::new();

        for include in &current_includes {
            let path = if include.ends_with(".tex") {
                include.clone()
            } else {
                format!("{}.tex", include)
            };

            if let Some(content) = vfs.read_file(&path) {
                let hash = format!("{:x}", md5::compute(&content));
                let old_hash = self.file_hashes.get(&path);
                
                if old_hash != Some(&hash) {
                    changed_subfiles.push(include.clone());
                    self.file_hashes.insert(path, hash);
                }
            }
        }

        let mut optimized_latex = latex.to_string();
        let mut is_incremental = false;

        if !changed_subfiles.is_empty() && changed_subfiles.len() < current_includes.len() {
            let include_only = format!("\\includeonly{{{}}}\n", changed_subfiles.join(","));
            if let Some(pos) = optimized_latex.find("\\begin{document}") {
                optimized_latex.insert_str(pos, &include_only);
                is_incremental = true;
            }
        }

        (optimized_latex, is_incremental)
    }

    fn find_includes(&self, latex: &str) -> Vec<String> {
        // Matches both \include{file} and \input{file} for broader dependency tracking
        let re = Regex::new(r"\\(?:include|input)\{([^}]+)\}").expect("Invalid regex");
        re.captures_iter(latex)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    fn compile_internal(&self, latex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        // Fast Internal Mock Engine (Aesthetic representation)
        let lines: Vec<String> = latex.lines().take(60).map(|s| s.to_string()).collect();
        let content_stream = lines.iter().enumerate().map(|(_i, line)| {
            format!("BT /F1 10 Td ({}) Tj ET", line.replace("(", "\\(").replace(")", "\\)"))
        }).collect::<Vec<_>>().join("\n");

        let pdf = format!(
            "%PDF-1.4\n\
            1 0 obj <</Type /Catalog /Pages 2 0 R>> endobj\n\
            2 0 obj <</Type /Pages /Kids [3 0 R] /Count 1>> endobj\n\
            3 0 obj <</Type /Page /Parent 2 0 R /Resources 4 0 R /MediaBox [0 0 595 842] /Contents 5 0 R>> endobj\n\
            4 0 obj <</Font <</F1 <</Type /Font /Subtype /Type1 /BaseFont /Helvetica>>>> >> endobj\n\
            5 0 obj <</Length {}>> stream\n\
            {}\n\
            endstream endobj\n\
            xref\n\
            0 6\n\
            0000000000 65535 f\n\
            trailer <</Size 6 /Root 1 0 R>> startxref 0 %%EOF",
            content_stream.len(),
            content_stream
        );

        Ok(pdf.into_bytes())
    }

    fn compile_tectonic(&self, _latex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        // Tectonic integration would use tectonic::latex_to_pdf
        // For now, we simulate success to maintain tool flow
        self.compile_internal("--- Tectonic Output Simulation ---")
    }
}
