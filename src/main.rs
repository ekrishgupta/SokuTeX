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

use pdf_renderer::PdfRenderer;

#[tokio::main]
async fn main() {
    env_logger::init();
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
        let width = state.size.width as u16;
        let height = state.size.height as u16;
        if let Ok(pixels) = pdf_renderer.render_page(pdf_data, 0, width, height) {
            state.update_texture(width as u32, height as u32, &pixels);
        }
    }

    render_pdf(&mut state, &pdf_renderer, &pdf_data);

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
                            logical_key: Key::Named(NamedKey::Escape),
                            ..
                        },
                        ..
                    } => target.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                        // TODO: Re-render PDF on resize?
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {}
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
