use egui::{Color32, FontId, RichText, Visuals};

#[derive(PartialEq)]
pub enum View {
    Dashboard,
    Editor,
}

pub struct Gui {
    pub view: View,
    pub ui_text: String,
    pub compile_status: String,
    pub selected_project: Option<String>,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            view: View::Dashboard,
            ui_text: String::new(),
            compile_status: "Idle".to_string(),
            selected_project: None,
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
        egui::SidePanel::left("dashboard_sidebar")
            .min_width(200.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(13, 15, 17)))
            .show(ctx, |ui| {
                ui.add_space(24.0);
                ui.horizontal(|ui| {
                    ui.add_space(64.0);
                    ui.label(RichText::new("SokuTeX")
                        .font(FontId::new(16.0, egui::FontFamily::Name("logo_font".into())))
                        .color(Color32::WHITE));
                });

                ui.add_space(32.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;
                    
                    self.nav_item(ui, "Recent", true);
                    self.nav_item(ui, "Projects", false);
                    self.nav_item(ui, "Templates", false);
                    
                    ui.add_space(ui.available_height() - 60.0);
                    self.nav_item(ui, "Settings", false);
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(10, 12, 14)))
            .show(ctx, |ui| {
                ui.add_space(24.0);
                ui.horizontal(|ui| {
                    ui.add_space(32.0);
                    
                    // Search Bar Mock
                    egui::Frame::none()
                        .fill(Color32::from_rgb(18, 20, 23))
                        .rounding(6.0)
                        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(30, 33, 38)))
                        .inner_margin(egui::Margin::symmetric(12.0, 6.0))
                        .show(ui, |ui| {
                            ui.set_width(300.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("🔍").size(10.0));
                                ui.label(RichText::new("Search documents...").color(Color32::from_rgb(80, 85, 95)).size(12.0));
                            });
                        });
                    
                    ui.add_space(ui.available_width() - 120.0);
                    
                    // Profile Mock
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("KG").color(Color32::WHITE).font(FontId::new(12.0, egui::FontFamily::Proportional)).strong());
                        ui.add_space(4.0);
                        egui::Frame::none()
                            .fill(Color32::from_rgb(40, 45, 60))
                            .rounding(4.0)
                            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new("PRO").color(Color32::from_rgb(150, 170, 255)).size(9.0).strong());
                            });
                    });
                });

                ui.add_space(40.0);
                ui.horizontal(|ui| {
                    ui.add_space(32.0);
                    ui.label(RichText::new("Recent Work").font(FontId::new(20.0, egui::FontFamily::Proportional)).color(Color32::WHITE).strong());
                });
                
                ui.add_space(24.0);
                
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.add_space(32.0);
                        ui.spacing_mut().item_spacing = egui::vec2(24.0, 24.0);
                        
                        // Action Card: Create New
                        if self.project_card(ui, "+ New Project", "Create from template", true).clicked() {
                            // Logic to create new
                        }

                        // Mock Projects
                        if self.project_card(ui, "Quantum Mech Notes", "2 hours ago", false).clicked() {
                            self.view = View::Editor;
                            self.selected_project = Some("Quantum Mech Notes".into());
                        }
                        
                        if self.project_card(ui, "Graph Theory HW", "Yesterday", false).clicked() {
                            self.view = View::Editor;
                            self.selected_project = Some("Graph Theory HW".into());
                        }

                        if self.project_card(ui, "Dissertation Draft", "3 days ago", false).clicked() {
                            self.view = View::Editor;
                            self.selected_project = Some("Dissertation Draft".into());
                        }
                    });
                });
            });
    }

    fn project_card(&mut self, ui: &mut egui::Ui, title: &str, subtitle: &str, is_action: bool) -> egui::Response {
        let (bg, border, text) = if is_action {
            (Color32::from_rgb(30, 35, 45), Color32::from_rgb(50, 60, 80), Color32::WHITE)
        } else {
            (Color32::from_rgb(15, 17, 20), Color32::from_rgb(30, 33, 38), Color32::from_rgb(180, 190, 200))
        };

        let response = egui::Frame::none()
            .fill(bg)
            .rounding(8.0)
            .stroke(egui::Stroke::new(1.0, border))
            .inner_margin(egui::Margin::same(20.0))
            .show(ui, |ui| {
                ui.set_width(200.0);
                ui.set_height(140.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).color(text).font(FontId::new(14.0, egui::FontFamily::Proportional)).strong());
                    ui.add_space(4.0);
                    ui.label(RichText::new(subtitle).color(Color32::from_rgb(100, 110, 120)).font(FontId::new(11.0, egui::FontFamily::Proportional)));
                    
                    ui.add_space(ui.available_height() - 20.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("LATEX").color(Color32::from_rgb(60, 70, 80)).font(FontId::new(9.0, egui::FontFamily::Monospace)));
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
        let color = if active { Color32::WHITE } else { Color32::from_rgb(120, 130, 140) };
        let bg = if active { Color32::from_rgb(25, 27, 30) } else { Color32::TRANSPARENT };
        
        egui::Frame::none()
            .fill(bg)
            .rounding(4.0)
            .inner_margin(egui::Margin::symmetric(32.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(label).color(color).font(FontId::new(13.0, egui::FontFamily::Proportional)));
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

                            // Branding - Vertically centered with buttons via item_spacing or manual offset
                            ui.label(RichText::new("SokuTeX")
                                .font(FontId::new(17.0, egui::FontFamily::Name("logo_font".into())))
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
