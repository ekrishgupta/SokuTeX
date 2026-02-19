use ahash::AHashMap;

#[derive(Default)]
pub struct AutocompleteNode {
    children: AHashMap<char, AutocompleteNode>,
    is_word: bool,
    command: String,
}

pub struct AutocompleteEngine {
    root: AutocompleteNode,
}

impl AutocompleteEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            root: AutocompleteNode::default(),
        };

        // Seed with common high-frequency LaTeX commands
        let commands = vec![
            "\\begin", "\\end", "\\section", "\\subsection", "\\textbf",
            "\\textit", "\\include", "\\input", "\\usepackage", "\\documentclass",
            "\\maketitle", "\\tableofcontents", "\\itemize", "\\enumerate",
            "\\alpha", "\\beta", "\\gamma", "\\sum", "\\prod", "\\int", "\\infty",
        ];

        for cmd in commands {
            engine.insert(cmd);
        }

        engine
    }

    pub fn insert(&mut self, word: &str) {
        let mut curr = &mut self.root;
        for c in word.chars() {
            curr = curr.children.entry(c).or_default();
        }
        curr.is_word = true;
        curr.command = word.to_string();
    }

    pub fn suggest(&self, prefix: &str) -> Vec<String> {
        if prefix.is_empty() { return vec![]; }
        
        let mut curr = &self.root;
        for c in prefix.chars() {
            if let Some(node) = curr.children.get(&c) {
                curr = node;
            } else {
                return vec![];
            }
        }

        let mut results = Vec::new();
        self.collect_words(curr, &mut results);
        results
    }

    fn collect_words(&self, node: &AutocompleteNode, results: &mut Vec<String>) {
        if node.is_word {
            results.push(node.command.clone());
        }
        // Early exit if we have enough suggestions
        if results.len() > 10 { return; }
        
        for child in node.children.values() {
            self.collect_words(child, results);
            if results.len() > 10 { return; }
        }
    }
}
