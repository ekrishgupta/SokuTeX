use egui::{Color32, TextFormat};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LatexTokenType {
    Command,
    Math,
    Comment,
    Generic,
    Bracket,
}

pub struct LatexSyntaxHighlighter;

impl LatexSyntaxHighlighter {
    pub fn tokenize(text: &str) -> Vec<(String, LatexTokenType)> {
        let mut tokens = Vec::new();
        let mut chars = text.chars().peekable();
        let mut current = String::new();
        let mut in_math = false;

        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    if !current.is_empty() {
                        tokens.push((current.clone(), if in_math { LatexTokenType::Math } else { LatexTokenType::Generic }));
                        current.clear();
                    }
                    current.push(c);
                    while let Some(&next) = chars.peek() {
                        if next.is_alphabetic() {
                            current.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push((current.clone(), LatexTokenType::Command));
                    current.clear();
                }
                '$' => {
                    if !current.is_empty() {
                        tokens.push((current.clone(), if in_math { LatexTokenType::Math } else { LatexTokenType::Generic }));
                        current.clear();
                    }
                    in_math = !in_math;
                    tokens.push(("$".into(), LatexTokenType::Math));
                }
                '%' => {
                    if !current.is_empty() {
                        tokens.push((current.clone(), if in_math { LatexTokenType::Math } else { LatexTokenType::Generic }));
                        current.clear();
                    }
                    current.push(c);
                    while let Some(&next) = chars.peek() {
                        if next == '\n' { break; }
                        current.push(chars.next().unwrap());
                    }
                    tokens.push((current.clone(), LatexTokenType::Comment));
                    current.clear();
                }
                '{' | '}' | '[' | ']' => {
                    if !current.is_empty() {
                        tokens.push((current.clone(), if in_math { LatexTokenType::Math } else { LatexTokenType::Generic }));
                        current.clear();
                    }
                    tokens.push((c.to_string(), LatexTokenType::Bracket));
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if !current.is_empty() {
            tokens.push((current, if in_math { LatexTokenType::Math } else { LatexTokenType::Generic }));
        }

        tokens
    }

    pub fn format_text(text: &str) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        let tokens = Self::tokenize(text);

        for (content, token_type) in tokens {
            let color = match token_type {
                LatexTokenType::Command => Color32::from_rgb(200, 100, 255), // Purple
                LatexTokenType::Math => Color32::from_rgb(100, 255, 100),    // Green
                LatexTokenType::Comment => Color32::from_rgb(80, 85, 95),    // Dim gray
                LatexTokenType::Bracket => Color32::from_rgb(255, 200, 100), // Orange
                LatexTokenType::Generic => Color32::from_rgb(200, 200, 200), // Off-white
            };

            job.append(
                &content,
                0.0,
                TextFormat {
                    font_id: egui::FontId::monospace(12.0),
                    color,
                    ..Default::default()
                },
            );
        }

        job
    }
}
