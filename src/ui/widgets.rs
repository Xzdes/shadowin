// src/ui/widgets.rs

use rusttype::{point, Font, Scale};

// --- Код Button остается без изменений ---
#[derive(PartialEq, Clone, Copy)]
pub enum ButtonState {
    Idle,
    Hovered,
    Pressed,
}

pub struct Button<'a> {
    pub id: usize,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub state: ButtonState,
    was_pressed: bool,
    text: String,
    font: &'a Font<'a>,
}

impl<'a> Button<'a> {
    pub fn new(id: usize, x: i32, y: i32, width: u32, height: u32, text: String, font: &'a Font) -> Self {
        Self { id, x, y, width, height, state: ButtonState::Idle, was_pressed: false, text, font }
    }

    pub fn update(&mut self, mouse_pos: (i32, i32), mouse_pressed: bool) -> bool {
        let is_over = mouse_pos.0 >= self.x && mouse_pos.0 <= self.x + self.width as i32 && mouse_pos.1 >= self.y && mouse_pos.1 <= self.y + self.height as i32;
        let mut clicked = false;
        let new_state = if is_over {
            if mouse_pressed { ButtonState::Pressed } else { ButtonState::Hovered }
        } else {
            ButtonState::Idle
        };
        if self.was_pressed && new_state == ButtonState::Hovered {
            clicked = true;
        }
        self.state = new_state;
        self.was_pressed = self.state == ButtonState::Pressed;
        clicked
    }

    pub fn draw(&self, frame: &mut [u8], screen_width: u32) {
        let color = match self.state {
            ButtonState::Idle => [0, 123, 255, 255],
            ButtonState::Hovered => [0, 153, 255, 255],
            ButtonState::Pressed => [0, 93, 205, 255],
        };
        for row in 0..self.height {
            for col in 0..self.width {
                let px = self.x + col as i32;
                let py = self.y + row as i32;
                if px >= 0 && py >= 0 {
                    let index = ((py as u32 * screen_width) + px as u32) as usize * 4;
                    if index + 3 < frame.len() {
                        frame[index..index + 4].copy_from_slice(&color);
                    }
                }
            }
        }
        let scale = Scale { x: 30.0, y: 30.0 };
        let text_color = [255, 255, 255, 255];
        let v_metrics = self.font.v_metrics(scale);
        let text_height = v_metrics.ascent - v_metrics.descent;
        let glyphs: Vec<_> = self.font.layout(&self.text, scale, point(0.0, 0.0)).collect();
        let text_width = glyphs.iter().rev().map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width).next().unwrap_or(0.0);
        let text_start_x = self.x + ((self.width as f32 - text_width) / 2.0) as i32;
        let text_start_y = self.y + ((self.height as f32 + text_height) / 2.0) as i32;
        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    if v > 0.1 {
                        let px = text_start_x + bounding_box.min.x + gx as i32;
                        let py = text_start_y + bounding_box.min.y + gy as i32;
                        if px >= 0 && py >= 0 {
                            let index = ((py as u32 * screen_width) + px as u32) as usize * 4;
                            if index + 3 < frame.len() {
                                frame[index..index + 4].copy_from_slice(&text_color);
                            }
                        }
                    }
                });
            }
        }
    }
}

// ---- НОВЫЙ ВИДЖЕТ: TextPanel ----
pub struct TextPanel<'a> {
    pub x: i32,
    pub y: i32,
    // Текст для этой панели будет приходить извне
    font: &'a Font<'a>,
}

impl<'a> TextPanel<'a> {
    pub fn new(x: i32, y: i32, font: &'a Font) -> Self {
        Self { x, y, font }
    }

    // Метод draw теперь принимает текст для отрисовки как аргумент
    pub fn draw(&self, text: &str, frame: &mut [u8], screen_width: u32) {
        let scale = Scale { x: 40.0, y: 40.0 }; // Сделаем шрифт побольше
        let text_color = [200, 200, 200, 255]; // Серый цвет
        let glyphs: Vec<_> = self.font.layout(text, scale, point(0.0, 0.0)).collect();
        
        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    if v > 0.1 {
                        let px = self.x + bounding_box.min.x + gx as i32;
                        let py = self.y + bounding_box.min.y + gy as i32;
                        if px >= 0 && py >= 0 {
                            let index = ((py as u32 * screen_width) + px as u32) as usize * 4;
                            if index + 3 < frame.len() {
                                frame[index..index + 4].copy_from_slice(&text_color);
                            }
                        }
                    }
                });
            }
        }
    }
}