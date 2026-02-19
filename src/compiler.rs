use std::error::Error;
use ahash::AHashMap;
use regex::Regex;
use crate::vfs::Vfs;
use crate::config::CompileBackend;
use std::hash::{Hash, Hasher};

pub struct Compiler {
    cache: AHashMap<u64, Vec<u8>>,
    backend: CompileBackend,
    file_hashes: AHashMap<String, u64>,
    pub active_file: Option<String>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            cache: AHashMap::new(),
            backend: CompileBackend::Internal,
            file_hashes: AHashMap::new(),
            active_file: None,
        }
    }

    pub fn set_backend(&mut self, backend: CompileBackend) {
        self.backend = backend;
    }

    pub fn compile(&mut self, latex: &str, draft: bool, vfs: &Vfs) -> Result<Vec<u8>, Box<dyn Error>> {
        let (optimized_latex, _) = self.optimize_latex(latex, draft, vfs);

        // 3. Cache Check
        let mut hasher = ahash::AHasher::default();
        optimized_latex.hash(&mut hasher);
        self.backend.hash(&mut hasher);
        let final_hash = hasher.finish();
        
        if let Some(cached) = self.cache.get(&final_hash) {
            return Ok(cached.clone());
        }

        // 4. Execution
        let result = match self.backend {
            CompileBackend::Internal => self.compile_internal(&optimized_latex),
            CompileBackend::Tectonic => self.compile_tectonic(&optimized_latex, vfs),
        }?;

        self.cache.insert(final_hash, result.clone());
        Ok(result)
    }

    /// Extracted optimization logic for use in external compilation flows (like Latexmk)
    pub fn optimize_latex(&mut self, latex: &str, draft: bool, vfs: &Vfs) -> (String, bool) {
        let all_deps = self.find_includes(latex);
        let mut changed_subfiles = Vec::new();

        for dep in &all_deps {
            let path = if dep.ends_with(".tex") {
                dep.clone()
            } else {
                format!("{}.tex", dep)
            };

            if let Some(content) = vfs.read_file(&path) {
                let mut hasher = ahash::AHasher::default();
                content.hash(&mut hasher);
                let hash = hasher.finish();
                let old_hash = self.file_hashes.get(&path);
                
                if old_hash != Some(&hash) {
                    changed_subfiles.push(dep.clone());
                    self.file_hashes.insert(path, hash);
                }
            }
        }

        let mut optimized_latex = latex.to_string();
        let mut is_incremental = false;

        // Dynamic \includeonly Optimization
        let mut target_includes = changed_subfiles;
        
        // If there's an active file in the editor, prioritize it to force LaTeX to skip others
        if let Some(ref active) = self.active_file {
            let active_base = active.strip_suffix(".tex").unwrap_or(active);
            // Only use it if it's actually part of the includes
            if all_deps.iter().any(|d| d == active_base) {
                target_includes = vec![active_base.to_string()];
            }
        }

        // Only inject \includeonly for files that are actually using \include (not \input)
        let actual_includes: Vec<String> = Regex::new(r"\\include\{([^}]+)\}")
            .unwrap()
            .captures_iter(latex)
            .map(|cap| cap[1].to_string())
            .collect();

        let final_targets: Vec<String> = target_includes.into_iter()
            .filter(|t| actual_includes.contains(t))
            .collect();

        if !final_targets.is_empty() && final_targets.len() < actual_includes.len() {
            let include_only = format!("\\includeonly{{{}}}\n", final_targets.join(","));
            if let Some(pos) = optimized_latex.find("\\begin{document}") {
                optimized_latex.insert_str(pos, &include_only);
                is_incremental = true;
            }
        }


        if draft {
            if optimized_latex.contains("\\documentclass") && !optimized_latex.contains("[draft]") {
                optimized_latex = optimized_latex.replace("\\documentclass", "\\documentclass[draft]");
            }
            if let Some(pos) = optimized_latex.find("\\begin{document}") {
                optimized_latex.insert_str(pos + "\\begin{document}".len(), "\n\\textbf{--- DRAFT MODE ACTIVE ---}\n");
            }
        } else {
            optimized_latex = optimized_latex.replace("\\documentclass[draft]", "\\documentclass");
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

    fn compile_tectonic(&self, latex: &str, vfs: &Vfs) -> Result<Vec<u8>, Box<dyn Error>> {
        use tectonic::driver::{Driver, DriverHooks};
        use tectonic::io::memory::MemoryIo;
        use tectonic::status::plain::PlainStatusBackend;
        use tectonic::config::PersistentConfig;

        let mut mem_io = MemoryIo::new();
        
        // Populate MemoryIo from VFS
        for entry in vfs.get_all_files() {
            mem_io.write_file(entry.key(), entry.value().clone())
                .map_err(|e| format!("VFS sync failed: {}", e))?;
        }

        // Write the main document (which might be optimized_latex)
        mem_io.write_file("main.tex", latex.as_bytes().to_vec())
            .map_err(|e| format!("Main file sync failed: {}", e))?;

        let config = PersistentConfig::default();
        let mut status = PlainStatusBackend::new();
        
        // Tectonic's default bundle
        let bundle = tectonic::io::ite_bundle::IteBundle::default();

        let mut hooks = DriverHooks::new(
            Box::new(mem_io),
            Box::new(bundle),
            Box::new(status),
        );

        let mut driver = Driver::new(&config);
        driver.run(&mut hooks, "main.tex")
            .map_err(|e| format!("Tectonic compilation failed: {}", e))?;

        let output_files = hooks.io.into_inner().unwrap().into_inner();
        output_files.get("main.pdf")
            .cloned()
            .ok_or_else(|| "Tectonic produced no PDF output".into())
    }
}
