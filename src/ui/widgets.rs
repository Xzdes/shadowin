// src/ui/widgets.rs

use rusttype::{point, Font, Scale};
use std::time::Instant;
use std::collections::HashMap;
use image::{DynamicImage, GenericImageView};
use std::sync::Arc;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ButtonState { Idle, Hovered, Pressed }

// ИСПРАВЛЕНИЕ: Убираем `'a`
pub struct Button {
    pub id: usize,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub state: ButtonState,
    was_pressed: bool,
    text: String,
    // Поле `font` больше не нужно
}

// ИСПРАВЛЕНИЕ: Убираем `'a` и `font` из параметров
impl Button {
    pub fn new(id: usize, x: i32, y: i32, width: u32, height: u32, text: String, _font: &Font) -> Self {
        Self { id, x, y, width, height, state: ButtonState::Idle, was_pressed: false, text }
    }

    pub fn update(&mut self, mouse_pos: (i32, i32), mouse_pressed: bool) -> bool {
        let is_over = mouse_pos.0 >= self.x && mouse_pos.0 <= self.x + self.width as i32 && mouse_pos.1 >= self.y && mouse_pos.1 <= self.y + self.height as i32;
        let mut clicked = false;
        let new_state = if is_over { if mouse_pressed { ButtonState::Pressed } else { ButtonState::Hovered } } else { ButtonState::Idle };
        if self.was_pressed && new_state == ButtonState::Hovered { clicked = true; }
        self.state = new_state;
        self.was_pressed = self.state == ButtonState::Pressed;
        clicked
    }

    pub fn get_render_prompts(&self) -> HashMap<String, String> {
        let mut prompts = HashMap::new();
        let base_prompt = format!(
            "a crisp UI button with the text '{}', photorealistic, octane render, trending on artstation, dark sci-fi style, neon blue highlights",
            self.text
        );
        prompts.insert(format!("{}-{:?}", self.id, ButtonState::Idle), format!("{}, normal state", base_prompt));
        prompts.insert(format!("{}-{:?}", self.id, ButtonState::Hovered), format!("{}, glowing, hovered state", base_prompt));
        prompts.insert(format!("{}-{:?}", self.id, ButtonState::Pressed), format!("{}, pressed, indented", base_prompt));
        prompts
    }

    pub fn draw(&self, frame: &mut [u8], screen_width: u32, cache: &HashMap<String, Arc<DynamicImage>>) {
        let key = format!("{}-{:?}", self.id, self.state);
        if let Some(image) = cache.get(&key) {
            for (px, py, pixel) in image.pixels() {
                let screen_x = self.x + px as i32;
                let screen_y = self.y + py as i32;
                if screen_x >= 0 && screen_y >= 0 {
                    let index = ((screen_y as u32 * screen_width) + screen_x as u32) as usize * 4;
                    if index + 3 < frame.len() {
                        let mut rgba = pixel.0;
                        rgba[3] = 255;
                        frame[index..index + 4].copy_from_slice(&rgba);
                    }
                }
            }
        } else {
            let color = [50, 50, 50, 255]; // Серый запасной вариант
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
        }
    }
}

// --- TextPanel (без изменений) ---
pub struct TextPanel<'a> {
    pub x: i32,
    pub y: i32,
    font: &'a Font<'a>,
}
impl<'a> TextPanel<'a> {
    pub fn new(x: i32, y: i32, font: &'a Font) -> Self { Self { x, y, font } }
    pub fn draw(&self, text: &str, frame: &mut [u8], screen_width: u32) {
        let scale = Scale { x: 40.0, y: 40.0 }; let text_color = [200, 200, 200, 255];
        let glyphs: Vec<_> = self.font.layout(text, scale, point(self.x as f32, self.y as f32)).collect();
        for glyph in glyphs { if let Some(bounding_box) = glyph.pixel_bounding_box() { glyph.draw(|gx, gy, v| { if v > 0.1 { let px = bounding_box.min.x + gx as i32; let py = bounding_box.min.y + gy as i32; if px >= 0 && py >= 0 { let index = ((py as u32 * screen_width) + px as u32) as usize * 4; if index + 3 < frame.len() { frame[index..index + 4].copy_from_slice(&text_color); } } } }); } }
    }
}

// --- TextInput (без изменений) ---
pub struct TextInput<'a> {
    pub x: i32, pub y: i32, pub width: u32, pub height: u32, pub text: String, font: &'a Font<'a>,
    pub is_focused: bool, cursor_timer: Instant, cursor_visible: bool,
}
impl<'a> TextInput<'a> {
    pub fn new(x: i32, y: i32, width: u32, height: u32, font: &'a Font) -> Self { Self { x, y, width, height, text: String::new(), font, is_focused: false, cursor_timer: Instant::now(), cursor_visible: false } }
    pub fn is_over(&self, mouse_pos: (i32, i32)) -> bool { mouse_pos.0 >= self.x && mouse_pos.0 <= self.x + self.width as i32 && mouse_pos.1 >= self.y && mouse_pos.1 <= self.y + self.height as i32 }
    pub fn key_press(&mut self, chars: &str) { if self.is_focused { self.text.push_str(chars); } }
    pub fn backspace(&mut self) { if self.is_focused { self.text.pop(); } }
    pub fn draw(&mut self, frame: &mut [u8], screen_width: u32) {
        let bg_color = if self.is_focused { [50, 50, 60, 255] } else { [30, 30, 40, 255] };
        for row in 0..self.height { for col in 0..self.width { let px = self.x + col as i32; let py = self.y + row as i32; if px >= 0 && py >= 0 { let index = ((py as u32 * screen_width) + px as u32) as usize * 4; if index + 3 < frame.len() { frame[index..index + 4].copy_from_slice(&bg_color); } } } }
        let scale = Scale { x: 24.0, y: 24.0 }; let text_color = [220, 220, 220, 255];
        let v_metrics = self.font.v_metrics(scale); let text_y = self.y + ((self.height as f32 - (v_metrics.ascent - v_metrics.descent)) / 2.0 + v_metrics.ascent) as i32;
        let glyphs: Vec<_> = self.font.layout(&self.text, scale, point(self.x as f32 + 10.0, text_y as f32)).collect();
        let cursor_x = glyphs.iter().last().map_or(self.x + 10, |g| (g.position().x + g.unpositioned().h_metrics().advance_width) as i32);
        for glyph in glyphs { if let Some(bounding_box) = glyph.pixel_bounding_box() { glyph.draw(|gx, gy, v| { if v > 0.1 { let px = bounding_box.min.x + gx as i32; let py = bounding_box.min.y + gy as i32; if px >= 0 && py >= 0 { let index = ((py as u32 * screen_width) + px as u32) as usize * 4; if index + 3 < frame.len() { frame[index..index + 4].copy_from_slice(&text_color); } } } }); } }
        if self.is_focused { if self.cursor_timer.elapsed().as_millis() > 500 { self.cursor_visible = !self.cursor_visible; self.cursor_timer = Instant::now(); } if self.cursor_visible { let cursor_height = (v_metrics.ascent - v_metrics.descent) as u32; for i in 0..cursor_height { let px = cursor_x; let py = text_y - v_metrics.ascent as i32 + i as i32; if px >= 0 && py >= 0 { let index = ((py as u32 * screen_width) + px as u32) as usize * 4; if index + 3 < frame.len() { frame[index..index + 4].copy_from_slice(&text_color); } } } } }
    }
}