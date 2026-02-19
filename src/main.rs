use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    keyboard::{Key, NamedKey},
};

mod compiler;
mod renderer;
mod editor;
mod pdf_renderer;
mod palette;
mod vfs;
mod io;
mod compiler_daemon;
mod synctex;
mod config;
mod bib;
mod perf;

use pdf_renderer::PdfRenderer;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    // Start Compiler Daemon
    let (_compile_tx, compile_rx) = tokio::sync::mpsc::channel(10);
    let daemon = compiler_daemon::CompilerDaemon::new(compile_rx);
    tokio::spawn(daemon.run());

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("SokuTeX")
        .build(&event_loop)
        .unwrap();

    let mut state = renderer::State::new(&window).await;
    
    // Initialize PDF Renderer
    let pdf_renderer = PdfRenderer::new().expect("Failed to initialize PdfRenderer");
    let pdf_data = std::fs::read("test.pdf").expect("Failed to read test.pdf");

    fn render_pdf(state: &mut renderer::State, pdf_renderer: &PdfRenderer, pdf_data: &[u8]) {
        let timer = perf::PerfTimer::start("PDF Render");
        let width = state.size.width as u16;
        let height = state.size.height as u16;
        if let Ok(pixels) = pdf_renderer.render_page(pdf_data, 0, width, height) {
            state.update_texture(width as u32, height as u32, &pixels);
        }
        timer.stop();
    }

    render_pdf(&mut state, &pdf_renderer, &pdf_data);
    let mut palette = palette::CommandPalette::new();
    let mut vfs = vfs::Vfs::new();
    vfs.write_file("main.tex", b"\\documentclass{article}\n\\begin{document}\nHello SokuTeX!\n\\end{document}".to_vec());

    let mut ui_text = String::new();
    let mut editor = editor::Editor::new();
    if let Some(content) = vfs.read_file("main.tex") {
        ui_text = String::from_utf8_lossy(content).to_string();
        editor.buffer = ropey::Rope::from_str(&ui_text);
    }

    let mut modifiers = winit::event::Modifiers::default();

    event_loop.run(|event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::KeyboardInput {
                        event: KeyEvent {
                            state: ElementState::Pressed,
                            logical_key,
                            ..
                        },
                        ..
                    } => {
                        let consumed = state.handle_event(&window, &event).consumed;
                        if !consumed {
                            match logical_key {
                                Key::Named(NamedKey::Escape) => target.exit(),
                                Key::Character(c) if c == "p" && (modifiers.state().super_key() || modifiers.state().control_key()) => {
                                    palette.toggle();
                                }
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                        render_pdf(&mut state, &pdf_renderer, &pdf_data);
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {}
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = *new_modifiers;
                    }
                    WindowEvent::RedrawRequested => {
                        let pdf_texture_id = state.pdf_texture_id;
                        
                        let render_res = state.render(&window, |ctx| {
                            let mut visuals = egui::Visuals::dark();
                            visuals.window_fill = egui::Color32::from_rgb(15, 15, 15);
                            visuals.panel_fill = egui::Color32::from_rgb(15, 15, 15);
                            ctx.set_visuals(visuals);

                            egui::TopBottomPanel::top("header").show(ctx, |ui| {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.add_space(8.0);
                                    ui.label(egui::RichText::new("SokuTeX").strong().size(14.0));
                                    ui.separator();
                                    
                                    if ui.button("Compile").clicked() {
                                        println!("Manual compile triggered");
                                    }
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(8.0);
                                        if ui.button("Expand").clicked() {}
                                        if ui.button("Close").clicked() {
                                            target.exit();
                                        }
                                    });
                                });
                                ui.add_space(4.0);
                            });

                            egui::SidePanel::left("editor_panel")
                                .min_width(300.0)
                                .frame(egui::Frame::none().fill(egui::Color32::from_rgb(10, 10, 10)))
                                .show(ctx, |ui| {
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(8.0);
                                        ui.heading("Editor");
                                    });
                                    
                                    egui::ScrollArea::vertical().show(ui, |ui| {
                                        let resp = ui.add_sized(ui.available_size(), egui::TextEdit::multiline(&mut ui_text)
                                            .font(egui::TextStyle::Monospace)
                                            .frame(false)
                                            .code_editor()
                                            .lock_focus(true));
                                        
                                        if resp.changed() {
                                            editor.buffer = ropey::Rope::from_str(&ui_text);
                                            // Trigger auto-save
                                            let text = ui_text.clone();
                                            tokio::spawn(async move {
                                                let _ = io::IoHandler::auto_save(text, "autosave.tex").await;
                                            });
                                        }
                                    });
                                });

                            egui::CentralPanel::default()
                                .frame(egui::Frame::none().fill(egui::Color32::from_rgb(20, 20, 20)))
                                .show(ctx, |ui| {
                                if let Some(tex_id) = pdf_texture_id {
                                    ui.centered_and_justified(|ui| {
                                        ui.image(egui::load::SizedTexture::new(tex_id, ui.available_size()));
                                    });
                                } else {
                                    ui.centered_and_justified(|ui| {
                                        ui.label(egui::RichText::new("Waiting for PDF...").italics());
                                    });
                                }
                            });
                        });

                        match render_res {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {
                        let _ = state.handle_event(&window, &event);
                    }
                }
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
