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
    }

    pub fn draw(&mut self, ctx: &egui::Context, pdf_tex_id: Option<egui::TextureId>) {
        egui::SidePanel::left("editor_panel")
            .min_width(350.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(10, 12, 14)))
            .show(ctx, |ui| {
                ui.add_space(24.0);
                ui.horizontal(|ui| {
                    ui.add_space(32.0);
                    ui.spacing_mut().item_spacing.x = 20.0;
                    ui.label(RichText::new("STX").strong().color(Color32::WHITE).extra_letter_spacing(4.0));
                    
                    if ui.button(RichText::new("COMP").size(9.0)).clicked() {
                        self.compile_status = "BUSY".to_string();
                    }
                    let _ = ui.button(RichText::new("SYNC").size(9.0));
                });
                
                ui.add_space(16.0);
                
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
            .frame(egui::Frame::none().fill(Color32::from_rgb(18, 20, 23)))
            .show(ctx, |ui| {
                if let Some(tex_id) = pdf_tex_id {
                    let image_size = ui.available_size() * 0.95; // Add some breathing room
                    ui.centered_and_justified(|ui| {
                        ui.image(egui::load::SizedTexture::new(tex_id, image_size));
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("...").color(Color32::from_rgb(40, 45, 50)));
                    });
                }
            });
    }
}
