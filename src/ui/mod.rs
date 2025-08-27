// src/ui/mod.rs

pub mod widgets;

use crate::AppState;
use widgets::{Button, TextPanel};
use rusttype::Font;

pub struct Ui<'a> {
    buttons: Vec<Button<'a>>,
    text_panel: TextPanel<'a>,
}

impl<'a> Ui<'a> {
    pub fn new(font: &'a Font) -> Self {
        let mut buttons = Vec::new();
        // ---- ИСПРАВЛЕНИЕ 1 ----
        buttons.push(Button::new(0, 50, 50, 200, 60, "Click Me!".to_string(), font));
        // ---- ИСПРАВЛЕНИЕ 2 ----
        buttons.push(Button::new(1, 270, 50, 150, 60, "Reset".to_string(), font));

        let text_panel = TextPanel::new(50, 200, font);
        
        Self { buttons, text_panel }
    }

    pub fn update(&mut self, mouse_pos: (i32, i32), mouse_pressed: bool) -> Option<usize> {
        let mut clicked_id = None;
        for button in self.buttons.iter_mut() {
            if button.update(mouse_pos, mouse_pressed) {
                clicked_id = Some(button.id);
            }
        }
        clicked_id
    }

    pub fn draw(&self, app_state: &AppState, frame: &mut [u8], screen_width: u32) {
        for button in &self.buttons {
            button.draw(frame, screen_width);
        }
        self.text_panel.draw(&app_state.message, frame, screen_width);
    }
}