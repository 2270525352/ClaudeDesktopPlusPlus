use anyhow::{anyhow, bail, Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use tungstenite::client::{client, IntoClientRequest};
use tungstenite::{Message, WebSocket};

const CDP_HTTP_TIMEOUT: Duration = Duration::from_secs(3);
const CDP_WS_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Deserialize)]
pub struct CdpTarget {
    pub id: String,
    #[serde(rename = "type")]
    pub target_type: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(default, rename = "webSocketDebuggerUrl")]
    pub websocket_debugger_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CdpVersion {
    #[serde(default)]
    pub browser: String,
    #[serde(default)]
    pub protocol_version: String,
    #[serde(default, rename = "webSocketDebuggerUrl")]
    pub websocket_debugger_url: Option<String>,
}

pub fn is_local_port_open(port: u16) -> bool {
    let Ok(address) = format!("127.0.0.1:{port}").parse() else {
        return false;
    };
    TcpStream::connect_timeout(&address, Duration::from_millis(250)).is_ok()
}

pub fn query_version(port: u16) -> Result<CdpVersion> {
    let client = cdp_http_client()?;
    let url = format!("http://127.0.0.1:{port}/json/version");
    let response = client
        .get(url)
        .send()
        .context("failed to query CDP version")?
        .error_for_status()
        .context("CDP version query failed")?;

    response
        .json::<CdpVersion>()
        .context("failed to parse CDP version")
}

pub fn wait_for_version(port: u16, timeout: Duration) -> Result<CdpVersion> {
    let started = Instant::now();
    let mut last_error = None;

    while started.elapsed() < timeout {
        match query_version(port) {
            Ok(version) => return Ok(version),
            Err(error) => last_error = Some(error),
        }

        std::thread::sleep(Duration::from_millis(250));
    }

    match last_error {
        Some(error) => Err(error).context("timed out waiting for CDP version"),
        None => bail!("timed out waiting for CDP version"),
    }
}

pub fn list_targets(port: u16) -> Result<Vec<CdpTarget>> {
    let client = cdp_http_client()?;

    let url = format!("http://127.0.0.1:{port}/json/list");
    let response = client
        .get(url)
        .send()
        .context("failed to query CDP targets")?
        .error_for_status()
        .context("CDP target query failed")?;

    response
        .json::<Vec<CdpTarget>>()
        .context("failed to parse CDP targets")
}

pub fn wait_for_injectable_target(port: u16, timeout: Duration) -> Result<CdpTarget> {
    let started = Instant::now();
    let mut last_error = None;
    let mut last_targets = Vec::new();

    while started.elapsed() < timeout {
        match list_targets(port) {
            Ok(targets) => {
                if let Some(target) = pick_claude_target(&targets) {
                    return Ok(target);
                }
                last_targets = targets;
            }
            Err(error) => last_error = Some(error),
        }

        std::thread::sleep(Duration::from_millis(250));
    }

    if !last_targets.is_empty() {
        let summary = last_targets
            .iter()
            .map(|target| format!("{} {} {}", target.target_type, target.title, target.url))
            .collect::<Vec<_>>()
            .join(" | ");
        bail!("timed out waiting for a Claude CDP target; last targets: {summary}");
    }

    match last_error {
        Some(error) => Err(error).context("timed out waiting for a Claude CDP target"),
        None => bail!("timed out waiting for a Claude CDP target"),
    }
}

pub fn wait_for_targets(port: u16, timeout: Duration) -> Result<Vec<CdpTarget>> {
    let started = Instant::now();
    let mut last_error = None;

    while started.elapsed() < timeout {
        match list_targets(port) {
            Ok(targets) if !targets.is_empty() => return Ok(targets),
            Ok(_) => {}
            Err(error) => last_error = Some(error),
        }

        std::thread::sleep(Duration::from_millis(250));
    }

    match last_error {
        Some(error) => Err(error).context("timed out waiting for CDP targets"),
        None => bail!("timed out waiting for CDP targets"),
    }
}

pub fn pick_injectable_target(targets: &[CdpTarget]) -> Option<CdpTarget> {
    targets
        .iter()
        .find(|target| {
            target.target_type == "page"
                && target.websocket_debugger_url.is_some()
                && looks_like_claude_target(target)
        })
        .cloned()
        .or_else(|| {
            targets
                .iter()
                .find(|target| {
                    target.target_type == "page" && target.websocket_debugger_url.is_some()
                })
                .cloned()
        })
}

pub fn pick_claude_target(targets: &[CdpTarget]) -> Option<CdpTarget> {
    targets
        .iter()
        .find(|target| {
            target.target_type == "page"
                && target.websocket_debugger_url.is_some()
                && looks_like_claude_target(target)
        })
        .cloned()
}

fn looks_like_claude_target(target: &CdpTarget) -> bool {
    let haystack = format!("{} {}", target.title, target.url).to_ascii_lowercase();
    haystack.contains("claude") || haystack.contains("anthropic")
}

#[derive(Debug, Serialize)]
struct CdpCommand<'a> {
    id: u64,
    method: &'a str,
    params: Value,
}

pub fn inject_script(websocket_url: &str, script: &str) -> Result<()> {
    let mut socket = connect_cdp_socket(websocket_url)?;

    send_cdp_command(
        &mut socket,
        1,
        "Page.addScriptToEvaluateOnNewDocument",
        json!({ "source": script }),
    )?;
    send_cdp_command(
        &mut socket,
        2,
        "Runtime.evaluate",
        json!({
            "expression": script,
            "awaitPromise": false,
            "returnByValue": true,
        }),
    )?;

    let _ = socket.close(None);
    Ok(())
}

fn cdp_http_client() -> Result<Client> {
    Client::builder()
        .timeout(CDP_HTTP_TIMEOUT)
        .build()
        .context("failed to build CDP HTTP client")
}

fn connect_cdp_socket(websocket_url: &str) -> Result<WebSocket<TcpStream>> {
    let address = websocket_tcp_address(websocket_url)?;
    let stream = TcpStream::connect_timeout(&address, CDP_WS_TIMEOUT)
        .with_context(|| format!("failed to connect CDP websocket tcp stream at {address}"))?;
    stream
        .set_read_timeout(Some(CDP_WS_TIMEOUT))
        .context("failed to set CDP websocket read timeout")?;
    stream
        .set_write_timeout(Some(CDP_WS_TIMEOUT))
        .context("failed to set CDP websocket write timeout")?;

    let request = websocket_url
        .into_client_request()
        .context("failed to build CDP websocket request")?;
    let (socket, _) = client(request, stream).context("failed to open CDP websocket")?;
    Ok(socket)
}

fn websocket_tcp_address(websocket_url: &str) -> Result<SocketAddr> {
    let without_scheme = websocket_url
        .strip_prefix("ws://")
        .context("only ws:// CDP websocket URLs are supported")?;
    let host_port = without_scheme
        .split('/')
        .next()
        .filter(|value| !value.is_empty())
        .context("CDP websocket URL is missing host and port")?;

    host_port
        .to_socket_addrs()
        .context("failed to resolve CDP websocket host")?
        .find(|address| address.ip().is_loopback())
        .or_else(|| host_port.to_socket_addrs().ok()?.next())
        .context("failed to resolve CDP websocket address")
}

fn send_cdp_command(
    socket: &mut tungstenite::WebSocket<impl Read + Write>,
    id: u64,
    method: &str,
    params: Value,
) -> Result<Value> {
    let command = CdpCommand { id, method, params };
    let payload = serde_json::to_string(&command)?;
    socket
        .send(Message::Text(payload))
        .with_context(|| format!("failed to send CDP command {method}"))?;

    loop {
        let message = socket
            .read()
            .with_context(|| format!("failed to read CDP response for {method}"))?;

        let Message::Text(text) = message else {
            continue;
        };

        let value: Value = serde_json::from_str(&text).context("failed to parse CDP response")?;
        if value.get("id").and_then(Value::as_u64) != Some(id) {
            continue;
        }

        if let Some(error) = value.get("error") {
            return Err(anyhow!("CDP command {method} failed: {error}"));
        }

        return Ok(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn target(
        target_type: &str,
        title: &str,
        url: &str,
        websocket_debugger_url: Option<&str>,
    ) -> CdpTarget {
        CdpTarget {
            id: "target-id".to_string(),
            target_type: target_type.to_string(),
            title: title.to_string(),
            url: url.to_string(),
            websocket_debugger_url: websocket_debugger_url.map(str::to_string),
        }
    }

    #[test]
    fn local_port_open_detects_bound_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral listener");
        let port = listener.local_addr().expect("read listener addr").port();

        assert!(is_local_port_open(port));
    }

    #[test]
    fn local_port_open_returns_false_for_released_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral listener");
        let port = listener.local_addr().expect("read listener addr").port();
        drop(listener);

        assert!(!is_local_port_open(port));
    }

    #[test]
    fn pick_claude_target_requires_claude_page_with_websocket() {
        let targets = vec![
            target(
                "page",
                "Settings",
                "https://example.com",
                Some("ws://127.0.0.1:49321/devtools/page/other"),
            ),
            target(
                "page",
                "Claude",
                "https://claude.ai/new",
                Some("ws://127.0.0.1:49321/devtools/page/claude"),
            ),
        ];

        let picked = pick_claude_target(&targets).expect("pick Claude target");

        assert_eq!(picked.url, "https://claude.ai/new");
    }

    #[test]
    fn pick_claude_target_ignores_non_claude_page() {
        let targets = vec![target(
            "page",
            "Settings",
            "https://example.com",
            Some("ws://127.0.0.1:49321/devtools/page/other"),
        )];

        assert!(pick_claude_target(&targets).is_none());
    }

    #[test]
    fn pick_injectable_target_falls_back_to_generic_page() {
        let targets = vec![target(
            "page",
            "Settings",
            "https://example.com",
            Some("ws://127.0.0.1:49321/devtools/page/other"),
        )];

        let picked = pick_injectable_target(&targets).expect("pick fallback page");

        assert_eq!(picked.url, "https://example.com");
    }

    #[test]
    fn pick_injectable_target_rejects_pages_without_websocket() {
        let targets = vec![target("page", "Claude", "https://claude.ai/new", None)];

        assert!(pick_injectable_target(&targets).is_none());
    }
}
