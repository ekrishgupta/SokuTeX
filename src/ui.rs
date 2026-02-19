use egui::{Color32, FontId, RichText, Visuals};

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
                    extrusion: 30.0, 
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
            ui.horizontal_wrapped(|ui| {
                ui.add_space(24.0);
                ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
                
                for (icon, code) in symbols {
                    if self.symbol_card(ui, icon, code).clicked() {
                        // In real app, copy to clipboard or insert at cursor
                    }
                }
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
            ui.horizontal_wrapped(|ui| {
                ui.add_space(24.0);
                ui.spacing_mut().item_spacing = egui::vec2(20.0, 20.0);
                
                for i in 0..self.templates.len() {
                    let template = &self.templates[i];
                    if self.template_card(ui, template).clicked() {
                        self.template_selected_index = i;
                        // For now just open editor with placeholder
                        self.view = View::Editor;
                        self.selected_project = Some(format!("New {}", template.name));
                        self.ui_text = format!("% New {} template\n\\documentclass{{article}}\n\\begin{{document}}\nHello SokuTeX!\n\\end{{document}}", template.name);
                    }
                }
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
                                self.show_errors = !self.show_errors; // Toggle for demo
                            }
                            let _ = ui.button(RichText::new("SYNC").size(9.0).strong());
                            
                            if ui.button(RichText::new("STX").size(9.0).strong()).clicked() {
                                self.view = View::Dashboard;
                            }
                        });
                    });
                
                ui.add_space(4.0);

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
                        let _resp = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(&mut self.ui_text)
                                .font(FontId::monospace(13.0))
                                .frame(false)
                                .margin(egui::Margin::same(32.0))
                                .code_editor()
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                        );
                    });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(255, 255, 255))) // PDF usually white base
            .show(ctx, |ui| {
                if let Some(tex_id) = pdf_tex_id {
                    // Fill vertically and horizontally
                    ui.image(egui::load::SizedTexture::new(tex_id, ui.available_size()));
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("...").color(Color32::from_rgb(200, 200, 200)));
                    });
                }
            });
    }
}
