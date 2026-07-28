# ClaudeDesktopPlusPlus

<p align="center">
  <img src="docs/images/claude-desktop-plus-plus.png" alt="ClaudeDesktopPlusPlus icon" width="150">
</p>

<p align="center">
  <a href="README.md">中文</a> | English
</p>

<p align="center">
  <img alt="Release" src="https://img.shields.io/github/v/release/2270525352/ClaudeDesktopPlusPlus">
  <img alt="Stars" src="https://img.shields.io/github/stars/2270525352/ClaudeDesktopPlusPlus">
  <img alt="License" src="https://img.shields.io/github/license/2270525352/ClaudeDesktopPlusPlus">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-1.85%2B-orange">
  <img alt="Tauri" src="https://img.shields.io/badge/Tauri-2.x-24C8DB">
  <img alt="Windows" src="https://img.shields.io/badge/Windows-x64-0078D4">
  <img alt="macOS" src="https://img.shields.io/badge/macOS-x64%20%7C%20arm64-111111">
</p>

ClaudeDesktopPlusPlus (Claude++) is a third-party desktop companion for Claude Desktop. It brings launching, API providers, `cc-switch`, plugins, localization, conversation recovery, and system diagnostics into one control console.

Move beyond limitations introduced by local Gateway forwarding with one-click direct provider configuration; when Claude account and system requirements are met, unlock the Cowork workflow.

![Claude++ overview](docs/images/screenshot-overview.png)

## Highlights

- Sync `cc-switch` and manage Anthropic or OpenAI / Codex compatible APIs.
- Switch between direct and local Gateway modes with credential tests and model discovery.
- Detect and launch Claude Desktop; check Modern Installer, VMP, and Hypervisor readiness.
- Sync, search, and install plugins from the official Claude plugin directory.
- Install, update, or safely uninstall the Built-in Broken Skills Pack with 17 Claude Code skills.
- Apply Chinese localization and recover local conversations after account or provider changes.
- Chinese and English UI, system tray support, silent command execution, and update checks.

<table>
  <tr>
    <td width="50%"><img src="docs/images/screenshot-api-config.png" alt="API configuration"></td>
    <td width="50%"><img src="docs/images/screenshot-plugins.png" alt="Plugins and built-in skills"></td>
  </tr>
  <tr>
    <td align="center">API providers and cc-switch sync</td>
    <td align="center">Plugins, localization, and built-in skills</td>
  </tr>
</table>

## Download

Get the latest build from [GitHub Releases](https://github.com/2270525352/ClaudeDesktopPlusPlus/releases):

- Windows x64: `ClaudePlus-Windows-x64-Setup.exe`
- macOS Apple Silicon: `ClaudePlus-macOS-arm64.dmg`
- macOS Intel: `ClaudePlus-macOS-x64.dmg`

After installing `Claude++`:

1. Check Claude Desktop under **System Ready**.
2. Sync or add a provider under **API Config**.
3. Test credentials, then choose Direct or Gateway mode.
4. Install the plugins and skills you need under **Plugins**.

## Notes

- Anthropic-compatible providers can usually use Direct mode.
- Direct mode for OpenAI / Codex providers requires Claude-compatible protocol handling and model mapping upstream; otherwise use Gateway.
- Cowork, Code, organization plugins, and other official features still depend on a real Claude sign-in and Anthropic policy.
- Local plugin management uses the official Claude CLI and plugin directory. It is separate from organization-managed plugin delivery.
- Conversation recovery backs up the target profile first and does not copy sign-in credentials by default.
- The Windows uninstaller removes 3P routing written by Claude++; the same repair is available from **API Config**.
- `v0.1.46` adds code signing and DMG signature verification for both macOS ARM64 and Intel x64 builds, fixing the damaged-app warning caused by unsigned downloads.
- `v0.1.47` uses concise installer names and adds automatic Claude Desktop version checks plus one-click install/update on Windows and macOS.
- `v0.1.49` adds live progress bars for Claude Desktop and Claude++ updates, fixes the hidden PowerShell redirect `exit 1`, requests administrator approval when Windows installation needs it, and surfaces detailed failures.
- `v0.1.48` fixes latest-version redirect parsing, checks for updates automatically at startup, and surfaces update-check failures in the version card.

## Development

```powershell
cd apps/desktop
npm install
npm run dev
```

Check and build:

```powershell
node --check ui\cyber-console\app.js
cargo +stable-x86_64-pc-windows-gnullvm test --target x86_64-pc-windows-gnullvm -q
cd apps/desktop
npm run bundle
```

See [development tracking](docs/development-tracking.md) for current work.

## Feedback

- [Issues](https://github.com/2270525352/ClaudeDesktopPlusPlus/issues)
- [Releases](https://github.com/2270525352/ClaudeDesktopPlusPlus/releases)
- Email: <a href="mailto:2270525352@qq.com">2270525352@qq.com</a>
- Reference project: [Codex++](https://github.com/BigPizzaV3/CodexPlusPlus)

## License

[MIT License](LICENSE)
