use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    keyboard::{Key, NamedKey},
    dpi::PhysicalSize,
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
    let (compile_tx, compile_rx) = tokio::sync::mpsc::channel(10);
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
    let mut editor = editor::Editor::new();

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
                    } => match logical_key {
                        Key::Named(NamedKey::Escape) => target.exit(),
                        Key::Character(c) if c == "p" && (modifiers.state().super_key() || modifiers.state().control_key()) => {
                            palette.toggle();
                            println!("Palette visible: {}", palette.visible);
                        }
                        Key::Character(c) => {
                            // Basic text input for now
                            if c.chars().count() == 1 {
                                editor.insert_char(c.chars().next().unwrap());
                                let text = editor.get_text();
                                tokio::spawn(async move {
                                    let _ = io::IoHandler::auto_save(text, "autosave.tex").await;
                                });
                            }
                        }
                        Key::Named(NamedKey::Backspace) => {
                            editor.delete_back();
                            let text = editor.get_text();
                            tokio::spawn(async move {
                                let _ = io::IoHandler::auto_save(text, "autosave.tex").await;
                            });
                        }
                        Key::Named(NamedKey::ArrowLeft) => {
                            editor.move_left();
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            editor.move_right();
                        }
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                        render_pdf(&mut state, &pdf_renderer, &pdf_data);
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {}
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = *new_modifiers;
                    }
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        button: MouseButton::Left,
                        ..
                    } => {
                        // In a real app we'd get the actual cursor position
                        let offset = synctex::SyncTex::pdf_to_editor(0.0, 0.0, 0);
                        editor.cursor = offset;
                        println!("SyncTeX jump to offset: {}", offset);
                    }
                    WindowEvent::RedrawRequested => {
                        match state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                }
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}
