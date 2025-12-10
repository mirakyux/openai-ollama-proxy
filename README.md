# OpenAI-Ollama Proxy

[English](README_en.md) | 中文

将在线 AI 接口（OpenAI 兼容格式）伪装成本地 Ollama 服务。

## 起因

PowerToys 的 AI 功能支持 OpenAI 格式接口，但不支持自定义 `base_url`；同时支持 Ollama 本地服务。本工具通过模拟 Ollama API，让 PowerToys 可以使用任意 OpenAI 兼容的在线 AI 服务。

## 使用方法

1. 编辑 `config.json` 配置你的 AI 服务：

```json
{
  "base_url": "https://api.openai.com/v1",
  "api_key": "sk-your-api-key",
  "model": "gpt-4o-mini",
  "port": 11434
}
```

2. 运行 `openai-ollama-proxy.exe`

3. 在 PowerToys 中选择 Ollama，并设置url `127.0.0.1:11434`

## 功能

- Windows 系统托盘运行，右键退出
- Linux 命令行模式
- 支持任意 OpenAI 兼容 API

## 构建

```bash
cargo build --release
```

## License

MIT License
