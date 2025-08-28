// src/ai_renderer.rs

use serde::{Serialize, Deserialize};
use reqwest::blocking::Client; // <-- ВАЖНО: импортируем БЛОКИРУЮЩИЙ клиент
use image::{DynamicImage, ImageFormat};
use base64::{Engine as _, engine::general_purpose};

// Адрес API нашего локального сервера Stable Diffusion
const API_URL: &str = "http://127.0.0.1:7860/sdapi/v1/txt2img";

// Структура, которая описывает наш JSON-запрос к API
#[derive(Serialize)]
struct Txt2ImgRequest {
    prompt: String,
    negative_prompt: String,
    steps: u32,
    width: u32,
    height: u32,
    cfg_scale: f32,
    sampler_name: String,
}

// Структура, которая описывает JSON-ответ от API
#[derive(Deserialize)]
struct Txt2ImgResponse {
    images: Vec<String>,
}

// Наш главный "AI-Коммуникатор"
pub struct AiRenderer {
    client: Client,
}

impl AiRenderer {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    // Функция теперь СИНХРОННАЯ. Она будет "замораживать" программу,
    // пока нейросеть думает, и возвращать результат, когда он будет готов.
    pub fn generate_image(&self, prompt: &str, width: u32, height: u32) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        println!("AI Renderer: Sending prompt: '{}'", prompt);

        // 1. Создаем тело запроса
        let request_body = Txt2ImgRequest {
            prompt: prompt.to_string(),
            negative_prompt: "blurry, worst quality, low quality, deformed, text, watermark, signature".to_string(),
            steps: 20,
            width,
            height,
            cfg_scale: 7.0,
            sampler_name: "Euler a".to_string(),
        };

        // 2. Отправляем POST-запрос и ждем ответа
        let response = self.client.post(API_URL).json(&request_body).send()?;

        // 3. Проверяем, успешен ли ответ
        if !response.status().is_success() {
            return Err(format!("API Error: {}", response.text()?).into());
        }

        // 4. Парсим JSON-ответ
        let response_data: Txt2ImgResponse = response.json()?;

        // 5. Декодируем изображение из Base64
        let base64_image = response_data.images.get(0).ok_or("No images in API response")?;
        let image_bytes = general_purpose::STANDARD.decode(base64_image)?;

        // 6. Превращаем байты в объект изображения
        let image = image::load_from_memory_with_format(&image_bytes, ImageFormat::Png)?;

        println!("AI Renderer: Image received successfully!");
        Ok(image)
    }
}