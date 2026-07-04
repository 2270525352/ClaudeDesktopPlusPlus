# Claude++

Claude++ is an unofficial desktop enhancement tool for Claude Desktop. It is
built with Rust, Tauri, and a terminal-style cyberpunk control console.

The project focuses on local configuration, launch orchestration, third-party
API provider setup, Claude Desktop readiness checks, official plugin discovery,
history repair, and one-click Chinese localization.

> Claude++ is not affiliated with Anthropic. Use it only in environments where
> you are allowed to manage Claude Desktop configuration and local resources.

## Features

- Windows desktop app packaged with Tauri / NSIS.
- Chinese-first UI with English switching.
- cc-switch configuration sync and provider list cleanup.
- Direct / Gateway connection mode management for third-party APIs.
- OpenAI / Codex compatible provider model discovery and mapping hints.
- One-click Claude Desktop launch through the installed modern app package.
- System readiness page for Claude Desktop installer and Windows VMP checks.
- Official Claude plugin marketplace sync, search, pagination, and install.
- One-click Chinese localization resource patch.
- One-click local history repair with backup.
- Local sponsor / recommendation page support.

## Repository Layout

```text
apps/desktop/                 Tauri desktop shell
apps/desktop/src-tauri/       Rust backend for the desktop app
crates/claude-plus-core/      Shared launcher, install, CDP, and ASAR helpers
crates/claude-plus-launcher/  CLI prototype
ui/cyber-console/             Static control-console UI
assets/inject/                Runtime injection scripts
assets/localization/          Bundled zh-CN resources with third-party notices
docs/                         Research and implementation notes
```

## Build

Install Node.js, Rust, and the Windows build dependencies required by Tauri.
On Windows this project can build with the local `stable-x86_64-pc-windows-gnullvm`
toolchain route used by the desktop scripts.

```powershell
cd apps/desktop
npm install
npm run dev
```

Build the desktop executable:

```powershell
cd apps/desktop
npm run build
```

Build the Windows installer:

```powershell
cd apps/desktop
npm run bundle
```

The installer is generated under:

```text
apps/desktop/src-tauri/target/x86_64-pc-windows-gnullvm/release/bundle/nsis/
```

## Safety Notes

- API keys are stored in the user's local application configuration, not in the
  repository.
- Claude++ does not include Claude account credentials.
- Direct mode depends on the provider exposing Anthropic-compatible routes and,
  for OpenAI / Codex style providers, compatible model mapping.
- Gateway mode is available for compatibility when the upstream provider does
  not expose a Claude-compatible direct route.
- History repair only copies local cache/session data after creating a backup.
- Chinese localization resources include their upstream MIT license and notices
  in `assets/localization/zh-CN/`.

## License

Claude++ is released under the MIT License. See `LICENSE`.
