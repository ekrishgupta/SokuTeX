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

fn render_pdf(
    pdf_renderer: std::sync::Arc<PdfRenderer>,
    pdf_data: std::sync::Arc<Vec<u8>>,
    revision: u64,
    page: i32,
    width: u16,
    height: u16,
    tx: Option<tokio::sync::mpsc::Sender<(u32, u32, std::sync::Arc<Vec<u8>>, f32, f32)>>,
) {
    tokio::task::spawn_blocking(move || {
        let timer = perf::PerfTimer::start("PDF Render (Async)");
        if let Ok((pixels, pw, ph)) = pdf_renderer.render_page(&pdf_data, revision, page, width, height) {
            if let Some(tx) = tx {
                let _ = tx.blocking_send((width as u32, height as u32, pixels, pw, ph));
            }
        }
        timer.stop();
    });
}




#[tokio::main]
async fn main() {
    env_logger::init();
    
    // Initialize VFS
    let vfs = std::sync::Arc::new(vfs::Vfs::new());
    vfs.write_file("main.tex", b"\\documentclass{article}\n\\input{sections/intro}\n\\include{sections/chapter1}\n\\begin{document}\nHello SokuTeX!\n\\end{document}".to_vec());
    vfs.write_file("sections/intro.tex", b"\\section{Introduction}\nThis is a multi-file project.".to_vec());
    vfs.write_file("sections/chapter1.tex", b"\\section{Chapter 1}\n\\input{sections/details}\nMore content here.".to_vec());
    vfs.write_file("sections/details.tex", b"Detailed explanation...".to_vec());
    vfs.write_file("references.bib", b"@article{einstein1905,\n  author = {Einstein, Albert},\n  title = {On the Electrodynamics of Moving Bodies},\n  journal = {Annalen der Physik},\n  year = {1905}\n}\n@book{knuth1984,\n  author = {Knuth, Donald E.},\n  title = {The TeXbook},\n  year = {1984},\n  publisher = {Addison-Wesley}\n}".to_vec());

    // Start Compiler Daemon
    let (compile_tx, compile_rx) = tokio::sync::mpsc::channel(10);
    let (result_tx, mut result_rx) = tokio::sync::mpsc::channel::<(compiler_daemon::CompileResult, crate::dependencies::DependencyNode)>(1);
    let daemon = compiler_daemon::CompilerDaemon::new(compile_rx, vfs.clone());
    tokio::spawn(daemon.run());

    // Compile Debouncer
    let (debounce_tx, mut debounce_rx) = tokio::sync::mpsc::channel::<(String, crate::config::CompileBackend, bool, bool, Option<String>)>(10);
    let compile_tx_clone = compile_tx.clone();
    let result_tx_clone = result_tx.clone();
    let vfs_clone = vfs.clone();
    tokio::spawn(async move {
        let mut last_req = None;
        let mut last_compile_time: Option<tokio::time::Instant> = None;
        let sleep_duration = std::time::Duration::from_millis(150);
        let sleep = tokio::time::sleep(sleep_duration);
        tokio::pin!(sleep);
        
        loop {
            tokio::select! {
                req = debounce_rx.recv() => {
                    match req {
                        Some(r) => {
                            let now = tokio::time::Instant::now();
                            if last_compile_time.map_or(true, |t| now.duration_since(t) >= sleep_duration) {
                                // Leading edge: trigger immediately
                                last_compile_time = Some(now);
                                
                                let (text, backend, draft, focus_mode, active_file) = r.clone();
                                let vfs_c = vfs_clone.clone();
                                let ctx_c = compile_tx_clone.clone();
                                let rtx_c = result_tx_clone.clone();
                                
                                tokio::spawn(async move {
                                    vfs_c.write_file("main.tex", text.as_bytes().to_vec());
                                    let dep_tree = crate::dependencies::DependencyScanner::scan("main.tex", &vfs_c);
                                    let (otx, orx) = tokio::sync::oneshot::channel();
                                    use crate::compiler_daemon::CompileRequest;
                                    if ctx_c.send(CompileRequest::Compile { 
                                        latex: text, 
                                        backend, 
                                        draft,
                                        focus_mode,
                                        active_file,
                                        response: otx 
                                    }).await.is_ok() {
                                        if let Ok(res) = orx.await {
                                            let _ = rtx_c.send((res, dep_tree)).await;
                                        }
                                    }
                                });

                                // Ensure we don't process it a second time as a trailing edge
                                last_req = None; 
                                sleep.as_mut().reset(now + sleep_duration);
                            } else {
                                // Debounce subsequent keystrokes
                                last_req = Some(r);
                                sleep.as_mut().reset(now + sleep_duration);
                            }
                        }
                        None => break,
                    }
                }
                _ = &mut sleep, if last_req.is_some() => {
                    if let Some((text, backend, draft, focus_mode, active_file)) = last_req.take() {
                        last_compile_time = Some(tokio::time::Instant::now());
                        
                        let vfs_c = vfs_clone.clone();
                        let ctx_c = compile_tx_clone.clone();
                        let rtx_c = result_tx_clone.clone();
                        
                        tokio::spawn(async move {
                            vfs_c.write_file("main.tex", text.as_bytes().to_vec());
                            let dep_tree = crate::dependencies::DependencyScanner::scan("main.tex", &vfs_c);
                            let (otx, orx) = tokio::sync::oneshot::channel();
                            use crate::compiler_daemon::CompileRequest;
                            if ctx_c.send(CompileRequest::Compile { 
                                latex: text, 
                                backend, 
                                draft,
                                focus_mode,
                                active_file,
                                response: otx 
                            }).await.is_ok() {
                                if let Ok(res) = orx.await {
                                    let _ = rtx_c.send((res, dep_tree)).await;
                                }
                            }
                        });
                    }
                }
            }
        }
    });

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
    
    // Fallback if workspace_preview.pdf doesn't exist
    let mut current_pdf_data = if let Ok(data) = std::fs::read("workspace_preview.pdf") {
        std::sync::Arc::new(data)
    } else {
        // Create a dummy PDF if file missing to prevent crash
        std::sync::Arc::new(vec![])
    };
    let mut current_pdf_revision = 0u64;

    // PDF Render Channel
    let (pdf_tx, mut pdf_rx) = tokio::sync::mpsc::channel::<(u32, u32, std::sync::Arc<Vec<u8>>, f32, f32)>(2);


    // Dependency render channel
    let (dep_tx, mut dep_rx) = tokio::sync::mpsc::channel::<crate::dependencies::DependencyNode>(10);

    render_pdf(pdf_renderer.clone(), current_pdf_data.clone(), current_pdf_revision, 0, state.size.width as u16, state.size.height as u16, Some(pdf_tx.clone()));
    let mut palette = palette::CommandPalette::new();

    let mut gui = ui::Gui::new();
    gui.vfs = Some(vfs.clone());
    ui::Gui::setup_visuals(&state.egui_ctx);

    // Initial scan for .bib files in VFS
    let mut bib_contents = Vec::new();
    for entry in vfs.get_all_files().iter() {
        if entry.key().ends_with(".bib") {
            if let Ok(content) = String::from_utf8(entry.value().clone()) {
                bib_contents.push(content);
            }
        }
    }
    gui.refresh_bibliography(bib_contents);

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
                        render_pdf(pdf_renderer.clone(), current_pdf_data.clone(), current_pdf_revision, 0, state.size.width as u16, state.size.height as u16, Some(pdf_tx.clone()));
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {}
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = *new_modifiers;
                    }
                    WindowEvent::RedrawRequested => {
                        // Check for PDF render results
                        if let Ok((w, h, pixels, pw, ph)) = pdf_rx.try_recv() {
                            state.update_texture(w, h, &pixels);
                            gui.pdf_page_size = egui::vec2(pw, ph);
                        }



                        let pdf_texture_id = state.pdf_texture_id;

                        // Handle Backward Sync: Update internal editor state before drawing
                        if let Some(line) = gui.sync_to_editor_request {
                            editor.move_to_line(line);
                        }

                        
                        // Update GPU Uniforms for PDF Transform
                        let zoom = gui.pdf_zoom;
                        let pan = gui.pdf_pan;
                        
                        // Simple 2D transform matrix: [zoom, 0, 0, pan.x], [0, zoom, 0, pan.y], ...
                        let transform = [
                            [zoom, 0.0, 0.0, 0.0],
                            [0.0, zoom, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [pan.x, pan.y, 0.0, 1.0],
                        ];
                        state.update_uniforms(transform);

                        let render_res = state.render(&window, |ctx| {
                            gui.draw(ctx, pdf_texture_id);
                        });

                        // Check for Auto-Compile requests from GUI
                        if gui.compile_requested {
                            gui.compile_requested = false;
                            gui.last_compile_text = gui.ui_text.clone();
                            
                            let text = gui.ui_text.clone();
                            let backend = gui.compile_backend;
                            let draft = gui.draft_mode;
                            let focus_mode = gui.focus_mode;
                            let active_file = Some(gui.active_file_path.clone());

                            let tx = debounce_tx.clone();
                            tokio::spawn(async move {
                                let _ = tx.send((text, backend, draft, focus_mode, active_file)).await;
                            });
                        }

                        // Check for compilation results and updated dependency tree
                        if let Ok(dep_tree) = dep_rx.try_recv() {
                            gui.dependency_tree = Some(dep_tree);
                        }

                        if let Ok((res, dep_tree)) = result_rx.try_recv() {
                            gui.dependency_tree = Some(dep_tree);
                            current_pdf_revision = res.revision;
                            current_pdf_data = std::sync::Arc::new(res.pdf);
                            render_pdf(pdf_renderer.clone(), current_pdf_data.clone(), current_pdf_revision, 0, state.size.width as u16, state.size.height as u16, Some(pdf_tx.clone()));
                            
                            // Load SyncTeX if available
                            let mut stx = crate::synctex::SyncTex::new();
                            let mut loaded = false;
                            
                            if let Some(ref data) = res.synctex_data {
                                use std::io::Cursor;
                                use flate2::read::GzDecoder;
                                
                                // Try Gz first if it looks like one, or just try both
                                let cursor = Cursor::new(data);
                                let mut decoder = GzDecoder::new(cursor);
                                let mut decoded_data = Vec::new();
                                use std::io::Read;
                                if decoder.read_to_end(&mut decoded_data).is_ok() {
                                    if stx.load_from_reader(Cursor::new(decoded_data)).is_ok() {
                                        loaded = true;
                                    }
                                } else {
                                    if stx.load_from_reader(Cursor::new(data)).is_ok() {
                                        loaded = true;
                                    }
                                }
                            }
                            
                            if !loaded {
                                // Fallback to disk if not in result
                                if stx.load("main.synctex.gz").is_ok() || stx.load("main.synctex").is_ok() {
                                    loaded = true;
                                }
                            }
                            
                            if loaded {
                                gui.synctex = Some(stx);
                            }

                            // Lazy pre-render adjacent pages
                            for i in 1..5 {
                                render_pdf(pdf_renderer.clone(), current_pdf_data.clone(), current_pdf_revision, i, state.size.width as u16, state.size.height as u16, None);
                            }
                        }


                        // Check for external file changes
                        if let Ok(crate::watcher::FileEvent::Modified(path)) = file_rx.try_recv() {
                            if path.contains("main.tex") {
                                if let Some(content) = vfs.read_file("main.tex") {
                                    gui.ui_text = String::from_utf8_lossy(&content).to_string();
                                }
                            } else if path.ends_with(".bib") {
                                if let Some(_content_bytes) = vfs.read_file(&path) {
                                    // Refresh bibliography from all .bib files in VFS
                                    let mut bib_contents = Vec::new();
                                    for entry in vfs.get_all_files().iter() {
                                        if entry.key().ends_with(".bib") {
                                            if let Ok(content) = String::from_utf8(entry.value().clone()) {
                                                bib_contents.push(content);
                                            }
                                        }
                                    }
                                    gui.refresh_bibliography(bib_contents);
                                }
                            }
                        }


                        // Handle file change request from GUI
                        if let Some(new_file) = gui.file_change_request.take() {
                            // Save current text to VFS first
                            vfs.write_file(&gui.active_file_path, gui.ui_text.as_bytes().to_vec());
                            
                            if let Some(content) = vfs.read_file(&new_file) {
                                gui.active_file_path = new_file;
                                gui.ui_text = String::from_utf8_lossy(&content).to_string();
                                editor.buffer = ropey::Rope::from_str(&gui.ui_text);
                                gui.last_compile_text = gui.ui_text.clone();
                                gui.prev_ui_text = gui.ui_text.clone();
                                
                                let dtx = dep_tx.clone();
                                let rtx = compile_tx.clone();
                                tokio::spawn(async move {
                                    let (otx, orx) = tokio::sync::oneshot::channel();
                                    let _ = rtx.send(crate::compiler_daemon::CompileRequest::ScanDependencies {
                                        main_file: "main.tex".to_string(),
                                        response: otx,
                                    }).await;
                                    if let Ok(tree) = orx.await {
                                        let _ = dtx.send(tree).await;
                                    }
                                });
                            }
                        }

                        // Sync back to editor and update VFS if changed
                        let current_text = gui.ui_text.clone();
                        if editor.get_text() != current_text {
                            editor.buffer = ropey::Rope::from_str(&current_text);
                            
                            // Update VFS and request async dependency scan
                            vfs.write_file(&gui.active_file_path, current_text.as_bytes().to_vec());
                            
                            let dtx = dep_tx.clone();
                            let rtx = compile_tx.clone();
                            tokio::spawn(async move {
                                let (otx, orx) = tokio::sync::oneshot::channel();
                                let _ = rtx.send(crate::compiler_daemon::CompileRequest::ScanDependencies {
                                    main_file: "main.tex".to_string(),
                                    response: otx,
                                }).await;
                                if let Ok(tree) = orx.await {
                                    let _ = dtx.send(tree).await;
                                }
                            });

                            // Refresh bibliography from all .bib files in VFS
                            let mut bib_contents = Vec::new();
                            for entry in vfs.get_all_files().iter() {
                                if entry.key().ends_with(".bib") {
                                    if let Ok(content) = String::from_utf8(entry.value().clone()) {
                                        bib_contents.push(content);
                                    }
                                }
                            }
                            gui.refresh_bibliography(bib_contents);

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
