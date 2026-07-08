# ClaudeDesktopPlusPlus

<p align="center">
  <img src="docs/images/claude-desktop-plus-plus.png" alt="ClaudeDesktopPlusPlus icon" width="160">
</p>

<p align="center">
  <a href="README.md">中文</a> | English
</p>

<p align="center">
  <img alt="Release" src="https://img.shields.io/github/v/release/2270525352/ClaudeDesktopPlusPlus">
  <img alt="Stars" src="https://img.shields.io/github/stars/2270525352/ClaudeDesktopPlusPlus">
  <img alt="License" src="https://img.shields.io/github/license/2270525352/ClaudeDesktopPlusPlus">
  <img alt="Rust" src="https://img.shields.io/badge/rust-1.85%2B-orange">
  <img alt="Tauri" src="https://img.shields.io/badge/tauri-2.x-24C8DB">
  <img alt="Windows" src="https://img.shields.io/badge/Windows-x64-0078D4">
  <img alt="macOS" src="https://img.shields.io/badge/macOS-arm64-111111">
</p>

ClaudeDesktopPlusPlus is an external launcher and management console for Claude Desktop. It helps manage Claude Desktop launch, third-party API providers, cc-switch synchronization, plugin setup, Chinese localization, conversation recovery, and system readiness checks from a dedicated Tauri desktop app.

> This project is not affiliated with Anthropic. Claude Desktop, Claude, Claude Code, Cowork, and related names belong to their respective owners. Use this tool only on machines and profiles you are allowed to manage.

## Downloads

Download the latest installer from [GitHub Releases](https://github.com/2270525352/ClaudeDesktopPlusPlus/releases):

- Windows: `ClaudeDesktopPlusPlus-v*-windows-x64-setup.exe`
- macOS Apple Silicon: `ClaudeDesktopPlusPlus-v*-macos-arm64.dmg`
- macOS Intel: not published yet

## Highlights

- Rust backend and Tauri desktop console.
- Chinese-first UI with English switching.
- cc-switch provider synchronization and stale entry cleanup.
- Anthropic-compatible and OpenAI / Codex-compatible provider management.
- Direct mode and local Gateway mode.
- Model discovery, mapping hints, and credential checks.
- One-click Claude Desktop launch with Windows MSIX / modern installer detection.
- System readiness checks for Claude installation, VMP, Hypervisor, and reboot state.
- Official plugin directory sync, search, pagination, and installation.
- One-click Chinese localization.
- One-click local conversation recovery with backup-first behavior.
- Local action feedback dialogs.

## Provider Modes

Anthropic-compatible providers usually work best with Direct mode.

OpenAI / Codex-compatible providers often expose `/v1/models`, `/v1/chat/completions`, or similar routes. To use them directly in Claude Desktop, the upstream platform must map Claude model names to real upstream models, for example:

```text
claude-opus-4-5   -> gpt-5.5
claude-sonnet-4-5 -> gpt-5.4
claude-haiku-4-5  -> gpt-5.4-mini
```

If your provider does not support this kind of direct model mapping, use Gateway mode.

## Development

```powershell
node --check ui\cyber-console\app.js
cargo +stable-x86_64-pc-windows-gnullvm check --target x86_64-pc-windows-gnullvm -q
```

Run the desktop app in development:

```powershell
cd apps/desktop
npm install
npm run dev
```

Build installers:

```powershell
cd apps/desktop
npm run bundle
```

## Links

- Releases: <https://github.com/2270525352/ClaudeDesktopPlusPlus/releases>
- Issues: <https://github.com/2270525352/ClaudeDesktopPlusPlus/issues>
- Email: <a href="mailto:2270525352@qq.com">2270525352@qq.com</a>
- Codex++: <https://github.com/BigPizzaV3/CodexPlusPlus>

## License

MIT License. See [LICENSE](LICENSE).
