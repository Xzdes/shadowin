// src/ui/mod.rs

pub mod widgets;

use crate::{ai_renderer::AiRenderer, AppState};
use widgets::{Button, TextInput, TextPanel};
use rusttype::Font;
use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};
use std::collections::HashMap;
use image::DynamicImage;
use std::sync::Arc;

pub struct Ui<'a> {
    // ИСПРАВЛЕНИЕ: Убираем 'a, так как Button его больше не требует
    pub buttons: Vec<Button>,
    pub text_input: TextInput<'a>,
    pub text_panel: TextPanel<'a>,
    ai_renderer: Arc<AiRenderer>,
    render_cache: HashMap<String, Arc<DynamicImage>>,
}

impl<'a> Ui<'a> {
    pub fn new(font: &'a Font, ai_renderer: Arc<AiRenderer>) -> Self {
        let buttons = vec![
            Button::new(0, 50, 50, 200, 60, "Submit".to_string(), font),
            Button::new(1, 270, 50, 150, 60, "Clear".to_string(), font),
        ];
        let text_input = TextInput::new(50, 130, 370, 40, font);
        let text_panel = TextPanel::new(50, 200, font);
        
        Self { buttons, text_input, text_panel, ai_renderer, render_cache: HashMap::new() }
    }

    // ИСПРАВЛЕНИЕ: Этот метод теперь не асинхронный и разделен на два этапа
    pub fn update(&mut self, mouse_pos: (i32, i32), mouse_pressed: bool, mouse_clicked: bool) -> Option<usize> {
        if mouse_clicked {
            if self.text_input.is_over(mouse_pos) {
                self.text_input.is_focused = true;
            } else {
                self.text_input.is_focused = false;
            }
        }
        
        let mut clicked_id = None;
        
        // Этап 1: Обновляем состояние кнопок и собираем промпты в отдельный список
        let mut prompts_to_generate = Vec::new();
        for button in self.buttons.iter_mut() {
            if button.update(mouse_pos, mouse_pressed) {
                clicked_id = Some(button.id);
                self.text_input.is_focused = false;
            }
            
            for (state_key, prompt) in button.get_render_prompts() {
                if !self.render_cache.contains_key(&state_key) {
                    // Сохраняем все необходимое для генерации
                    prompts_to_generate.push((state_key, prompt, button.width, button.height));
                }
            }
        }
        
        // Этап 2: После того как мы закончили изменять `self.buttons`, мы можем безопасно изменять `self.render_cache`
        for (key, prompt, width, height) in prompts_to_generate {
            println!("Cache miss for: '{}'. Generating...", key);
            if let Ok(image) = self.ai_renderer.generate_image(&prompt, width, height) {
                self.render_cache.insert(key, Arc::new(image));
            }
        }
        
        clicked_id
    }

    pub fn handle_key_event(&mut self, key_event: &KeyEvent) {
        if !self.text_input.is_focused || !key_event.state.is_pressed() { return; }
        match &key_event.logical_key {
            Key::Named(NamedKey::Backspace) => {
                self.text_input.backspace();
            }
            Key::Character(chars) => {
                self.text_input.key_press(chars);
            }
            _ => (),
        }
    }
    
    pub fn draw(&mut self, app_state: &AppState, frame: &mut [u8], screen_width: u32) {
        for button in &self.buttons {
            button.draw(frame, screen_width, &self.render_cache);
        }
        
        let final_message = format!("{} (Clicks: {})", app_state.message, app_state.click_count);
        self.text_panel.draw(&final_message, frame, screen_width);
        
        self.text_input.draw(frame, screen_width);
    }
}