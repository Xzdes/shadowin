// src/ui/widgets.rs

use rusttype::{point, Font, Scale};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use image::{DynamicImage, GenericImageView, Rgba, Pixel}; // ИСПРАВЛЕНИЕ: Импортируем трейт Pixel
use std::sync::Arc;

const TRANSITION_DURATION: Duration = Duration::from_millis(200);

/// Визуальные состояния, которые может сгенерировать AI.
#[derive(PartialEq, Clone, Copy, Debug, Hash, Eq)]
pub enum VisualState {
    Idle,
    Hovered,
    Pressed,
}

/// Структура, описывающая переход из одного состояния в другое.
#[derive(Debug)]
struct Transition {
    from: VisualState,
    to: VisualState,
    start: Instant,
}

/// Текущее состояние кнопки: либо стабильное, либо в процессе анимации.
enum ButtonState {
    Stable(VisualState),
    Animating(Transition),
}

/// Наш новый, умный виджет кнопки.
pub struct Button {
    pub id: usize,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    state: ButtonState,
    text: String,
    font: Arc<Font<'static>>,
}

impl Button {
    pub fn new(id: usize, x: i32, y: i32, width: u32, height: u32, text: String, font: Arc<Font<'static>>) -> Self {
        Self {
            id, x, y, width, height,
            state: ButtonState::Stable(VisualState::Idle),
            text,
            font,
        }
    }

    /// Обновляет состояние кнопки, управляя анимациями. Возвращает `true` при клике.
    pub fn update(&mut self, mouse_pos: (i32, i32), mouse_pressed: bool, mouse_clicked: bool) -> bool {
        let is_over = mouse_pos.0 >= self.x
            && mouse_pos.0 <= self.x + self.width as i32
            && mouse_pos.1 >= self.y
            && mouse_pos.1 <= self.y + self.height as i32;

        let target_state = if is_over {
            if mouse_pressed { VisualState::Pressed } else { VisualState::Hovered }
        } else {
            VisualState::Idle
        };

        // ИСПРАВЛЕНИЕ: Рефакторинг для обхода borrow checker'a
        let mut transition_finished = false;
        let mut new_stable_state = VisualState::Idle; // временное значение
        let mut current_visual_state = VisualState::Idle; // временное значение

        match &self.state {
            ButtonState::Stable(s) => {
                current_visual_state = *s;
            }
            ButtonState::Animating(t) => {
                current_visual_state = t.to; // Для логики переходов считаем, что мы уже в целевом состоянии
                if t.start.elapsed() >= TRANSITION_DURATION {
                    transition_finished = true;
                    new_stable_state = t.to;
                }
            }
        };
        
        if transition_finished {
            self.state = ButtonState::Stable(new_stable_state);
        }

        if current_visual_state != target_state {
            let from_state = match self.state {
                ButtonState::Stable(s) => s,
                ButtonState::Animating(ref t) => t.to,
            };
            self.state = ButtonState::Animating(Transition {
                from: from_state,
                to: target_state,
                start: Instant::now(),
            });
        }
        
        is_over && mouse_clicked
    }

    /// Генерирует промпты для AI. ВАЖНО: теперь просим фон БЕЗ ТЕКСТА.
    pub fn get_render_prompts(&self) -> HashMap<String, String> {
        let mut prompts = HashMap::new();
        let base_prompt = format!(
            "a crisp UI button background, no text, photorealistic, octane render, trending on artstation, dark sci-fi style, neon blue highlights",
        );
        prompts.insert(format!("{}-{:?}", self.id, VisualState::Idle), format!("{}, normal state", base_prompt));
        prompts.insert(format!("{}-{:?}", self.id, VisualState::Hovered), format!("{}, glowing, hovered state", base_prompt));
        prompts.insert(format!("{}-{:?}", self.id, VisualState::Pressed), format!("{}, pressed down, indented", base_prompt));
        prompts
    }

    /// Отрисовывает кнопку, включая анимации и адаптивный текст.
    pub fn draw(&self, frame: &mut [u8], screen_width: u32, cache: &HashMap<String, Arc<DynamicImage>>) {
        let (background, top_layer, progress) = match &self.state {
            ButtonState::Stable(state) => {
                let key = format!("{}-{:?}", self.id, state);
                (cache.get(&key), None, 0.0)
            }
            ButtonState::Animating(t) => {
                let from_key = format!("{}-{:?}", self.id, t.from);
                let to_key = format!("{}-{:?}", self.id, t.to);
                let progress = (t.start.elapsed().as_secs_f32() / TRANSITION_DURATION.as_secs_f32()).min(1.0);
                (cache.get(&from_key), cache.get(&to_key), progress)
            }
        };

        // 1. Отрисовка фона (и псевдо-анимации)
        if let Some(bg_image) = background {
            draw_image(frame, screen_width, bg_image, self.x, self.y, 1.0 - progress);

            if let Some(top_image) = top_layer {
                draw_image(frame, screen_width, top_image, self.x, self.y, progress);
            }
        } else {
            // Запасной вариант, если картинка не найдена
            draw_fallback_rect(frame, screen_width, self.x, self.y, self.width, self.height, [50, 50, 50, 255]);
        }
        
        // 2. Отрисовка адаптивного текста
        let text_image_source = top_layer.or(background).map(|i| Arc::clone(i));
        if let Some(image_for_text) = text_image_source {
            let text_color = calculate_contrast_color(&image_for_text);
            draw_text(frame, screen_width, &self.font, &self.text, self.x, self.y, self.width, self.height, text_color);
        }
    }
}

// --- Хелперы для отрисовки ---

fn draw_image(frame: &mut [u8], screen_width: u32, image: &DynamicImage, x: i32, y: i32, alpha_multiplier: f32) {
    for (px, py, pixel) in image.pixels() {
        let screen_x = x + px as i32;
        let screen_y = y + py as i32;
        if screen_x >= 0 && screen_y >= 0 {
            let index = ((screen_y as u32 * screen_width) + screen_x as u32) as usize * 4;
            if index + 3 < frame.len() {
                // Только рисуем пиксели с ненулевой альфой
                if alpha_multiplier > 0.01 {
                    let mut new_pixel = pixel;
                    new_pixel.0[3] = (new_pixel.0[3] as f32 * alpha_multiplier) as u8;
                    
                    let old_pixel_slice = &mut frame[index..index + 4];
                    let mut old_pixel = Rgba([old_pixel_slice[0], old_pixel_slice[1], old_pixel_slice[2], old_pixel_slice[3]]);
                    
                    // ИСПРАВЛЕНИЕ: теперь blend() в области видимости
                    old_pixel.blend(&new_pixel);
                    
                    old_pixel_slice.copy_from_slice(&old_pixel.0);
                }
            }
        }
    }
}


fn draw_text(frame: &mut [u8], screen_width: u32, font: &Font, text: &str, x: i32, y: i32, w: u32, h: u32, color: [u8; 4]) {
    let scale = Scale { x: h as f32 * 0.5, y: h as f32 * 0.5 };
    let v_metrics = font.v_metrics(scale);
    let glyphs_height = v_metrics.ascent - v_metrics.descent;
    
    let glyphs: Vec<_> = font.layout(text, scale, point(0.0, 0.0)).collect();
    let text_width: f32 = glyphs.iter().map(|g| g.unpositioned().h_metrics().advance_width).sum();
    
    let text_x = x as f32 + (w as f32 - text_width) / 2.0;
    let text_y = y as f32 + (h as f32 - glyphs_height) / 2.0 + v_metrics.ascent;
    
    for glyph in font.layout(text, scale, point(text_x, text_y)) {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, v| {
                if v > 0.1 {
                    let px = bb.min.x + gx as i32;
                    let py = bb.min.y + gy as i32;
                    if px >= 0 && py >= 0 {
                        let index = ((py as u32 * screen_width) + px as u32) as usize * 4;
                        if index + 3 < frame.len() {
                            let text_pixel = Rgba([color[0], color[1], color[2], (color[3] as f32 * v) as u8]);
                            let old_pixel_slice = &mut frame[index..index + 4];
                            let mut old_pixel = Rgba([old_pixel_slice[0], old_pixel_slice[1], old_pixel_slice[2], old_pixel_slice[3]]);
                            old_pixel.blend(&text_pixel);
                            old_pixel_slice.copy_from_slice(&old_pixel.0);
                        }
                    }
                }
            });
        }
    }
}

fn draw_fallback_rect(frame: &mut[u8], screen_width: u32, x: i32, y: i32, w: u32, h: u32, color: [u8; 4]) {
    for row in 0..h {
        for col in 0..w {
            let px = x + col as i32;
            let py = y + row as i32;
            if px >= 0 && py >= 0 {
                let index = ((py as u32 * screen_width) + px as u32) as usize * 4;
                if index + 3 < frame.len() {
                    frame[index..index + 4].copy_from_slice(&color);
                }
            }
        }
    }
}

/// "Текст-Хамелеон": вычисляет, каким должен быть цвет текста, чтобы он был контрастным.
fn calculate_contrast_color(image: &DynamicImage) -> [u8; 4] {
    let mut total_luminance = 0.0;
    let pixel_count = (image.width() * image.height()) as f32;

    for (_, _, pixel) in image.pixels() {
        // Формула вычисления яркости (стандартная для sRGB)
        let r = pixel[0] as f32 / 255.0;
        let g = pixel[1] as f32 / 255.0;
        let b = pixel[2] as f32 / 255.0;
        total_luminance += 0.2126 * r + 0.7152 * g + 0.0722 * b;
    }
    
    let avg_luminance = total_luminance / pixel_count;

    if avg_luminance > 0.5 {
        [10, 10, 10, 255] // Темный текст на светлом фоне
    } else {
        [240, 240, 240, 255] // Светлый текст на темном фоне
    }
}


// --- TextPanel и TextInput (остаются без изменений, но принимают Arc<Font>) ---
pub struct TextPanel { pub x: i32, pub y: i32, font: Arc<Font<'static>>, }
impl TextPanel {
    pub fn new(x: i32, y: i32, font: Arc<Font<'static>>) -> Self { Self { x, y, font } }
    pub fn draw(&self, text: &str, frame: &mut [u8], screen_width: u32) {
        draw_text(frame, screen_width, &self.font, text, self.x, self.y, 700, 50, [200, 200, 200, 255]);
    }
}

pub struct TextInput {
    pub x: i32, pub y: i32, pub width: u32, pub height: u32, pub text: String, font: Arc<Font<'static>>,
    pub is_focused: bool, cursor_timer: Instant, cursor_visible: bool,
}
impl TextInput {
    pub fn new(x: i32, y: i32, width: u32, height: u32, font: Arc<Font<'static>>) -> Self { Self { x, y, width, height, text: String::new(), font, is_focused: false, cursor_timer: Instant::now(), cursor_visible: false } }
    pub fn is_over(&self, mouse_pos: (i32, i32)) -> bool { mouse_pos.0 >= self.x && mouse_pos.0 <= self.x + self.width as i32 && mouse_pos.1 >= self.y && mouse_pos.1 <= self.y + self.height as i32 }
    pub fn key_press(&mut self, chars: &str) { if self.is_focused { self.text.push_str(chars); } }
    pub fn backspace(&mut self) { if self.is_focused { self.text.pop(); } }
    pub fn draw(&mut self, frame: &mut [u8], screen_width: u32) {
        let bg_color = if self.is_focused { [50, 50, 60, 255] } else { [30, 30, 40, 255] };
        draw_fallback_rect(frame, screen_width, self.x, self.y, self.width, self.height, bg_color);
        
        let scale = Scale { x: 24.0, y: 24.0 }; let text_color = [220, 220, 220, 255];
        let v_metrics = self.font.v_metrics(scale); let text_y = self.y + ((self.height as f32 - (v_metrics.ascent - v_metrics.descent)) / 2.0 + v_metrics.ascent) as i32;
        let glyphs: Vec<_> = self.font.layout(&self.text, scale, point(self.x as f32 + 10.0, text_y as f32)).collect();
        let cursor_x = glyphs.iter().last().map_or(self.x + 10, |g| (g.position().x + g.unpositioned().h_metrics().advance_width) as i32);
        for glyph in glyphs { if let Some(bounding_box) = glyph.pixel_bounding_box() { glyph.draw(|gx, gy, v| { if v > 0.1 { let px = bounding_box.min.x + gx as i32; let py = bounding_box.min.y + gy as i32; if px >= 0 && py >= 0 { let index = ((py as u32 * screen_width) + px as u32) as usize * 4; if index + 3 < frame.len() { frame[index..index + 4].copy_from_slice(&text_color); } } } }); } }
        if self.is_focused { if self.cursor_timer.elapsed().as_millis() > 500 { self.cursor_visible = !self.cursor_visible; self.cursor_timer = Instant::now(); } if self.cursor_visible { let cursor_height = (v_metrics.ascent - v_metrics.descent) as u32; for i in 0..cursor_height { let px = cursor_x; let py = text_y - v_metrics.ascent as i32 + i as i32; if px >= 0 && py >= 0 { let index = ((py as u32 * screen_width) + px as u32) as usize * 4; if index + 3 < frame.len() { frame[index..index + 4].copy_from_slice(&text_color); } } } } }
    }
}