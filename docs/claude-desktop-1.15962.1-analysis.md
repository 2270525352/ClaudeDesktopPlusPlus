# Claude Desktop 1.15962.1 Notes

These notes come from a non-destructive extraction of:

`%LOCALAPPDATA%\AnthropicClaude\app-1.15962.1\resources\app.asar`

The extracted copy used for inspection was:

`%TEMP%\claude-asar-inspect-1.15962.1`

## CDP Guard

`index.pre.js` checks startup arguments before the app continues:

- `remote-debugging-port`
- `remote-debugging-pipe`

If either flag is present and `CLAUDE_CDP_AUTH` / `CLAUDE_USER_DATA_DIR` do not
pass the embedded Ed25519 public-key verification, the process exits during
preload startup.

This explains why a Codex++-style external CDP launcher cannot inject into this
Claude Desktop build by only adding `--remote-debugging-port`.

Do not attempt to forge the token. Treat this as a signed internal gate.

## Main Window Shape

The main app window is a `BrowserWindow` using:

`.vite/build/mainWindow.js`

The actual Claude web surface is a child `WebContentsView` created by the main
process. That view is tagged as `CLAUDE_AI_WEB`, uses:

`.vite/build/mainView.js`

and loads the URL returned by the app deployment mode. In normal first-party
mode this resolves to Claude's production web origin.

## Existing Internal CDP Use

Claude Desktop still uses Electron's internal `webContents.debugger` API for
preview and automation surfaces. That is separate from exposing a public remote
debugging port and does not provide an external attach route.

## Practical Injection Route

For Claude++ the realistic prototype is a controlled preload patch:

1. Unpack or patch a copied `app.asar`.
2. Patch the main Claude `WebContentsView` preload path or append a small
   renderer bootstrap to `mainView.js`.
3. Repack to a staged artifact.
4. Only after explicit user approval, back up the installed `app.asar` and swap
   in the staged artifact.
5. Provide a restore command before shipping an install patch command.

`claude-plus-launcher patch --stage-only` implements the non-destructive staging
flow. It does not replace files in the Claude Desktop install directory.
