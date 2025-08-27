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

    pub fn handle_key_event(&mut self, key_event: &KeyEvent) {
        if !self.text_input.is_focused || !key_event.state.is_pressed() {
            return;
        }
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
            button.draw(frame, screen_width);
        }
        let final_message = format!("{} (Clicks: {})", app_state.message, app_state.click_count);
        self.text_panel.draw(&final_message, frame, screen_width);
        self.text_input.draw(frame, screen_width);
    }
}