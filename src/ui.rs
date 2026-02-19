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

pub struct Gui {
    pub view: View,
    pub ui_text: String,
    pub compile_status: String,
    pub selected_project: Option<String>,
    pub dash_selected_index: usize,
    pub search_text: String,
    pub projects: Vec<ProjectItem>,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            view: View::Dashboard,
            ui_text: String::new(),
            compile_status: "Idle".to_string(),
            selected_project: None,
            dash_selected_index: 0,
            search_text: String::new(),
            projects: vec![
                ProjectItem { name: "Quantum Mech Notes".into(), modified: "2m ago".into(), path: "~/physics/mech.tex".into() },
                ProjectItem { name: "Graph Theory HW".into(), modified: "1h ago".into(), path: "~/math/hw4.tex".into() },
                ProjectItem { name: "Dissertation Draft".into(), modified: "3d ago".into(), path: "~/uni/thesis/main.tex".into() },
                ProjectItem { name: "Abstract Algebra".into(), modified: "1w ago".into(), path: "~/math/algebra.tex".into() },
                ProjectItem { name: "CV 2024".into(), modified: "2w ago".into(), path: "~/personal/cv.tex".into() },
            ],
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
        match self.view {
            View::Dashboard => self.draw_dashboard(ctx),
            View::Editor => self.draw_editor(ctx, pdf_tex_id),
        }
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
            .width_range(160.0..=160.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(13, 15, 17)))
            .show(ctx, |ui| {
                ui.add_space(24.0);
                ui.horizontal(|ui| {
                    ui.add_space(24.0);
                    ui.label(RichText::new("SokuTeX")
                        .font(FontId::new(16.0, egui::FontFamily::Name("logo_font".into())))
                        .color(Color32::WHITE));
                });

                ui.add_space(32.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    self.nav_item(ui, "Dashboard", true);
                    self.nav_item(ui, "Library", false);
                    self.nav_item(ui, "Templates", false);
                    ui.add_space(ui.available_height() - 40.0);
                    self.nav_item(ui, "Settings", false);
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(10, 12, 14)))
            .show(ctx, |ui| {
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
                    ui.horizontal(|ui| {
                        ui.add_space(24.0);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.set_width(ui.available_width() - 24.0);
                                ui.label(RichText::new("NAME").size(10.0).color(Color32::from_rgb(60, 65, 75)));
                                ui.add_space(200.0);
                                ui.label(RichText::new("PATH").size(10.0).color(Color32::from_rgb(60, 65, 75)));
                                ui.add_space(ui.available_width() - 100.0);
                                ui.label(RichText::new("MODIFIED").size(10.0).color(Color32::from_rgb(60, 65, 75)));
                            });
                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(8.0);

                            for (i, project) in self.projects.iter().enumerate() {
                                let is_selected = i == self.dash_selected_index;
                                if self.project_row(ui, project, is_selected).clicked() {
                                    self.view = View::Editor;
                                    self.selected_project = Some(project.name.clone());
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
            });
    }

    fn project_row(&mut self, ui: &mut egui::Ui, project: &ProjectItem, selected: bool) -> egui::Response {
        let bg = if selected { Color32::from_rgb(25, 28, 35) } else { Color32::TRANSPARENT };
        let text_color = if selected { Color32::WHITE } else { Color32::from_rgb(160, 170, 180) };

        let response = egui::Frame::none()
            .fill(bg)
            .rounding(2.0)
            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(&project.name).color(text_color).font(FontId::new(13.0, egui::FontFamily::Proportional)));
                    ui.add_space(150.0);
                    ui.label(RichText::new(&project.path).size(11.0).color(Color32::from_rgb(60, 70, 80)));
                    ui.with_layout(egui::Layout::right_to_right(egui::Align::Center), |ui| {
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

    fn nav_item(&mut self, ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response {
        let color = if active { Color32::WHITE } else { Color32::from_rgb(100, 110, 120) };
        let bg = if active { Color32::from_rgb(20, 22, 25) } else { Color32::TRANSPARENT };
        
        egui::Frame::none()
            .fill(bg)
            .rounding(2.0)
            .inner_margin(egui::Margin::symmetric(16.0, 6.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(label).color(color).font(FontId::new(12.0, egui::FontFamily::Proportional)));
                })
            }).response.interact(egui::Sense::click())
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
                            }
                            let _ = ui.button(RichText::new("SYNC").size(9.0).strong());
                            
                            if ui.button(RichText::new("STX").size(9.0).strong()).clicked() {
                                self.view = View::Dashboard;
                            }
                        });
                    });
                
                ui.add_space(4.0);
                
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
