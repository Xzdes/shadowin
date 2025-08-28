// src/main.rs

use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{Event, MouseButton, WindowEvent, ElementState, Ime};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;
use rusttype::Font;

// Подключаем наши модули
mod ui;
mod ai_renderer;
use ui::Ui;
use ai_renderer::AiRenderer;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

// `AppState` - это хранилище данных нашего приложения.
// UI будет просто "читать" эти данные и отображать их.
pub struct AppState {
    mouse_pos: PhysicalPosition<f64>,
    mouse_pressed: bool,
    message: String,
    bg_color: [u8; 4],
    click_count: u32,
}

// `main` - обычная, синхронная функция.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. Инициализация ---
    let font_data = include_bytes!("../assets/font.ttf");
    let font = Font::try_from_bytes(font_data).ok_or("Failed to load font")?;
    
    // Создаем наш AI-рендерер и "оборачиваем" его в Arc, чтобы безопасно передать в UI
    let ai_renderer = Arc::new(AiRenderer::new());
    
    let event_loop = EventLoop::new()?;
    // `Poll` заставляет цикл постоянно работать, что хорошо для UI и игр
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = Arc::new({
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let attributes = Window::default_attributes().with_title("Shadowin AI").with_inner_size(size);
        event_loop.create_window(attributes)?
    });

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        pollster::block_on(Pixels::new_async(WIDTH, HEIGHT, surface_texture))?
    };

    // Создаем наш UI-менеджер
    let mut ui = Ui::new(&font, Arc::clone(&ai_renderer));
    
    // Создаем начальное состояние приложения
    let mut app_state = AppState {
        mouse_pos: (0.0, 0.0).into(),
        mouse_pressed: false,
        message: "AI Renderer is ready.".to_string(),
        bg_color: [20, 20, 30, 255],
        click_count: 0,
    };

    let window_clone = Arc::clone(&window);

    // --- 2. Главный Цикл ---
    event_loop.run(move |event, elwt| {
        let mut mouse_clicked_this_frame = false;

        // Обрабатываем события окна
        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::CursorMoved { position, .. } => { app_state.mouse_pos = position; },
                WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                    let is_pressed = state == ElementState::Pressed;
                    // Если кнопка была нажата, а теперь отпущена - это клик
                    if !is_pressed && app_state.mouse_pressed {
                        mouse_clicked_this_frame = true;
                    }
                    app_state.mouse_pressed = is_pressed;
                },
                WindowEvent::KeyboardInput { event: key_event, .. } => {
                    if key_event.logical_key == Key::Named(NamedKey::Escape) { elwt.exit(); }
                    ui.handle_key_event(&key_event);
                },
                WindowEvent::Ime(Ime::Commit(text)) => {
                    ui.text_input.key_press(&text);
                },
                WindowEvent::Resized(size) => {
                    if let Err(_err) = pixels.resize_surface(size.width, size.height) { elwt.exit(); }
                },
                WindowEvent::RedrawRequested => {
                    let frame = pixels.frame_mut();
                    for pixel in frame.chunks_exact_mut(4) {
                        pixel.copy_from_slice(&app_state.bg_color);
                    }
                    ui.draw(&app_state, frame, WIDTH);
                    if let Err(_err) = pixels.render() { elwt.exit(); }
                },
                _ => (),
            }
        }
        
        // --- 3. Обновление Логики и Отрисовка ---
        // `update` теперь вызывается в каждом кадре.
        // Он проверит, нужно ли генерировать новые картинки (и "замрет", если нужно),
        // а также вернет ID кликнутой кнопки.
        let mouse_in_buffer = pixels.window_pos_to_pixel(app_state.mouse_pos.into()).ok().map(|(x, y)| (x as i32, y as i32)).unwrap_or((-1, -1));
        
        if let Some(clicked_id) = ui.update(mouse_in_buffer, app_state.mouse_pressed, mouse_clicked_this_frame) {
            app_state.click_count += 1;
            match clicked_id {
                0 => { // Submit
                    if ui.text_input.text == "shadowin" {
                        app_state.message = "Welcome, master.".to_string();
                        app_state.bg_color = [40, 20, 20, 255];
                    } else {
                        app_state.message = format!("Submitted: {}", ui.text_input.text);
                    }
                },
                1 => { // Clear
                    ui.text_input.text.clear();
                    app_state.message = "Cleared.".to_string();
                    app_state.bg_color = [20, 20, 30, 255];
                    app_state.click_count = 0;
                },
                _ => {}
            }
        }
        
        // Запрашиваем перерисовку окна в конце каждого кадра
        window_clone.request_redraw();
    })?;

    Ok(())
}