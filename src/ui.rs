use egui::{Color32, FontId, RichText, Visuals};

pub struct Gui {
    pub ui_text: String,
    pub compile_status: String,
}

impl Gui {
    pub fn new() -> Self {
        Self {
            ui_text: String::new(),
            compile_status: "Idle".to_string(),
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
        egui::SidePanel::left("editor_panel")
            .min_width(350.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(10, 12, 14)))
            .show(ctx, |ui| {
                // Top Control Bar Area - Centered with traffic lights
                egui::Frame::none()
                    .fill(Color32::from_rgb(10, 12, 14))
                    .inner_margin(egui::Margin { left: 16.0, right: 16.0, top: 4.0, bottom: 4.0 })
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.add_space(64.0); // Clear traffic lights
                            ui.spacing_mut().item_spacing.x = 16.0;
                            
                            // Visuals for buttons in the header
                            ui.visuals_mut().widgets.inactive.bg_fill = Color32::from_rgb(30, 32, 35);
                            ui.visuals_mut().widgets.hovered.bg_fill = Color32::from_rgb(45, 48, 52);

                            // SokuTeX Brand
                            ui.label(RichText::new("SokuTeX")
                                .font(FontId::new(17.0, egui::FontFamily::Name("logo_font".into())))
                                .color(Color32::WHITE)
                                .extra_letter_spacing(0.1));
                            
                            ui.add_space(4.0);
                            
                            // Buttons row
                            ui.spacing_mut().button_padding = egui::vec2(10.0, 3.0);
                            if ui.button(RichText::new("COMP").size(9.0).strong()).clicked() {
                                self.compile_status = "BUSY".to_string();
                            }
                            let _ = ui.button(RichText::new("SYNC").size(9.0).strong());
                            let _ = ui.button(RichText::new("PROJ").size(9.0).strong());
                        });
                    });
                
                ui.add_space(8.0);
                
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
