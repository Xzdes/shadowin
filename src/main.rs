// src/main.rs

use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{Event, MouseButton, WindowEvent, ElementState};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;
use rusttype::Font;

mod ui;
use ui::Ui;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

pub struct AppState {
    mouse_pos: PhysicalPosition<f64>,
    mouse_pressed: bool,
    message: String,
    click_count: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let font_data = include_bytes!("../assets/font.ttf");
    let font = Font::try_from_bytes(font_data).ok_or("Failed to load font")?;

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = Arc::new({
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let attributes = Window::default_attributes().with_title("Shadowin").with_inner_size(size);
        event_loop.create_window(attributes)?
    });

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        pollster::block_on(Pixels::new_async(WIDTH, HEIGHT, surface_texture))?
    };

    let mut ui = Ui::new(&font);
    
    let mut app_state = AppState {
        mouse_pos: (0.0, 0.0).into(),
        mouse_pressed: false,
        // ---- ИСПРАВЛЕНИЕ 3 ----
        message: "Hello, Shadowin!".to_string(),
        click_count: 0,
    };

    let window_clone = Arc::clone(&window);

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::CursorMoved { position, .. } => { app_state.mouse_pos = position; }
                WindowEvent::MouseInput { state, button, .. } => {
                    if button == MouseButton::Left { app_state.mouse_pressed = state == ElementState::Pressed; }
                }
                WindowEvent::KeyboardInput { event: key_event, .. } => {
                    if key_event.logical_key == Key::Named(NamedKey::Escape) { elwt.exit(); }
                }
                WindowEvent::Resized(size) => {
                    if let Err(_err) = pixels.resize_surface(size.width, size.height) { elwt.exit(); }
                }
                WindowEvent::RedrawRequested => {
                    let frame = pixels.frame_mut();
                    for pixel in frame.chunks_exact_mut(4) {
                        pixel.copy_from_slice(&[20, 20, 30, 255]);
                    }
                    ui.draw(&app_state, frame, WIDTH);
                    
                    if let Err(_err) = pixels.render() { elwt.exit(); }
                }
                _ => (),
            },
            Event::AboutToWait => {
                let mouse_in_buffer = pixels.window_pos_to_pixel(app_state.mouse_pos.into()).ok().map(|(x, y)| (x as i32, y as i32)).unwrap_or((-1, -1)); 
                
                if let Some(clicked_id) = ui.update(mouse_in_buffer, app_state.mouse_pressed) {
                    match clicked_id {
                        0 => {
                            app_state.click_count += 1;
                            app_state.message = format!("Button clicked {} times!", app_state.click_count);
                        },
                        1 => {
                            app_state.click_count = 0;
                            // ---- ИСПРАВЛЕНИЕ 4 ----
                            app_state.message = "State has been reset.".to_string();
                        },
                        _ => {}
                    }
                }
                window_clone.request_redraw();
            }
            _ => (),
        }
    })?;

    Ok(())
}