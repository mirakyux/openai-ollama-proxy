#![cfg_attr(windows, windows_subsystem = "windows")]

use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::{fs, sync::Arc};

#[derive(Deserialize, Clone)]
struct Config {
    base_url: String,
    api_key: String,
    model: String,
    #[serde(default = "default_port")]
    port: u16,
}

fn default_port() -> u16 { 11434 }

#[derive(Deserialize)]
struct OllamaChatRequest {
    model: Option<String>,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Option<Message>,
}

#[derive(Serialize)]
struct OllamaResponse {
    model: String,
    message: Message,
    done: bool,
}


async fn chat(
    State(config): State<Arc<Config>>,
    Json(req): Json<OllamaChatRequest>,
) -> Result<Json<OllamaResponse>, (StatusCode, String)> {
    let client = reqwest::Client::new();
    let model = req.model.unwrap_or(config.model.clone());
    
    let openai_req = OpenAIRequest {
        model: config.model.clone(),
        messages: req.messages,
        stream: false,
    };

    let resp = client
        .post(format!("{}/chat/completions", config.base_url))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&openai_req)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    let openai_resp: OpenAIResponse = resp
        .json()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    let message = openai_resp
        .choices
        .first()
        .and_then(|c| c.message.clone())
        .unwrap_or(Message { role: "assistant".into(), content: "".into() });

    Ok(Json(OllamaResponse { model, message, done: true }))
}

async fn list_models(State(config): State<Arc<Config>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "models": [{ "name": config.model.clone(), "model": config.model.clone() }]
    }))
}

async fn run_server(config: Arc<Config>) {
    let port = config.port;
    let app = Router::new()
        .route("/api/chat", post(chat))
        .route("/api/tags", get(list_models))
        .with_state(config);

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


#[cfg(windows)]
mod tray {
    use muda::{Menu, MenuItem, MenuEvent};
    use tray_icon::{TrayIconBuilder, Icon};

    const ICON_BYTES: &[u8] = include_bytes!("static/tray.ico");

    pub fn run_tray_loop(port: u16) {
        let menu = Menu::new();
        let quit_item = MenuItem::new("Quit", true, None);
        menu.append(&quit_item).unwrap();

        let icon = Icon::from_resource(1, None)
            .or_else(|_| {
                let (rgba, width, height) = parse_ico(ICON_BYTES);
                Icon::from_rgba(rgba, width, height)
            })
            .unwrap();

        let _tray = TrayIconBuilder::new()
            .with_tooltip(format!("OpenAI-Ollama Proxy - :{}", port))
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build()
            .unwrap();

        let quit_id = quit_item.id().clone();
        let menu_receiver = MenuEvent::receiver();
        
        loop {
            unsafe {
                let mut msg = std::mem::zeroed();
                if winapi::um::winuser::GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                    winapi::um::winuser::TranslateMessage(&msg);
                    winapi::um::winuser::DispatchMessageW(&msg);
                }
            }
            
            if let Ok(event) = menu_receiver.try_recv() {
                if event.id == quit_id {
                    std::process::exit(0);
                }
            }
        }
    }

    fn parse_ico(data: &[u8]) -> (Vec<u8>, u32, u32) {
        // ICO header: 6 bytes, then image entry: 16 bytes each
        // Entry format: width(1), height(1), colors(1), reserved(1), planes(2), bpp(2), size(4), offset(4)
        let width = if data[6] == 0 { 256u32 } else { data[6] as u32 };
        let height = if data[7] == 0 { 256u32 } else { data[7] as u32 };
        let offset = u32::from_le_bytes([data[18], data[19], data[20], data[21]]) as usize;
        
        // Check if PNG (starts with PNG signature)
        if data.len() > offset + 8 && &data[offset..offset+4] == &[0x89, 0x50, 0x4E, 0x47] {
            // PNG format - just return a simple colored icon
            let mut rgba = Vec::with_capacity((width * height * 4) as usize);
            for _ in 0..(width * height) {
                rgba.extend_from_slice(&[0x00, 0x80, 0xFF, 0xFF]);
            }
            return (rgba, width, height);
        }
        
        // BMP format
        let bmp_header_size = u32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]) as usize;
        let bpp = u16::from_le_bytes([data[offset+14], data[offset+15]]);
        let pixel_offset = offset + bmp_header_size;
        
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        
        if bpp == 32 {
            // BGRA format, bottom-up
            for y in 0..height {
                for x in 0..width {
                    let src = pixel_offset + ((height - 1 - y) * width * 4 + x * 4) as usize;
                    let dst = ((y * width + x) * 4) as usize;
                    if src + 3 < data.len() {
                        rgba[dst] = data[src + 2];     // R
                        rgba[dst + 1] = data[src + 1]; // G
                        rgba[dst + 2] = data[src];     // B
                        rgba[dst + 3] = data[src + 3]; // A
                    }
                }
            }
        } else {
            // Fallback: blue icon
            for i in 0..(width * height) as usize {
                rgba[i * 4] = 0x00;
                rgba[i * 4 + 1] = 0x80;
                rgba[i * 4 + 2] = 0xFF;
                rgba[i * 4 + 3] = 0xFF;
            }
        }
        
        (rgba, width, height)
    }
}

#[cfg(windows)]
fn main() {
    unsafe { winapi::um::wincon::FreeConsole(); }
    
    let config_str = fs::read_to_string("config.json").expect("Failed to read config.json");
    let config: Config = serde_json::from_str(&config_str).expect("Invalid config.json");
    let config = Arc::new(config);

    let port = config.port;
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(run_server(config));

    tray::run_tray_loop(port);
}

#[cfg(not(windows))]
#[tokio::main]
async fn main() {
    let config_str = fs::read_to_string("config.json").expect("Failed to read config.json");
    let config: Config = serde_json::from_str(&config_str).expect("Invalid config.json");
    let config = Arc::new(config);

    println!("Ollama proxy running on http://127.0.0.1:{}", config.port);
    run_server(config).await;
}
