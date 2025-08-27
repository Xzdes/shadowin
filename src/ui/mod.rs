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

    pub fn handle_click(&mut self, mouse_pos: (i32, i32)) -> Option<usize> {
        for button in &self.buttons {
            if button.is_over(mouse_pos) {
                self.text_input.is_focused = false;
                return Some(button.id);
            }
        }
        
        if self.text_input.is_over(mouse_pos) {
            self.text_input.is_focused = true;
        } else {
            self.text_input.is_focused = false;
        }
        
        None
    }

    pub fn update_visuals(&mut self, mouse_pos: (i32, i32), mouse_pressed: bool) {
        for button in self.buttons.iter_mut() {
            button.update(mouse_pos, mouse_pressed);
        }
    }

    // ---- ПОЛНОСТЬЮ ПЕРЕПИСАННЫЙ МЕТОД ----
    pub fn handle_key_event(&mut self, key_event: &KeyEvent) {
        // Если поле ввода не в фокусе, ничего не делаем
        if !self.text_input.is_focused {
            return;
        }
        
        // Обрабатываем только НАЖАТИЯ клавиш
        if key_event.state.is_pressed() {
            match &key_event.logical_key {
                // Если это Backspace, стираем символ
                Key::Named(NamedKey::Backspace) => {
                    self.text_input.backspace();
                }
                // Если это вводимый символ (буква, цифра, знак)
                Key::Character(chars) => {
                    // `chars` - это уже готовая строка, которую нужно вставить
                    self.text_input.key_press(chars);
                }
                // Все остальные клавиши (Shift, Ctrl и т.д.) игнорируем
                _ => (),
            }
        }
    }
    // ------------------------------------

    pub fn draw(&mut self, app_state: &AppState, frame: &mut [u8], screen_width: u32) {
        for button in &self.buttons {
            button.draw(frame, screen_width);
        }
        self.text_panel.draw(&app_state.message, frame, screen_width);
        self.text_input.draw(frame, screen_width);
    }
}