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
        
        // Custom SokuTeX Midnight Theme
        visuals.panel_fill = Color32::from_rgb(10, 10, 10);
        visuals.window_fill = Color32::from_rgb(15, 15, 15);
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(10, 10, 10);
        visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(150, 150, 150);
        
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(25, 25, 25);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(35, 35, 35);
        visuals.widgets.active.bg_fill = Color32::from_rgb(45, 45, 45);
        
        visuals.selection.bg_fill = Color32::from_rgb(0, 102, 255); // Electric Blue
        
        ctx.set_visuals(visuals);
        
        let fonts = egui::FontDefinitions::default();
        // We can add custom fonts here if we include them in the binary
        ctx.set_fonts(fonts);
    }

    pub fn draw(&mut self, ctx: &egui::Context, pdf_tex_id: Option<egui::TextureId>) {
        egui::TopBottomPanel::top("header")
            .frame(egui::Frame::none()
                .fill(Color32::from_rgb(15, 15, 15))
                .inner_margin(egui::Margin::symmetric(16.0, 8.0))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(30, 30, 30))))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("SOKUTEX").strong().size(16.0).extra_letter_spacing(2.0).color(Color32::WHITE));
                    ui.add_space(20.0);
                    
                    ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
                    if ui.button(RichText::new("Compile").color(Color32::from_rgb(0, 162, 255))).clicked() {
                        self.compile_status = "Compiling...".to_string();
                    }
                    let _ = ui.button("Sync");
                    let _ = ui.button("Project");
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        if ui.button(RichText::new("Expand").size(12.0)).clicked() {}
                        if ui.button(RichText::new("Settings").size(12.0)).clicked() {}
                    });
                });
            });

        egui::TopBottomPanel::bottom("status_bar")
            .frame(egui::Frame::none()
                .fill(Color32::from_rgb(10, 10, 10))
                .inner_margin(egui::Margin::symmetric(12.0, 4.0))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(30, 30, 30))))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Status: {}", self.compile_status)).size(10.0).color(Color32::from_rgb(100, 100, 100)));
                    ui.add_space(20.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("UTF-8").size(10.0).color(Color32::from_rgb(100, 100, 100)));
                        ui.separator();
                        ui.label(RichText::new("LaTeX").size(10.0).color(Color32::from_rgb(100, 100, 100)));
                    });
                });
            });

        egui::SidePanel::left("editor_container")
            .min_width(400.0)
            .default_width(500.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(5, 5, 5)))
            .show(ctx, |ui| {
                ui.add_space(10.0);
                egui::ScrollArea::vertical()
                    .id_source("editor_scroll")
                    .show(ui, |ui| {
                        let resp = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(&mut self.ui_text)
                                .font(FontId::monospace(14.0))
                                .frame(false)
                                .margin(egui::Margin::same(20.0))
                                .code_editor()
                                .lock_focus(true)
                                .desired_width(f32::INFINITY)
                        );
                        
                        if resp.changed() {
                            // Parent will sync this
                        }
                    });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(20, 20, 20)))
            .show(ctx, |ui| {
                if let Some(tex_id) = pdf_tex_id {
                    let image_size = ui.available_size();
                    ui.centered_and_justified(|ui| {
                        ui.image(egui::load::SizedTexture::new(tex_id, image_size));
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(RichText::new("PREVIEW AREA").italics().color(Color32::from_rgb(60, 60, 60)));
                    });
                }
            });
    }
}
