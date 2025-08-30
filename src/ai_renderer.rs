// src/ai_renderer.rs

use serde::{Serialize, Deserialize};
use reqwest::blocking::Client;
use image::{DynamicImage, ImageFormat};
use base64::{Engine as _, engine::general_purpose};

const API_URL: &str = "http://127.0.0.1:7860/sdapi/v1/txt2img";

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

#[derive(Deserialize)]
struct Txt2ImgResponse {
    images: Vec<String>,
}

pub struct AiRenderer {
    client: Client,
}

impl AiRenderer {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }

    pub fn generate_image(&self, prompt: &str, width: u32, height: u32) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        println!("AI Renderer: Sending prompt: '{}'", prompt);

        let request_body = Txt2ImgRequest {
            prompt: prompt.to_string(),
            negative_prompt: "blurry, worst quality, low quality, deformed, text, watermark, signature".to_string(),
            steps: 20,
            width,
            height,
            cfg_scale: 7.0,
            sampler_name: "Euler a".to_string(),
        };

        let response = self.client.post(API_URL).json(&request_body).send()?;

        if !response.status().is_success() {
            return Err(format!("API Error: {}", response.text()?).into());
        }

        let response_data: Txt2ImgResponse = response.json()?;

        let base64_image = response_data.images.get(0).ok_or("No images in API response")?;
        let image_bytes = general_purpose::STANDARD.decode(base64_image)?;

        let image = image::load_from_memory_with_format(&image_bytes, ImageFormat::Png)?;

        println!("AI Renderer: Image received successfully!");
        Ok(image)
    }
}