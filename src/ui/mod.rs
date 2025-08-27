// src/ui/mod.rs

pub mod widgets;

use crate::AppState;
use widgets::{Button, TextPanel, TextInput};
use rusttype::Font;
use winit::event::KeyEvent;
use winit::keyboard::{Key, NamedKey};

pub struct Ui<'a> {
    pub buttons: Vec<Button<'a>>,
    pub text_panel: TextPanel<'a>,
    pub text_input: TextInput<'a>,
}

impl<'a> Ui<'a> {
    pub fn new(font: &'a Font) -> Self {
        let buttons = vec![
            Button::new(0, 50, 50, 200, 60, "Submit".to_string(), font),
            Button::new(1, 270, 50, 150, 60, "Clear".to_string(), font),
        ];
        let text_panel = TextPanel::new(50, 200, font);
        let text_input = TextInput::new(50, 130, 370, 40, font);
        
        Self { buttons, text_panel, text_input }
    }

    // Новый метод, обрабатывает только клики и фокус
    pub fn handle_click(&mut self, mouse_pos: (i32, i32)) -> Option<usize> {
        // Проверяем, кликнули ли по кнопке
        for button in &self.buttons {
            if button.is_over(mouse_pos) {
                self.text_input.is_focused = false; // Клик по кнопке снимает фокус
                return Some(button.id);
            }
        }
        
        // Проверяем, кликнули ли по полю ввода
        if self.text_input.is_over(mouse_pos) {
            self.text_input.is_focused = true;
        } else {
            self.text_input.is_focused = false; // Клик в пустом месте снимает фокус
        }
        
        None
    }

    // Старый `update` теперь `update_visuals` и он проще
    pub fn update_visuals(&mut self, mouse_pos: (i32, i32), mouse_pressed: bool) {
        for button in self.buttons.iter_mut() {
            button.update(mouse_pos, mouse_pressed);
        }
        // В будущем здесь можно обновлять визуал и для других виджетов
    }

    pub fn handle_key_event(&mut self, key_event: &KeyEvent) {
        if !self.text_input.is_focused || !key_event.state.is_pressed() {
            return;
        }
        if let Key::Named(NamedKey::Backspace) = key_event.logical_key {
            self.text_input.backspace();
        }
    }
    
    // В `draw` мы передаем `&mut self`, так как TextInput его требует
    pub fn draw(&mut self, app_state: &AppState, frame: &mut [u8], screen_width: u32) {
        for button in &self.buttons {
            button.draw(frame, screen_width);
        }
        self.text_panel.draw(&app_state.message, frame, screen_width);
        self.text_input.draw(frame, screen_width);
    }
}