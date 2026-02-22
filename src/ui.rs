use egui::{Color32, FontId, RichText, Visuals};
use crate::dependencies::DependencyNode;


#[derive(PartialEq)]
pub enum View {
    Dashboard,
    Editor,
}

pub struct ProjectItem {
    pub name: String,
    pub modified: String,
    pub path: String,
}

#[derive(PartialEq)]
pub enum DashTab {
    Dashboard,
    Library,
    Templates,
    Symbols,
    Settings,
}

pub struct Template {
    pub name: String,
    pub description: String,
    pub icon: String,
}

pub struct CompileError {
    pub line: usize,
    pub message: String,
}

#[derive(PartialEq, Clone, Copy)]
pub enum LatexTheme {
    Midnight,
    SoftGray,
}

pub struct Gui {
    pub view: View,
    pub theme: LatexTheme,
    pub active_tab: DashTab,
    pub ui_text: String,
    pub compile_status: String,
    pub selected_project: Option<String>,
    pub dash_selected_index: usize,
    pub template_selected_index: usize,
    pub search_text: String,
    pub projects: Vec<ProjectItem>,
    pub templates: Vec<Template>,
    pub errors: Vec<CompileError>,
    pub show_errors: bool,
    pub show_command_palette: bool,
    pub command_search_text: String,
    pub last_compile_text: String,
    pub prev_ui_text: String,
    pub compile_timer: std::time::Instant,
    pub compile_requested: bool,
    pub autocomplete: crate::autocomplete::AutocompleteEngine,
    pub draft_mode: bool,
    pub focus_mode: bool,
    pub compile_backend: crate::config::CompileBackend,
    pub dependency_tree: Option<DependencyNode>,
    pub show_dependencies: bool,
    pub show_bib_panel: bool,
    pub bib_entries: Vec<crate::bib::BibEntry>,
    pub bib_search: String,
    pub synctex: Option<crate::synctex::SyncTex>,
    pub sync_to_editor_request: Option<usize>, // line to scroll to
    pub sync_to_pdf_request: bool,
    pub pdf_scroll_target: Option<(usize, f32, f32)>, // (page, x, y)
    pub pdf_highlight_rect: Option<egui::Rect>,
    pub active_file_path: String,
    pub pdf_page_size: egui::Vec2, // Width, Height in points
    pub file_change_request: Option<String>,
    pub cursor_override: Option<usize>,
    pub selection_override: Option<(usize, usize)>,
    
    // PDF Interactive State
    pub pdf_zoom: f32,
    pub pdf_pan: egui::Vec2,
    pub vfs: Option<std::sync::Arc<crate::vfs::Vfs>>,
    pub image_cache: std::collections::HashMap<String, egui::TextureHandle>,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            view: View::Dashboard,
            theme: LatexTheme::Midnight,
            active_tab: DashTab::Dashboard,
            ui_text: String::new(),
            compile_status: "Idle".to_string(),
            selected_project: None,
            dash_selected_index: 0,
            template_selected_index: 0,
            search_text: String::new(),
            projects: vec![
                ProjectItem { name: "Quantum Mech Notes".into(), modified: "2m ago".into(), path: "~/physics/mech.tex".into() },
                ProjectItem { name: "Graph Theory HW".into(), modified: "1h ago".into(), path: "~/math/hw4.tex".into() },
                ProjectItem { name: "Dissertation Draft".into(), modified: "3d ago".into(), path: "~/uni/thesis/main.tex".into() },
                ProjectItem { name: "Abstract Algebra".into(), modified: "1w ago".into(), path: "~/math/algebra.tex".into() },
                ProjectItem { name: "CV 2024".into(), modified: "2w ago".into(), path: "~/personal/cv.tex".into() },
            ],
            templates: vec![
                Template { name: "Scientific Paper".into(), description: "Nature-style two column layout".into(), icon: "📄".into() },
                Template { name: "Modern CV".into(), description: "Minimalist engineering resume".into(), icon: "👤".into() },
                Template { name: "Presentation".into(), description: "Beamer-based slide deck".into(), icon: "🖼".into() },
                Template { name: "Lab Report".into(), description: "Structured data and formulas".into(), icon: "🧪".into() },
            ],
            errors: vec![
                CompileError { line: 12, message: "Undefined control sequence \\textbfz".into() },
            ],
            show_errors: false,
            show_command_palette: false,
            command_search_text: String::new(),
            last_compile_text: String::new(),
            prev_ui_text: String::new(),
            compile_timer: std::time::Instant::now(),
            compile_requested: false,
            autocomplete: crate::autocomplete::AutocompleteEngine::new(),
            draft_mode: false,
            focus_mode: false,
            compile_backend: crate::config::CompileBackend::Internal,
            dependency_tree: None,
            show_dependencies: true,
            show_bib_panel: false,
            bib_entries: Vec::new(),
            bib_search: String::new(),
            synctex: None,
            sync_to_editor_request: None,
            sync_to_pdf_request: false,
            pdf_scroll_target: None,
            pdf_highlight_rect: None,
            active_file_path: "main.tex".to_string(),
            pdf_page_size: egui::vec2(612.0, 792.0), // Default to Letter
            file_change_request: None,
            cursor_override: None,
            selection_override: None,
            pdf_zoom: 1.0,
            pdf_pan: egui::vec2(0.0, 0.0),
            vfs: None,
            image_cache: std::collections::HashMap::new(),
        }
    }

    pub fn refresh_bibliography(&mut self, bib_contents: Vec<String>) {
        self.bib_entries.clear();
        for content in bib_contents {
            let mut entries = crate::bib::BibParser::parse(&content);
            self.bib_entries.append(&mut entries);
        }
    }

    pub fn setup_visuals(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();
        
        // "Cool Minimal" Slate Palette
        let bg_color = Color32::from_rgb(18, 20, 23);
        let panel_color = Color32::from_rgb(13, 15, 17);
        let border_color = Color32::from_rgb(30, 33, 38);
        let text_color = Color32::from_rgb(160, 170, 180);
        
        visuals.panel_fill = panel_color;
        visuals.window_fill = bg_color;
        visuals.widgets.noninteractive.bg_fill = panel_color;
        visuals.widgets.noninteractive.fg_stroke.color = text_color;
        visuals.widgets.noninteractive.bg_stroke.color = border_color;
        
        visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
        visuals.widgets.inactive.fg_stroke.color = text_color;
        
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(25, 28, 32);
        visuals.widgets.hovered.fg_stroke.color = Color32::WHITE;
        
        visuals.widgets.active.bg_fill = Color32::from_rgb(30, 35, 40);
        visuals.widgets.active.fg_stroke.color = Color32::WHITE;
        
        visuals.selection.bg_fill = Color32::from_rgb(50, 60, 80);
        
        ctx.set_visuals(visuals);

        // Load Custom Fonts
        let mut fonts = egui::FontDefinitions::default();
        
        fonts.font_data.insert("premium_bold".to_owned(), 
            egui::FontData::from_static(include_bytes!("../assets/fonts/Outfit-Bold.ttf")));
        fonts.font_data.insert("premium_regular".to_owned(), 
            egui::FontData::from_static(include_bytes!("../assets/fonts/Outfit-Regular.ttf")));
        fonts.font_data.insert("mono_refined".to_owned(), 
            egui::FontData::from_static(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf")));

        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
            .insert(0, "premium_regular".to_owned());
        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap()
            .insert(0, "mono_refined".to_owned());
        
        // Custom family for the logo
        fonts.families.insert(egui::FontFamily::Name("logo_font".into()), vec!["premium_bold".into()]);

        ctx.set_fonts(fonts);
    }

    pub fn draw(&mut self, ctx: &egui::Context, pdf_tex_id: Option<egui::TextureId>) {
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::K)) {
            self.show_command_palette = !self.show_command_palette;
        }

        // Auto-Compile Detection (Near-instant latency)
        if self.ui_text != self.prev_ui_text {
            self.compile_timer = std::time::Instant::now();
        }
        
        if self.ui_text != self.last_compile_text {
            self.compile_requested = true;
        }
        
        self.prev_ui_text = self.ui_text.clone();

        match self.view {
            View::Dashboard => self.draw_dashboard(ctx),
            View::Editor => self.draw_editor(ctx, pdf_tex_id),
        }

        if self.show_command_palette {
            self.draw_command_palette(ctx);
        }
    }

    fn draw_command_palette(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.show_command_palette = false;
        }

        egui::Window::new("command_palette")
            .collapsible(false)
            .title_bar(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
            .fixed_size([500.0, 300.0])
            .frame(egui::Frame::none()
                .fill(Color32::from_rgb(15, 17, 20))
                .rounding(8.0)
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(40, 45, 50)))
                .shadow(egui::epaint::Shadow { 
                    offset: egui::vec2(0.0, 10.0),
                    blur: 30.0, 
                    spread: 2.0,
                    color: Color32::from_black_alpha(150) 
                }))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Search area
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(16.0, 12.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("🔍").size(14.0));
                                let resp = ui.add(egui::TextEdit::singleline(&mut self.command_search_text)
                                    .hint_text("Search actions or projects...")
                                    .frame(false)
                                    .desired_width(f32::INFINITY)
                                    .font(FontId::proportional(14.0)));
                                resp.request_focus();
                            });
                        });
                    
                    ui.separator();
                    
                    // Results area
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add_space(8.0);
                        if self.command_item(ui, "🏠 Go to Dashboard", "View your projects").clicked() {
                            self.view = View::Dashboard;
                            self.show_command_palette = false;
                        }
                        if self.command_item(ui, "🚀 Compile Document", "Run Tectonic on current file").clicked() {
                            self.compile_status = "BUSY".to_string();
                            self.show_command_palette = false;
                        }
                        self.command_item(ui, "📚 Open Library", "Browse your LaTeX collection");
                        self.command_item(ui, "🎨 Change Theme", "Switch high-contrast or light mode");
                        ui.add_space(8.0);
                    });
                });
            });
    }

    fn command_item(&mut self, ui: &mut egui::Ui, title: &str, subtitle: &str) -> egui::Response {
        let response = egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(16.0, 8.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).color(Color32::WHITE).size(13.0));
                    ui.label(RichText::new(subtitle).color(Color32::from_rgb(80, 85, 95)).size(10.0));
                });
            }).response;
        
        let response = response.interact(egui::Sense::click());
        if response.hovered() {
            ui.painter().rect_filled(response.rect, 2.0, Color32::from_rgb(25, 28, 32));
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }
        response
    }

    fn draw_dashboard(&mut self, ctx: &egui::Context) {
        // High-density keyboard navigation
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.dash_selected_index = (self.dash_selected_index + 1) % self.projects.len();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.dash_selected_index = (self.dash_selected_index + self.projects.len() - 1) % self.projects.len();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(p) = self.projects.get(self.dash_selected_index) {
                self.view = View::Editor;
                self.selected_project = Some(p.name.clone());
            }
        }

        egui::SidePanel::left("dashboard_sidebar")
            .width_range(180.0..=180.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(13, 15, 17)))
            .show(ctx, |ui| {
                // Branding Area - Aligned with traffic lights
                egui::Frame::none()
                    .inner_margin(egui::Margin { left: 16.0, right: 16.0, top: 12.0, bottom: 4.0 })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(64.0); // Clear traffic lights
                            ui.label(RichText::new("SokuTeX")
                                .font(FontId::new(16.0, egui::FontFamily::Name("logo_font".into())))
                                .color(Color32::WHITE)
                                .extra_letter_spacing(0.1));
                        });
                    });

                ui.add_space(32.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    
                    if self.nav_item(ui, "Dashboard", self.active_tab == DashTab::Dashboard).clicked() {
                        self.active_tab = DashTab::Dashboard;
                    }
                    if self.nav_item(ui, "Library", self.active_tab == DashTab::Library).clicked() {
                        self.active_tab = DashTab::Library;
                    }
                    if self.nav_item(ui, "Templates", self.active_tab == DashTab::Templates).clicked() {
                        self.active_tab = DashTab::Templates;
                    }
                    if self.nav_item(ui, "Symbols", self.active_tab == DashTab::Symbols).clicked() {
                        self.active_tab = DashTab::Symbols;
                    }
                    
                    ui.add_space(ui.available_height() - 40.0);
                    if self.nav_item(ui, "Settings", self.active_tab == DashTab::Settings).clicked() {
                        self.active_tab = DashTab::Settings;
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(10, 12, 14)))
            .show(ctx, |ui| {
                match self.active_tab {
                    DashTab::Dashboard => self.render_dashboard_content(ui),
                    DashTab::Library => {
                        ui.centered_and_justified(|ui| ui.label(RichText::new("Library View").color(Color32::WHITE)));
                    },
                    DashTab::Templates => self.render_templates_content(ui),
                    DashTab::Symbols => self.render_symbols_content(ui),
                    DashTab::Settings => self.render_settings_content(ui),
                }
            });
    }

    fn render_settings_content(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.add_space(24.0);
            ui.label(RichText::new("Application Settings").font(FontId::new(20.0, egui::FontFamily::Proportional)).color(Color32::WHITE).strong());
        });
        
        ui.add_space(32.0);
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("APPEARANCE").size(10.0).color(Color32::from_rgb(60, 65, 75)));
                    ui.add_space(12.0);
                    
                    ui.horizontal(|ui| {
                        if self.theme_option(ui, "Midnight", self.theme == LatexTheme::Midnight) {
                            self.theme = LatexTheme::Midnight;
                        }
                        ui.add_space(12.0);
                        if self.theme_option(ui, "Soft Gray", self.theme == LatexTheme::SoftGray) {
                            self.theme = LatexTheme::SoftGray;
                        }
                    });

                    ui.add_space(32.0);
                    ui.label(RichText::new("EDITOR BEHAVIOR").size(10.0).color(Color32::from_rgb(60, 65, 75)));
                    ui.add_space(12.0);
                    
                    ui.checkbox(&mut true, "Auto-compile on save");
                    ui.add_space(8.0);
                    ui.checkbox(&mut true, "Enable SyncTeX (Double-click to navigate)");
                    ui.add_space(8.0);
                    ui.checkbox(&mut false, "Show line numbers");
                });
            });
        });
    }

    fn theme_option(&self, ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
        let (bg, border) = if selected {
            (Color32::from_rgb(30, 35, 45), Color32::from_rgb(60, 100, 200))
        } else {
            (Color32::from_rgb(15, 17, 20), Color32::from_rgb(30, 33, 38))
        };

        let response = egui::Frame::none()
            .fill(bg)
            .rounding(6.0)
            .stroke(egui::Stroke::new(1.0, border))
            .inner_margin(egui::Margin::symmetric(24.0, 12.0))
            .show(ui, |ui| {
                ui.label(RichText::new(label).color(Color32::WHITE).size(13.0));
            }).response;
        
        let response = response.interact(egui::Sense::click());
        if response.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }
        response.clicked()
    }

    fn render_symbols_content(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.add_space(24.0);
            ui.label(RichText::new("Mathematical Symbols").font(FontId::new(20.0, egui::FontFamily::Proportional)).color(Color32::WHITE).strong());
        });
        
        ui.add_space(32.0);
        
        let symbols = vec![
            ("Σ", "\\sum"), ("Π", "\\prod"), ("∫", "\\int"), ("∞", "\\infty"),
            ("α", "\\alpha"), ("β", "\\beta"), ("γ", "\\gamma"), ("δ", "\\delta"),
            ("λ", "\\lambda"), ("μ", "\\mu"), ("π", "\\pi"), ("ω", "\\omega"),
            ("√", "\\sqrt{x}"), ("∂", "\\partial"), ("∇", "\\nabla"), ("∈", "\\in"),
            ("∀", "\\forall"), ("∃", "\\exists"), ("∄", "\\nexists"), ("∅", "\\emptyset"),
        ];

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
                        for (icon, code) in symbols {
                            if self.symbol_card(ui, icon, code).clicked() {
                                // Action item
                            }
                        }
                    });
                });
                ui.add_space(24.0);
            });
        });
    }

    fn symbol_card(&self, ui: &mut egui::Ui, icon: &str, code: &str) -> egui::Response {
        let response = egui::Frame::none()
            .fill(Color32::from_rgb(18, 20, 23))
            .rounding(4.0)
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(30, 33, 38)))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                ui.set_width(100.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(icon).size(20.0).color(Color32::WHITE));
                    ui.add_space(4.0);
                    ui.label(RichText::new(code).size(10.0).color(Color32::from_rgb(80, 85, 95)).font(FontId::monospace(9.0)));
                });
            }).response;
        
        let response = response.interact(egui::Sense::click());
        if response.hovered() {
            ui.painter().rect_stroke(response.rect, 4.0, egui::Stroke::new(1.0, Color32::from_rgb(60, 100, 200)));
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }
        response
    }

    fn render_templates_content(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.add_space(24.0);
            ui.label(RichText::new("Explore Templates").font(FontId::new(20.0, egui::FontFamily::Proportional)).color(Color32::WHITE).strong());
        });
        
        ui.add_space(32.0);
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(20.0, 20.0);
                        for i in 0..self.templates.len() {
                            let template = &self.templates[i];
                            if self.template_card(ui, template).clicked() {
                                self.template_selected_index = i;
                                self.view = View::Editor;
                                self.selected_project = Some(format!("New {}", template.name));
                                self.ui_text = format!("% New {} template\n\\documentclass{{article}}\n\\begin{{document}}\nHello SokuTeX!\n\\end{{document}}", template.name);
                            }
                        }
                    });
                });
                ui.add_space(24.0);
            });
        });
    }

    fn template_card(&self, ui: &mut egui::Ui, template: &Template) -> egui::Response {
        let (bg, border) = (Color32::from_rgb(15, 17, 20), Color32::from_rgb(30, 33, 38));
        
        let response = egui::Frame::none()
            .fill(bg)
            .rounding(8.0)
            .stroke(egui::Stroke::new(1.0, border))
            .inner_margin(egui::Margin::same(20.0))
            .show(ui, |ui| {
                ui.set_width(180.0);
                ui.set_height(140.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(&template.icon).size(24.0));
                    ui.add_space(12.0);
                    ui.label(RichText::new(&template.name).color(Color32::WHITE).font(FontId::new(14.0, egui::FontFamily::Proportional)).strong());
                    ui.add_space(4.0);
                    ui.label(RichText::new(&template.description).color(Color32::from_rgb(100, 110, 120)).font(FontId::new(11.0, egui::FontFamily::Proportional)));
                });
            }).response;

        let response = response.interact(egui::Sense::click());
        if response.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }
        response
    }

    fn render_dashboard_content(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.add_space(24.0);
            
            // Unified Search/Command Bar
            egui::Frame::none()
                .fill(Color32::from_rgb(18, 20, 23))
                .rounding(4.0)
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(30, 33, 38)))
                .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                .show(ui, |ui| {
                    ui.set_width(400.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⌘").color(Color32::from_rgb(80, 85, 95)));
                        ui.add(egui::TextEdit::singleline(&mut self.search_text)
                            .hint_text("Search or run command...")
                            .frame(false)
                            .desired_width(f32::INFINITY));
                    });
                });
            
            ui.add_space(ui.available_width() - 80.0);
            ui.label(RichText::new("KG").strong().color(Color32::WHITE));
        });

        ui.add_space(32.0);
        
        // High-Density Project List
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(24.0);
                ui.vertical(|ui| {
                    // Table Header
                    ui.horizontal(|ui| {
                        ui.add_sized([240.0, 10.0], egui::Label::new(RichText::new("NAME").size(10.0).color(Color32::from_rgb(60, 65, 75))));
                        ui.add_sized([350.0, 10.0], egui::Label::new(RichText::new("PATH").size(10.0).color(Color32::from_rgb(60, 65, 75))));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new("MODIFIED").size(10.0).color(Color32::from_rgb(60, 65, 75)));
                        });
                    });
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    for i in 0..self.projects.len() {
                        let is_selected = i == self.dash_selected_index;
                        let project = &self.projects[i];
                        let mut open_project = None;
                        
                        if self.project_row(ui, project, is_selected).clicked() {
                            open_project = Some(project.name.clone());
                        }

                        if open_project.is_some() {
                            self.view = View::Editor;
                            self.selected_project = open_project;
                        }
                    }
                });
            });
        });
        
        // Keyboard hints
        ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("↑↓ Navigate  •  ⏎ Open  •  ⌘N New").size(10.0).color(Color32::from_rgb(40, 45, 50)));
                ui.add_space(24.0);
            });
        });
    }

    fn project_row(&self, ui: &mut egui::Ui, project: &ProjectItem, selected: bool) -> egui::Response {
        let bg = if selected { Color32::from_rgb(25, 28, 35) } else { Color32::TRANSPARENT };
        let text_color = if selected { Color32::WHITE } else { Color32::from_rgb(160, 170, 180) };

        let response = egui::Frame::none()
            .fill(bg)
            .rounding(2.0)
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_sized([240.0, 18.0], egui::Label::new(RichText::new(&project.name).color(text_color).font(FontId::new(13.0, egui::FontFamily::Proportional))));
                    ui.add_sized([350.0, 18.0], egui::Label::new(RichText::new(&project.path).size(11.0).color(Color32::from_rgb(60, 70, 80))));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(&project.modified).size(11.0).color(Color32::from_rgb(100, 110, 120)));
                    });
                });
            }).response;

        let response = response.interact(egui::Sense::click());
        if response.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }
        response
    }

    fn nav_item(&self, ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response {
        let (color, bg) = if active { 
            (Color32::WHITE, Color32::from_rgb(25, 28, 32)) 
        } else { 
            (Color32::from_rgb(110, 120, 130), Color32::TRANSPARENT) 
        };
        
        let response = egui::Frame::none()
            .fill(bg)
            .rounding(4.0)
            .inner_margin(egui::Margin::symmetric(14.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_width(ui.available_width());
                    if active {
                        ui.add_space(4.0);
                    }
                    ui.label(RichText::new(label).color(color).font(FontId::new(13.0, egui::FontFamily::Proportional)));
                })
            }).response;

        let response = response.interact(egui::Sense::click());
        if response.hovered() {
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
        }
        response
    }

    fn draw_editor(&mut self, ctx: &egui::Context, pdf_tex_id: Option<egui::TextureId>) {
        egui::SidePanel::left("editor_panel")
            .min_width(350.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(10, 12, 14)))
            .show(ctx, |ui| {
                // Top Control Bar Area - Pinned to top edge
                egui::Frame::none()
                    .fill(Color32::from_rgb(10, 12, 14))
                    .inner_margin(egui::Margin { left: 16.0, right: 16.0, top: 12.0, bottom: 4.0 })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 24.0;
                            ui.add_space(60.0); // Offset for macOS traffic lights

                            // Visuals for buttons - integrated into the title bar
                            ui.visuals_mut().widgets.inactive.bg_fill = Color32::from_rgb(30, 32, 35);
                            ui.visuals_mut().widgets.hovered.bg_fill = Color32::from_rgb(45, 48, 52);

                            // Branding / Project Name
                            let title = self.selected_project.as_deref().unwrap_or("SokuTeX");
                            ui.label(RichText::new(title)
                                .font(FontId::new(16.0, egui::FontFamily::Name("logo_font".into())))
                                .color(Color32::WHITE)
                                .extra_letter_spacing(0.1));
                            
                            ui.add_space(-4.0);
                            
                            ui.spacing_mut().button_padding = egui::vec2(10.0, 3.0);
                            if ui.button(RichText::new("COMP").size(9.0).strong()).clicked() {
                                self.compile_status = "BUSY".to_string();
                                self.draft_mode = false; // Demand a full render
                                self.compile_requested = true;
                            }
                            
                            let draft_text = if self.draft_mode { "DRAFT: ON" } else { "DRAFT: OFF" };
                            if ui.button(RichText::new(draft_text).size(9.0).strong()).clicked() {
                                self.draft_mode = !self.draft_mode;
                                self.compile_requested = true; // Trigger re-compile on toggle
                            }

                            let focus_text = if self.focus_mode { "FOCUS: ON" } else { "FOCUS: OFF" };
                            if ui.button(RichText::new(focus_text).size(9.0).strong()).clicked() {
                                self.focus_mode = !self.focus_mode;
                                self.compile_requested = true; // Trigger re-compile on toggle
                            }

                            if ui.button(RichText::new("SYNC").size(9.0).strong()).clicked() {
                                self.sync_to_pdf_request = true;
                            }
                            
                            if ui.button(RichText::new("STX").size(9.0).strong()).clicked() {
                                self.view = View::Dashboard;
                            }

                            ui.separator();
                            
                            egui::ComboBox::from_id_source("backend_selector")
                                .selected_text(RichText::new(format!("{:?}", self.compile_backend)).size(9.0).strong())
                                .width(80.0)
                                .show_ui(ui, |ui| {
                                    use crate::config::CompileBackend;
                                    ui.selectable_value(&mut self.compile_backend, CompileBackend::Internal, "Internal");
                                    ui.selectable_value(&mut self.compile_backend, CompileBackend::Tectonic, "Tectonic");
                                    ui.selectable_value(&mut self.compile_backend, CompileBackend::Latexmk, "Latexmk");
                                });

                            if ui.button(RichText::new("TREE").size(9.0).strong()).clicked() {
                                self.show_dependencies = !self.show_dependencies;
                            }

                            if ui.button(RichText::new("BIB").size(9.0).strong()).clicked() {
                                self.show_bib_panel = !self.show_bib_panel;
                            }
                        });
                    });
                
                ui.add_space(4.0);

                if self.show_dependencies {
                    egui::SidePanel::left("dependency_tree_panel")
                        .resizable(true)
                        .default_width(150.0)
                        .width_range(100.0..=300.0)
                        .frame(egui::Frame::none().fill(Color32::from_rgb(13, 15, 17)))
                        .show_inside(ui, |ui| {
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);
                                ui.label(RichText::new("PROJECT TREE").size(10.0).color(Color32::from_rgb(100, 110, 120)).strong());
                            });
                            ui.add_space(8.0);
                            
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                let tree = self.dependency_tree.clone();
                                if let Some(tree) = tree {
                                    self.render_node_recursive(ui, &tree);
                                    
                                    ui.add_space(16.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(16.0);
                                        ui.label(RichText::new("OUTLINE").size(10.0).color(Color32::from_rgb(100, 110, 120)).strong());
                                    });
                                    ui.add_space(8.0);
                                    self.render_outline_recursive(ui, &tree);
                                } else {
                                    ui.horizontal(|ui| {
                                        ui.add_space(16.0);
                                        ui.label(RichText::new("No dependencies found").size(11.0).color(Color32::from_rgb(60, 65, 75)));
                                    });
                                }
                            });
                        });
                }

                if self.show_errors {
                    egui::TopBottomPanel::bottom("error_gutter")
                        .resizable(true)
                        .default_height(100.0)
                        .frame(egui::Frame::none().fill(Color32::from_rgb(18, 10, 12)))
                        .show_inside(ui, |ui| {
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.add_space(16.0);
                                ui.label(RichText::new("DIAGNOSTICS").size(10.0).color(Color32::from_rgb(180, 80, 90)).strong());
                            });
                            ui.add_space(8.0);
                            
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for error in &self.errors {
                                    ui.horizontal(|ui| {
                                        ui.add_space(16.0);
                                        ui.label(RichText::new(format!("L{}", error.line)).color(Color32::from_rgb(100, 110, 120)).font(FontId::monospace(11.0)));
                                        ui.add_space(8.0);
                                        ui.label(RichText::new(&error.message).color(Color32::from_rgb(200, 210, 220)).font(FontId::proportional(12.0)));
                                    });
                                    ui.add_space(4.0);
                                }
                            });
                        });
                }
                
                egui::ScrollArea::vertical()
                    .id_source("editor_scroll")
                    .show(ui, |ui| {
                        let mut tab_pressed = false;
                        ui.input_mut(|i| {
                            if i.consume_key(egui::Modifiers::NONE, egui::Key::Tab) {
                                tab_pressed = true;
                            }
                        });

                        let edit_output = egui::TextEdit::multiline(&mut self.ui_text)
                            .font(FontId::monospace(13.0))
                            .frame(false)
                            .margin(egui::Margin::same(32.0))
                            .code_editor()
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
                            .min_size(ui.available_size())
                            .layouter(&mut |ui, string, wrap_width| {
                                let mut layout_job = crate::syntax::LatexSyntaxHighlighter::format_text(string);
                                layout_job.wrap.max_width = wrap_width;
                                ui.fonts(|f| f.layout_job(layout_job))
                            })
                            .show(ui);
                            
                        let galley_pos = edit_output.galley_pos;
                        let galley = edit_output.galley.clone();
                        let resp = edit_output.response;

                        if let Some(pos) = resp.hover_pos() {
                            let relative_pos = pos - galley_pos;
                            let cursor = galley.cursor_from_pos(relative_pos);
                            let char_idx = cursor.ccursor.index;
                            
                            if char_idx <= self.ui_text.len() {
                                let start_search = char_idx.saturating_sub(60);
                                let prefix = &self.ui_text[start_search..char_idx];
                                
                                if let Some(slash_idx) = prefix.rfind('\\') {
                                    let start = start_search + slash_idx;
                                    
                                    if !self.ui_text[start..char_idx].contains('}') {
                                        if let Some(closing_idx) = self.ui_text[char_idx..].find('}') {
                                            let end = char_idx + closing_idx + 1;
                                            let hovered_text = &self.ui_text[start..end];
                                            
                                            if hovered_text.starts_with("\\cite{") {
                                                let key = &hovered_text[6..hovered_text.len()-1];
                                                if let Some(entry) = self.bib_entries.iter().find(|e| e.key == key) {
                                                    egui::show_tooltip_at_pointer(ui.ctx(), resp.id.with("hover_cite"), |ui| {
                                                        egui::Frame::none()
                                                            .fill(Color32::from_rgb(25, 28, 35))
                                                            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(45, 50, 60)))
                                                            .rounding(4.0)
                                                            .inner_margin(egui::Margin::same(12.0))
                                                            .show(ui, |ui| {
                                                                ui.set_max_width(320.0);
                                                                ui.label(RichText::new(entry.title.as_deref().unwrap_or("")).strong().color(Color32::WHITE));
                                                                ui.add_space(4.0);
                                                                ui.label(RichText::new(entry.author.as_deref().unwrap_or("")).color(Color32::from_rgb(150, 160, 170)));
                                                                ui.add_space(8.0);
                                                                ui.label(RichText::new(&entry.key).size(10.0).color(Color32::from_rgb(100, 110, 120)));
                                                            });
                                                    });
                                                }
                                            } else if hovered_text.starts_with("\\ref{") {
                                                let label = &hovered_text[5..hovered_text.len()-1];
                                                if let Some(ref vfs) = self.vfs {
                                                    if !self.image_cache.contains_key(label) {
                                                        let search_term = label.replace("fig:", "");
                                                        let mut image_data = None;
                                                        for entry in vfs.get_all_files().iter() {
                                                            let file_name = entry.key();
                                                            if (file_name.ends_with(".png") || file_name.ends_with(".jpg") || file_name.ends_with(".jpeg")) && file_name.contains(&search_term) {
                                                                image_data = Some(entry.value().clone());
                                                                break;
                                                            }
                                                        }
                                                        
                                                        let dummy_img = egui::ColorImage::from_rgba_unmultiplied([1, 1], &[0, 0, 0, 0]);
                                                        let mut texture = ui.ctx().load_texture(format!("dummy_{}", label), dummy_img, egui::TextureOptions::LINEAR);
                                                        
                                                        if let Some(data) = image_data {
                                                            if let Ok(img) = image::load_from_memory(&data) {
                                                                let size = [img.width() as _, img.height() as _];
                                                                let image_buffer = img.to_rgba8();
                                                                let pixels = image_buffer.as_flat_samples();
                                                                let slice = pixels.as_slice();
                                                                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, slice);
                                                                texture = ui.ctx().load_texture(format!("img_{}", label), color_image, egui::TextureOptions::LINEAR);
                                                            }
                                                        }
                                                        self.image_cache.insert(label.to_string(), texture);
                                                    }
                                                    
                                                    if let Some(texture) = self.image_cache.get(label) {
                                                        if texture.size() != [1, 1] { // not dummy
                                                            egui::show_tooltip_at_pointer(ui.ctx(), resp.id.with("hover_ref"), |ui| {
                                                                egui::Frame::none()
                                                                    .fill(Color32::from_rgb(25, 28, 35))
                                                                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(45, 50, 60)))
                                                                    .rounding(4.0)
                                                                    .inner_margin(egui::Margin::same(8.0))
                                                                    .show(ui, |ui| {
                                                                        let size = texture.size();
                                                                        let aspect = size[0] as f32 / size[1] as f32;
                                                                        let mut w = 250.0;
                                                                        let mut h = w / aspect;
                                                                        if h > 200.0 {
                                                                            h = 200.0;
                                                                            w = h * aspect;
                                                                        }
                                                                        ui.image(egui::load::SizedTexture::new(texture.id(), egui::vec2(w, h)));
                                                                        ui.add_space(4.0);
                                                                        ui.label(RichText::new(format!("Figure: {}", label)).size(10.0).color(Color32::from_rgb(150, 160, 170)));
                                                                    });
                                                            });
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if resp.double_clicked() {
                            self.sync_to_pdf_request = true;
                        }

                        if let Some(line) = self.sync_to_editor_request.take() {
                            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), resp.id) {
                                let mut char_idx = 0;
                                for (i, l) in self.ui_text.lines().enumerate() {
                                    if i + 1 == line {
                                        break;
                                    }
                                    char_idx += l.len() + 1;
                                }
                                let ccursor = egui::text::CCursor::new(char_idx);
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                                state.store(ui.ctx(), resp.id);
                                ui.ctx().memory_mut(|m| m.request_focus(resp.id));
                            }
                        }

                        if let Some(char_idx) = self.cursor_override.take() {
                            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), resp.id) {
                                let ccursor = egui::text::CCursor::new(char_idx);
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                                state.store(ui.ctx(), resp.id);
                                ui.ctx().memory_mut(|m| m.request_focus(resp.id));
                            }
                        }

                        if let Some((start_idx, end_idx)) = self.selection_override.take() {
                            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), resp.id) {
                                let ccursor_start = egui::text::CCursor::new(start_idx);
                                let ccursor_end = egui::text::CCursor::new(end_idx);
                                state.cursor.set_char_range(Some(egui::text::CCursorRange::two(ccursor_start, ccursor_end)));
                                state.store(ui.ctx(), resp.id);
                                ui.ctx().memory_mut(|m| m.request_focus(resp.id));
                            }
                        }

                        if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), resp.id) {
                            // Snippet / Tab logic
                            if tab_pressed {
                                let char_idx = state.cursor.char_range().map(|r| r.primary.index).unwrap_or(0);
                                let mut temp_editor = crate::editor::Editor::new();
                                temp_editor.buffer = ropey::Rope::from_str(&self.ui_text);
                                temp_editor.cursor = char_idx;
                                
                                if let Some((start, end)) = temp_editor.expand_or_jump_snippet(&self.autocomplete) {
                                    self.ui_text = temp_editor.buffer.to_string();
                                    
                                    let ccursor_start = egui::text::CCursor::new(start);
                                    let ccursor_end = egui::text::CCursor::new(end);
                                    state.cursor.set_char_range(Some(egui::text::CCursorRange::two(ccursor_start, ccursor_end)));
                                    state.clone().store(ui.ctx(), resp.id);
                                    ui.ctx().memory_mut(|m| m.request_focus(resp.id));
                                } else {
                                    // Normally insert tab if no snippet stuff
                                    let mut new_text: String = self.ui_text.chars().take(char_idx).collect();
                                    new_text.push_str("    ");
                                    let suffix: String = self.ui_text.chars().skip(char_idx).collect();
                                    new_text.push_str(&suffix);
                                    self.ui_text = new_text;
                                    
                                    let ccursor = egui::text::CCursor::new(char_idx + 4);
                                    state.cursor.set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                                    state.clone().store(ui.ctx(), resp.id);
                                    ui.ctx().memory_mut(|m| m.request_focus(resp.id));
                                }
                            }

                            // Simple prefix matching based on cursor
                            let char_idx = state.cursor.char_range().map(|r| r.primary.index).unwrap_or(0);
                            let text_up_to_cursor: String = self.ui_text.chars().take(char_idx).collect();
                            if let Some(last_backslash) = text_up_to_cursor.rfind('\\') {
                                let prefix = &text_up_to_cursor[last_backslash..];
                                if !prefix.contains(' ') && prefix.len() > 1 {
                                    let suggestions = self.autocomplete.suggest(prefix);
                                    if !suggestions.is_empty() {
                                        egui::Area::new(egui::Id::new("autocomplete_area"))
                                            .fixed_pos(resp.rect.left_top() + egui::vec2(64.0, 64.0)) 
                                            .show(ui.ctx(), |ui| {
                                                egui::Frame::popup(ui.style())
                                                    .fill(Color32::from_rgb(25, 28, 32))
                                                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(45, 50, 60)))
                                                    .show(ui, |ui| {
                                                        ui.set_width(150.0);
                                                        for suggestion in suggestions {
                                                            if ui.selectable_label(false, &suggestion).clicked() {
                                                                let mut new_text: String = self.ui_text.chars().take(last_backslash).collect();
                                                                new_text.push_str(&suggestion);
                                                                let suffix: String = self.ui_text.chars().skip(char_idx).collect();
                                                                new_text.push_str(&suffix);
                                                                self.ui_text = new_text;
                                                            }
                                                        }
                                                    });
                                            });
                                    }
                                }
                            }

                            // Handle Forward Sync
                            if self.sync_to_pdf_request {
                                self.sync_to_pdf_request = false;
                                if let Some(ref stx) = self.synctex {
                                    // 07. Calculate line number from char index
                                    let mut line_num = 1;
                                    let mut current_idx = 0;
                                    for line in self.ui_text.lines() {
                                        let line_len = line.len();
                                        if current_idx + line_len >= char_idx {
                                            break;
                                        }
                                        current_idx += line_len + 1; // +1 for newline
                                        line_num += 1;
                                    }
                                    
                                    if let Some(node) = stx.forward_sync(line_num, 1) {
                                        self.pdf_scroll_target = Some((node.page as usize, node.x, node.y));
                                        
                                        // Calculate highlight rect (Letter size: 612 x 792)
                                        let x_ratio = node.x / 612.0;
                                        let y_ratio = node.y / 792.0;
                                        let w_ratio = node.width / 612.0;
                                        let h_ratio = node.height / 792.0;
                                        let d_ratio = node.depth / 792.0;
                                        
                                        self.pdf_highlight_rect = Some(egui::Rect::from_min_size(
                                            egui::pos2(x_ratio, y_ratio - h_ratio),
                                            egui::vec2(w_ratio, h_ratio + d_ratio)
                                        ));
                                    }
                                }
                            }
                        }
                    });

                if self.show_bib_panel {
                    egui::SidePanel::right("bib_sidebar")
                        .resizable(true)
                        .default_width(280.0)
                        .frame(egui::Frame::none().fill(Color32::from_rgb(15, 17, 20)))
                        .show_inside(ui, |ui| {
                            self.draw_bib_panel(ui);
                        });
                }
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(255, 255, 255))) // PDF usually white base
            .show(ctx, |ui| {
                if let Some(tex_id) = pdf_tex_id {
                    let image_size = ui.available_size();
                    
                    // Handle Zoom/Pan Interactions
                    let response = ui.interact(ui.available_rect_before_wrap(), ui.id(), egui::Sense::drag());
                    
                    if response.dragged() {
                        self.pdf_pan += response.drag_delta() / (image_size / 2.0);
                    }

                    // Handle Scroll-based Zoom
                    let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
                    if scroll_delta != 0.0 {
                        let zoom_factor = 1.0 + (scroll_delta / 100.0);
                        self.pdf_zoom *= zoom_factor;
                        // Limit zoom
                        self.pdf_zoom = self.pdf_zoom.clamp(0.1, 10.0);
                    }
                    
                    // We draw the PDF as a background in wgpu now, but we still want to show the texture in egui
                    // if we want to use egui's layout. However, the requirement was to use a custom shader.
                    // My custom shader is already drawing the background.
                    // Let's draw a transparent image here to capture clicks and provide interaction.
                    let image_response = ui.image(egui::load::SizedTexture::new(tex_id, image_size));
                    
                    // Render SyncTeX highlight (needs to be adjusted for zoom/pan)
                    if let Some(rect_ratio) = self.pdf_highlight_rect {
                        // Adjusted for zoom/pan
                        let scaled_rect = egui::Rect::from_min_size(
                            egui::pos2(rect_ratio.min.x * self.pdf_zoom + self.pdf_pan.x, rect_ratio.min.y * self.pdf_zoom + self.pdf_pan.y),
                            rect_ratio.size() * self.pdf_zoom
                        );

                        let screen_rect = egui::Rect::from_min_size(
                            image_response.rect.min + egui::vec2(scaled_rect.min.x * image_size.x, scaled_rect.min.y * image_size.y),
                            egui::vec2(scaled_rect.width() * image_size.x, scaled_rect.height() * image_size.y)
                        );
                        ui.painter().rect_filled(screen_rect, 0.0, Color32::from_rgba_unmultiplied(255, 255, 0, 80));
                    }

                    // Floating Controls for PDF
                    egui::Area::new(egui::Id::new("pdf_controls"))
                        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
                        .show(ctx, |ui| {
                            egui::Frame::none()
                                .fill(Color32::from_rgb(30, 32, 35))
                                .rounding(4.0)
                                .inner_margin(egui::Margin::same(8.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("{:.0}%", self.pdf_zoom * 100.0)).size(10.0).color(Color32::WHITE));
                                        if ui.button(RichText::new("RESET").size(10.0)).clicked() {
                                            self.pdf_zoom = 1.0;
                                            self.pdf_pan = egui::vec2(0.0, 0.0);
                                        }
                                    });
                                });
                        });

                    if response.double_clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let relative_pos = pos - response.rect.min;
                            let x_ratio = relative_pos.x / image_size.x;
                            let y_ratio = relative_pos.y / image_size.y;
                            
                            if let Some(ref stx) = self.synctex {
                                // Use the actual page dimensions instead of hardcoded 612x792
                                let pdf_x = x_ratio * self.pdf_page_size.x;
                                let pdf_y = y_ratio * self.pdf_page_size.y;
                                
                                if let Some(node) = stx.backward_sync(1, pdf_x, pdf_y) {
                                    self.sync_to_editor_request = Some(node.line as usize);
                                    
                                    // Update highlight for inverse sync too
                                    let x_ratio = node.x / self.pdf_page_size.x;
                                    let y_ratio = node.y / self.pdf_page_size.y;
                                    let w_ratio = node.width / self.pdf_page_size.x;
                                    let h_ratio = node.height / self.pdf_page_size.y;
                                    let d_ratio = node.depth / self.pdf_page_size.y;
                                    
                                    self.pdf_highlight_rect = Some(egui::Rect::from_min_size(
                                        egui::pos2(x_ratio, y_ratio - h_ratio),
                                        egui::vec2(w_ratio, h_ratio + d_ratio)
                                    ));
                                }
                            }

                        }
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("...").color(Color32::from_rgb(200, 200, 200)));
                    });
                }
            });
    }

    fn render_node_recursive(&mut self, ui: &mut egui::Ui, node: &DependencyNode) {
        let has_children = !node.children.is_empty();
        let is_active = self.active_file_path == node.name;
        
        let icon = if node.name.ends_with(".tex") {
            "📄"
        } else if node.name.ends_with(".bib") {
            "📚"
        } else if node.name.ends_with(".sty") || node.name.ends_with(".cls") {
            "🛠"
        } else {
            "📝"
        };

        let color = if is_active {
            Color32::from_rgb(100, 160, 255)
        } else if has_children {
            Color32::WHITE
        } else {
            Color32::from_rgb(160, 170, 180)
        };

        let label = RichText::new(format!("{} {}", icon, node.name))
            .size(11.5)
            .color(color);

        if has_children {
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id(&node.name), true)
                .show_header(ui, |ui| {
                    if ui.selectable_label(is_active, label).clicked() {
                        self.file_change_request = Some(node.name.clone());
                    }
                })
                .body(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    for i in 0..node.children.len() {
                        let child = node.children[i].clone();
                        self.render_node_recursive(ui, &child);
                    }
                });
        } else {
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                if ui.selectable_label(is_active, label).clicked() {
                    self.file_change_request = Some(node.name.clone());
                }
            });
        }
    }

    fn render_outline_recursive(&mut self, ui: &mut egui::Ui, node: &DependencyNode) {
        for item in &node.outline {
            ui.horizontal(|ui| {
                ui.add_space(16.0 + (item.level as f32) * 12.0);
                
                let icon = match item.level {
                    0 => "📖",
                    1 => "🔖",
                    2 => "🔹",
                    _ => "▫",
                };

                let label = RichText::new(format!("{} {}", icon, item.title))
                    .size(11.0)
                    .color(Color32::from_rgb(200, 200, 200));

                if ui.selectable_label(false, label).clicked() {
                    self.file_change_request = Some(item.file_name.clone());
                    self.sync_to_editor_request = Some(item.line);
                    self.sync_to_pdf_request = true;
                }
            });
        }
        for child in &node.children {
            self.render_outline_recursive(ui, child);
        }
    }

    fn draw_bib_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(RichText::new("BIBLIOGRAPHY").size(10.0).color(Color32::from_rgb(100, 110, 120)).strong());
        });
        ui.add_space(8.0);

        // Search bar
        egui::Frame::none()
            .fill(Color32::from_rgb(25, 28, 32))
            .rounding(4.0)
            .inner_margin(egui::Margin::symmetric(8.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔍").size(10.0));
                    ui.add(egui::TextEdit::singleline(&mut self.bib_search)
                        .hint_text("Search references...")
                        .frame(false)
                        .desired_width(f32::INFINITY));
                });
            });

        ui.add_space(8.0);
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let search = self.bib_search.to_lowercase();
            for entry in &self.bib_entries {
                let matches = search.is_empty() || 
                    entry.key.to_lowercase().contains(&search) ||
                    entry.author.as_ref().map(|a| a.to_lowercase().contains(&search)).unwrap_or(false) ||
                    entry.title.as_ref().map(|t| t.to_lowercase().contains(&search)).unwrap_or(false);

                if matches {
                    let response = egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&entry.key).color(Color32::from_rgb(60, 120, 220)).strong().size(11.0));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(&entry.entry_type).size(9.0).color(Color32::from_rgb(80, 85, 95)));
                                    });
                                });
                                if let Some(title) = &entry.title {
                                    ui.label(RichText::new(title).size(12.0).color(Color32::WHITE));
                                }
                                if let Some(author) = &entry.author {
                                    ui.label(RichText::new(author).size(10.0).color(Color32::from_rgb(120, 130, 140)));
                                }
                            });
                        }).response;

                    let response = response.interact(egui::Sense::click());
                    if response.hovered() {
                        ui.painter().rect_filled(response.rect, 2.0, Color32::from_rgb(30, 35, 45));
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    }

                    if response.clicked() {
                        let cite = format!("\\cite{{{}}}", entry.key);
                        // Simple insertion at the end of the current buffer for now
                        self.ui_text.push_str(&cite);
                    }
                }
            }
        });
    }
}

