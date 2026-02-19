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
mod syntax;
mod autocomplete;
mod watcher;
mod latexmk;
mod dependencies;


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

    // Start File Watcher
    let (file_tx, mut file_rx) = tokio::sync::mpsc::channel(10);
    let mut watcher = watcher::FileWatcher::new(file_tx).expect("Failed to setup file watcher");
    watcher.watch(".").expect("Failed to start watching directory");
    
    // Initialize PDF Renderer with the workspace preview
    let pdf_renderer = std::sync::Arc::new(PdfRenderer::new().expect("Failed to initialize PdfRenderer"));
    let mut current_pdf_data = std::fs::read("workspace_preview.pdf").expect("Failed to read workspace_preview.pdf");
    let mut current_pdf_revision = 0u64;

    // PDF Render Channel
    let (pdf_tx, mut pdf_rx) = tokio::sync::mpsc::channel::<(u32, u32, Vec<u8>)>(2);

    let request_render = |pdf_renderer: std::sync::Arc<PdfRenderer>, pdf_data: Vec<u8>, revision: u64, width: u16, height: u16, tx: tokio::sync::mpsc::Sender<(u32, u32, Vec<u8>)>| {
        tokio::task::spawn_blocking(move || {
            let timer = perf::PerfTimer::start("PDF Render (Async)");
            if let Ok(pixels) = pdf_renderer.render_page(&pdf_data, revision, 0, width, height) {
                let _ = tx.blocking_send((width as u32, height as u32, pixels));
            }
            timer.stop();
        });
    };

    request_render(pdf_renderer.clone(), current_pdf_data.clone(), current_pdf_revision, state.size.width as u16, state.size.height as u16, pdf_tx.clone());
    let mut palette = palette::CommandPalette::new();
    let vfs = vfs::Vfs::new();
    vfs.write_file("main.tex", b"\\documentclass{article}\n\\input{sections/intro}\n\\include{sections/chapter1}\n\\begin{document}\nHello SokuTeX!\n\\end{document}".to_vec());
    vfs.write_file("sections/intro.tex", b"\\section{Introduction}\nThis is a multi-file project.".to_vec());
    vfs.write_file("sections/chapter1.tex", b"\\section{Chapter 1}\n\\input{sections/details}\nMore content here.".to_vec());
    vfs.write_file("sections/details.tex", b"Detailed explanation...".to_vec());

    let mut gui = ui::Gui::new();
    ui::Gui::setup_visuals(&state.egui_ctx);

    let mut editor = editor::Editor::new();
    if let Some(content) = vfs.read_file("main.tex") {
        gui.ui_text = String::from_utf8_lossy(&content).to_string();
        editor.buffer = ropey::Rope::from_str(&gui.ui_text);
        gui.dependency_tree = Some(crate::dependencies::DependencyScanner::scan("main.tex", &vfs));
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
                        request_render(pdf_renderer.clone(), current_pdf_data.clone(), current_pdf_revision, state.size.width as u16, state.size.height as u16, pdf_tx.clone());
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {}
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = *new_modifiers;
                    }
                    WindowEvent::RedrawRequested => {
                        // Check for PDF render results
                        if let Ok((w, h, pixels)) = pdf_rx.try_recv() {
                            state.update_texture(w, h, &pixels);
                        }

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
                            let backend = gui.compile_backend;
                            tokio::spawn(async move {
                                let (otx, orx) = tokio::sync::oneshot::channel();
                                use crate::compiler_daemon::CompileRequest;
                                if tx.send(CompileRequest::Compile { 
                                    latex: text, 
                                    backend, 
                                    response: otx 
                                }).await.is_ok() {
                                    if let Ok(pdf) = orx.await {
                                        let _ = r_tx.send(pdf).await;
                                    }
                                }
                            });
                        }

                        // Check for compilation results
                        if let Ok(new_pdf) = result_rx.try_recv() {
                            current_pdf_revision += 1;
                            current_pdf_data = new_pdf;
                            request_render(pdf_renderer.clone(), current_pdf_data.clone(), current_pdf_revision, state.size.width as u16, state.size.height as u16, pdf_tx.clone());
                            
                            // Lazy pre-render adjacent pages
                            let renderer = pdf_renderer.clone();
                            let pdf = current_pdf_data.clone();
                            let revision = current_pdf_revision;
                            let width = state.size.width as u16;
                            let height = state.size.height as u16;
                            tokio::spawn(async move {
                                // Render next few pages into cache
                                for i in 1..5 {
                                    let _ = renderer.render_page(&pdf, revision, i, width, height);
                                }
                            });
                        }

                        // Check for external file changes
                        if let Ok(crate::watcher::FileEvent::Modified(path)) = file_rx.try_recv() {
                            if path.contains("main.tex") {
                                if let Some(content) = vfs.read_file("main.tex") {
                                    gui.ui_text = String::from_utf8_lossy(&content).to_string();
                                }
                            }
                        }

                        // Sync back to editor and autosave if changed
                        let current_text = gui.ui_text.clone();
                        if editor.get_text() != current_text {
                            editor.buffer = ropey::Rope::from_str(&current_text);
                            
                            // Update VFS and scan for dependencies
                            vfs.write_file("main.tex", current_text.as_bytes().to_vec());
                            gui.dependency_tree = Some(crate::dependencies::DependencyScanner::scan("main.tex", &vfs));

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
