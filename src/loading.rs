// src/loading.rs

use crate::ai_renderer::AiRenderer;
// ИСПРАВЛЕНИЕ: Убираем TextInput и TextPanel, так как они не используются здесь
use crate::ui::widgets::Button;
use crate::ui::AppUi;
use image::{DynamicImage, ImageFormat};
use rusttype::{point, Font, Scale};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

const CACHE_DIR: &str = "cache";

/// Структура, описывающая один ассет, который нужно сгенерировать.
struct AssetRequest {
    key: String,
    prompt: String,
    width: u32,
    height: u32,
}

/// Состояние экрана загрузки.
pub struct LoadingState {
    font: Arc<Font<'static>>,
    ai_renderer: Arc<AiRenderer>,
    render_cache: HashMap<String, Arc<DynamicImage>>,
    assets_to_generate: Vec<AssetRequest>,
    current_status: String,
    is_done: bool,
}

impl LoadingState {
    pub fn new(font: Arc<Font<'static>>, ai_renderer: Arc<AiRenderer>) -> Self {
        // 1. Создать директорию кэша, если ее нет
        fs::create_dir_all(CACHE_DIR).unwrap();

        // 2. Определить все необходимые ассеты
        let mut required_assets = Vec::new();
        let mut temp_buttons = vec![
            // ИСПРАВЛЕНИЕ: Конвертируем &str в String
            Button::new(0, 50, 50, 200, 60, "Submit".to_string(), Arc::clone(&font)),
            Button::new(1, 270, 50, 150, 60, "Clear".to_string(), Arc::clone(&font)),
        ];

        for button in &mut temp_buttons {
            for (state_key, prompt) in button.get_render_prompts() {
                required_assets.push(AssetRequest {
                    key: state_key,
                    prompt,
                    width: button.width,
                    height: button.height,
                });
            }
        }

        // 3. Проверить кэш на диске
        let mut render_cache = HashMap::new();
        let mut assets_to_generate = Vec::new();

        for asset in required_assets {
            let path = Path::new(CACHE_DIR).join(format!("{}.png", asset.key));
            if path.exists() {
                // Загружаем из кэша
                let image_bytes = fs::read(&path).unwrap();
                if let Ok(image) = image::load_from_memory_with_format(&image_bytes, ImageFormat::Png) {
                    render_cache.insert(asset.key, Arc::new(image));
                }
            } else {
                // Добавляем в очередь на генерацию
                assets_to_generate.push(asset);
            }
        }
        
        let initial_status = if assets_to_generate.is_empty() {
            "All assets loaded from cache. Starting...".to_string()
        } else {
            format!("Need to generate {} assets.", assets_to_generate.len())
        };
        // Инвертируем очередь, чтобы pop() доставал элементы в правильном порядке
        assets_to_generate.reverse();

        Self {
            font,
            ai_renderer,
            render_cache,
            assets_to_generate,
            current_status: initial_status,
            is_done: false,
        }
    }

    /// Обновляет состояние загрузки. Если генерация закончена, возвращает готовый UI.
    pub fn update(&mut self) -> Option<AppUi> {
        if self.is_done {
            return None; // Уже отдали UI, больше ничего не делаем
        }
        
        if let Some(asset) = self.assets_to_generate.pop() {
            self.current_status = format!("Generating: '{}'...", asset.key);
            println!("{}", self.current_status); // Лог в консоль

            // Это блокирующий вызов, он "заморозит" экран загрузки на время генерации.
            if let Ok(image) = self.ai_renderer.generate_image(&asset.prompt, asset.width, asset.height) {
                // Сохраняем в кэш на диске
                let path = Path::new(CACHE_DIR).join(format!("{}.png", asset.key));
                image.save(&path).unwrap();
                // Сохраняем в кэш в памяти
                self.render_cache.insert(asset.key, Arc::new(image));
            } else {
                self.current_status = format!("Error generating: '{}'!", asset.key);
                println!("{}", self.current_status);
            }
        } else {
            // Генерация закончена!
            self.current_status = "Generation complete! Finalizing...".to_string();
            self.is_done = true;
            
            // Создаем и возвращаем полностью готовый AppUi
            return Some(AppUi::new(
                Arc::clone(&self.font),
                Arc::clone(&self.ai_renderer),
                std::mem::take(&mut self.render_cache), // Передаем владение кэшем
            ));
        }

        None
    }

    /// Рисует нативный UI загрузки.
    pub fn draw(&self, frame: &mut [u8], screen_width: u32) {
        let text_color = [200, 200, 200, 255];
        let scale = Scale { x: 30.0, y: 30.0 };
        let text = &self.current_status;
        
        let v_metrics = self.font.v_metrics(scale);
        let glyphs_height = (v_metrics.ascent - v_metrics.descent).round() as u32;

        let glyphs: Vec<_> = self.font.layout(text, scale, point(0.0, 0.0)).collect();
        let width = glyphs.iter().map(|g| g.unpositioned().h_metrics().advance_width).sum::<f32>().round() as u32;
        
        let text_x = (screen_width.saturating_sub(width)) / 2;
        let text_y = (600 - glyphs_height) / 2;
        
        let final_glyphs: Vec<_> = self.font.layout(text, scale, point(text_x as f32, text_y as f32 + v_metrics.ascent)).collect();

        for glyph in final_glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|gx, gy, v| {
                    if v > 0.1 {
                        let px = bounding_box.min.x + gx as i32;
                        let py = bounding_box.min.y + gy as i32;
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