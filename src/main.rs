use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};
use winit::platform::macos::WindowBuilderExtMacOS;

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
mod ui;

use pdf_renderer::PdfRenderer;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    // Start Compiler Daemon
    let (compile_tx, compile_rx) = tokio::sync::mpsc::channel(10);
    let (result_tx, mut result_rx) = tokio::sync::mpsc::channel(1);
    let daemon = compiler_daemon::CompilerDaemon::new(compile_rx);
    tokio::spawn(daemon.run());

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("SokuTeX")
        .with_transparent(true)
        .with_fullsize_content_view(true)
        .with_titlebar_transparent(true)
        .with_title_hidden(true)
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

    let mut gui = ui::Gui::new();
    ui::Gui::setup_visuals(&state.egui_ctx);

    let mut editor = editor::Editor::new();
    if let Some(content) = vfs.read_file("main.tex") {
        gui.ui_text = String::from_utf8_lossy(content).to_string();
        editor.buffer = ropey::Rope::from_str(&gui.ui_text);
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
                            gui.draw(ctx, pdf_texture_id);
                        });

                        // Check for Auto-Compile requests from GUI
                        if gui.compile_requested {
                            gui.compile_requested = false;
                            gui.last_compile_text = gui.ui_text.clone();
                            let tx = compile_tx.clone();
                            let text = gui.ui_text.clone();
                            let r_tx = result_tx.clone();
                            tokio::spawn(async move {
                                let (otx, orx) = tokio::sync::oneshot::channel();
                                use crate::compiler_daemon::CompileRequest;
                                if tx.send(CompileRequest::Compile { latex: text, response: otx }).await.is_ok() {
                                    if let Ok(pdf) = orx.await {
                                        let _ = r_tx.send(pdf).await;
                                    }
                                }
                            });
                        }

                        // Check for compilation results
                        if let Ok(new_pdf) = result_rx.try_recv() {
                            render_pdf(&mut state, &pdf_renderer, &new_pdf);
                        }

                        // Sync back to editor and autosave if changed
                        let current_text = gui.ui_text.clone();
                        if editor.get_text() != current_text {
                            editor.buffer = ropey::Rope::from_str(&current_text);
                            tokio::spawn(async move {
                                let _ = io::IoHandler::auto_save(current_text, "autosave.tex").await;
                            });
                        }

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
