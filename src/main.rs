// src/main.rs

use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
// ИСПРАВЛЕНИЕ (warning): Убран неиспользуемый импорт `PhysicalPosition`.
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;

// Подключаем наши модули
mod ai_renderer;
mod ui;
mod loading; 

use ai_renderer::AiRenderer;
use loading::LoadingState;
use ui::AppUi;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

/// Глобальное состояние, которое хранит данные, нужные для логики приложения.
pub struct AppState {
    pub mouse_pos: (i32, i32),
    pub mouse_pressed: bool,
    pub message: String,
    pub bg_color: [u8; 4],
    pub click_count: u32,
    pub text_input_content: String,
}

/// Перечисление, управляющее тем, какой "экран" сейчас активен.
enum AppMode {
    Loading(LoadingState),
    Running(AppUi),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- 1. Инициализация ---
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    // Примечание: `winit` рекомендует создавать окно внутри замыкания `run`,
    // но для совместимости с `pixels` мы создаем его здесь.
    // Это вызывает предупреждение о `deprecated`, но на работу не влияет.
    let window = Arc::new({
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let attributes = Window::default_attributes()
            .with_title("Shadowin AI")
            .with_inner_size(size);
        event_loop.create_window(attributes)?
    });

    // ИСПРАВЛЕНИЕ (E0505): Создаем клон `Arc` специально для замыкания.
    // `window_clone` будет перемещен в замыкание, в то время как
    // оригинальный `window` будет использован для создания `pixels`,
    // что решает конфликт владения.
    let window_clone = Arc::clone(&window);

    let mut pixels = {
        let window_size = window.inner_size();
        // `pixels` заимствует `window` здесь. Это заимствование будет жить
        // столько же, сколько и `pixels`.
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &*window);
        pollster::block_on(Pixels::new_async(WIDTH, HEIGHT, surface_texture))?
    };

    let font_data = include_bytes!("../assets/font.ttf");
    let font = Arc::new(rusttype::Font::try_from_bytes(font_data).ok_or("Failed to load font")?);
    
    let ai_renderer = Arc::new(AiRenderer::new());

    let mut app_state = AppState {
        mouse_pos: (-1, -1),
        mouse_pressed: false,
        message: "AI Renderer is initializing...".to_string(),
        bg_color: [20, 20, 30, 255],
        click_count: 0,
        text_input_content: String::new(),
    };

    let mut mode = AppMode::Loading(LoadingState::new(Arc::clone(&font), Arc::clone(&ai_renderer)));

    // --- 2. Главный Цикл ---
    // Примечание: `event_loop.run` также является `deprecated`.
    // Современный API `winit` использует `run_app`, но это требует
    // рефакторинга всей структуры приложения.
    event_loop.run(move |event, elwt| {
        let mut mouse_clicked_this_frame = false;
        if let Event::WindowEvent { event, .. } = &event {
            match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::KeyboardInput { event: key_event, .. } if key_event.state.is_pressed() && key_event.logical_key == Key::Named(NamedKey::Escape) => {
                    elwt.exit()
                }
                WindowEvent::CursorMoved { position, .. } => {
                    if let Ok((x, y)) = pixels.window_pos_to_pixel((*position).into()) {
                        app_state.mouse_pos = (x as i32, y as i32);
                    } else {
                        app_state.mouse_pos = (-1, -1);
                    }
                }
                WindowEvent::MouseInput { state, button: winit::event::MouseButton::Left, .. } => {
                    let is_pressed = *state == winit::event::ElementState::Pressed;
                    if !is_pressed && app_state.mouse_pressed {
                        mouse_clicked_this_frame = true;
                    }
                    app_state.mouse_pressed = is_pressed;
                }
                WindowEvent::Resized(size) => {
                    if let Err(_) = pixels.resize_surface(size.width, size.height) { elwt.exit(); }
                }
                _ => {}
            }
        }

        // --- 3. Логика и Отрисовка в зависимости от режима ---
        match &mut mode {
            AppMode::Loading(loading_state) => {
                if let Some(finished_ui) = loading_state.update() {
                    mode = AppMode::Running(finished_ui);
                    app_state.message = "AI Renderer is ready.".to_string();
                }
            }
            AppMode::Running(app_ui) => {
                if let Some(clicked_id) = app_ui.update(&app_state, mouse_clicked_this_frame, &event) {
                    app_state.click_count += 1;
                    match clicked_id {
                        0 => { // Submit
                            if app_state.text_input_content == "shadowin" {
                                app_state.message = "Welcome, master.".to_string();
                                app_state.bg_color = [40, 20, 20, 255];
                            } else {
                                app_state.message = format!("Submitted: {}", app_state.text_input_content);
                            }
                        },
                        1 => { // Clear
                            app_ui.text_input.text.clear();
                            app_state.text_input_content.clear(); 
                            app_state.message = "Cleared.".to_string();
                            app_state.bg_color = [20, 20, 30, 255];
                            app_state.click_count = 0;
                        },
                        _ => {}
                    }
                }
                app_state.text_input_content = app_ui.text_input.text.clone();
            }
        }
        
        // --- 4. Отрисовка ---
        if let Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } = event {
            let frame = pixels.frame_mut();
            frame.chunks_exact_mut(4).for_each(|pixel| pixel.copy_from_slice(&app_state.bg_color));

            match &mut mode {
                AppMode::Loading(loading_state) => loading_state.draw(frame, WIDTH),
                AppMode::Running(app_ui) => app_ui.draw(&app_state, frame, WIDTH),
            }

            if let Err(_) = pixels.render() { elwt.exit(); }
        }
        
        // Используем клон, который был перемещен в замыкание.
        window_clone.request_redraw();
    })?;

    Ok(())
}