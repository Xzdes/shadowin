// src/ui/mod.rs

pub mod widgets;

use crate::{ai_renderer::AiRenderer, AppState};
use widgets::{Button, TextInput, TextPanel};
use rusttype::Font;
use winit::event::{Event, KeyEvent, WindowEvent, Ime};
use std::collections::HashMap;
use image::DynamicImage;
use std::sync::Arc;

/// Менеджер UI основного приложения.
pub struct AppUi {
    pub buttons: Vec<Button>,
    pub text_input: TextInput,
    pub text_panel: TextPanel,
    #[allow(dead_code)]
    ai_renderer: Arc<AiRenderer>, // Сохраняем на случай будущих генераций
    render_cache: HashMap<String, Arc<DynamicImage>>,
}

impl AppUi {
    /// Создается с уже готовым, заполненным кэшем.
    pub fn new(
        font: Arc<Font<'static>>,
        ai_renderer: Arc<AiRenderer>,
        render_cache: HashMap<String, Arc<DynamicImage>>,
    ) -> Self {
        let buttons = vec![
            // ИСПРАВЛЕНИЕ: Конвертируем &str в String
            Button::new(0, 50, 50, 200, 60, "Submit".to_string(), Arc::clone(&font)),
            Button::new(1, 270, 50, 150, 60, "Clear".to_string(), Arc::clone(&font)),
        ];
        let text_input = TextInput::new(50, 130, 370, 40, Arc::clone(&font));
        let text_panel = TextPanel::new(50, 200, Arc::clone(&font));

        Self {
            buttons,
            text_input,
            text_panel,
            ai_renderer,
            render_cache,
        }
    }

    /// Обновляет состояние всех виджетов.
    pub fn update(
        &mut self,
        app_state: &AppState,
        mouse_clicked: bool,
        event: &Event<()>,
    ) -> Option<usize> {
        // Фокус текстового поля
        if mouse_clicked {
            if self.text_input.is_over(app_state.mouse_pos) {
                self.text_input.is_focused = true;
            } else {
                self.text_input.is_focused = false;
            }
        }

        // Обработка ввода текста
        if let Event::WindowEvent { event, .. } = event {
            self.handle_key_event(event);
        }

        let mut clicked_id = None;
        for button in self.buttons.iter_mut() {
            // `button.update` теперь возвращает `bool` только если был произведен КЛИК
            if button.update(app_state.mouse_pos, app_state.mouse_pressed, mouse_clicked) {
                clicked_id = Some(button.id);
                // Если кликнули на кнопку, снимаем фокус с поля ввода
                self.text_input.is_focused = false;
            }
        }

        clicked_id
    }

    /// Обработка событий клавиатуры для текстового поля
    fn handle_key_event(&mut self, event: &WindowEvent) {
        if !self.text_input.is_focused { return; }
        match event {
            WindowEvent::KeyboardInput { event: KeyEvent { state, logical_key, .. }, .. } if *state == winit::event::ElementState::Pressed => {
                use winit::keyboard::{Key, NamedKey};
                match logical_key {
                    Key::Named(NamedKey::Backspace) => self.text_input.backspace(),
                    _ => {}
                }
            },
            WindowEvent::Ime(Ime::Commit(text)) => {
                self.text_input.key_press(text);
            }
            _ => (),
        }
    }
    
    /// Отрисовка всех виджетов.
    pub fn draw(&mut self, app_state: &AppState, frame: &mut [u8], screen_width: u32) {
        for button in &self.buttons {
            // Передаем кэш в каждый виджет для отрисовки
            button.draw(frame, screen_width, &self.render_cache);
        }

        let final_message = format!("{} (Clicks: {})", app_state.message, app_state.click_count);
        self.text_panel.draw(&final_message, frame, screen_width);
        
        self.text_input.draw(frame, screen_width);
    }
}