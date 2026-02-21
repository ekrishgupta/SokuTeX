use std::error::Error;

use regex::Regex;
use crate::vfs::Vfs;
use crate::config::CompileBackend;
use crate::bib::BibParser;
use std::hash::{Hash, Hasher};
use std::collections::HashSet;
use log::info;
use rayon::prelude::*;

use std::sync::OnceLock;

static INCLUDE_REGEX: OnceLock<Regex> = OnceLock::new();
static INCLUDE_INPUT_REGEX: OnceLock<Regex> = OnceLock::new();
static BIB_REGEX: OnceLock<Regex> = OnceLock::new();
static BIBRESOURCE_REGEX: OnceLock<Regex> = OnceLock::new();

use dashmap::DashMap;

#[derive(Debug, Clone)]
pub struct FileDelta {
    pub path: String,
    pub old_hash: u64,
    pub new_hash: u64,
    pub content_size: usize,
}


pub struct Compiler {
    cache: DashMap<u64, Vec<u8>>,
    backend: CompileBackend,
    file_hashes: DashMap<String, u64>,
    bib_cache: DashMap<String, (u64, Vec<String>)>,
    pub active_file: Option<String>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            backend: CompileBackend::Internal,
            file_hashes: DashMap::new(),
            bib_cache: DashMap::new(),
            active_file: None,
        }
    }

    pub fn set_backend(&mut self, backend: CompileBackend) {
        self.backend = backend;
    }

    pub fn update_bib_cache(&self, latex: &str, vfs: &Vfs) {
        let bib_re = BIB_REGEX.get_or_init(|| Regex::new(r"\\bibliography\{([^}]+)\}").unwrap());
        let bibresource_re = BIBRESOURCE_REGEX.get_or_init(|| Regex::new(r"\\addbibresource\{([^}]+)\}").unwrap());

        let mut bib_files = Vec::new();
        for cap in bib_re.captures_iter(latex) {
            for file in cap[1].split(',') {
                let mut f = file.trim().to_string();
                if !f.ends_with(".bib") { f.push_str(".bib"); }
                bib_files.push(f);
            }
        }
        for cap in bibresource_re.captures_iter(latex) {
            bib_files.push(cap[1].trim().to_string());
        }

        for path in bib_files {
            if let Some(content_bytes) = vfs.read_file(&path) {
                let mut hasher = ahash::AHasher::default();
                content_bytes.hash(&mut hasher);
                let hash = hasher.finish();

                let needs_update = match self.bib_cache.get(&path) {
                    Some(val) => val.0 != hash,
                    None => true,
                };

                if needs_update {
                    info!("Updating BibTeX cache for {}", path);
                    let content = String::from_utf8_lossy(&content_bytes);
                    let entries = BibParser::parse(&content);
                    let keys: Vec<String> = entries.into_iter().map(|e| e.key).collect();
                    self.bib_cache.insert(path, (hash, keys));
                }
            }
        }
    }

    pub fn compile(&self, latex: &str, draft: bool, focus_mode: bool, vfs: &Vfs) -> Result<Vec<u8>, Box<dyn Error>> {
        let (optimized_latex, _, deltas) = self.optimize_latex(latex, draft, focus_mode, vfs);

        for delta in &deltas {
            info!("Delta detected: {} ({} bytes) {} -> {}", delta.path, delta.content_size, delta.old_hash, delta.new_hash);
        }

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
            CompileBackend::Tectonic => self.compile_tectonic(&optimized_latex),
            CompileBackend::Latexmk => {
                return Err("Latexmk backend is handled asynchronously by the daemon".into());
            }
        }?;

        self.cache.insert(final_hash, result.clone());
        Ok(result)
    }

    fn compile_tectonic(&self, latex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        use std::process::Command;
        use std::io::Write;
        
        // Tectonic: A complete, self-contained TeX/LaTeX engine
        let temp_dir = std::env::temp_dir().join("sokutex_tectonic");
        std::fs::create_dir_all(&temp_dir)?;
        
        let file_path = temp_dir.join("main.tex");
        let mut file = std::fs::File::create(&file_path)?;
        file.write_all(latex.as_bytes())?;
        
        let output = Command::new("tectonic")
            .arg(&file_path)
            .output()?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Tectonic compilation failed: {}", stderr).into());
        }
        
        let pdf_path = temp_dir.join("main.pdf");
        Ok(std::fs::read(pdf_path)?)
    }

    /// Extracted optimization logic for use in external compilation flows (like Latexmk)
    pub fn optimize_latex(&self, latex: &str, draft: bool, focus_mode: bool, vfs: &Vfs) -> (String, bool, Vec<FileDelta>) {

        // 1. Identify all top-level \include units
        let include_re = INCLUDE_REGEX.get_or_init(|| Regex::new(r"\\include\{([^}]+)\}").unwrap());
        let top_level_units: Vec<String> = include_re
            .captures_iter(latex)
            .map(|cap| cap[1].to_string())
            .collect();

        if top_level_units.is_empty() {
            return (self.apply_draft_mode(latex.to_string(), draft), false, Vec::new());
        }

        // 2. Track changes across the entire dependency tree
        
        // This is a bit expensive but necessary for "affected subtree" logic
        // We could optimize by only scanning if we haven't scanned recently
        let mut all_known_files = Vec::new();
        self.collect_all_dependencies("main.tex", vfs, &mut all_known_files, &mut HashSet::new());

        // Update BibTeX cache for all dependencies (Parallel)
        self.update_bib_cache(latex, vfs);
        for path in &all_known_files {
            if let Some(content) = vfs.read_file(path) {
                let content_str = String::from_utf8_lossy(&content);
                self.update_bib_cache(&content_str, vfs);
            }
        }

        let deltas: Vec<FileDelta> = all_known_files.par_iter()
            .filter_map(|path| {
                if let Some(content) = vfs.read_file(path) {
                    let mut hasher = ahash::AHasher::default();
                    content.hash(&mut hasher);
                    let new_hash = hasher.finish();
                    let content_size = content.len();

                    let (old_hash, is_changed) = match self.file_hashes.entry(path.clone()) {
                        dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                            let old = *entry.get();
                            if old != new_hash {
                                entry.insert(new_hash);
                                (old, true)
                            } else {
                                (old, false)
                            }
                        }
                        dashmap::mapref::entry::Entry::Vacant(entry) => {
                            entry.insert(new_hash);
                            (0, true)
                        }
                    };
                    
                    if is_changed {
                        return Some(FileDelta {
                            path: path.clone(),
                            old_hash,
                            new_hash,
                            content_size,
                        });
                    }
                }
                None
            })
            .collect();

        let changed_files: HashSet<String> = deltas.iter().map(|d| d.path.clone()).collect();

        // 3. Determine which top-level units are affected (Parallel)
        let mut affected_units: Vec<String> = top_level_units.par_iter()
            .filter(|unit| {
                let unit_path = if unit.ends_with(".tex") { unit.to_string() } else { format!("{}.tex", unit) };
                
                // Check if unit itself changed
                if changed_files.contains(&unit_path) {
                    return true;
                }

                // Check if any recursive dependency of this unit changed
                let mut unit_deps = Vec::new();
                self.collect_all_dependencies(&unit_path, vfs, &mut unit_deps, &mut HashSet::new());
                
                unit_deps.iter().any(|d| changed_files.contains(d))
            })
            .cloned()
            .collect();

        // 4. Focus Mode & Priority logic
        if let Some(ref active) = self.active_file {
            let active_base = active.strip_suffix(".tex").unwrap_or(active);
            if top_level_units.iter().any(|u| u == active_base) {
                if focus_mode {
                    // Power User Focus Mode: Ignore other changes, focus only on active unit + bibliography
                    info!("Focus Mode: concentrating on unit {}", active_base);
                    affected_units = vec![active_base.to_string()];

                    // Auto-include bibliography units if they exist
                    for unit in &top_level_units {
                        let unit_path = if unit.ends_with(".tex") { unit.clone() } else { format!("{}.tex", unit) };
                        if let Some(content) = vfs.read_file(&unit_path) {
                            let content_str = String::from_utf8_lossy(&content);
                            if content_str.contains("\\bibliography") || content_str.contains("\\printbibliography") {
                                if !affected_units.contains(unit) {
                                    info!("Focus Mode: auto-including bibliography unit {}", unit);
                                    affected_units.push(unit.clone());
                                }
                            }
                        }
                    }
                } else {
                    // Standard Incremental: If we are actively editing an include unit, it's the primary candidate for includeonly
                    if !affected_units.contains(&active_base.to_string()) {
                        affected_units.push(active_base.to_string());
                    }
                }
            }
        }

        // 5. Automated Transparent \includeonly injection
        let mut optimized_latex = latex.to_string();
        let mut is_incremental = false;

        // Only inject if we have a subset of units affected
        // If everything changed or nothing changed (first compile), we don't necessarily want \includeonly
        // unless we want to force focus on what the user is editing.
        if !affected_units.is_empty() && affected_units.len() < top_level_units.len() {
            let include_only = format!("\\includeonly{{{}}}\n", affected_units.join(","));
            if let Some(pos) = optimized_latex.find("\\begin{document}") {
                optimized_latex.insert_str(pos, &include_only);
                is_incremental = true;
                info!("Incremental: only recompiling affected subtree: {:?}", affected_units);
            }
        }

        (self.apply_draft_mode(optimized_latex, draft), is_incremental, deltas)
    }

    fn apply_draft_mode(&self, mut latex: String, draft: bool) -> String {
        if draft {
            // Regex to find \documentclass and its optional arguments
            let re = Regex::new(r"\\documentclass(?:\[([^\]]*)\])?\{([^}]+)\}").unwrap();
            if let Some(caps) = re.captures(&latex) {
                let options = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let class = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                
                if !options.contains("draft") {
                    let new_options = if options.is_empty() {
                        "draft".to_string()
                    } else {
                        format!("draft,{}", options)
                    };
                    let new_documentclass = format!("\\documentclass[{}]{{{}}}", new_options, class);
                    latex = latex.replace(&caps[0], &new_documentclass);
                }
            }
            
            if let Some(pos) = latex.find("\\begin{document}") {
                let injection = "\n\\textbf{--- DRAFT MODE ACTIVE ---}\n";
                if !latex.contains(injection) {
                    latex.insert_str(pos + "\\begin{document}".len(), injection);
                }
            }
        } else {
            // Remove draft from options if present
            let re = Regex::new(r"\\documentclass\[([^\]]*)\]\{([^}]+)\}").unwrap();
            if let Some(caps) = re.captures(&latex) {
                let options = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let class = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                
                if options.contains("draft") {
                    let new_options: Vec<&str> = options.split(',')
                        .map(|s| s.trim())
                        .filter(|&s| s != "draft")
                        .collect();
                    
                    let new_documentclass = if new_options.is_empty() {
                        format!("\\documentclass{{{}}}", class)
                    } else {
                        format!("\\documentclass[{}]{{{}}}", new_options.join(","), class)
                    };
                    latex = latex.replace(&caps[0], &new_documentclass);
                }
            }
            // Also remove the draft mode visual indicator if it exists
            latex = latex.replace("\n\\textbf{--- DRAFT MODE ACTIVE ---}\n", "");
        }
        latex
    }

    fn collect_all_dependencies(&self, path: &str, vfs: &Vfs, out: &mut Vec<String>, visited: &mut HashSet<String>) {
        if visited.contains(path) { return; }
        visited.insert(path.to_string());
        
        if let Some(content_bytes) = vfs.read_file(path) {
            let content = String::from_utf8_lossy(&content_bytes);
            let re = INCLUDE_INPUT_REGEX.get_or_init(|| Regex::new(r"\\(?:include|input)\{([^}]+)\}").unwrap());
            for cap in re.captures_iter(&content) {
                let mut sub = cap[1].to_string();
                if !sub.ends_with(".tex") { sub.push_str(".tex"); }
                out.push(sub.clone());
                self.collect_all_dependencies(&sub, vfs, out, visited);
            }
        }
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
}
