# OpenAI-Ollama Proxy

English | [中文](README.md)

Disguise online AI APIs (OpenAI-compatible format) as a local Ollama service.

## Why

PowerToys AI feature supports OpenAI format API but doesn't allow custom `base_url`; it also supports local Ollama service. This tool simulates the Ollama API, enabling PowerToys to use any OpenAI-compatible online AI service.

## Usage

1. Edit `config.json` to configure your AI service:

```json
{
  "base_url": "https://api.openai.com/v1",
  "api_key": "sk-your-api-key",
  "model": "gpt-4o-mini",
  "port": 11434
}
```

2. Run `openai-ollama-proxy.exe`

3. In PowerToys, select Ollama. It will automatically connect to `127.0.0.1:11434`

## Features

- Windows system tray, right-click to quit
- Linux command-line mode
- Supports any OpenAI-compatible API

## Build

```bash
cargo build --release
```

## License

MIT License
