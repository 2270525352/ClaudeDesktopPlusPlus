#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use claude_plus_core::asar_patch::{find_app_asar, stage_preload_patch};
use claude_plus_core::cdp;
use claude_plus_core::install::{detect_claude_install, ClaudeInstall};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Method, Response, Server, StatusCode};
use uuid::Uuid;

const INJECT_SCRIPT: &str = include_str!("../../../../assets/inject/renderer-inject.js");
const CHINESE_LOCALIZATION_SCRIPT: &str =
    include_str!("../../../../assets/inject/chinese-localization.js");
const CHINESE_LOCALIZATION_SCRIPT_ID: &str = "builtin-chinese-localization";
const CHINESE_LOCALIZATION_SCRIPT_NAME: &str = "Chinese Localization";
const LOCALIZATION_DESKTOP_ZH_CN: &str =
    include_str!("../../../../assets/localization/zh-CN/zh-CN.json");
const LOCALIZATION_FRONTEND_ZH_CN: &str =
    include_str!("../../../../assets/localization/zh-CN/ion-dist/i18n/zh-CN.json");
const LOCALIZATION_STATSIG_ZH_CN: &str =
    include_str!("../../../../assets/localization/zh-CN/ion-dist/i18n/statsig/zh-CN.json");
const DEFAULT_GATEWAY_PORT: u16 = 49331;
const DEFAULT_CDP_PORT: u16 = 49321;
const GATEWAY_BIND_HOST: &str = "127.0.0.1";
const PROVIDER_PROTOCOL_ANTHROPIC: &str = "anthropic";
const PROVIDER_PROTOCOL_OPENAI: &str = "openai";
const CLAUDE_3P_DIR_NAME: &str = "Claude-3p";
const CLAUDE_3P_CONFIG_FILE: &str = "claude_desktop_config.json";
const CLAUDE_3P_LIBRARY_DIR: &str = "configLibrary";
const CLAUDE_3P_META_FILE: &str = "_meta.json";
const CLAUDE_PLUS_CONFIG_NAME: &str = "Claude++ Provider";
const LEGACY_CLAUDE_PLUS_CONFIG_NAME: &str = "Claude++ Gateway";
const OFFICIAL_PLUGIN_MARKETPLACE_NAME: &str = "claude-plugins-official";
const OFFICIAL_PLUGIN_MARKETPLACE_REPO: &str = "anthropics/claude-plugins-official";
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

static GATEWAY_STATE: OnceLock<Mutex<Option<GatewayRuntime>>> = OnceLock::new();

#[derive(Debug, Serialize)]
struct InstallInfo {
    executable: String,
    working_dir: String,
    source: String,
    app_user_model_id: Option<String>,
    app_asar: Option<String>,
    launcher_route: String,
    live_injection_supported: bool,
}

#[derive(Debug, Serialize)]
struct PatchStatus {
    install: Option<InstallInfo>,
    stage_only: bool,
    install_write_enabled: bool,
}

#[derive(Debug, Serialize)]
struct StagedPatchInfo {
    source_asar: String,
    source_unpacked: Option<String>,
    stage_dir: String,
    original_copy: String,
    original_unpacked_copy: Option<String>,
    extract_dir: String,
    staged_asar: String,
    staged_unpacked: Option<String>,
    patched_preload: String,
    install_write_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    #[serde(default)]
    active_provider_id: Option<String>,
    #[serde(default)]
    providers: Vec<ApiProvider>,
    #[serde(default)]
    scripts: Vec<UserScript>,
    #[serde(default)]
    sandbox: SandboxConfig,
    #[serde(default)]
    gateway: GatewayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiProvider {
    id: String,
    name: String,
    app_type: String,
    source: String,
    base_url: String,
    api_key: String,
    #[serde(default)]
    protocol: String,
    #[serde(default)]
    model_mappings: Vec<ModelMapping>,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelMapping {
    claude_route: String,
    target_model: String,
    label: String,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserScript {
    id: String,
    name: String,
    enabled: bool,
    code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SandboxConfig {
    inject_provider: bool,
    inject_api_key: bool,
    relax_sandbox: bool,
    acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GatewayConfig {
    enabled: bool,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct ProviderInput {
    id: Option<String>,
    name: String,
    app_type: String,
    base_url: String,
    api_key: String,
    #[serde(default)]
    protocol: Option<String>,
    #[serde(default)]
    model_mappings: Option<Vec<ModelMapping>>,
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct SandboxInput {
    inject_provider: bool,
    inject_api_key: bool,
    relax_sandbox: bool,
    acknowledged: bool,
}

#[derive(Debug, Deserialize)]
struct GatewayInput {
    enabled: bool,
    port: u16,
}

#[derive(Debug, Deserialize)]
struct ScriptInput {
    id: Option<String>,
    name: String,
    enabled: bool,
    code: String,
}

#[derive(Debug, Serialize)]
struct PublicConfig {
    active_provider_id: Option<String>,
    providers: Vec<PublicProvider>,
    scripts: Vec<PublicScript>,
    sandbox: SandboxConfig,
    gateway: GatewayConfig,
    config_path: String,
}

#[derive(Debug, Serialize)]
struct PublicProvider {
    id: String,
    name: String,
    app_type: String,
    source: String,
    base_url: String,
    protocol: String,
    model_mappings: Vec<ModelMapping>,
    key_mask: String,
    has_key: bool,
    injectable: bool,
    enabled: bool,
    active: bool,
}

#[derive(Debug, Serialize)]
struct PublicScript {
    id: String,
    name: String,
    enabled: bool,
    code: String,
}

#[derive(Debug, Serialize)]
struct AppState {
    config: PublicConfig,
    cc_switch: CcSwitchStatus,
    install: Option<InstallInfo>,
    claude_3p: Claude3pStatus,
    system: SystemReadiness,
    history: HistoryScan,
}

#[derive(Debug, Serialize)]
struct DeveloperCapabilitiesStatus {
    config_paths: Vec<CapabilityConfigPath>,
    browser_mcp_configured: bool,
    workspace_mcp_configured: bool,
    npx_available: bool,
    workspace_path: String,
    chrome_connector_url: String,
    chrome_help_url: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct OfficialPluginsStatus {
    claude_cli_available: bool,
    claude_cli_path: Option<String>,
    marketplace_configured: bool,
    marketplace_name: String,
    marketplace_path: Option<String>,
    marketplace_last_updated: Option<String>,
    marketplace_plugin_count: usize,
    installed_plugins: Vec<String>,
    plugins: Vec<OfficialPluginEntry>,
    featured_plugins: Vec<OfficialPluginEntry>,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct OfficialPluginEntry {
    name: String,
    plugin_id: String,
    description: String,
    category: Option<String>,
    author: Option<String>,
    homepage: Option<String>,
    source: String,
    install_count: Option<u64>,
    installed: bool,
}

#[derive(Debug, Serialize)]
struct OfficialPluginActionResult {
    ok: bool,
    exit_code: Option<i32>,
    message: String,
    stdout: String,
    stderr: String,
    status: OfficialPluginsStatus,
}

#[derive(Debug, Serialize)]
struct CapabilityConfigPath {
    label: String,
    path: String,
    exists: bool,
    writable: bool,
    browser_mcp: bool,
    workspace_mcp: bool,
}

#[derive(Debug, Serialize)]
struct SystemReadiness {
    is_windows: bool,
    is_admin: bool,
    os_name: Option<String>,
    os_build: Option<String>,
    virtualization_firmware_enabled: Option<bool>,
    hypervisor_present: Option<bool>,
    hypervisor_launch_type: Option<String>,
    claude_installed: bool,
    claude_modern_installer: bool,
    claude_appx_package: Option<String>,
    virtual_machine_platform: Option<String>,
    hypervisor_platform: Option<String>,
    hyper_v: Option<String>,
    reboot_required: bool,
}

#[derive(Debug, Serialize)]
struct SystemActionResult {
    ok: bool,
    exit_code: Option<i32>,
    message: String,
    stdout: String,
    stderr: String,
    reboot_required: bool,
    downloaded_path: Option<String>,
    system: SystemReadiness,
}

#[derive(Debug, Clone, Serialize, Default)]
struct LocalizationPatchStatus {
    resources_dir: String,
    desktop_json: bool,
    frontend_json: bool,
    statsig_json: bool,
    whitelist_patched: bool,
    locale_paths: Vec<String>,
    current_locale: Option<String>,
}

#[derive(Debug, Serialize)]
struct LocalizationActionResult {
    ok: bool,
    message: String,
    stdout: String,
    stderr: String,
    config: PublicConfig,
    status: LocalizationPatchStatus,
}

#[derive(Debug, Serialize)]
struct HistoryScan {
    target_path: String,
    backup_root: String,
    profiles: Vec<HistoryProfile>,
}

#[derive(Debug, Serialize)]
struct HistoryProfile {
    name: String,
    path: String,
    exists: bool,
    is_target: bool,
    item_count: usize,
    file_count: u64,
    total_bytes: u64,
    latest_write_ms: Option<u128>,
    items: Vec<HistoryItem>,
}

#[derive(Debug, Serialize)]
struct HistoryItem {
    key: String,
    label: String,
    relative_path: String,
    default_restore: bool,
    exists: bool,
    file_count: u64,
    total_bytes: u64,
    latest_write_ms: Option<u128>,
}

#[derive(Debug, Deserialize)]
struct HistoryRepairInput {
    source_path: String,
    item_keys: Vec<String>,
}

#[derive(Debug, Serialize)]
struct HistoryRepairResult {
    ok: bool,
    source_path: String,
    target_path: String,
    backup_path: String,
    copied_files: u64,
    copied_bytes: u64,
    restored_items: Vec<String>,
    message: String,
    scan: HistoryScan,
}

#[derive(Debug, Serialize)]
struct CcSwitchStatus {
    found: bool,
    root: String,
    settings_path: Option<String>,
    database_path: Option<String>,
    provider_count: usize,
    current_provider_id: Option<String>,
    last_error: Option<String>,
}

#[derive(Debug, Serialize)]
struct SyncResult {
    imported: usize,
    updated: usize,
    removed: usize,
    skipped: usize,
    active_provider_id: Option<String>,
    config: PublicConfig,
}

#[derive(Debug, Serialize)]
struct LaunchResult {
    executable: String,
    process_id: u32,
    injected_provider_id: Option<String>,
    sandbox_relaxed: bool,
    clean_environment: bool,
    launcher_route: String,
    injection_channel: String,
    live_injection_supported: bool,
    live_injection_attempted: bool,
    gateway_url: Option<String>,
    cdp_port: Option<u16>,
    cdp_injected: bool,
    cdp_error: Option<String>,
    claude_3p: Option<Claude3pStatus>,
    verification: Option<LaunchVerification>,
}

#[derive(Debug, Serialize)]
struct GatewayStatus {
    enabled: bool,
    running: bool,
    url: String,
    port: u16,
    provider_id: Option<String>,
    provider_name: Option<String>,
    target_base_url: Option<String>,
    request_count: u64,
    forwarded_count: u64,
    last_request_path: Option<String>,
    last_request_at_ms: Option<u128>,
    last_upstream_status: Option<u16>,
    last_upstream_error: Option<String>,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalGatewayHealth {
    ok: bool,
    provider: Option<String>,
    target: Option<String>,
    requests: Option<u64>,
    forwarded: Option<u64>,
    last_upstream_status: Option<u16>,
    last_upstream_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct Claude3pStatus {
    user_data_dir: String,
    desktop_config_path: String,
    config_library_dir: String,
    meta_path: String,
    config_path: String,
    config_id: String,
    deployment_mode: Option<String>,
    applied_id: Option<String>,
    file_exists: bool,
    meta_exists: bool,
    active_config_exists: bool,
    applied: bool,
}

#[derive(Debug, Clone, Serialize)]
struct LaunchVerification {
    gateway_hit: bool,
    request_delta: u64,
    forwarded_delta: u64,
    last_request_path: Option<String>,
    last_upstream_status: Option<u16>,
    last_upstream_error: Option<String>,
    deployment_mode: Option<String>,
    claude_log_evidence: Vec<String>,
    verdict: String,
}

#[derive(Debug, Clone, Serialize)]
struct ProviderTestResult {
    provider_id: String,
    provider_name: String,
    base_url: String,
    url: String,
    protocol: String,
    model_count: usize,
    claude_desktop_compatible: bool,
    compatibility_message: String,
    ok: bool,
    status: Option<u16>,
    code: Option<String>,
    message: Option<String>,
    body_excerpt: String,
    key_mask: String,
}

#[derive(Debug, Clone, Serialize)]
struct ModelDiscoveryResult {
    protocol: String,
    model_count: usize,
    models: Vec<String>,
    model_mappings: Vec<ModelMapping>,
    message: String,
}

#[derive(Debug, Clone)]
struct GatewayRuntime {
    port: u16,
    provider_id: String,
    provider_name: String,
    target_base_url: String,
    shared: Arc<GatewayShared>,
}

#[derive(Debug, Clone)]
struct GatewayProvider {
    id: String,
    name: String,
    base_url: String,
    api_key: String,
    protocol: String,
    model_mappings: Vec<ModelMapping>,
}

#[derive(Debug, Default)]
struct GatewayCounters {
    request_count: AtomicU64,
    forwarded_count: AtomicU64,
    last_request_path: Mutex<Option<String>>,
    last_request_at_ms: Mutex<Option<u128>>,
    last_upstream_status: Mutex<Option<u16>>,
    last_upstream_error: Mutex<Option<String>>,
}

#[derive(Debug)]
struct GatewayShared {
    provider: Mutex<GatewayProvider>,
    counters: GatewayCounters,
    stop_requested: AtomicBool,
}

#[derive(Debug, Clone)]
struct GatewaySnapshot {
    request_count: u64,
    forwarded_count: u64,
    last_request_path: Option<String>,
    last_request_at_ms: Option<u128>,
    last_upstream_status: Option<u16>,
    last_upstream_error: Option<String>,
}

struct GatewayForwardResult {
    status: u16,
    upstream_error: Option<String>,
    response: Response<std::io::Cursor<Vec<u8>>>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            active_provider_id: None,
            providers: Vec::new(),
            scripts: Vec::new(),
            sandbox: SandboxConfig::default(),
            gateway: GatewayConfig::default(),
        }
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            inject_provider: true,
            inject_api_key: false,
            relax_sandbox: false,
            acknowledged: false,
        }
    }
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: DEFAULT_GATEWAY_PORT,
        }
    }
}

#[tauri::command]
fn detect_install() -> Result<Option<InstallInfo>, String> {
    Ok(detect_claude_install().map(install_info))
}

#[tauri::command]
fn patch_status() -> PatchStatus {
    PatchStatus {
        install: detect_claude_install().map(install_info),
        stage_only: true,
        install_write_enabled: false,
    }
}

#[tauri::command]
fn patch_stage_only() -> Result<StagedPatchInfo, String> {
    let install = detect_claude_install().ok_or("Claude Desktop install was not found")?;
    let config = read_config().map_err(|error| error.to_string())?;
    let staged = stage_preload_patch(&install, &build_inject_script(&config))
        .map_err(|error| error.to_string())?;

    Ok(StagedPatchInfo {
        source_asar: path_string(&staged.source_asar),
        source_unpacked: staged
            .source_unpacked
            .as_ref()
            .map(|path| path_string(path)),
        stage_dir: path_string(&staged.stage_dir),
        original_copy: path_string(&staged.original_copy),
        original_unpacked_copy: staged
            .original_unpacked_copy
            .as_ref()
            .map(|path| path_string(path)),
        extract_dir: path_string(&staged.extract_dir),
        staged_asar: path_string(&staged.staged_asar),
        staged_unpacked: staged
            .staged_unpacked
            .as_ref()
            .map(|path| path_string(path)),
        patched_preload: path_string(&staged.patched_preload),
        install_write_enabled: false,
    })
}

#[tauri::command]
fn read_app_state() -> Result<AppState, String> {
    let config = read_config().map_err(|error| error.to_string())?;
    let install = detect_claude_install();
    Ok(AppState {
        config: public_config(&config),
        cc_switch: cc_switch_status(),
        install: install.clone().map(install_info),
        claude_3p: claude_3p_status(),
        system: system_readiness_placeholder(install.as_ref()),
        history: history_scan_placeholder(),
    })
}

#[tauri::command]
fn gateway_status() -> Result<GatewayStatus, String> {
    let config = read_config().map_err(|error| error.to_string())?;
    Ok(build_gateway_status(&config, None))
}

#[tauri::command]
async fn system_readiness_status() -> Result<SystemReadiness, String> {
    tauri::async_runtime::spawn_blocking(system_readiness)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn history_scan_status() -> Result<HistoryScan, String> {
    tauri::async_runtime::spawn_blocking(history_scan)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn developer_capabilities_status() -> Result<DeveloperCapabilitiesStatus, String> {
    tauri::async_runtime::spawn_blocking(developer_capabilities_status_sync)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn official_plugins_status() -> Result<OfficialPluginsStatus, String> {
    tauri::async_runtime::spawn_blocking(official_plugins_status_sync)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn sync_official_plugin_marketplace() -> Result<OfficialPluginActionResult, String> {
    tauri::async_runtime::spawn_blocking(sync_official_plugin_marketplace_sync)
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn install_official_plugin(plugin: String) -> Result<OfficialPluginActionResult, String> {
    tauri::async_runtime::spawn_blocking(move || install_official_plugin_sync(plugin))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn enable_developer_capabilities() -> Result<DeveloperCapabilitiesStatus, String> {
    tauri::async_runtime::spawn_blocking(enable_developer_capabilities_sync)
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn repair_history(input: HistoryRepairInput) -> Result<HistoryRepairResult, String> {
    tauri::async_runtime::spawn_blocking(move || repair_history_from_profile(input))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn enable_virtual_machine_platform() -> Result<SystemActionResult, String> {
    tauri::async_runtime::spawn_blocking(enable_virtual_machine_platform_sync)
        .await
        .map_err(|error| error.to_string())?
}

fn enable_virtual_machine_platform_sync() -> Result<SystemActionResult, String> {
    let script = r#"
$ErrorActionPreference = 'Continue'
$lines = New-Object System.Collections.Generic.List[string]
$exitCode = 0

function Add-Line([string]$line) {
  $script:lines.Add($line) | Out-Null
}

function Invoke-NativeStep([string]$label, [scriptblock]$block) {
  Add-Line "== $label =="
  $output = & $block 2>&1
  if ($output) {
    $output | ForEach-Object { Add-Line ([string]$_) }
  }
  $code = if ($null -eq $LASTEXITCODE) { 0 } else { $LASTEXITCODE }
  Add-Line "exit=$code"
  if (($code -ne 0) -and ($code -ne 3010) -and ($code -ne 1641)) {
    $script:exitCode = $code
  }
}

Invoke-NativeStep "Set hypervisor launch type" { bcdedit /set hypervisorlaunchtype auto }
Invoke-NativeStep "Enable Windows Hypervisor Platform" { dism /English /online /Enable-Feature /FeatureName:HypervisorPlatform /All /NoRestart }
Invoke-NativeStep "Enable Virtual Machine Platform" { dism /English /online /Enable-Feature /FeatureName:VirtualMachinePlatform /All /NoRestart }

Add-Line "== Current feature state =="
foreach ($name in @("HypervisorPlatform", "VirtualMachinePlatform", "Microsoft-Hyper-V-All")) {
  try {
    $state = (Get-WindowsOptionalFeature -Online -FeatureName $name -ErrorAction Stop).State
    Add-Line "$name=$state"
  } catch {
    Add-Line "$name=$($_.Exception.Message)"
  }
}

$logPath = Join-Path $env:WINDIR "Logs\DISM\dism.log"
if (Test-Path $logPath) {
  Add-Line "== Recent DISM diagnostics =="
  Get-Content $logPath -Tail 80 |
    Select-String -Pattern "Parent features|requires a reboot|Reboot required|Error|Failed" |
    ForEach-Object { Add-Line $_.Line }
}

Write-Output ($lines -join "`n")
exit $exitCode
"#;
    let output = run_powershell_script(script)?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code();
    let system = system_readiness();
    let reboot_required = system.reboot_required;
    let features_ready = feature_state_enabled(&system.virtual_machine_platform)
        && feature_state_enabled(&system.hypervisor_platform);
    let staged_for_reboot = reboot_required
        && system.is_admin
        && (stdout.contains("requires a reboot")
            || stdout.contains("Reboot required")
            || stdout.contains("需要重新启动")
            || stdout.contains("已暂存")
            || stdout.contains("staged"));
    let ok = output.status.success() || features_ready || staged_for_reboot;
    Ok(SystemActionResult {
        ok,
        exit_code,
        message: vmp_enablement_message(&system, output.status.success(), &stdout, &stderr),
        stdout,
        stderr,
        reboot_required,
        downloaded_path: None,
        system,
    })
}

#[tauri::command]
async fn install_claude_modern() -> Result<SystemActionResult, String> {
    tauri::async_runtime::spawn_blocking(install_claude_modern_sync)
        .await
        .map_err(|error| error.to_string())?
}

fn install_claude_modern_sync() -> Result<SystemActionResult, String> {
    if !cfg!(target_os = "windows") {
        return Err(
            "Claude modern installer automation is only implemented on Windows".to_string(),
        );
    }

    if let Some(package) = claude_appx_package_name() {
        return Ok(SystemActionResult {
            ok: true,
            exit_code: Some(0),
            message: "Claude Desktop modern package is already installed.".to_string(),
            stdout: format!("Installed Appx package: {package}"),
            stderr: String::new(),
            reboot_required: reboot_required_by_windows(),
            downloaded_path: None,
            system: system_readiness(),
        });
    }

    let download_dir = config_path()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("downloads");
    fs::create_dir_all(&download_dir).map_err(|error| error.to_string())?;
    let package_path = download_dir.join("Claude-modern.msix");
    let package = path_string(&package_path).replace('\'', "''");
    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$redirect = 'https://claude.ai/api/desktop/win32/x64/msix/latest/redirect'
$package = '{package}'
$response = Invoke-WebRequest -Uri $redirect -MaximumRedirection 0 -ErrorAction SilentlyContinue
$location = $response.Headers.Location
if (-not $location) {{
  $response = Invoke-WebRequest -Uri $redirect -MaximumRedirection 5 -UseBasicParsing
  if ($response.BaseResponse.ResponseUri) {{ $location = $response.BaseResponse.ResponseUri.AbsoluteUri }}
}}
if (-not $location) {{ throw 'Could not resolve Claude MSIX download URL' }}
Invoke-WebRequest -Uri $location -OutFile $package -UseBasicParsing
Add-AppxPackage -Path $package
Write-Output "Installed Claude modern package from $location"
"#
    );
    let output = run_powershell_script(&script)?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code();
    let ok = output.status.success();
    Ok(SystemActionResult {
        ok,
        exit_code,
        message: if ok {
            "Claude Desktop modern installer is installed.".to_string()
        } else {
            "Claude Desktop modern installer installation failed.".to_string()
        },
        stdout,
        stderr,
        reboot_required: reboot_required_by_windows(),
        downloaded_path: Some(path_string(&package_path)),
        system: system_readiness(),
    })
}

#[tauri::command]
fn relaunch_as_admin() -> Result<(), String> {
    if !cfg!(target_os = "windows") {
        return Err("Administrator relaunch is only implemented on Windows".to_string());
    }
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    let exe = path_string(&exe).replace('\'', "''");
    let script = format!("Start-Process -FilePath '{exe}' -Verb RunAs");
    let output = run_powershell_script(&script)?;
    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("{stdout}\n{stderr}").trim().to_string())
    }
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    let url = url.trim();
    if url.is_empty() || url.chars().any(char::is_control) {
        return Err("URL is empty or malformed".to_string());
    }
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("Only http and https URLs can be opened".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let script = format!("Start-Process -FilePath {}", powershell_single_quoted(url));
        let output = run_powershell_script(&script)?;
        if output.status.success() {
            return Ok(());
        }
        return Err(format_command_failure(&output));
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("open")
            .arg(url)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| error.to_string())?;
        if output.status.success() {
            return Ok(());
        }
        return Err(format_command_failure(&output));
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let output = Command::new("xdg-open")
            .arg(url)
            .stdin(Stdio::null())
            .output()
            .map_err(|error| error.to_string())?;
        if output.status.success() {
            return Ok(());
        }
        Err(format_command_failure(&output))
    }
}

#[tauri::command]
fn save_gateway_options(options: GatewayInput) -> Result<PublicConfig, String> {
    if options.port == 0 {
        return Err("Gateway port is required".to_string());
    }

    let mut config = read_config().map_err(|error| error.to_string())?;
    config.gateway = GatewayConfig {
        enabled: options.enabled,
        port: options.port,
    };
    write_config(&config).map_err(|error| error.to_string())?;

    if !config.gateway.enabled {
        stop_gateway_runtime();
    } else {
        refresh_gateway_runtime_after_config_change(&config);
    }

    Ok(public_config(&config))
}

#[tauri::command]
fn start_gateway() -> Result<GatewayStatus, String> {
    let config = read_config().map_err(|error| error.to_string())?;
    ensure_gateway_runtime(&config)?;
    Ok(build_gateway_status(&config, None))
}

#[tauri::command]
fn stop_gateway() -> Result<GatewayStatus, String> {
    let config = read_config().map_err(|error| error.to_string())?;
    stop_gateway_runtime();
    Ok(build_gateway_status(&config, None))
}

#[tauri::command]
async fn sync_cc_switch_config() -> Result<SyncResult, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut config = read_config().map_err(|error| error.to_string())?;
        let (imported, updated, removed, skipped) = apply_cc_switch_sync(&mut config)?;
        write_config(&config).map_err(|error| error.to_string())?;
        refresh_gateway_runtime_after_config_change(&config);
        Ok(SyncResult {
            imported,
            updated,
            removed,
            skipped,
            active_provider_id: config.active_provider_id.clone(),
            config: public_config(&config),
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
fn save_api_provider(provider: ProviderInput) -> Result<PublicConfig, String> {
    let mut config = read_config().map_err(|error| error.to_string())?;
    let id = provider
        .id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("manual-{}", unix_millis()));
    let existing = config.providers.iter().find(|item| item.id == id);
    let api_key = if provider.api_key.trim().is_empty() {
        existing
            .map(|item| item.api_key.clone())
            .unwrap_or_default()
    } else {
        provider.api_key.trim().to_string()
    };
    let source = existing
        .map(|item| item.source.clone())
        .unwrap_or_else(|| "manual".to_string());
    let api_provider = ApiProvider {
        id: id.clone(),
        name: provider.name.trim().to_string(),
        app_type: normalize_app_type(&provider.app_type),
        source,
        base_url: provider.base_url.trim().to_string(),
        api_key,
        protocol: provider
            .protocol
            .as_deref()
            .map(normalize_provider_protocol)
            .unwrap_or_else(|| infer_provider_protocol(&provider.name, &provider.base_url)),
        model_mappings: provider
            .model_mappings
            .unwrap_or_else(|| {
                existing
                    .map(|item| item.model_mappings.clone())
                    .unwrap_or_default()
            })
            .into_iter()
            .filter_map(normalize_model_mapping)
            .collect(),
        enabled: provider.enabled,
    };

    if api_provider.name.is_empty() {
        return Err("Provider name is required".to_string());
    }
    if api_provider.base_url.is_empty() {
        return Err("Base URL is required".to_string());
    }

    match config.providers.iter_mut().find(|item| item.id == id) {
        Some(existing) => *existing = api_provider,
        None => config.providers.push(api_provider),
    }

    config.active_provider_id = Some(id);
    write_config(&config).map_err(|error| error.to_string())?;
    refresh_gateway_runtime_after_config_change(&config);
    Ok(public_config(&config))
}

#[tauri::command]
fn delete_api_provider(id: String) -> Result<PublicConfig, String> {
    let mut config = read_config().map_err(|error| error.to_string())?;
    let before = config.providers.len();
    config.providers.retain(|provider| provider.id != id);
    if config.providers.len() == before {
        return Err("Provider was not found".to_string());
    }

    if config.active_provider_id.as_deref() == Some(id.as_str())
        || config
            .active_provider_id
            .as_ref()
            .is_some_and(|active_id| !config.providers.iter().any(|provider| &provider.id == active_id))
    {
        config.active_provider_id = config
            .providers
            .iter()
            .find(|provider| provider.enabled)
            .map(|provider| provider.id.clone());
    }

    write_config(&config).map_err(|error| error.to_string())?;
    refresh_gateway_runtime_after_config_change(&config);
    Ok(public_config(&config))
}

#[tauri::command]
fn set_active_provider(id: String) -> Result<PublicConfig, String> {
    let mut config = read_config().map_err(|error| error.to_string())?;
    if !config.providers.iter().any(|provider| provider.id == id) {
        return Err("Provider was not found".to_string());
    }
    config.active_provider_id = Some(id);
    write_config(&config).map_err(|error| error.to_string())?;
    Ok(public_config(&config))
}

#[tauri::command]
fn discover_provider_models(provider: ProviderInput) -> Result<ModelDiscoveryResult, String> {
    let config = read_config().map_err(|error| error.to_string())?;
    let existing = provider
        .id
        .as_ref()
        .and_then(|id| config.providers.iter().find(|item| item.id == *id));
    let api_key = if provider.api_key.trim().is_empty() {
        existing
            .map(|item| item.api_key.clone())
            .unwrap_or_default()
    } else {
        provider.api_key.trim().to_string()
    };
    let protocol = provider
        .protocol
        .as_deref()
        .map(normalize_provider_protocol)
        .unwrap_or_else(|| infer_provider_protocol(&provider.name, &provider.base_url));
    let base_url = provider.base_url.trim();
    if base_url.is_empty() {
        return Err("Base URL is required".to_string());
    }
    let mut models = discover_provider_model_ids(base_url, &api_key)?;
    models = prioritize_model_ids(models);
    let model_mappings = if protocol == PROVIDER_PROTOCOL_OPENAI {
        openai_route_models_from_ids(models.clone())
            .into_iter()
            .map(|route| ModelMapping {
                claude_route: route.claude_route,
                target_model: route.target_model,
                label: route.label,
                enabled: true,
            })
            .collect()
    } else {
        Vec::new()
    };
    let message = if protocol == PROVIDER_PROTOCOL_OPENAI {
        format!(
            "Discovered {} OpenAI/Codex model(s); generated Claude route mappings.",
            models.len()
        )
    } else {
        format!(
            "Discovered {} Anthropic-compatible model(s); direct model discovery can be used.",
            models.len()
        )
    };
    Ok(ModelDiscoveryResult {
        protocol,
        model_count: models.len(),
        models,
        model_mappings,
        message,
    })
}

#[tauri::command]
fn test_active_provider() -> Result<ProviderTestResult, String> {
    let mut config = read_config().map_err(|error| error.to_string())?;
    let _ = apply_cc_switch_sync(&mut config);
    let provider = active_provider(&config).ok_or("No enabled active provider was selected")?;
    let mut provider = provider.clone();
    if provider.api_key.trim().is_empty() {
        provider.api_key = provider_key_with_fallback(&config, &provider);
    }
    write_config(&config).map_err(|error| error.to_string())?;
    test_api_provider(&provider)
}

#[tauri::command]
fn test_provider(id: String) -> Result<ProviderTestResult, String> {
    let mut config = read_config().map_err(|error| error.to_string())?;
    let _ = apply_cc_switch_sync(&mut config);
    let provider = config
        .providers
        .iter()
        .find(|provider| provider.id == id)
        .ok_or("Provider was not found")?;
    if !provider.enabled {
        return Err("Provider is disabled".to_string());
    }
    let mut provider = provider.clone();
    if provider.api_key.trim().is_empty() {
        provider.api_key = provider_key_with_fallback(&config, &provider);
    }
    write_config(&config).map_err(|error| error.to_string())?;
    test_api_provider(&provider)
}

#[tauri::command]
fn save_sandbox_options(options: SandboxInput) -> Result<PublicConfig, String> {
    if options.relax_sandbox && !options.acknowledged {
        return Err("Relaxed sandbox mode requires explicit acknowledgement".to_string());
    }

    let mut config = read_config().map_err(|error| error.to_string())?;
    config.sandbox = SandboxConfig {
        inject_provider: options.inject_provider,
        inject_api_key: options.inject_api_key,
        relax_sandbox: options.relax_sandbox,
        acknowledged: options.acknowledged,
    };
    write_config(&config).map_err(|error| error.to_string())?;
    Ok(public_config(&config))
}

#[tauri::command]
fn save_user_script(script: ScriptInput) -> Result<PublicConfig, String> {
    let mut config = read_config().map_err(|error| error.to_string())?;
    let id = script
        .id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("script-{}", unix_millis()));
    let user_script = UserScript {
        id: id.clone(),
        name: script.name.trim().to_string(),
        enabled: script.enabled,
        code: script.code,
    };

    if user_script.name.is_empty() {
        return Err("Script name is required".to_string());
    }

    match config.scripts.iter_mut().find(|item| item.id == id) {
        Some(existing) => *existing = user_script,
        None => config.scripts.push(user_script),
    }

    write_config(&config).map_err(|error| error.to_string())?;
    Ok(public_config(&config))
}

#[tauri::command]
async fn enable_chinese_localization() -> Result<LocalizationActionResult, String> {
    tauri::async_runtime::spawn_blocking(|| set_chinese_localization_sync(true))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn disable_chinese_localization() -> Result<LocalizationActionResult, String> {
    tauri::async_runtime::spawn_blocking(|| set_chinese_localization_sync(false))
        .await
        .map_err(|error| error.to_string())?
}

fn set_chinese_localization_sync(enabled: bool) -> Result<LocalizationActionResult, String> {
    let action = if enabled {
        install_chinese_localization_patch()
    } else {
        uninstall_chinese_localization_patch()
    };

    let (ok, message, stdout, stderr) = match action {
        Ok((message, stdout, stderr)) => (true, message, stdout, stderr),
        Err(error) => (false, error.clone(), String::new(), error),
    };

    let mut config = read_config().map_err(|error| error.to_string())?;
    upsert_builtin_script(
        &mut config,
        CHINESE_LOCALIZATION_SCRIPT_ID,
        CHINESE_LOCALIZATION_SCRIPT_NAME,
        CHINESE_LOCALIZATION_SCRIPT,
        enabled && ok,
    );
    write_config(&config).map_err(|error| error.to_string())?;

    Ok(LocalizationActionResult {
        ok,
        message,
        stdout,
        stderr,
        config: public_config(&config),
        status: localization_patch_status(),
    })
}

struct LocalizationStage {
    desktop: PathBuf,
    frontend: PathBuf,
    statsig: PathBuf,
    script: PathBuf,
    log: PathBuf,
    done: PathBuf,
    failed: PathBuf,
}

fn install_chinese_localization_patch() -> Result<(String, String, String), String> {
    let install = detect_claude_install().ok_or("Claude Desktop install was not found")?;
    let resources_dir = claude_resources_dir(&install);
    if !resources_dir.is_dir() {
        return Err(format!(
            "Claude Desktop resources directory was not found: {}",
            path_string(&resources_dir)
        ));
    }

    let stage = write_localization_stage_files()?;
    let (stdout, stderr) =
        apply_chinese_localization_resources(&install, &resources_dir, &stage, true)?;
    let locale_paths = set_claude_locale("zh-CN")?;
    let status = localization_patch_status();
    let missing = [
        (!status.desktop_json).then_some("resources/zh-CN.json"),
        (!status.frontend_json).then_some("resources/ion-dist/i18n/zh-CN.json"),
        (!status.statsig_json).then_some("resources/ion-dist/i18n/statsig/zh-CN.json"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    if missing.is_empty() {
        Ok((
            format!(
                "Chinese localization resources installed. Locale written to {} config file(s). Restart Claude Desktop to reload zh-CN.",
                locale_paths.len()
            ),
            stdout,
            stderr,
        ))
    } else {
        Err(format!(
            "Localization task ran, but required files are still missing: {}",
            missing.join(", ")
        ))
    }
}

fn uninstall_chinese_localization_patch() -> Result<(String, String, String), String> {
    let install = detect_claude_install().ok_or("Claude Desktop install was not found")?;
    let resources_dir = claude_resources_dir(&install);
    if !resources_dir.is_dir() {
        return Err(format!(
            "Claude Desktop resources directory was not found: {}",
            path_string(&resources_dir)
        ));
    }

    let stage = write_localization_stage_files()?;
    let (stdout, stderr) =
        apply_chinese_localization_resources(&install, &resources_dir, &stage, false)?;
    let locale_paths = set_claude_locale("en-US")?;
    Ok((
        format!(
            "Chinese localization resources removed. Locale restored to en-US in {} config file(s).",
            locale_paths.len()
        ),
        stdout,
        stderr,
    ))
}

fn write_localization_stage_files() -> Result<LocalizationStage, String> {
    let root = config_path()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("localization")
        .join("zh-CN");
    let desktop = root.join("zh-CN.json");
    let frontend = root.join("ion-dist").join("i18n").join("zh-CN.json");
    let statsig = root
        .join("ion-dist")
        .join("i18n")
        .join("statsig")
        .join("zh-CN.json");
    fs::create_dir_all(desktop.parent().unwrap_or(&root)).map_err(|error| error.to_string())?;
    fs::create_dir_all(frontend.parent().unwrap_or(&root)).map_err(|error| error.to_string())?;
    fs::create_dir_all(statsig.parent().unwrap_or(&root)).map_err(|error| error.to_string())?;
    fs::write(&desktop, LOCALIZATION_DESKTOP_ZH_CN).map_err(|error| error.to_string())?;
    fs::write(&frontend, LOCALIZATION_FRONTEND_ZH_CN).map_err(|error| error.to_string())?;
    fs::write(&statsig, LOCALIZATION_STATSIG_ZH_CN).map_err(|error| error.to_string())?;

    Ok(LocalizationStage {
        desktop,
        frontend,
        statsig,
        script: root.join("apply-localization.ps1"),
        log: root.join("apply-localization.log"),
        done: root.join("apply-localization.done"),
        failed: root.join("apply-localization.failed"),
    })
}

fn claude_resources_dir(install: &ClaudeInstall) -> PathBuf {
    if cfg!(target_os = "macos") {
        if install
            .working_dir
            .file_name()
            .is_some_and(|name| name == "MacOS")
        {
            if let Some(contents_dir) = install.working_dir.parent() {
                return contents_dir.join("Resources");
            }
        }
    }
    install.working_dir.join("resources")
}

fn apply_chinese_localization_resources(
    install: &ClaudeInstall,
    resources_dir: &Path,
    stage: &LocalizationStage,
    install_patch: bool,
) -> Result<(String, String), String> {
    #[cfg(target_os = "windows")]
    {
        return run_windows_localization_task(install, resources_dir, stage, install_patch);
    }

    #[cfg(not(target_os = "windows"))]
    {
        apply_localization_resources_direct(resources_dir, stage, install_patch)
    }
}

#[cfg(target_os = "windows")]
fn run_windows_localization_task(
    install: &ClaudeInstall,
    resources_dir: &Path,
    stage: &LocalizationStage,
    install_patch: bool,
) -> Result<(String, String), String> {
    if !is_running_as_admin() {
        return Err("Administrator permission is required to patch the MSIX WindowsApps resource directory.".to_string());
    }

    let script = if install_patch {
        windows_install_localization_script(install, resources_dir, stage)
    } else {
        windows_uninstall_localization_script(install, resources_dir, stage)
    };
    fs::write(&stage.script, script).map_err(|error| error.to_string())?;
    let _ = fs::remove_file(&stage.done);
    let _ = fs::remove_file(&stage.failed);
    let _ = fs::remove_file(&stage.log);

    let task_name = format!("ClaudePlusLocalization-{}", unix_millis());
    let orchestrator = format!(
        r#"
$ErrorActionPreference = 'Continue'
$TaskName = {task_name}
$ScriptPath = {script_path}
$DonePath = {done_path}
$FailedPath = {failed_path}
$LogPath = {log_path}
$TaskAction = 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "' + $ScriptPath + '"'
& schtasks.exe /Create /TN $TaskName /TR $TaskAction /SC ONCE /ST 23:59 /RU SYSTEM /RL HIGHEST /F
$createCode = $LASTEXITCODE
if ($createCode -ne 0) {{ exit $createCode }}
& schtasks.exe /Run /TN $TaskName
$runCode = $LASTEXITCODE
if ($runCode -ne 0) {{
  & schtasks.exe /Delete /TN $TaskName /F | Out-Null
  exit $runCode
}}
$deadline = (Get-Date).AddSeconds(90)
while ((Get-Date) -lt $deadline) {{
  if ((Test-Path -LiteralPath $DonePath) -or (Test-Path -LiteralPath $FailedPath)) {{ break }}
  Start-Sleep -Milliseconds 500
}}
& schtasks.exe /Delete /TN $TaskName /F | Out-Null
if (Test-Path -LiteralPath $LogPath) {{
  Get-Content -LiteralPath $LogPath -Raw
}}
if (Test-Path -LiteralPath $FailedPath) {{ exit 1 }}
if (-not (Test-Path -LiteralPath $DonePath)) {{
  Write-Error 'Localization task timed out before writing completion marker.'
  exit 1
}}
"#,
        task_name = powershell_single_quoted(&task_name),
        script_path = powershell_single_quoted(&path_string(&stage.script)),
        done_path = powershell_single_quoted(&path_string(&stage.done)),
        failed_path = powershell_single_quoted(&path_string(&stage.failed)),
        log_path = powershell_single_quoted(&path_string(&stage.log)),
    );

    let output = run_powershell_script(&orchestrator)?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        Ok((stdout, stderr))
    } else {
        Err(format!(
            "Localization resource task failed. {}",
            format_command_failure(&output)
        ))
    }
}

#[cfg(target_os = "windows")]
fn windows_install_localization_script(
    install: &ClaudeInstall,
    resources_dir: &Path,
    stage: &LocalizationStage,
) -> String {
    let app_dir = path_string(&install.working_dir);
    format!(
        r#"
$ErrorActionPreference = 'Stop'
$ResourcesDir = {resources_dir}
$AppDir = {app_dir}
$DesktopSource = {desktop}
$FrontendSource = {frontend}
$StatsigSource = {statsig}
$LogPath = {log}
$DonePath = {done}
$FailedPath = {failed}
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
function Log([string]$Message) {{ Add-Content -LiteralPath $LogPath -Value $Message -Encoding UTF8 }}
Remove-Item -LiteralPath $DonePath -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $FailedPath -ErrorAction SilentlyContinue
try {{
  Log 'install zh-CN localization start'
  if (-not (Test-Path -LiteralPath $ResourcesDir)) {{ throw "resources directory not found: $ResourcesDir" }}
  Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
    Where-Object {{ $_.Name -ieq 'Claude.exe' -and $_.ExecutablePath -and $_.ExecutablePath.StartsWith($AppDir, [System.StringComparison]::OrdinalIgnoreCase) }} |
    ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }}
  New-Item -ItemType Directory -Force -Path (Join-Path $ResourcesDir 'ion-dist\i18n') | Out-Null
  New-Item -ItemType Directory -Force -Path (Join-Path $ResourcesDir 'ion-dist\i18n\statsig') | Out-Null
  [System.IO.File]::Copy($DesktopSource, (Join-Path $ResourcesDir 'zh-CN.json'), $true)
  [System.IO.File]::Copy($FrontendSource, (Join-Path $ResourcesDir 'ion-dist\i18n\zh-CN.json'), $true)
  [System.IO.File]::Copy($StatsigSource, (Join-Path $ResourcesDir 'ion-dist\i18n\statsig\zh-CN.json'), $true)
  $BaseList = '["en-US","de-DE","fr-FR","ko-KR","ja-JP","es-419","es-ES","it-IT","hi-IN","pt-BR","id-ID"]'
  $WithZh = '["en-US","de-DE","fr-FR","ko-KR","ja-JP","es-419","es-ES","it-IT","hi-IN","pt-BR","id-ID","zh-CN"]'
  $patched = 0
  $roots = @((Join-Path $ResourcesDir 'ion-dist\assets\v1'), (Join-Path $ResourcesDir 'ion-dist\assets'))
  foreach ($root in $roots) {{
    if (-not (Test-Path -LiteralPath $root)) {{ continue }}
    Get-ChildItem -LiteralPath $root -Recurse -File -Filter '*.js' -ErrorAction SilentlyContinue | ForEach-Object {{
      $text = [System.IO.File]::ReadAllText($_.FullName, [System.Text.Encoding]::UTF8)
      if ($text.Contains('"zh-CN"')) {{ return }}
      $next = $text.Replace($BaseList, $WithZh)
      if ($next -ne $text) {{
        [System.IO.File]::WriteAllText($_.FullName, $next, $Utf8NoBom)
        $script:patched += 1
        Log ("patched whitelist: " + $_.FullName)
      }}
    }}
  }}
  Log ("whitelist patched count=" + $patched)
  Set-Content -LiteralPath $DonePath -Value 'ok' -Encoding ASCII
  exit 0
}} catch {{
  Log ("ERROR: " + $_.Exception.Message)
  Set-Content -LiteralPath $FailedPath -Value $_.Exception.Message -Encoding UTF8
  exit 1
}}
"#,
        resources_dir = powershell_single_quoted(&path_string(resources_dir)),
        app_dir = powershell_single_quoted(&app_dir),
        desktop = powershell_single_quoted(&path_string(&stage.desktop)),
        frontend = powershell_single_quoted(&path_string(&stage.frontend)),
        statsig = powershell_single_quoted(&path_string(&stage.statsig)),
        log = powershell_single_quoted(&path_string(&stage.log)),
        done = powershell_single_quoted(&path_string(&stage.done)),
        failed = powershell_single_quoted(&path_string(&stage.failed)),
    )
}

#[cfg(target_os = "windows")]
fn windows_uninstall_localization_script(
    install: &ClaudeInstall,
    resources_dir: &Path,
    stage: &LocalizationStage,
) -> String {
    let app_dir = path_string(&install.working_dir);
    format!(
        r#"
$ErrorActionPreference = 'Stop'
$ResourcesDir = {resources_dir}
$AppDir = {app_dir}
$LogPath = {log}
$DonePath = {done}
$FailedPath = {failed}
$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
function Log([string]$Message) {{ Add-Content -LiteralPath $LogPath -Value $Message -Encoding UTF8 }}
Remove-Item -LiteralPath $DonePath -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $FailedPath -ErrorAction SilentlyContinue
try {{
  Log 'uninstall zh-CN localization start'
  if (-not (Test-Path -LiteralPath $ResourcesDir)) {{ throw "resources directory not found: $ResourcesDir" }}
  Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
    Where-Object {{ $_.Name -ieq 'Claude.exe' -and $_.ExecutablePath -and $_.ExecutablePath.StartsWith($AppDir, [System.StringComparison]::OrdinalIgnoreCase) }} |
    ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }}
  @(
    (Join-Path $ResourcesDir 'zh-CN.json'),
    (Join-Path $ResourcesDir 'ion-dist\i18n\zh-CN.json'),
    (Join-Path $ResourcesDir 'ion-dist\i18n\statsig\zh-CN.json')
  ) | ForEach-Object {{ Remove-Item -LiteralPath $_ -Force -ErrorAction SilentlyContinue }}
  $roots = @((Join-Path $ResourcesDir 'ion-dist\assets\v1'), (Join-Path $ResourcesDir 'ion-dist\assets'))
  $patched = 0
  foreach ($root in $roots) {{
    if (-not (Test-Path -LiteralPath $root)) {{ continue }}
    Get-ChildItem -LiteralPath $root -Recurse -File -Filter '*.js' -ErrorAction SilentlyContinue | ForEach-Object {{
      $text = [System.IO.File]::ReadAllText($_.FullName, [System.Text.Encoding]::UTF8)
      $next = $text.Replace(',"zh-CN"', '').Replace('"zh-CN",', '')
      if ($next -ne $text) {{
        [System.IO.File]::WriteAllText($_.FullName, $next, $Utf8NoBom)
        $script:patched += 1
        Log ("removed whitelist entry: " + $_.FullName)
      }}
    }}
  }}
  Log ("whitelist cleanup count=" + $patched)
  Set-Content -LiteralPath $DonePath -Value 'ok' -Encoding ASCII
  exit 0
}} catch {{
  Log ("ERROR: " + $_.Exception.Message)
  Set-Content -LiteralPath $FailedPath -Value $_.Exception.Message -Encoding UTF8
  exit 1
}}
"#,
        resources_dir = powershell_single_quoted(&path_string(resources_dir)),
        app_dir = powershell_single_quoted(&app_dir),
        log = powershell_single_quoted(&path_string(&stage.log)),
        done = powershell_single_quoted(&path_string(&stage.done)),
        failed = powershell_single_quoted(&path_string(&stage.failed)),
    )
}

#[cfg(not(target_os = "windows"))]
fn apply_localization_resources_direct(
    resources_dir: &Path,
    stage: &LocalizationStage,
    install_patch: bool,
) -> Result<(String, String), String> {
    if install_patch {
        fs::create_dir_all(resources_dir.join("ion-dist").join("i18n"))
            .map_err(|error| error.to_string())?;
        fs::create_dir_all(
            resources_dir
                .join("ion-dist")
                .join("i18n")
                .join("statsig"),
        )
        .map_err(|error| error.to_string())?;
        fs::copy(&stage.desktop, resources_dir.join("zh-CN.json"))
            .map_err(|error| error.to_string())?;
        fs::copy(
            &stage.frontend,
            resources_dir
                .join("ion-dist")
                .join("i18n")
                .join("zh-CN.json"),
        )
        .map_err(|error| error.to_string())?;
        fs::copy(
            &stage.statsig,
            resources_dir
                .join("ion-dist")
                .join("i18n")
                .join("statsig")
                .join("zh-CN.json"),
        )
        .map_err(|error| error.to_string())?;
    } else {
        let _ = fs::remove_file(resources_dir.join("zh-CN.json"));
        let _ = fs::remove_file(
            resources_dir
                .join("ion-dist")
                .join("i18n")
                .join("zh-CN.json"),
        );
        let _ = fs::remove_file(
            resources_dir
                .join("ion-dist")
                .join("i18n")
                .join("statsig")
                .join("zh-CN.json"),
        );
    }
    let patched = patch_language_whitelist_in_resources(resources_dir, install_patch)?;
    Ok((
        format!("local resource patch finished; whitelist changed={patched}"),
        String::new(),
    ))
}

fn localization_patch_status() -> LocalizationPatchStatus {
    let Some(install) = detect_claude_install() else {
        return LocalizationPatchStatus::default();
    };
    let resources_dir = claude_resources_dir(&install);
    LocalizationPatchStatus {
        resources_dir: path_string(&resources_dir),
        desktop_json: resources_dir.join("zh-CN.json").is_file(),
        frontend_json: resources_dir
            .join("ion-dist")
            .join("i18n")
            .join("zh-CN.json")
            .is_file(),
        statsig_json: resources_dir
            .join("ion-dist")
            .join("i18n")
            .join("statsig")
            .join("zh-CN.json")
            .is_file(),
        whitelist_patched: localization_whitelist_patched(&resources_dir),
        locale_paths: claude_locale_config_paths()
            .into_iter()
            .filter(|path| path.is_file())
            .map(|path| path_string(&path))
            .collect(),
        current_locale: current_claude_locale(),
    }
}

fn set_claude_locale(locale: &str) -> Result<Vec<String>, String> {
    let mut touched = Vec::new();
    for path in claude_locale_config_paths() {
        let mut value = read_json_file(&path).unwrap_or_else(|_| json!({}));
        if !value.is_object() {
            value = json!({});
        }
        let Some(root) = value.as_object_mut() else {
            continue;
        };
        root.insert("locale".to_string(), Value::String(locale.to_string()));
        write_json_file(&path, &value)?;
        touched.push(path_string(&path));
    }
    Ok(touched)
}

fn current_claude_locale() -> Option<String> {
    claude_locale_config_paths().into_iter().find_map(|path| {
        read_json_file(&path)
            .ok()?
            .get("locale")
            .and_then(Value::as_str)
            .map(ToString::to_string)
    })
}

fn claude_locale_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    if let Ok(path) = claude_3p_user_data_dir() {
        push_unique_path(&mut paths, &mut seen, path.join("config.json"));
    }
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let local = PathBuf::from(local_app_data);
        push_unique_path(&mut paths, &mut seen, local.join("Claude-3p").join("config.json"));
        push_unique_path(&mut paths, &mut seen, local.join("Claude").join("config.json"));
        push_unique_path(
            &mut paths,
            &mut seen,
            local
                .join("Packages")
                .join("Claude_pzs8sxrjxfjjc")
                .join("LocalCache")
                .join("Roaming")
                .join("Claude")
                .join("config.json"),
        );
        push_unique_path(
            &mut paths,
            &mut seen,
            local
                .join("Packages")
                .join("Claude_pzs8sxrjxfjjc")
                .join("LocalCache")
                .join("Roaming")
                .join("Claude-3p")
                .join("config.json"),
        );
    }
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let roaming = PathBuf::from(appdata);
        push_unique_path(
            &mut paths,
            &mut seen,
            roaming.join("Claude-3p").join("config.json"),
        );
        push_unique_path(
            &mut paths,
            &mut seen,
            roaming.join("Claude").join("config.json"),
        );
    }
    paths
}

fn push_unique_path(paths: &mut Vec<PathBuf>, seen: &mut HashSet<String>, path: PathBuf) {
    let key = normalized_path_key(&path);
    if seen.insert(key) {
        paths.push(path);
    }
}

fn localization_whitelist_patched(resources_dir: &Path) -> bool {
    let mut files = Vec::new();
    collect_js_files(&resources_dir.join("ion-dist").join("assets"), &mut files);
    files.into_iter().any(|path| {
        fs::read_to_string(path)
            .map(|text| text.contains("\"zh-CN\""))
            .unwrap_or(false)
    })
}

#[cfg(not(target_os = "windows"))]
fn patch_language_whitelist_in_resources(
    resources_dir: &Path,
    install_patch: bool,
) -> Result<bool, String> {
    let base = r#"["en-US","de-DE","fr-FR","ko-KR","ja-JP","es-419","es-ES","it-IT","hi-IN","pt-BR","id-ID"]"#;
    let with_zh = r#"["en-US","de-DE","fr-FR","ko-KR","ja-JP","es-419","es-ES","it-IT","hi-IN","pt-BR","id-ID","zh-CN"]"#;
    let mut changed = false;
    let mut files = Vec::new();
    collect_js_files(&resources_dir.join("ion-dist").join("assets"), &mut files);
    for path in files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let next = if install_patch {
            if text.contains("\"zh-CN\"") {
                continue;
            }
            text.replace(base, with_zh)
        } else {
            text.replace(r#","zh-CN""#, "").replace(r#""zh-CN","#, "")
        };
        if next != text {
            fs::write(&path, next).map_err(|error| error.to_string())?;
            changed = true;
        }
    }
    Ok(changed)
}

fn collect_js_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_js_files(&path, files);
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("js"))
        {
            files.push(path);
        }
    }
}

fn upsert_builtin_script(config: &mut AppConfig, id: &str, name: &str, code: &str, enabled: bool) {
    let script = UserScript {
        id: id.to_string(),
        name: name.to_string(),
        enabled,
        code: code.to_string(),
    };
    match config.scripts.iter_mut().find(|item| item.id == id) {
        Some(existing) => *existing = script,
        None => config.scripts.insert(0, script),
    }
}

#[tauri::command]
fn launch_claude_desktop() -> Result<LaunchResult, String> {
    let install = detect_claude_install().ok_or("Claude Desktop install was not found")?;
    let config = read_config().map_err(|error| error.to_string())?;
    terminate_claude_processes().map_err(|error| error.to_string())?;
    let launched = launch_claude_plain(&install, &config, LaunchMode::Clean)
        .map_err(|error| error.to_string())?;
    let process_id = launched.process_id;
    wait_for_launched_child(launched);

    Ok(LaunchResult {
        executable: path_string(&install.executable),
        process_id,
        injected_provider_id: None,
        sandbox_relaxed: false,
        clean_environment: true,
        launcher_route: launcher_route_label(&install).to_string(),
        injection_channel: "diagnostic_clean_launch".to_string(),
        live_injection_supported: live_injection_supported_for_install(&install),
        live_injection_attempted: false,
        gateway_url: None,
        cdp_port: None,
        cdp_injected: false,
        cdp_error: None,
        claude_3p: clear_claude_3p_deployment_mode().ok(),
        verification: None,
    })
}

#[tauri::command]
fn launch_claude_desktop_current_provider() -> Result<LaunchResult, String> {
    let install = detect_claude_install().ok_or("Claude Desktop install was not found")?;
    let mut config = read_config().map_err(|error| error.to_string())?;
    let _ = apply_cc_switch_sync(&mut config);
    if validate_active_provider_for_injection(&config).is_err() {
        let fallback_id = config
            .providers
            .iter()
            .find(|provider| provider_can_inject(provider, &config.sandbox))
            .map(|provider| provider.id.clone())
            .ok_or(
                "No injectable provider was found. Select a provider with Base URL or enable API Key injection.",
            )?;
        config.active_provider_id = Some(fallback_id);
    }
    validate_active_provider_for_injection(&config)?;
    write_config(&config).map_err(|error| error.to_string())?;
    let uses_local_gateway = config_uses_local_gateway(&config);
    let gateway_url = if uses_local_gateway {
        ensure_gateway_runtime(&config)?;
        Some(gateway_url(config.gateway.port))
    } else {
        None
    };
    let before = gateway_snapshot();
    let claude_3p = Some(apply_claude_3p_provider_config(&config)?);

    let launch_install = prepare_launch_install(&install, &config)?;
    let live_injection_supported = live_injection_supported_for_install(&launch_install);
    let cdp_port = cdp_injection_supported_for_install(&launch_install).then(find_cdp_port);
    terminate_claude_processes().map_err(|error| error.to_string())?;
    let launched = launch_claude_plain(
        &launch_install,
        &config,
        LaunchMode::Inject {
            cdp_port: cdp_port.unwrap_or(DEFAULT_CDP_PORT),
        },
    )
    .map_err(|error| error.to_string())?;
    let process_id = launched.process_id;
    let cdp_result = cdp_port.map(|port| inject_claude_via_cdp(&config, port));
    let cdp_injected = cdp_result.as_ref().is_some_and(Result::is_ok);
    let cdp_error = cdp_result.and_then(Result::err);
    wait_for_launched_child(launched);

    Ok(LaunchResult {
        executable: path_string(&launch_install.executable),
        process_id,
        injected_provider_id: config.active_provider_id,
        sandbox_relaxed: config.sandbox.relax_sandbox && config.sandbox.acknowledged,
        clean_environment: false,
        launcher_route: launcher_route_label(&launch_install).to_string(),
        injection_channel: injection_channel_label(&launch_install, uses_local_gateway).to_string(),
        live_injection_supported,
        live_injection_attempted: cdp_port.is_some(),
        gateway_url,
        cdp_port,
        cdp_injected,
        cdp_error,
        claude_3p,
        verification: Some(verify_launch(before, uses_local_gateway)),
    })
}

#[tauri::command]
fn launch_claude_desktop_with_provider(id: String) -> Result<LaunchResult, String> {
    let install = detect_claude_install().ok_or("Claude Desktop install was not found")?;
    let mut config = read_config().map_err(|error| error.to_string())?;
    let _ = apply_cc_switch_sync(&mut config);
    if !config.providers.iter().any(|provider| provider.id == id) {
        return Err("Provider was not found".to_string());
    }
    config.active_provider_id = Some(id);
    validate_active_provider_for_injection(&config)?;
    write_config(&config).map_err(|error| error.to_string())?;
    let uses_local_gateway = config_uses_local_gateway(&config);
    let gateway_url = if uses_local_gateway {
        ensure_gateway_runtime(&config)?;
        Some(gateway_url(config.gateway.port))
    } else {
        None
    };
    let before = gateway_snapshot();
    let claude_3p = Some(apply_claude_3p_provider_config(&config)?);

    let launch_install = prepare_launch_install(&install, &config)?;
    let live_injection_supported = live_injection_supported_for_install(&launch_install);
    let cdp_port = cdp_injection_supported_for_install(&launch_install).then(find_cdp_port);
    terminate_claude_processes().map_err(|error| error.to_string())?;
    let launched = launch_claude_plain(
        &launch_install,
        &config,
        LaunchMode::Inject {
            cdp_port: cdp_port.unwrap_or(DEFAULT_CDP_PORT),
        },
    )
    .map_err(|error| error.to_string())?;
    let process_id = launched.process_id;
    let cdp_result = cdp_port.map(|port| inject_claude_via_cdp(&config, port));
    let cdp_injected = cdp_result.as_ref().is_some_and(Result::is_ok);
    let cdp_error = cdp_result.and_then(Result::err);
    wait_for_launched_child(launched);

    Ok(LaunchResult {
        executable: path_string(&launch_install.executable),
        process_id,
        injected_provider_id: config.active_provider_id,
        sandbox_relaxed: config.sandbox.relax_sandbox && config.sandbox.acknowledged,
        clean_environment: false,
        launcher_route: launcher_route_label(&launch_install).to_string(),
        injection_channel: injection_channel_label(&launch_install, uses_local_gateway).to_string(),
        live_injection_supported,
        live_injection_attempted: cdp_port.is_some(),
        gateway_url,
        cdp_port,
        cdp_injected,
        cdp_error,
        claude_3p,
        verification: Some(verify_launch(before, uses_local_gateway)),
    })
}

fn install_info(install: ClaudeInstall) -> InstallInfo {
    let app_asar = find_app_asar(&install).map(|path| path_string(&path));
    InstallInfo {
        executable: path_string(&install.executable),
        working_dir: path_string(&install.working_dir),
        source: install.source.to_string(),
        app_user_model_id: install.app_user_model_id.clone(),
        app_asar,
        launcher_route: launcher_route_label(&install).to_string(),
        live_injection_supported: live_injection_supported_for_install(&install),
    }
}

#[derive(Clone, Copy)]
enum LaunchMode {
    Clean,
    Inject { cdp_port: u16 },
}

struct LaunchedClaude {
    process_id: u32,
    child: Option<Child>,
}

fn launch_claude_plain(
    install: &ClaudeInstall,
    config: &AppConfig,
    mode: LaunchMode,
) -> anyhow::Result<LaunchedClaude> {
    if let Some(app_user_model_id) = install.app_user_model_id.as_ref() {
        let process_id =
            activate_windows_app(app_user_model_id, "").map_err(|error| anyhow::anyhow!(error))?;
        return Ok(LaunchedClaude {
            process_id,
            child: None,
        });
    }

    let mut command = Command::new(&install.executable);
    hide_child_console(&mut command);
    command
        .current_dir(&install.working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    for argument in launch_arguments(config, mode) {
        command.arg(argument);
    }

    clear_network_override_env(&mut command);

    if matches!(mode, LaunchMode::Inject { .. }) {
        if let Some(provider) = active_provider(config) {
            if config.sandbox.inject_provider {
                if config.gateway.enabled {
                    command.env("ANTHROPIC_BASE_URL", gateway_url(config.gateway.port));
                } else if !provider.base_url.is_empty() {
                    command.env("ANTHROPIC_BASE_URL", &provider.base_url);
                }
            }
            if config.sandbox.inject_api_key
                && !config.gateway.enabled
                && !provider.api_key.is_empty()
            {
                command.env("ANTHROPIC_AUTH_TOKEN", &provider.api_key);
                command.env("ANTHROPIC_API_KEY", &provider.api_key);
            }
        }
    }

    if matches!(mode, LaunchMode::Inject { .. })
        && config.sandbox.relax_sandbox
        && config.sandbox.acknowledged
    {
        command.env("ELECTRON_DISABLE_SANDBOX", "1");
    }

    let child = command.spawn()?;
    Ok(LaunchedClaude {
        process_id: child.id(),
        child: Some(child),
    })
}

fn launch_arguments(config: &AppConfig, mode: LaunchMode) -> Vec<String> {
    let mut arguments = Vec::new();
    if let LaunchMode::Inject { cdp_port } = mode {
        arguments.push(format!("--remote-debugging-port={cdp_port}"));
        arguments.push("--remote-allow-origins=*".to_string());
    }
    if matches!(mode, LaunchMode::Inject { .. })
        && config.sandbox.relax_sandbox
        && config.sandbox.acknowledged
    {
        arguments.push("--no-sandbox".to_string());
    }
    arguments
}

fn wait_for_launched_child(launched: LaunchedClaude) {
    if let Some(mut child) = launched.child {
        std::thread::spawn(move || {
            let _ = child.wait();
        });
    }
}

fn prepare_launch_install(
    install: &ClaudeInstall,
    config: &AppConfig,
) -> Result<ClaudeInstall, String> {
    let _ = config;
    Ok(install.clone())
}

fn live_injection_supported_for_install(install: &ClaudeInstall) -> bool {
    install.app_user_model_id.is_none()
}

fn cdp_injection_supported_for_install(install: &ClaudeInstall) -> bool {
    install.app_user_model_id.is_none() && !preload_injection_install(install)
}

fn preload_injection_install(install: &ClaudeInstall) -> bool {
    let _ = install;
    false
}

fn launcher_route_label(install: &ClaudeInstall) -> &'static str {
    if preload_injection_install(install) {
        return "external_launcher_localized_sidecar";
    }
    if install.app_user_model_id.is_some() {
        "external_launcher_app_activation"
    } else {
        "external_launcher_process"
    }
}

fn injection_channel_label(install: &ClaudeInstall, uses_local_gateway: bool) -> &'static str {
    if preload_injection_install(install) {
        return if uses_local_gateway {
            "preload_script_plus_gateway_config"
        } else {
            "preload_script_plus_direct_config"
        };
    }
    match (
        live_injection_supported_for_install(install),
        uses_local_gateway,
    ) {
        (true, true) => "live_script_plus_gateway_config",
        (true, false) => "live_script_plus_direct_config",
        (false, true) => "config_injection_plus_gateway",
        (false, false) => "config_injection_direct",
    }
}

fn activate_windows_app(app_user_model_id: &str, arguments: &str) -> Result<u32, String> {
    if !cfg!(target_os = "windows") {
        return Err("Windows app activation is only available on Windows".to_string());
    }

    let script = format!(
        r#"
$code = @'
using System;
using System.Runtime.InteropServices;

[Flags]
public enum ActivateOptions
{{
    None = 0,
    DesignMode = 1,
    NoErrorUI = 2,
    NoSplashScreen = 4
}}

[ComImport, Guid("45BA127D-10A8-46EA-8AB7-56EA9078943C")]
public class ApplicationActivationManager {{ }}

[ComImport, InterfaceType(ComInterfaceType.InterfaceIsIUnknown), Guid("2e941141-7f97-4756-ba1d-9decde894a3d")]
public interface IApplicationActivationManager
{{
    [PreserveSig]
    int ActivateApplication(
        [MarshalAs(UnmanagedType.LPWStr)] string appUserModelId,
        [MarshalAs(UnmanagedType.LPWStr)] string arguments,
        ActivateOptions options,
        out uint processId);
}}

public static class ClaudePlusAppActivation
{{
    public static uint Activate(string appUserModelId, string arguments)
    {{
        var manager = (IApplicationActivationManager)new ApplicationActivationManager();
        uint processId;
        int hr = manager.ActivateApplication(appUserModelId, arguments, ActivateOptions.None, out processId);
        if (hr != 0)
        {{
            Marshal.ThrowExceptionForHR(hr);
        }}
        return processId;
    }}
}}
'@
Add-Type -TypeDefinition $code -ErrorAction Stop
[ClaudePlusAppActivation]::Activate({app_id}, {arguments})
"#,
        app_id = powershell_single_quoted(app_user_model_id),
        arguments = powershell_single_quoted(arguments)
    );
    let output = run_powershell_script(&script)?;
    if !output.status.success() {
        return Err(format_command_failure(&output));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .rev()
        .find_map(|line| line.trim().parse::<u32>().ok())
        .ok_or_else(|| format!("Windows app activation did not return a process id: {stdout}"))
}

fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn format_command_failure(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let status = output
        .status
        .code()
        .map_or_else(|| "terminated".to_string(), |code| code.to_string());
    format!("Command failed with exit code {status}. stdout: {stdout}. stderr: {stderr}")
}

fn find_cdp_port() -> u16 {
    (DEFAULT_CDP_PORT..DEFAULT_CDP_PORT + 50)
        .find(|port| !cdp::is_local_port_open(*port))
        .unwrap_or(DEFAULT_CDP_PORT)
}

fn inject_claude_via_cdp(config: &AppConfig, cdp_port: u16) -> Result<(), String> {
    cdp::wait_for_version(cdp_port, Duration::from_secs(10)).map_err(|error| error.to_string())?;
    let target = cdp::wait_for_injectable_target(cdp_port, Duration::from_secs(20))
        .map_err(|error| error.to_string())?;
    let websocket_url = target
        .websocket_debugger_url
        .ok_or_else(|| "Claude CDP target did not expose a websocket URL".to_string())?;
    cdp::inject_script(&websocket_url, &build_inject_script(config))
        .map_err(|error| error.to_string())
}

fn validate_active_provider_for_injection(config: &AppConfig) -> Result<(), String> {
    let provider = active_provider(config).ok_or("No enabled active provider was selected")?;
    if provider_can_inject(provider, &config.sandbox) {
        return Ok(());
    }
    Err(
        "The active provider has no injectable value. Select a provider with Base URL or enable API Key injection."
            .to_string(),
    )
}

fn provider_can_inject(provider: &ApiProvider, sandbox: &SandboxConfig) -> bool {
    provider.enabled
        && ((sandbox.inject_provider && !provider.base_url.is_empty())
            || (sandbox.inject_api_key && !provider.api_key.is_empty()))
}

fn test_api_provider(provider: &ApiProvider) -> Result<ProviderTestResult, String> {
    let base_url = provider.base_url.trim().trim_end_matches('/').to_string();
    if base_url.is_empty() {
        return Err("Provider Base URL is required".to_string());
    }
    let url = join_gateway_url(&base_url, "/v1/models?limit=1000");
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(30))
        .build();
    let mut request = agent
        .get(&url)
        .set("anthropic-version", "2023-06-01")
        .set("accept", "application/json");

    if !provider.api_key.trim().is_empty() {
        request = request
            .set("Authorization", &format!("Bearer {}", provider.api_key))
            .set("x-api-key", &provider.api_key);
    }

    let (status, body) = match request.call() {
        Ok(response) => read_ureq_response(response).map(|body| (Some(200), body))?,
        Err(ureq::Error::Status(status, response)) => {
            let body = read_ureq_response(response)?;
            (Some(status), body)
        }
        Err(error) => {
            return Ok(ProviderTestResult {
                provider_id: provider.id.clone(),
                provider_name: provider.name.clone(),
                base_url,
                url,
                protocol: effective_provider_protocol(provider),
                model_count: 0,
                claude_desktop_compatible: false,
                compatibility_message: format!(
                    "Network test failed before model compatibility could be checked: {error}"
                ),
                ok: false,
                status: None,
                code: Some("NETWORK_ERROR".to_string()),
                message: Some(error.to_string()),
                body_excerpt: String::new(),
                key_mask: mask_secret(&provider.api_key),
            });
        }
    };
    let status_code = status.unwrap_or_default();
    let body_text = String::from_utf8_lossy(&body).to_string();
    let parsed = parse_provider_error(&body_text);
    let model_ids = parse_model_ids(&body_text);
    let protocol = effective_provider_protocol(provider);
    let requires_adapter = protocol == PROVIDER_PROTOCOL_OPENAI;
    let model_count = model_ids.len();
    let claude_desktop_compatible = (200..300).contains(&status_code) && !requires_adapter;
    let compatibility_message = if !(200..300).contains(&status_code) {
        "Provider did not return a successful model list response".to_string()
    } else if requires_adapter {
        format!(
            "Detected {model_count} OpenAI/Codex-style model(s). Claude++ will use the local adapter Gateway for Claude Desktop."
        )
    } else if model_count == 0 {
        "Model endpoint is reachable but returned no model IDs; configure explicit models or check provider permissions."
            .to_string()
    } else {
        format!("Detected {model_count} Claude-compatible model(s).")
    };

    Ok(ProviderTestResult {
        provider_id: provider.id.clone(),
        provider_name: provider.name.clone(),
        base_url,
        url,
        protocol,
        model_count,
        claude_desktop_compatible,
        compatibility_message,
        ok: (200..300).contains(&status_code),
        status,
        code: parsed.0,
        message: parsed.1,
        body_excerpt: redact_body_excerpt(&body_text),
        key_mask: mask_secret(&provider.api_key),
    })
}

fn apply_claude_3p_provider_config(config: &AppConfig) -> Result<Claude3pStatus, String> {
    let provider = active_provider(config).ok_or("No enabled active provider was selected")?;
    let user_data_dir = claude_3p_user_data_dir()?;
    let library_dir = user_data_dir.join(CLAUDE_3P_LIBRARY_DIR);
    let desktop_config_path = user_data_dir.join(CLAUDE_3P_CONFIG_FILE);
    let meta_path = library_dir.join(CLAUDE_3P_META_FILE);
    fs::create_dir_all(&library_dir).map_err(|error| error.to_string())?;

    let config_id =
        existing_claude_plus_3p_config_id(&meta_path).unwrap_or_else(|| Uuid::new_v4().to_string());
    let config_path = library_dir.join(format!("{config_id}.json"));
    let gateway = build_claude_3p_provider_json(config, provider)?;

    write_json_file(&config_path, &gateway)?;
    let mut meta = read_json_file(&meta_path).unwrap_or_else(|_| json!({}));
    let entries = meta
        .get_mut("entries")
        .and_then(Value::as_array_mut)
        .map(|entries| {
            entries.retain(|entry| {
                entry
                    .get("name")
                    .and_then(Value::as_str)
                    .is_none_or(|name| !is_claude_plus_3p_config_name(name))
            });
            entries.push(json!({
                "id": config_id,
                "name": CLAUDE_PLUS_CONFIG_NAME,
                "provider": "gateway",
                "note": claude_3p_config_note(config_uses_local_gateway(config)),
            }));
        });
    if entries.is_none() {
        meta = json!({
            "appliedId": config_id,
            "isManaged": false,
            "platform": current_platform(),
            "entries": [{
                "id": config_id,
                "name": CLAUDE_PLUS_CONFIG_NAME,
                "provider": "gateway",
                "note": claude_3p_config_note(config_uses_local_gateway(config)),
            }],
        });
    }
    meta["appliedId"] = Value::String(config_id.clone());
    meta["isManaged"] = Value::Bool(false);
    meta["platform"] = Value::String(current_platform().to_string());
    write_json_file(&meta_path, &meta)?;

    let mut desktop_config = read_json_file(&desktop_config_path).unwrap_or_else(|_| json!({}));
    if !desktop_config.is_object() {
        desktop_config = json!({});
    }
    desktop_config["deploymentMode"] = Value::String("3p".to_string());
    write_json_file(&desktop_config_path, &desktop_config)?;

    let mut status = claude_3p_status();
    status.config_id = config_id;
    status.config_path = path_string(&config_path);
    status.applied = true;
    Ok(status)
}

fn clear_claude_3p_deployment_mode() -> anyhow::Result<Claude3pStatus> {
    let user_data_dir = claude_3p_user_data_dir().map_err(anyhow::Error::msg)?;
    let desktop_config_path = user_data_dir.join(CLAUDE_3P_CONFIG_FILE);
    if desktop_config_path.is_file() {
        let mut desktop_config = read_json_file(&desktop_config_path).unwrap_or_else(|_| json!({}));
        if let Some(object) = desktop_config.as_object_mut() {
            object.remove("deploymentMode");
            write_json_file(&desktop_config_path, &desktop_config).map_err(anyhow::Error::msg)?;
        }
    }
    Ok(claude_3p_status())
}

fn build_claude_3p_provider_json(
    config: &AppConfig,
    provider: &ApiProvider,
) -> Result<Value, String> {
    let uses_local_gateway = config.gateway.enabled;
    let provider_base_url = if uses_local_gateway {
        gateway_url(config.gateway.port)
    } else {
        provider.base_url.trim_end_matches('/').to_string()
    };
    let api_key = provider_key_with_fallback(config, provider);
    if provider_base_url.trim().is_empty() {
        return Err("Direct 3P mode requires a provider Base URL".to_string());
    }
    if !uses_local_gateway && api_key.trim().is_empty() {
        return Err("Direct 3P mode requires an API key for the selected provider".to_string());
    }
    let injected_gateway_key = if uses_local_gateway || api_key.is_empty() {
        "claude-plus-local-gateway".to_string()
    } else {
        api_key.clone()
    };
    let inference_models = build_provider_inference_model_entries(provider, &api_key);
    let mut gateway = json!({
        "inferenceProvider": "gateway",
        "inferenceGatewayBaseUrl": provider_base_url,
        "inferenceCredentialKind": "static",
        "inferenceGatewayAuthScheme": "bearer",
        "inferenceGatewayApiKey": injected_gateway_key,
        "disableDeploymentModeChooser": true,
        "coworkTabEnabled": true,
        "isClaudeCodeForDesktopEnabled": true,
        "chatTabEnabled": true,
        "modelDiscoveryEnabled": true,
        "autoModeEnabled": true,
        "coworkEgressAllowedHosts": ["*"]
    });

    if !inference_models.is_empty() {
        gateway["modelDiscoveryEnabled"] = Value::Bool(false);
        gateway["inferenceModels"] = Value::Array(inference_models);
    }

    Ok(gateway)
}

fn claude_3p_config_note(uses_local_gateway: bool) -> &'static str {
    if uses_local_gateway {
        "Managed by Claude++ (local Gateway)"
    } else {
        "Managed by Claude++ (direct 3P provider)"
    }
}

fn build_provider_inference_model_entries(provider: &ApiProvider, api_key: &str) -> Vec<Value> {
    if provider_requires_gateway_adapter(provider) {
        return build_openai_route_inference_model_entries(provider, api_key);
    }
    build_anthropic_inference_model_entries(provider, api_key)
}

fn build_openai_route_inference_model_entries(provider: &ApiProvider, api_key: &str) -> Vec<Value> {
    openai_route_models_for_provider(provider, api_key)
        .into_iter()
        .map(|route| {
            json!({
                "name": route.claude_route,
                "labelOverride": route.label,
            })
        })
        .collect()
}

fn build_anthropic_inference_model_entries(provider: &ApiProvider, api_key: &str) -> Vec<Value> {
    let model_ids = discover_provider_model_ids(&provider.base_url, api_key)
        .unwrap_or_else(|_| default_anthropic_model_ids());
    dedupe_model_ids(model_ids)
        .into_iter()
        .map(|model_id| {
            let label = model_label(&model_id);
            json!({
                "name": model_id,
                "labelOverride": label,
            })
        })
        .collect()
}

fn default_anthropic_model_ids() -> Vec<String> {
    vec![
        "claude-3-5-haiku-20241022".to_string(),
        "claude-3-5-sonnet-20241022".to_string(),
        "claude-3-7-sonnet-20250219".to_string(),
        "claude-haiku-4-5-20251001".to_string(),
        "claude-opus-4-20250514".to_string(),
        "claude-opus-4-1-20250805".to_string(),
    ]
}

fn dedupe_model_ids(model_ids: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    model_ids
        .into_iter()
        .filter(|id| !id.trim().is_empty())
        .filter(|id| seen.insert(id.to_ascii_lowercase()))
        .collect()
}

#[derive(Debug, Clone)]
struct OpenAiModelRoute {
    claude_route: String,
    target_model: String,
    label: String,
}

fn openai_route_models_for_provider(
    provider: &ApiProvider,
    api_key: &str,
) -> Vec<OpenAiModelRoute> {
    let configured = provider
        .model_mappings
        .iter()
        .filter(|mapping| mapping.enabled)
        .filter_map(|mapping| {
            normalize_model_mapping(mapping.clone()).map(|mapping| OpenAiModelRoute {
                label: if mapping.label.trim().is_empty() {
                    format!(
                        "{} via {}",
                        model_label(&mapping.target_model),
                        mapping.claude_route
                    )
                } else {
                    mapping.label
                },
                target_model: mapping.target_model,
                claude_route: mapping.claude_route,
            })
        })
        .collect::<Vec<_>>();
    if !configured.is_empty() {
        return configured;
    }

    let model_ids = discover_provider_model_ids(&provider.base_url, api_key)
        .unwrap_or_else(|_| default_openai_model_ids(provider));
    openai_route_models_from_ids(model_ids)
}

fn openai_route_models_from_ids(model_ids: Vec<String>) -> Vec<OpenAiModelRoute> {
    let model_ids = prioritize_model_ids(model_ids);
    let route_names = [
        "claude-opus-4-5",
        "claude-sonnet-4-5",
        "claude-haiku-4-5",
        "anthropic/claude-opus-4-5",
        "anthropic/claude-sonnet-4-5",
        "anthropic/claude-haiku-4-5",
        "claude-opus-4",
        "claude-sonnet-4",
        "claude-haiku-4",
    ];
    model_ids
        .into_iter()
        .zip(route_names)
        .map(|(target_model, claude_route)| OpenAiModelRoute {
            label: format!("{} via {}", model_label(&target_model), claude_route),
            target_model,
            claude_route: claude_route.to_string(),
        })
        .collect()
}

fn discover_provider_model_ids(base_url: &str, api_key: &str) -> Result<Vec<String>, String> {
    let url = join_gateway_url(base_url, "/v1/models?limit=1000");
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(20))
        .build();
    let mut request = agent
        .get(&url)
        .set("anthropic-version", "2023-06-01")
        .set("accept", "application/json");
    if !api_key.trim().is_empty() {
        request = request
            .set("Authorization", &format!("Bearer {}", api_key.trim()))
            .set("x-api-key", api_key.trim());
    }
    let body = match request.call() {
        Ok(response) => read_ureq_response(response)?,
        Err(ureq::Error::Status(status, response)) => {
            let body = read_ureq_response(response)?;
            return Err(upstream_error_summary(status, &body)
                .unwrap_or_else(|| format!("Model list returned HTTP {status}")));
        }
        Err(error) => return Err(error.to_string()),
    };
    let ids = parse_model_ids(&String::from_utf8_lossy(&body));
    if ids.is_empty() {
        Err("Model list returned no IDs".to_string())
    } else {
        Ok(ids)
    }
}

fn default_openai_model_ids(provider: &ApiProvider) -> Vec<String> {
    let lower = format!("{} {}", provider.name, provider.base_url).to_ascii_lowercase();
    if lower.contains("moonshot") || lower.contains("kimi") {
        return vec![
            "kimi-k2-0711-preview".to_string(),
            "moonshot-v1-128k".to_string(),
        ];
    }
    if lower.contains("siliconflow") {
        return vec![
            "deepseek-ai/DeepSeek-V3".to_string(),
            "Qwen/Qwen3-Coder-480B-A35B-Instruct".to_string(),
        ];
    }
    vec![
        "gpt-5.5".to_string(),
        "gpt-5.4".to_string(),
        "gpt-5.4-mini".to_string(),
    ]
}

fn prioritize_model_ids(model_ids: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let deduped = model_ids
        .into_iter()
        .filter(|id| seen.insert(id.to_ascii_lowercase()))
        .collect::<Vec<_>>();
    let mut ordered = Vec::new();
    push_best_model(&deduped, &mut ordered, |lower| {
        (lower.contains("5.5") || lower.contains("opus"))
            && !lower.contains("compact")
            && !lower.contains("mini")
    });
    push_best_model(&deduped, &mut ordered, |lower| {
        (lower.contains("5.4") || lower.contains("sonnet") || lower.contains("codex"))
            && !lower.contains("mini")
            && !lower.contains("compact")
            && !lower.contains("spark")
            && !lower.contains("review")
    });
    push_best_model(&deduped, &mut ordered, |lower| {
        lower.contains("mini") || lower.contains("haiku") || lower.contains("compact")
    });
    for id in deduped {
        if !ordered
            .iter()
            .any(|item: &String| item.eq_ignore_ascii_case(&id))
        {
            ordered.push(id);
        }
    }
    ordered
}

fn push_best_model(
    candidates: &[String],
    ordered: &mut Vec<String>,
    predicate: impl Fn(&str) -> bool,
) {
    if let Some(id) = candidates.iter().find(|id| {
        !ordered
            .iter()
            .any(|item| item.eq_ignore_ascii_case(id.as_str()))
            && predicate(&id.to_ascii_lowercase())
    }) {
        ordered.push(id.clone());
    }
}

fn model_label(model_id: &str) -> String {
    model_id
        .split(['/', ':'])
        .next_back()
        .unwrap_or(model_id)
        .replace('-', " ")
        .replace('_', " ")
}

fn is_claude_plus_3p_config_name(name: &str) -> bool {
    name == CLAUDE_PLUS_CONFIG_NAME || name == LEGACY_CLAUDE_PLUS_CONFIG_NAME
}

fn existing_claude_plus_3p_config_id(meta_path: &Path) -> Option<String> {
    let meta = read_json_file(meta_path).ok()?;
    let entries = meta.get("entries")?.as_array()?;
    entries.iter().find_map(|entry| {
        let name = entry.get("name").and_then(Value::as_str)?;
        let id = entry.get("id").and_then(Value::as_str)?;
        is_claude_plus_3p_config_name(name).then(|| id.to_string())
    })
}

fn claude_3p_status() -> Claude3pStatus {
    let user_data_dir =
        claude_3p_user_data_dir().unwrap_or_else(|_| PathBuf::from(CLAUDE_3P_DIR_NAME));
    let library_dir = user_data_dir.join(CLAUDE_3P_LIBRARY_DIR);
    let desktop_config_path = user_data_dir.join(CLAUDE_3P_CONFIG_FILE);
    let meta_path = library_dir.join(CLAUDE_3P_META_FILE);
    let meta = read_json_file(&meta_path).ok();
    let applied_id = meta
        .as_ref()
        .and_then(|value| value.get("appliedId"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let config_id = existing_claude_plus_3p_config_id(&meta_path)
        .or_else(|| applied_id.clone())
        .unwrap_or_default();
    let config_path = if config_id.is_empty() {
        library_dir.join("none.json")
    } else {
        library_dir.join(format!("{config_id}.json"))
    };
    let deployment_mode = read_claude_3p_deployment_mode();
    let file_exists = desktop_config_path.is_file();
    let meta_exists = meta_path.is_file();
    let active_config_exists = config_path.is_file();
    let applied = deployment_mode.as_deref() == Some("3p")
        && active_config_exists
        && !config_id.is_empty()
        && applied_id.as_deref() == Some(config_id.as_str());

    Claude3pStatus {
        user_data_dir: path_string(&user_data_dir),
        desktop_config_path: path_string(&desktop_config_path),
        config_library_dir: path_string(&library_dir),
        meta_path: path_string(&meta_path),
        config_path: path_string(&config_path),
        config_id,
        deployment_mode,
        applied_id,
        file_exists,
        meta_exists,
        active_config_exists,
        applied,
    }
}

fn claude_3p_user_data_dir() -> Result<PathBuf, String> {
    if cfg!(target_os = "windows") {
        if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
            return Ok(PathBuf::from(local_appdata).join(CLAUDE_3P_DIR_NAME));
        }
    }
    if cfg!(target_os = "macos") {
        if let Some(home) = std::env::var_os("HOME") {
            return Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join(CLAUDE_3P_DIR_NAME));
        }
    }
    Err("Claude 3P user-data directory is not supported on this platform".to_string())
}

struct HistoryItemDef {
    key: &'static str,
    label: &'static str,
    relative_path: &'static str,
    default_restore: bool,
}

#[derive(Default)]
struct FileTreeStats {
    file_count: u64,
    total_bytes: u64,
    latest_write_ms: Option<u128>,
}

fn history_item_defs() -> Vec<HistoryItemDef> {
    vec![
        HistoryItemDef {
            key: "indexeddb",
            label: "Chat IndexedDB",
            relative_path: "IndexedDB",
            default_restore: true,
        },
        HistoryItemDef {
            key: "local_storage",
            label: "Local Storage",
            relative_path: "Local Storage",
            default_restore: false,
        },
        HistoryItemDef {
            key: "session_storage",
            label: "Session Storage",
            relative_path: "Session Storage",
            default_restore: false,
        },
        HistoryItemDef {
            key: "local_agent_sessions",
            label: "Local Agent Sessions",
            relative_path: "local-agent-mode-sessions",
            default_restore: true,
        },
        HistoryItemDef {
            key: "claude_code_sessions",
            label: "Claude Code Sessions",
            relative_path: "claude-code-sessions",
            default_restore: true,
        },
        HistoryItemDef {
            key: "blob_storage",
            label: "Blob Attachments",
            relative_path: "blob_storage",
            default_restore: true,
        },
    ]
}

fn history_scan() -> HistoryScan {
    let target = claude_3p_user_data_dir().unwrap_or_else(|_| PathBuf::from(CLAUDE_3P_DIR_NAME));
    let backup_root = history_backup_root();
    let mut seen = HashSet::new();
    let mut profiles = Vec::new();

    for (name, path) in history_candidate_profiles(&target) {
        if !seen.insert(normalized_path_key(&path)) {
            continue;
        }
        let profile = history_profile(name, path, &target);
        if profile.exists && (profile.is_target || profile.item_count > 0) {
            profiles.push(profile);
        }
    }

    profiles.sort_by(|a, b| {
        b.is_target
            .cmp(&a.is_target)
            .then_with(|| b.latest_write_ms.cmp(&a.latest_write_ms))
            .then_with(|| a.name.cmp(&b.name))
    });

    HistoryScan {
        target_path: path_string(&target),
        backup_root: path_string(&backup_root),
        profiles,
    }
}

fn history_scan_placeholder() -> HistoryScan {
    let target = claude_3p_user_data_dir().unwrap_or_else(|_| PathBuf::from(CLAUDE_3P_DIR_NAME));
    HistoryScan {
        target_path: path_string(&target),
        backup_root: path_string(&history_backup_root()),
        profiles: Vec::new(),
    }
}

fn history_candidate_profiles(target: &Path) -> Vec<(String, PathBuf)> {
    let mut profiles = vec![("Current Claude-3p".to_string(), target.to_path_buf())];
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let appdata = PathBuf::from(appdata);
        profiles.push((
            "Official Claude profile".to_string(),
            appdata.join("Claude"),
        ));
        profiles.push((
            "Roaming Claude-3p profile".to_string(),
            appdata.join("Claude-3p"),
        ));
    }
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        let local = PathBuf::from(local_app_data);
        profiles.push(("Local Claude profile".to_string(), local.join("Claude")));
        profiles.push((
            "Local Claude-3p profile".to_string(),
            local.join("Claude-3p"),
        ));
        let package_roaming = local
            .join("Packages")
            .join("Claude_pzs8sxrjxfjjc")
            .join("LocalCache")
            .join("Roaming");
        profiles.push((
            "MSIX virtualized Claude profile".to_string(),
            package_roaming.join("Claude"),
        ));
        profiles.push((
            "MSIX virtualized Claude-3p profile".to_string(),
            package_roaming.join("Claude-3p"),
        ));
    }
    profiles
}

fn history_profile(name: String, path: PathBuf, target: &Path) -> HistoryProfile {
    let exists = path.is_dir();
    let is_target = same_path(&path, target);
    let mut items = Vec::new();
    let mut file_count = 0;
    let mut total_bytes = 0;
    let mut latest_write_ms = None;

    if exists {
        for definition in history_item_defs() {
            let item_path = path.join(definition.relative_path);
            let stats = file_tree_stats(&item_path);
            let item = HistoryItem {
                key: definition.key.to_string(),
                label: definition.label.to_string(),
                relative_path: definition.relative_path.to_string(),
                default_restore: definition.default_restore,
                exists: item_path.exists(),
                file_count: stats.file_count,
                total_bytes: stats.total_bytes,
                latest_write_ms: stats.latest_write_ms,
            };
            if item.exists {
                file_count += item.file_count;
                total_bytes += item.total_bytes;
                latest_write_ms = max_optional_millis(latest_write_ms, item.latest_write_ms);
            }
            items.push(item);
        }
    }

    let item_count = items.iter().filter(|item| item.exists).count();
    HistoryProfile {
        name,
        path: path_string(&path),
        exists,
        is_target,
        item_count,
        file_count,
        total_bytes,
        latest_write_ms,
        items,
    }
}

fn repair_history_from_profile(input: HistoryRepairInput) -> Result<HistoryRepairResult, String> {
    let source = PathBuf::from(input.source_path.trim());
    let target = claude_3p_user_data_dir()?;
    if !source.is_dir() {
        return Err("History source profile was not found".to_string());
    }
    if same_path(&source, &target) {
        return Err(
            "Select a different source profile; the current Claude-3p profile is already active"
                .to_string(),
        );
    }

    let selected_keys = if input.item_keys.is_empty() {
        history_item_defs()
            .into_iter()
            .filter(|item| item.default_restore)
            .map(|item| item.key.to_string())
            .collect::<HashSet<_>>()
    } else {
        input.item_keys.into_iter().collect::<HashSet<_>>()
    };

    let selected_defs = history_item_defs()
        .into_iter()
        .filter(|item| selected_keys.contains(item.key))
        .filter(|item| source.join(item.relative_path).exists())
        .collect::<Vec<_>>();

    if selected_defs.is_empty() {
        return Err(
            "No repairable history data was found in the selected source profile".to_string(),
        );
    }

    terminate_claude_processes().map_err(|error| error.to_string())?;
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;
    let backup_path = history_backup_root().join(format!("repair-{}", unix_millis()));
    fs::create_dir_all(&backup_path).map_err(|error| error.to_string())?;

    let mut copied_files = 0;
    let mut copied_bytes = 0;
    let mut restored_items = Vec::new();

    for definition in selected_defs {
        let source_item = source.join(definition.relative_path);
        let target_item = target.join(definition.relative_path);
        let backup_item = backup_path.join(definition.relative_path);

        if target_item.exists() {
            copy_path(
                &target_item,
                &backup_item,
                &mut copied_files,
                &mut copied_bytes,
            )
            .map_err(|error| format!("Backup failed for {}: {error}", definition.label))?;
            remove_path(&target_item)
                .map_err(|error| format!("Unable to replace {}: {error}", definition.label))?;
        }

        copy_path(
            &source_item,
            &target_item,
            &mut copied_files,
            &mut copied_bytes,
        )
        .map_err(|error| format!("Restore failed for {}: {error}", definition.label))?;
        restored_items.push(definition.label.to_string());
    }

    Ok(HistoryRepairResult {
        ok: true,
        source_path: path_string(&source),
        target_path: path_string(&target),
        backup_path: path_string(&backup_path),
        copied_files,
        copied_bytes,
        restored_items,
        message: "History repair finished. Relaunch Claude Desktop to reload restored local data."
            .to_string(),
        scan: history_scan(),
    })
}

fn history_backup_root() -> PathBuf {
    config_path()
        .parent()
        .map(|path| path.join("history-backups"))
        .unwrap_or_else(|| PathBuf::from("history-backups"))
}

fn file_tree_stats(path: &Path) -> FileTreeStats {
    let mut stats = FileTreeStats::default();
    collect_file_tree_stats(path, &mut stats);
    stats
}

fn collect_file_tree_stats(path: &Path, stats: &mut FileTreeStats) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    if let Ok(modified) = metadata.modified() {
        if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
            stats.latest_write_ms =
                max_optional_millis(stats.latest_write_ms, Some(duration.as_millis()));
        }
    }
    if metadata.is_file() {
        stats.file_count += 1;
        stats.total_bytes = stats.total_bytes.saturating_add(metadata.len());
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        collect_file_tree_stats(&entry.path(), stats);
    }
}

fn copy_path(
    source: &Path,
    target: &Path,
    copied_files: &mut u64,
    copied_bytes: &mut u64,
) -> std::io::Result<()> {
    let metadata = fs::metadata(source)?;
    if metadata.is_file() {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = fs::copy(source, target)?;
        *copied_files += 1;
        *copied_bytes = (*copied_bytes).saturating_add(bytes);
        return Ok(());
    }

    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_child = entry.path();
        let target_child = target.join(entry.file_name());
        copy_path(&source_child, &target_child, copied_files, copied_bytes)?;
    }
    Ok(())
}

fn remove_path(path: &Path) -> std::io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else if path.exists() {
        fs::remove_file(path)
    } else {
        Ok(())
    }
}

fn same_path(a: &Path, b: &Path) -> bool {
    normalized_path_key(a) == normalized_path_key(b)
}

fn normalized_path_key(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('/', "\\")
        .to_ascii_lowercase()
}

fn max_optional_millis(a: Option<u128>, b: Option<u128>) -> Option<u128> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn current_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "win32"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    }
}

fn read_claude_3p_deployment_mode() -> Option<String> {
    let path = claude_3p_user_data_dir().ok()?.join(CLAUDE_3P_CONFIG_FILE);
    read_json_file(&path)
        .ok()?
        .get("deploymentMode")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn read_json_file(path: &Path) -> anyhow::Result<Value> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

fn system_readiness() -> SystemReadiness {
    let install = detect_claude_install();
    SystemReadiness {
        is_windows: cfg!(target_os = "windows"),
        is_admin: is_running_as_admin(),
        os_name: windows_os_caption(),
        os_build: windows_os_build(),
        virtualization_firmware_enabled: virtualization_firmware_enabled(),
        hypervisor_present: hypervisor_present(),
        hypervisor_launch_type: hypervisor_launch_type(),
        claude_installed: install.is_some(),
        claude_modern_installer: install
            .as_ref()
            .is_some_and(|install| install.app_user_model_id.is_some())
            || claude_appx_package_name().is_some(),
        claude_appx_package: claude_appx_package_name(),
        virtual_machine_platform: windows_feature_state("VirtualMachinePlatform"),
        hypervisor_platform: windows_feature_state("HypervisorPlatform"),
        hyper_v: windows_feature_state("Microsoft-Hyper-V-All"),
        reboot_required: reboot_required_by_windows(),
    }
}

fn system_readiness_placeholder(install: Option<&ClaudeInstall>) -> SystemReadiness {
    SystemReadiness {
        is_windows: cfg!(target_os = "windows"),
        is_admin: false,
        os_name: None,
        os_build: None,
        virtualization_firmware_enabled: None,
        hypervisor_present: None,
        hypervisor_launch_type: None,
        claude_installed: install.is_some(),
        claude_modern_installer: install.is_some_and(|install| install.app_user_model_id.is_some()),
        claude_appx_package: None,
        virtual_machine_platform: None,
        hypervisor_platform: None,
        hyper_v: None,
        reboot_required: false,
    }
}

fn run_powershell_script(script: &str) -> Result<std::process::Output, String> {
    let script = format!(
        "$__claudePlusUtf8 = New-Object System.Text.UTF8Encoding $false; \
         [Console]::OutputEncoding = $__claudePlusUtf8; \
         $OutputEncoding = $__claudePlusUtf8; \
         {script}"
    );
    let mut command = Command::new("powershell.exe");
    hide_child_console(&mut command);
    command
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .stdin(Stdio::null())
        .output()
        .map_err(|error| error.to_string())
}

fn hide_child_console(command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

fn windows_feature_state(feature_name: &str) -> Option<String> {
    if !cfg!(target_os = "windows") {
        return None;
    }
    let feature_name = feature_name.replace('\'', "''");
    let script = format!(
        "(Get-WindowsOptionalFeature -Online -FeatureName '{feature_name}').State.ToString()"
    );
    let output = run_powershell_script(&script).ok()?;
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn claude_appx_package_name() -> Option<String> {
    if !cfg!(target_os = "windows") {
        return None;
    }
    let output = run_powershell_script(
        "Get-AppxPackage Claude | Select-Object -First 1 -ExpandProperty PackageFullName",
    )
    .ok()?;
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn powershell_trimmed(script: &str) -> Option<String> {
    let output = run_powershell_script(script).ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn windows_os_caption() -> Option<String> {
    if !cfg!(target_os = "windows") {
        return None;
    }
    powershell_trimmed("(Get-CimInstance Win32_OperatingSystem).Caption")
}

fn windows_os_build() -> Option<String> {
    if !cfg!(target_os = "windows") {
        return None;
    }
    powershell_trimmed("(Get-CimInstance Win32_OperatingSystem).BuildNumber")
}

fn virtualization_firmware_enabled() -> Option<bool> {
    if !cfg!(target_os = "windows") {
        return None;
    }
    powershell_trimmed(
        "Get-CimInstance Win32_Processor | Select-Object -First 1 -ExpandProperty VirtualizationFirmwareEnabled",
    )
    .and_then(|value| parse_bool_string(&value))
}

fn hypervisor_present() -> Option<bool> {
    if !cfg!(target_os = "windows") {
        return None;
    }
    powershell_trimmed("(Get-CimInstance Win32_ComputerSystem).HypervisorPresent")
        .and_then(|value| parse_bool_string(&value))
}

fn hypervisor_launch_type() -> Option<String> {
    if !cfg!(target_os = "windows") {
        return None;
    }
    let output = run_powershell_script(
        "(bcdedit /enum '{current}' | Select-String -Pattern 'hypervisorlaunchtype').Line",
    )
    .ok()?;
    let raw = String::from_utf8_lossy(&output.stdout);
    raw.lines()
        .find_map(|line| line.split_whitespace().last().map(ToString::to_string))
}

fn parse_bool_string(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" => Some(true),
        "false" | "no" | "0" => Some(false),
        _ => None,
    }
}

fn feature_state_enabled(state: &Option<String>) -> bool {
    state
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("enabled"))
}

fn vmp_enablement_message(
    system: &SystemReadiness,
    command_success: bool,
    stdout: &str,
    stderr: &str,
) -> String {
    if !system.is_admin {
        return "Administrator permission is required. Relaunch Claude++ as administrator, then run this again.".to_string();
    }
    if system.virtualization_firmware_enabled == Some(false) {
        return "CPU virtualization is disabled in firmware/BIOS. Enable VT-x or SVM first, then restart Windows.".to_string();
    }
    if feature_state_enabled(&system.virtual_machine_platform) {
        if system.hypervisor_present == Some(false) {
            return "Virtual Machine Platform is enabled. Restart Windows so the hypervisor can start.".to_string();
        }
        return "Virtual Machine Platform is enabled.".to_string();
    }
    if system.reboot_required {
        return "Windows staged virtualization components but has not finished enabling them. Restart Windows, then run this check again.".to_string();
    }
    let combined = format!("{stdout}\n{stderr}");
    if combined.contains("Parent features must be enabled")
        || combined.contains("可能未启用必要的父功能")
    {
        return "Windows reports that parent virtualization features are not enabled yet. Restart first; if it still fails, enable Hyper-V / Windows Hypervisor Platform in Windows Features.".to_string();
    }
    if combined.contains("Error: 50") {
        return "Windows returned DISM Error 50 while enabling virtualization features. This usually means a parent feature is pending, blocked, or the component store needs a restart/repair.".to_string();
    }
    if command_success {
        "Virtual Machine Platform enablement was requested. Restart Windows to finish.".to_string()
    } else {
        "Virtual Machine Platform enablement failed. Check Windows optional features, system policy, or component store health.".to_string()
    }
}

fn reboot_required_by_windows() -> bool {
    if !cfg!(target_os = "windows") {
        return false;
    }
    for key in [
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending",
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired",
    ] {
        let mut command = Command::new("reg");
        hide_child_console(&mut command);
        let status = command
            .args(["query", &format!(r"HKLM\{key}")])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if status.is_ok_and(|status| status.success()) {
            return true;
        }
    }
    false
}

fn is_running_as_admin() -> bool {
    if !cfg!(target_os = "windows") {
        return false;
    }
    let mut command = Command::new("net");
    hide_child_console(&mut command);
    command
        .arg("session")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn write_json_file(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(value).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

fn verify_launch(before: GatewaySnapshot, expect_gateway: bool) -> LaunchVerification {
    std::thread::sleep(Duration::from_millis(1200));
    let after = gateway_snapshot();
    let request_delta = after.request_count.saturating_sub(before.request_count);
    let forwarded_delta = after.forwarded_count.saturating_sub(before.forwarded_count);
    let deployment_mode = read_claude_3p_deployment_mode();
    let claude_log_evidence = recent_claude_log_evidence();
    let gateway_hit = request_delta > 0;
    let upstream_status = after.last_upstream_status;
    let has_3p_log = claude_log_evidence
        .iter()
        .any(|line| line.contains("[custom-3p]") || line.contains("deploymentMode"));
    let has_login_log = claude_log_evidence
        .iter()
        .any(|line| line.contains("claude.ai/login") || line.contains("api.anthropic.com"));
    let verdict = if expect_gateway
        && gateway_hit
        && deployment_mode.as_deref() == Some("3p")
        && upstream_status.is_some_and(|status| status == 401 || status == 403)
    {
        "provider_credentials_rejected".to_string()
    } else if expect_gateway
        && gateway_hit
        && deployment_mode.as_deref() == Some("3p")
        && upstream_status.is_some_and(|status| !(200..300).contains(&status))
    {
        "gateway_hit_upstream_error".to_string()
    } else if expect_gateway && gateway_hit && deployment_mode.as_deref() == Some("3p") {
        "verified_gateway_hit".to_string()
    } else if !expect_gateway && deployment_mode.as_deref() == Some("3p") && has_3p_log {
        "verified_direct_3p_config".to_string()
    } else if deployment_mode.as_deref() == Some("3p") && has_3p_log {
        "3p_config_applied_but_no_gateway_request_yet".to_string()
    } else if has_login_log {
        "still_official_login_or_1p_network".to_string()
    } else {
        "not_verified".to_string()
    };

    LaunchVerification {
        gateway_hit,
        request_delta,
        forwarded_delta,
        last_request_path: after.last_request_path,
        last_upstream_status: after.last_upstream_status,
        last_upstream_error: after.last_upstream_error,
        deployment_mode,
        claude_log_evidence,
        verdict,
    }
}

fn gateway_snapshot() -> GatewaySnapshot {
    let runtime = gateway_state().lock().ok().and_then(|guard| guard.clone());
    if let Some(runtime) = runtime {
        let counters = &runtime.shared.counters;
        return GatewaySnapshot {
            request_count: counters.request_count.load(Ordering::SeqCst),
            forwarded_count: counters.forwarded_count.load(Ordering::SeqCst),
            last_request_path: counters
                .last_request_path
                .lock()
                .ok()
                .and_then(|value| value.clone()),
            last_request_at_ms: counters
                .last_request_at_ms
                .lock()
                .ok()
                .and_then(|value| *value),
            last_upstream_status: counters
                .last_upstream_status
                .lock()
                .ok()
                .and_then(|value| *value),
            last_upstream_error: counters
                .last_upstream_error
                .lock()
                .ok()
                .and_then(|value| value.clone()),
        };
    }
    GatewaySnapshot {
        request_count: 0,
        forwarded_count: 0,
        last_request_path: None,
        last_request_at_ms: None,
        last_upstream_status: None,
        last_upstream_error: None,
    }
}

fn recent_claude_log_evidence() -> Vec<String> {
    let mut evidence = Vec::new();
    for path in claude_main_log_paths() {
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let label = path
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("log");
        evidence.extend(
            raw.lines()
                .rev()
                .filter(|line| {
                    line.contains("[custom-3p]")
                        || line.contains("deploymentMode")
                        || line.contains("ConfigHealth")
                        || line.contains("claude.ai/login")
                        || line.contains("api.anthropic.com")
                        || line.contains("gateway")
                        || line.contains("Gateway /v1/models")
                        || line.contains("credential rejected")
                })
                .take(8)
                .map(|line| format!("[{label}] {}", redact_log_line(line))),
        );
    }
    evidence
        .into_iter()
        .rev()
        .take(16)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn redact_log_line(line: &str) -> String {
    line.replace("Authorization", "Auth")
        .replace("oauth:tokenCache", "oauth:tokenCache=<redacted>")
}

fn claude_main_log_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if cfg!(target_os = "windows") {
        if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
            paths.push(
                PathBuf::from(local_appdata)
                    .join(CLAUDE_3P_DIR_NAME)
                    .join("logs")
                    .join("main.log"),
            );
        }
        if let Some(appdata) = std::env::var_os("APPDATA") {
            paths.push(
                PathBuf::from(appdata)
                    .join("Claude")
                    .join("logs")
                    .join("main.log"),
            );
        }
        return paths;
    }
    if cfg!(target_os = "macos") {
        if let Some(home) = std::env::var_os("HOME") {
            let support = PathBuf::from(&home)
                .join("Library")
                .join("Application Support");
            paths.push(
                support
                    .join(CLAUDE_3P_DIR_NAME)
                    .join("logs")
                    .join("main.log"),
            );
            paths.push(
                PathBuf::from(home)
                    .join("Library")
                    .join("Logs")
                    .join("Claude")
                    .join("main.log"),
            );
        }
        return paths;
    }
    paths
}

fn terminate_claude_processes() -> anyhow::Result<()> {
    if cfg!(target_os = "windows") {
        let mut command = Command::new("taskkill");
        hide_child_console(&mut command);
        let _ = command
            .args(["/IM", "claude.exe", "/T", "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        std::thread::sleep(Duration::from_millis(900));
        return Ok(());
    }
    if cfg!(target_os = "macos") {
        let _ = Command::new("pkill")
            .args(["-x", "Claude"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        std::thread::sleep(Duration::from_millis(900));
    }
    Ok(())
}

fn ensure_gateway_runtime(config: &AppConfig) -> Result<(), String> {
    if !config_uses_local_gateway(config) {
        return Ok(());
    }

    let provider = gateway_provider(config)?;
    let port = config.gateway.port;

    {
        let mut guard = gateway_state()
            .lock()
            .map_err(|_| "Gateway state lock was poisoned".to_string())?;
        if let Some(runtime) = guard.as_mut() {
            if runtime.port == port {
                runtime.provider_id = provider.id.clone();
                runtime.provider_name = provider.name.clone();
                runtime.target_base_url = provider.base_url.clone();
                if let Ok(mut active_provider) = runtime.shared.provider.lock() {
                    *active_provider = provider;
                }
                return Ok(());
            }
        }
    }

    if is_port_bound(port) {
        let guard = gateway_state()
            .lock()
            .map_err(|_| "Gateway state lock was poisoned".to_string())?;
        let owned_by_us = guard.as_ref().is_some_and(|runtime| runtime.port == port);
        if !owned_by_us {
            drop(guard);
            if let Some(health) = probe_local_gateway_health(port) {
                let target = health.target.clone().unwrap_or_default();
                if same_gateway_target(&target, &provider.base_url) {
                    return Ok(());
                }
                let owner = health.provider.as_deref().unwrap_or("Claude++ Gateway");
                return Err(format!(
                    "Gateway port {port} is already used by {owner} -> {target}. Stop that Gateway or choose another port."
                ));
            }
            wake_gateway_listener(port);
            if is_port_bound(port) {
                return Err(format!("Gateway port {port} is already in use"));
            }
        }
    }

    let server = Server::http((GATEWAY_BIND_HOST, port)).map_err(|error| error.to_string())?;
    let shared = Arc::new(GatewayShared {
        provider: Mutex::new(provider.clone()),
        counters: GatewayCounters::default(),
        stop_requested: AtomicBool::new(false),
    });
    let runtime = GatewayRuntime {
        port,
        provider_id: provider.id.clone(),
        provider_name: provider.name.clone(),
        target_base_url: provider.base_url.clone(),
        shared: shared.clone(),
    };
    let thread_runtime = runtime.clone();
    std::thread::spawn(move || run_gateway_server(server, shared, thread_runtime));

    let mut guard = gateway_state()
        .lock()
        .map_err(|_| "Gateway state lock was poisoned".to_string())?;
    *guard = Some(runtime);
    Ok(())
}

fn run_gateway_server(server: Server, shared: Arc<GatewayShared>, runtime: GatewayRuntime) {
    eprintln!(
        "[Claude++] gateway listening on {} -> {} ({})",
        gateway_url(runtime.port),
        runtime.target_base_url,
        runtime.provider_name
    );

    for mut request in server.incoming_requests() {
        if shared.stop_requested.load(Ordering::SeqCst) {
            break;
        }
        if request.url() == "/__claude_plus/health" {
            let provider = shared
                .provider
                .lock()
                .ok()
                .map(|provider| provider.clone())
                .unwrap_or(GatewayProvider {
                    id: runtime.provider_id.clone(),
                    name: runtime.provider_name.clone(),
                    base_url: runtime.target_base_url.clone(),
                    api_key: String::new(),
                    protocol: PROVIDER_PROTOCOL_ANTHROPIC.to_string(),
                    model_mappings: Vec::new(),
                });
            let body = json!({
                "ok": true,
                "provider": provider.name,
                "target": provider.base_url,
                "requests": shared.counters.request_count.load(Ordering::SeqCst),
                "forwarded": shared.counters.forwarded_count.load(Ordering::SeqCst),
                "last_upstream_status": shared
                    .counters
                    .last_upstream_status
                    .lock()
                    .ok()
                    .and_then(|value| *value),
                "last_upstream_error": shared
                    .counters
                    .last_upstream_error
                    .lock()
                    .ok()
                    .and_then(|value| value.clone()),
            })
            .to_string();
            let _ = request.respond(json_response(StatusCode(200), body));
            continue;
        }

        if request.method() == &Method::Options {
            let _ = request.respond(cors_response(StatusCode(204), Vec::new()));
            continue;
        }

        let method = request.method().clone();
        let url = request.url().to_string();
        shared.counters.request_count.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut path) = shared.counters.last_request_path.lock() {
            *path = Some(url.clone());
        }
        if let Ok(mut at) = shared.counters.last_request_at_ms.lock() {
            *at = Some(now_millis());
        }
        let provider = match shared.provider.lock() {
            Ok(provider) => provider.clone(),
            Err(_) => {
                record_gateway_upstream_result(
                    &shared.counters,
                    Some(500),
                    Some("Claude++ gateway state lock failed".to_string()),
                );
                let _ = request.respond(text_response(
                    StatusCode(500),
                    b"Claude++ gateway state lock failed".to_vec(),
                ));
                continue;
            }
        };
        let response = match forward_gateway_request(&method, &url, request.as_reader(), &provider)
        {
            Ok(result) => {
                let status = result.status;
                let error = if (200..300).contains(&status) {
                    shared
                        .counters
                        .forwarded_count
                        .fetch_add(1, Ordering::SeqCst);
                    None
                } else {
                    result
                        .upstream_error
                        .or_else(|| Some(format!("Upstream returned HTTP {status}")))
                };
                record_gateway_upstream_result(&shared.counters, Some(status), error);
                result.response
            }
            Err(error) => {
                record_gateway_upstream_result(&shared.counters, Some(502), Some(error.clone()));
                text_response(
                    StatusCode(502),
                    format!("Claude++ gateway error: {error}").into_bytes(),
                )
            }
        };
        let _ = request.respond(response);
    }
}

fn forward_gateway_request(
    method: &Method,
    path_and_query: &str,
    reader: &mut dyn Read,
    provider: &GatewayProvider,
) -> Result<GatewayForwardResult, String> {
    let mut body = Vec::new();
    reader
        .read_to_end(&mut body)
        .map_err(|error| error.to_string())?;

    if provider.protocol == PROVIDER_PROTOCOL_OPENAI {
        return forward_openai_compatible_request(method, path_and_query, &body, provider);
    }

    forward_raw_gateway_request(method, path_and_query, &body, provider)
}

fn forward_raw_gateway_request(
    method: &Method,
    path_and_query: &str,
    body: &[u8],
    provider: &GatewayProvider,
) -> Result<GatewayForwardResult, String> {
    let target = join_gateway_url(&provider.base_url, path_and_query);
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(180))
        .build();
    let method_text = method.as_str();
    let mut request = agent.request(method_text, &target);

    if !provider.api_key.is_empty() {
        request = request
            .set("Authorization", &format!("Bearer {}", provider.api_key))
            .set("x-api-key", &provider.api_key);
    }
    request = request
        .set("anthropic-version", "2023-06-01")
        .set("content-type", "application/json")
        .set("accept", "application/json");

    let upstream = if body.is_empty() {
        request.call()
    } else {
        request.send_bytes(body)
    };

    match upstream {
        Ok(response) => {
            let status = response.status();
            let response_body = read_ureq_response(response)?;
            let upstream_error = upstream_error_summary(status, &response_body);
            Ok(GatewayForwardResult {
                status,
                upstream_error,
                response: jsonish_response(StatusCode(status), response_body),
            })
        }
        Err(ureq::Error::Status(status, response)) => {
            let response_body = read_ureq_response(response)?;
            let upstream_error = upstream_error_summary(status, &response_body);
            Ok(GatewayForwardResult {
                status,
                upstream_error,
                response: jsonish_response(StatusCode(status), response_body),
            })
        }
        Err(error) => Err(error.to_string()),
    }
}

fn forward_openai_compatible_request(
    method: &Method,
    path_and_query: &str,
    body: &[u8],
    provider: &GatewayProvider,
) -> Result<GatewayForwardResult, String> {
    let path = path_and_query
        .split('?')
        .next()
        .unwrap_or(path_and_query)
        .trim_end_matches('/');
    match (method.as_str(), path) {
        ("GET", "/v1/models") => forward_openai_models(provider),
        ("POST", "/v1/messages/count_tokens") => Ok(openai_count_tokens_response(body)),
        ("POST", "/v1/messages") => forward_openai_chat_completion(body, provider),
        _ => forward_raw_gateway_request(method, path_and_query, body, provider),
    }
}

fn forward_openai_models(provider: &GatewayProvider) -> Result<GatewayForwardResult, String> {
    let provider_for_routes = ApiProvider {
        id: provider.id.clone(),
        name: provider.name.clone(),
        app_type: "claude".to_string(),
        source: "gateway".to_string(),
        base_url: provider.base_url.clone(),
        api_key: provider.api_key.clone(),
        protocol: provider.protocol.clone(),
        model_mappings: provider.model_mappings.clone(),
        enabled: true,
    };
    let routes = openai_route_models_for_provider(&provider_for_routes, &provider.api_key);
    let data = routes
        .iter()
        .map(|route| {
            json!({
                "type": "model",
                "id": route.claude_route,
                "display_name": route.label,
                "created_at": "2026-01-01T00:00:00Z",
            })
        })
        .collect::<Vec<_>>();
    let body = json!({
        "data": data,
        "has_more": false,
        "first_id": routes.first().map(|route| route.claude_route.clone()).unwrap_or_default(),
        "last_id": routes.last().map(|route| route.claude_route.clone()).unwrap_or_default(),
    })
    .to_string();
    Ok(GatewayForwardResult {
        status: 200,
        upstream_error: None,
        response: json_response(StatusCode(200), body),
    })
}

fn openai_count_tokens_response(body: &[u8]) -> GatewayForwardResult {
    let value = serde_json::from_slice::<Value>(body).unwrap_or(Value::Null);
    let input_tokens = estimate_anthropic_tokens(&value);
    GatewayForwardResult {
        status: 200,
        upstream_error: None,
        response: json_response(
            StatusCode(200),
            json!({
                "input_tokens": input_tokens,
            })
            .to_string(),
        ),
    }
}

fn forward_openai_chat_completion(
    body: &[u8],
    provider: &GatewayProvider,
) -> Result<GatewayForwardResult, String> {
    let anthropic_request: Value =
        serde_json::from_slice(body).map_err(|error| format!("Invalid JSON request: {error}"))?;
    let wants_stream = anthropic_request
        .get("stream")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let openai_request = anthropic_messages_to_openai_chat(&anthropic_request, provider)?;
    let (status, response_body) =
        send_openai_json_request(provider, "/v1/chat/completions", &openai_request)?;
    let upstream_error = upstream_error_summary(status, &response_body);
    if !(200..300).contains(&status) {
        return Ok(GatewayForwardResult {
            status,
            upstream_error,
            response: jsonish_response(StatusCode(status), response_body),
        });
    }

    let message = openai_chat_to_anthropic_message(&anthropic_request, &response_body)?;
    let response = if wants_stream {
        event_stream_response(
            StatusCode(200),
            anthropic_message_to_sse(&message).into_bytes(),
        )
    } else {
        json_response(StatusCode(200), message.to_string())
    };
    Ok(GatewayForwardResult {
        status: 200,
        upstream_error: None,
        response,
    })
}

fn send_openai_json_request(
    provider: &GatewayProvider,
    path: &str,
    payload: &Value,
) -> Result<(u16, Vec<u8>), String> {
    let target = join_gateway_url(&provider.base_url, path);
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(180))
        .build();
    let mut request = agent
        .post(&target)
        .set("content-type", "application/json")
        .set("accept", "application/json");
    if !provider.api_key.trim().is_empty() {
        request = request
            .set(
                "Authorization",
                &format!("Bearer {}", provider.api_key.trim()),
            )
            .set("x-api-key", provider.api_key.trim());
    }
    match request.send_string(&payload.to_string()) {
        Ok(response) => {
            let status = response.status();
            Ok((status, read_ureq_response(response)?))
        }
        Err(ureq::Error::Status(status, response)) => Ok((status, read_ureq_response(response)?)),
        Err(error) => Err(error.to_string()),
    }
}

fn anthropic_messages_to_openai_chat(
    request: &Value,
    provider: &GatewayProvider,
) -> Result<Value, String> {
    let requested_model = request
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let model = select_openai_model(requested_model, provider);
    let mut messages = Vec::new();

    if let Some(system) = request.get("system") {
        let content = extract_anthropic_text(system);
        if !content.trim().is_empty() {
            messages.push(json!({
                "role": "system",
                "content": content,
            }));
        }
    }

    for message in request
        .get("messages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        append_openai_messages_from_anthropic_message(&mut messages, message);
    }

    let mut payload = json!({
        "model": model,
        "messages": messages,
        "stream": false,
        "max_tokens": request.get("max_tokens").and_then(Value::as_u64).unwrap_or(1024),
    });

    if let Some(temperature) = request.get("temperature") {
        payload["temperature"] = temperature.clone();
    }
    if let Some(top_p) = request.get("top_p") {
        payload["top_p"] = top_p.clone();
    }
    if let Some(tools) = request.get("tools").and_then(Value::as_array) {
        let converted = tools
            .iter()
            .filter_map(anthropic_tool_to_openai_tool)
            .collect::<Vec<_>>();
        if !converted.is_empty() {
            payload["tools"] = Value::Array(converted);
            payload["tool_choice"] = Value::String("auto".to_string());
        }
    }

    Ok(payload)
}

fn append_openai_messages_from_anthropic_message(messages: &mut Vec<Value>, message: &Value) {
    let role = message
        .get("role")
        .and_then(Value::as_str)
        .unwrap_or("user");
    let content = message.get("content").unwrap_or(&Value::Null);

    if role == "assistant" {
        let text = extract_anthropic_text(content);
        let tool_calls = extract_anthropic_tool_calls(content);
        let mut item = json!({
            "role": "assistant",
            "content": if text.trim().is_empty() { Value::Null } else { Value::String(text) },
        });
        if !tool_calls.is_empty() {
            item["tool_calls"] = Value::Array(tool_calls);
        }
        messages.push(item);
        return;
    }

    if let Some(blocks) = content.as_array() {
        let mut user_blocks = Vec::new();
        for block in blocks {
            match block.get("type").and_then(Value::as_str) {
                Some("tool_result") => {
                    let tool_call_id = block
                        .get("tool_use_id")
                        .and_then(Value::as_str)
                        .unwrap_or("toolu_claude_plus");
                    messages.push(json!({
                        "role": "tool",
                        "tool_call_id": tool_call_id,
                        "content": extract_anthropic_text(block.get("content").unwrap_or(&Value::Null)),
                    }));
                }
                Some("image") => {
                    if let Some(image) = anthropic_image_to_openai_content(block) {
                        user_blocks.push(image);
                    }
                }
                _ => {
                    let text = extract_anthropic_text(block);
                    if !text.trim().is_empty() {
                        user_blocks.push(json!({ "type": "text", "text": text }));
                    }
                }
            }
        }
        if !user_blocks.is_empty() {
            messages.push(json!({
                "role": "user",
                "content": user_blocks,
            }));
        }
    } else {
        let text = extract_anthropic_text(content);
        if !text.trim().is_empty() {
            messages.push(json!({
                "role": "user",
                "content": text,
            }));
        }
    }
}

fn anthropic_tool_to_openai_tool(tool: &Value) -> Option<Value> {
    let name = tool.get("name").and_then(Value::as_str)?;
    Some(json!({
        "type": "function",
        "function": {
            "name": name,
            "description": tool.get("description").and_then(Value::as_str).unwrap_or_default(),
            "parameters": tool.get("input_schema").cloned().unwrap_or_else(|| json!({"type": "object"})),
        }
    }))
}

fn extract_anthropic_tool_calls(content: &Value) -> Vec<Value> {
    content
        .as_array()
        .map(|blocks| {
            blocks
                .iter()
                .filter_map(|block| {
                    if block.get("type").and_then(Value::as_str) != Some("tool_use") {
                        return None;
                    }
                    let id = block
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or("toolu_claude_plus");
                    let name = block.get("name").and_then(Value::as_str)?;
                    let input = block.get("input").cloned().unwrap_or_else(|| json!({}));
                    Some(json!({
                        "id": id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": input.to_string(),
                        }
                    }))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn anthropic_image_to_openai_content(block: &Value) -> Option<Value> {
    let source = block.get("source")?;
    let media_type = source.get("media_type").and_then(Value::as_str)?;
    let data = source.get("data").and_then(Value::as_str)?;
    Some(json!({
        "type": "image_url",
        "image_url": {
            "url": format!("data:{media_type};base64,{data}"),
        }
    }))
}

fn extract_anthropic_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .map(extract_anthropic_text)
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(_) => {
            if let Some(text) = value.get("text").and_then(Value::as_str) {
                text.to_string()
            } else if let Some(content) = value.get("content") {
                extract_anthropic_text(content)
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

fn select_openai_model(requested_model: &str, provider: &GatewayProvider) -> String {
    let requested = requested_model.trim();
    let lower = requested.to_ascii_lowercase();
    if !requested.is_empty()
        && !lower.starts_with("claude")
        && !lower.starts_with("anthropic/claude")
        && lower != "opus"
        && lower != "sonnet"
        && lower != "haiku"
    {
        return requested.to_string();
    }

    let provider_for_routes = ApiProvider {
        id: provider.id.clone(),
        name: provider.name.clone(),
        app_type: "claude".to_string(),
        source: "gateway".to_string(),
        base_url: provider.base_url.clone(),
        api_key: provider.api_key.clone(),
        protocol: provider.protocol.clone(),
        model_mappings: provider.model_mappings.clone(),
        enabled: true,
    };
    let routes = openai_route_models_for_provider(&provider_for_routes, &provider.api_key);
    if let Some(route) = routes
        .iter()
        .find(|route| route.claude_route.eq_ignore_ascii_case(requested))
    {
        return route.target_model.clone();
    }

    if lower.contains("haiku") {
        routes
            .iter()
            .find(|route| {
                let value = route.target_model.to_ascii_lowercase();
                value.contains("mini") || value.contains("haiku") || value.contains("lite")
            })
            .map(|route| route.target_model.clone())
            .or_else(|| routes.last().map(|route| route.target_model.clone()))
            .unwrap_or_else(|| "gpt-5.4-mini".to_string())
    } else if lower.contains("sonnet") {
        routes
            .iter()
            .find(|route| {
                let value = route.target_model.to_ascii_lowercase();
                value.contains("5.4") || value.contains("sonnet") || value.contains("codex")
            })
            .map(|route| route.target_model.clone())
            .or_else(|| routes.get(1).map(|route| route.target_model.clone()))
            .or_else(|| routes.first().map(|route| route.target_model.clone()))
            .unwrap_or_else(|| "gpt-5.4".to_string())
    } else {
        routes
            .first()
            .map(|route| route.target_model.clone())
            .unwrap_or_else(|| "gpt-5.5".to_string())
    }
}

fn openai_chat_to_anthropic_message(
    request: &Value,
    response_body: &[u8],
) -> Result<Value, String> {
    let response: Value = serde_json::from_slice(response_body)
        .map_err(|error| format!("OpenAI response was not JSON: {error}"))?;
    let choice = response
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .ok_or_else(|| "OpenAI response did not include choices".to_string())?;
    let message = choice.get("message").unwrap_or(&Value::Null);
    let mut content = Vec::new();

    let text = message
        .get("content")
        .map(extract_openai_content_text)
        .unwrap_or_default();
    if !text.trim().is_empty() {
        content.push(json!({
            "type": "text",
            "text": text,
        }));
    }

    if let Some(tool_calls) = message.get("tool_calls").and_then(Value::as_array) {
        for call in tool_calls {
            let id = call
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("toolu_claude_plus");
            let function = call.get("function").unwrap_or(&Value::Null);
            let name = function
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("tool");
            let arguments = function
                .get("arguments")
                .and_then(Value::as_str)
                .unwrap_or("{}");
            let input = serde_json::from_str::<Value>(arguments).unwrap_or_else(|_| json!({}));
            content.push(json!({
                "type": "tool_use",
                "id": id,
                "name": name,
                "input": input,
            }));
        }
    }

    if content.is_empty() {
        content.push(json!({
            "type": "text",
            "text": "",
        }));
    }

    let finish_reason = choice
        .get("finish_reason")
        .and_then(Value::as_str)
        .unwrap_or("stop");
    let stop_reason = match finish_reason {
        "length" => "max_tokens",
        "tool_calls" | "function_call" => "tool_use",
        _ => "end_turn",
    };
    let usage = response.get("usage").unwrap_or(&Value::Null);
    Ok(json!({
        "id": format!("msg_claude_plus_{}", unix_millis()),
        "type": "message",
        "role": "assistant",
        "content": content,
        "model": response
            .get("model")
            .and_then(Value::as_str)
            .or_else(|| request.get("model").and_then(Value::as_str))
            .unwrap_or("openai-compatible"),
        "stop_reason": stop_reason,
        "stop_sequence": Value::Null,
        "usage": {
            "input_tokens": usage.get("prompt_tokens").and_then(Value::as_u64).unwrap_or(0),
            "output_tokens": usage.get("completion_tokens").and_then(Value::as_u64).unwrap_or(0),
        }
    }))
}

fn extract_openai_content_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                item.get("text")
                    .and_then(Value::as_str)
                    .or_else(|| item.get("content").and_then(Value::as_str))
                    .map(ToString::to_string)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn anthropic_message_to_sse(message: &Value) -> String {
    let mut sse = String::new();
    let mut start = message.clone();
    start["content"] = Value::Array(Vec::new());
    push_sse_event(
        &mut sse,
        "message_start",
        json!({ "type": "message_start", "message": start }),
    );

    for (index, block) in message
        .get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let mut start_block = block.clone();
        if start_block.get("type").and_then(Value::as_str) == Some("text") {
            start_block["text"] = Value::String(String::new());
        }
        push_sse_event(
            &mut sse,
            "content_block_start",
            json!({ "type": "content_block_start", "index": index, "content_block": start_block }),
        );
        if block.get("type").and_then(Value::as_str) == Some("text") {
            push_sse_event(
                &mut sse,
                "content_block_delta",
                json!({
                    "type": "content_block_delta",
                    "index": index,
                    "delta": {
                        "type": "text_delta",
                        "text": block.get("text").and_then(Value::as_str).unwrap_or_default(),
                    }
                }),
            );
        }
        push_sse_event(
            &mut sse,
            "content_block_stop",
            json!({ "type": "content_block_stop", "index": index }),
        );
    }

    push_sse_event(
        &mut sse,
        "message_delta",
        json!({
            "type": "message_delta",
            "delta": {
                "stop_reason": message.get("stop_reason").and_then(Value::as_str).unwrap_or("end_turn"),
                "stop_sequence": Value::Null,
            },
            "usage": {
                "output_tokens": message.pointer("/usage/output_tokens").and_then(Value::as_u64).unwrap_or(0),
            }
        }),
    );
    push_sse_event(&mut sse, "message_stop", json!({ "type": "message_stop" }));
    sse
}

fn push_sse_event(buffer: &mut String, event: &str, data: Value) {
    buffer.push_str("event: ");
    buffer.push_str(event);
    buffer.push('\n');
    buffer.push_str("data: ");
    buffer.push_str(&data.to_string());
    buffer.push_str("\n\n");
}

fn estimate_anthropic_tokens(value: &Value) -> u64 {
    let text = extract_anthropic_text(value);
    ((text.chars().count() as u64) / 4).max(1)
}

fn stop_gateway_runtime() {
    let mut wake_port = None;
    if let Ok(mut guard) = gateway_state().lock() {
        if let Some(runtime) = guard.as_ref() {
            runtime.shared.stop_requested.store(true, Ordering::SeqCst);
            wake_port = Some(runtime.port);
        }
        *guard = None;
    }
    if let Some(port) = wake_port {
        wake_gateway_listener(port);
    }
}

fn build_gateway_status(config: &AppConfig, last_error: Option<String>) -> GatewayStatus {
    let runtime = gateway_state().lock().ok().and_then(|guard| guard.clone());
    let snapshot = gateway_snapshot();
    let provider = active_provider(config);
    let enabled = config_uses_local_gateway(config);
    let external_health = if runtime.is_none() && enabled {
        probe_local_gateway_health(config.gateway.port)
    } else {
        None
    };
    GatewayStatus {
        enabled,
        running: runtime.is_some() || external_health.as_ref().is_some_and(|health| health.ok),
        url: gateway_url(config.gateway.port),
        port: config.gateway.port,
        provider_id: runtime
            .as_ref()
            .map(|item| item.provider_id.clone())
            .or_else(|| provider.map(|item| item.id.clone())),
        provider_name: runtime
            .as_ref()
            .map(|item| item.provider_name.clone())
            .or_else(|| {
                external_health
                    .as_ref()
                    .and_then(|health| health.provider.clone())
            })
            .or_else(|| provider.map(|item| item.name.clone())),
        target_base_url: runtime
            .as_ref()
            .map(|item| item.target_base_url.clone())
            .or_else(|| {
                external_health
                    .as_ref()
                    .and_then(|health| health.target.clone())
            })
            .or_else(|| provider.map(|item| item.base_url.clone())),
        request_count: external_health
            .as_ref()
            .and_then(|health| health.requests)
            .unwrap_or(snapshot.request_count),
        forwarded_count: external_health
            .as_ref()
            .and_then(|health| health.forwarded)
            .unwrap_or(snapshot.forwarded_count),
        last_request_path: snapshot.last_request_path,
        last_request_at_ms: snapshot.last_request_at_ms,
        last_upstream_status: external_health
            .as_ref()
            .and_then(|health| health.last_upstream_status)
            .or(snapshot.last_upstream_status),
        last_upstream_error: external_health
            .as_ref()
            .and_then(|health| health.last_upstream_error.clone())
            .or(snapshot.last_upstream_error),
        last_error,
    }
}

fn refresh_gateway_runtime_after_config_change(config: &AppConfig) {
    let runtime_exists = gateway_state()
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(|_| ()))
        .is_some();
    if runtime_exists && config_uses_local_gateway(config) {
        if let Err(error) = ensure_gateway_runtime(config) {
            eprintln!("[Claude++] gateway refresh skipped: {error}");
        }
    }
}

fn gateway_provider(config: &AppConfig) -> Result<GatewayProvider, String> {
    let provider = active_provider(config).ok_or("No enabled active provider was selected")?;
    if provider.base_url.trim().is_empty() {
        return Err("Gateway mode requires an active provider with Base URL".to_string());
    }
    let api_key = provider_key_with_fallback(config, provider);
    Ok(GatewayProvider {
        id: provider.id.clone(),
        name: provider.name.clone(),
        base_url: provider.base_url.trim_end_matches('/').to_string(),
        api_key,
        protocol: effective_provider_protocol(provider),
        model_mappings: provider.model_mappings.clone(),
    })
}

fn provider_key_with_fallback(config: &AppConfig, provider: &ApiProvider) -> String {
    let key = provider.api_key.trim();
    if !key.is_empty() {
        return key.to_string();
    }
    let base_url = provider.base_url.trim_end_matches('/');
    config
        .providers
        .iter()
        .find(|item| {
            item.enabled
                && !item.api_key.trim().is_empty()
                && !base_url.is_empty()
                && item.base_url.trim_end_matches('/') == base_url
        })
        .map(|item| item.api_key.trim().to_string())
        .unwrap_or_default()
}

fn gateway_state() -> &'static Mutex<Option<GatewayRuntime>> {
    GATEWAY_STATE.get_or_init(|| Mutex::new(None))
}

fn gateway_url(port: u16) -> String {
    format!("http://{GATEWAY_BIND_HOST}:{port}")
}

fn probe_local_gateway_health(port: u16) -> Option<LocalGatewayHealth> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_millis(700))
        .build();
    let response = agent
        .get(&format!("{}/__claude_plus/health", gateway_url(port)))
        .call()
        .ok()?;
    let body = read_ureq_response(response).ok()?;
    serde_json::from_slice::<LocalGatewayHealth>(&body).ok()
}

fn wake_gateway_listener(port: u16) {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_millis(250))
        .build();
    let _ = agent
        .get(&format!("{}/__claude_plus/health", gateway_url(port)))
        .call();
    std::thread::sleep(Duration::from_millis(120));
}

fn same_gateway_target(left: &str, right: &str) -> bool {
    left.trim_end_matches('/')
        .eq_ignore_ascii_case(right.trim_end_matches('/'))
}

fn is_port_bound(port: u16) -> bool {
    TcpListener::bind((GATEWAY_BIND_HOST, port)).is_err()
}

fn join_gateway_url(base_url: &str, path_and_query: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let mut path = if path_and_query.starts_with('/') {
        path_and_query.to_string()
    } else {
        format!("/{path_and_query}")
    };
    if base.ends_with("/v1")
        && (path == "/v1" || path.starts_with("/v1/") || path.starts_with("/v1?"))
    {
        path = path.trim_start_matches("/v1").to_string();
        if path.is_empty() {
            path = "/".to_string();
        }
    }
    format!("{base}{path}")
}

fn read_ureq_response(response: ureq::Response) -> Result<Vec<u8>, String> {
    let mut response_body = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut response_body)
        .map_err(|error| error.to_string())?;
    Ok(response_body)
}

fn upstream_error_summary(status: u16, response_body: &[u8]) -> Option<String> {
    if (200..300).contains(&status) {
        return None;
    }
    let body_text = String::from_utf8_lossy(response_body);
    let (code, message) = parse_provider_error(&body_text);
    let mut parts = vec![format!("HTTP {status}")];
    if let Some(code) = code {
        parts.push(code);
    }
    if let Some(message) = message {
        parts.push(message);
    } else {
        let excerpt = redact_body_excerpt(&body_text);
        if !excerpt.is_empty() {
            parts.push(excerpt);
        }
    }
    Some(parts.join(" / "))
}

fn parse_provider_error(body: &str) -> (Option<String>, Option<String>) {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return (None, non_empty_excerpt(body));
    };
    let code = value
        .get("code")
        .or_else(|| value.pointer("/error/code"))
        .or_else(|| value.get("type"))
        .or_else(|| value.pointer("/error/type"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let message = value
        .get("message")
        .or_else(|| value.pointer("/error/message"))
        .or_else(|| value.get("msg"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| non_empty_excerpt(body));
    (code, message)
}

fn parse_model_ids(body: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return Vec::new();
    };
    value
        .get("data")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    if let Some(id) = item.get("id").and_then(Value::as_str) {
                        Some(id.trim().to_string())
                    } else {
                        item.as_str().map(|id| id.trim().to_string())
                    }
                })
                .filter(|id| !id.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn non_empty_excerpt(value: &str) -> Option<String> {
    let excerpt = redact_body_excerpt(value);
    (!excerpt.is_empty()).then_some(excerpt)
}

fn redact_body_excerpt(value: &str) -> String {
    let mut text = value
        .replace('\r', " ")
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for marker in ["sk-", "sk-ant-", "Bearer "] {
        while let Some(index) = text.find(marker) {
            let end = text[index..]
                .find(['"', '\'', ' ', ',', '}'])
                .map(|offset| index + offset)
                .unwrap_or(text.len());
            text.replace_range(index..end, "<redacted>");
        }
    }
    text.chars().take(480).collect()
}

fn record_gateway_upstream_result(
    counters: &GatewayCounters,
    status: Option<u16>,
    error: Option<String>,
) {
    if let Ok(mut last_status) = counters.last_upstream_status.lock() {
        *last_status = status;
    }
    if let Ok(mut last_error) = counters.last_upstream_error.lock() {
        *last_error = error;
    }
}

fn json_response(status: StatusCode, body: String) -> Response<std::io::Cursor<Vec<u8>>> {
    jsonish_response(status, body.into_bytes())
}

fn jsonish_response(status: StatusCode, body: Vec<u8>) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut response = Response::from_data(body).with_status_code(status);
    add_header(
        &mut response,
        "content-type",
        "application/json; charset=utf-8",
    );
    add_cors_headers(&mut response);
    response
}

fn text_response(status: StatusCode, body: Vec<u8>) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut response = Response::from_data(body).with_status_code(status);
    add_header(&mut response, "content-type", "text/plain; charset=utf-8");
    add_cors_headers(&mut response);
    response
}

fn event_stream_response(status: StatusCode, body: Vec<u8>) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut response = Response::from_data(body).with_status_code(status);
    add_header(
        &mut response,
        "content-type",
        "text/event-stream; charset=utf-8",
    );
    add_header(&mut response, "cache-control", "no-cache");
    add_cors_headers(&mut response);
    response
}

fn cors_response(status: StatusCode, body: Vec<u8>) -> Response<std::io::Cursor<Vec<u8>>> {
    let mut response = Response::from_data(body).with_status_code(status);
    add_cors_headers(&mut response);
    response
}

fn add_cors_headers(response: &mut Response<std::io::Cursor<Vec<u8>>>) {
    add_header(response, "access-control-allow-origin", "*");
    add_header(
        response,
        "access-control-allow-headers",
        "authorization,content-type,x-api-key,anthropic-version",
    );
    add_header(
        response,
        "access-control-allow-methods",
        "GET,POST,PUT,PATCH,DELETE,OPTIONS",
    );
}

fn add_header(response: &mut Response<std::io::Cursor<Vec<u8>>>, name: &str, value: &str) {
    if let Ok(header) = Header::from_bytes(name.as_bytes(), value.as_bytes()) {
        response.add_header(header);
    }
}

fn clear_network_override_env(command: &mut Command) {
    for key in [
        "ANTHROPIC_BASE_URL",
        "ANTHROPIC_AUTH_TOKEN",
        "ANTHROPIC_API_KEY",
        "CLAUDE_CODE_API_KEY",
        "CLAUDE_API_KEY",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "NO_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "no_proxy",
        "ELECTRON_DISABLE_SANDBOX",
    ] {
        command.env_remove(key);
    }
}

fn build_inject_script(config: &AppConfig) -> String {
    let active = active_provider(config);
    let providers = config
        .providers
        .iter()
        .filter(|provider| provider.enabled)
        .map(|provider| {
            json!({
                "id": provider.id,
                "name": provider.name,
                "baseUrl": provider.base_url,
                "source": provider.source,
                "appType": provider.app_type,
                "hasApiKey": !provider.api_key.is_empty(),
                "keyMask": mask_secret(&provider.api_key),
                "active": Some(&provider.id) == config.active_provider_id.as_ref(),
            })
        })
        .collect::<Vec<_>>();
    let payload = active.map(|provider| {
        json!({
            "providerId": provider.id,
            "providerName": provider.name,
            "baseUrl": provider.base_url,
            "apiKey": if config.sandbox.inject_api_key { provider.api_key.clone() } else { String::new() },
            "hasApiKey": !provider.api_key.is_empty(),
            "injectProvider": config.sandbox.inject_provider,
            "injectApiKey": config.sandbox.inject_api_key,
            "relaxSandbox": config.sandbox.relax_sandbox && config.sandbox.acknowledged,
            "gatewayEnabled": config.gateway.enabled,
            "gatewayUrl": if config.gateway.enabled { gateway_url(config.gateway.port) } else { String::new() },
            "providers": providers,
        })
    });
    let payload_json = serde_json::to_string(&payload).unwrap_or_else(|_| "null".to_string());
    let scripts = config
        .scripts
        .iter()
        .filter(|script| script.enabled && !script.code.trim().is_empty())
        .map(|script| json!({ "id": script.id, "name": script.name, "code": script.code }))
        .collect::<Vec<_>>();
    let scripts_json = serde_json::to_string(&scripts).unwrap_or_else(|_| "[]".to_string());

    format!(
        r#"(function claudePlusRuntimeConfig() {{
  window.__CLAUDE_PLUS_PROVIDER__ = {payload_json};
  window.__CLAUDE_PLUS_USER_SCRIPTS__ = {scripts_json};
}})();

(function claudePlusUserScripts() {{
  const scripts = window.__CLAUDE_PLUS_USER_SCRIPTS__ || [];
  for (const script of scripts) {{
    try {{
      (0, eval)(String(script.code || ""));
    }} catch (error) {{
      console.error("[Claude++] user script failed", script.name || script.id, error);
    }}
  }}
}})();

{INJECT_SCRIPT}"#
    )
}

fn active_provider(config: &AppConfig) -> Option<&ApiProvider> {
    config.active_provider_id.as_ref().and_then(|id| {
        config
            .providers
            .iter()
            .find(|provider| &provider.id == id && provider.enabled)
    })
}

fn read_config() -> anyhow::Result<AppConfig> {
    let path = config_path();
    if !path.is_file() {
        return Ok(AppConfig::default());
    }
    let raw = fs::read_to_string(&path)?;
    let mut config: AppConfig = serde_json::from_str(raw.trim_start_matches('\u{feff}'))?;
    normalize_config(&mut config);
    Ok(config)
}

fn normalize_config(config: &mut AppConfig) {
    for provider in &mut config.providers {
        provider.app_type = normalize_app_type(&provider.app_type);
        provider.protocol = if provider.protocol.trim().is_empty() {
            infer_provider_protocol(&provider.name, &provider.base_url)
        } else {
            normalize_provider_protocol(&provider.protocol)
        };
        provider.model_mappings = provider
            .model_mappings
            .clone()
            .into_iter()
            .filter_map(normalize_model_mapping)
            .collect();
    }
}

fn write_config(config: &AppConfig) -> anyhow::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(config)?)?;
    Ok(())
}

fn public_config(config: &AppConfig) -> PublicConfig {
    PublicConfig {
        active_provider_id: config.active_provider_id.clone(),
        providers: config
            .providers
            .iter()
            .map(|provider| PublicProvider {
                id: provider.id.clone(),
                name: provider.name.clone(),
                app_type: provider.app_type.clone(),
                source: provider.source.clone(),
                base_url: provider.base_url.clone(),
                protocol: effective_provider_protocol(provider),
                model_mappings: provider.model_mappings.clone(),
                key_mask: mask_secret(&provider.api_key),
                has_key: !provider.api_key.is_empty(),
                injectable: !provider.base_url.is_empty() || !provider.api_key.is_empty(),
                enabled: provider.enabled,
                active: Some(&provider.id) == config.active_provider_id.as_ref(),
            })
            .collect(),
        scripts: config
            .scripts
            .iter()
            .map(|script| PublicScript {
                id: script.id.clone(),
                name: script.name.clone(),
                enabled: script.enabled,
                code: script.code.clone(),
            })
            .collect(),
        sandbox: config.sandbox.clone(),
        gateway: config.gateway.clone(),
        config_path: path_string(&config_path()),
    }
}

struct CcSwitchSync {
    providers: Vec<ApiProvider>,
    current_provider_id: Option<String>,
}

fn apply_cc_switch_sync(config: &mut AppConfig) -> Result<(usize, usize, usize, usize), String> {
    let sync = read_cc_switch_providers()?;
    let mut imported = 0;
    let mut updated = 0;
    let skipped = 0;
    let synced_ids = sync
        .providers
        .iter()
        .map(|provider| provider.id.clone())
        .collect::<HashSet<_>>();

    for provider in sync.providers {
        match config
            .providers
            .iter_mut()
            .find(|item| item.id == provider.id)
        {
            Some(existing) => {
                *existing = provider;
                updated += 1;
            }
            None => {
                config.providers.push(provider);
                imported += 1;
            }
        }
    }

    let before_prune = config.providers.len();
    config
        .providers
        .retain(|provider| provider.source != "cc-switch" || synced_ids.contains(&provider.id));
    let removed = before_prune.saturating_sub(config.providers.len());

    if config
        .active_provider_id
        .as_ref()
        .is_some_and(|id| !config.providers.iter().any(|provider| &provider.id == id))
    {
        config.active_provider_id = None;
    }

    let active_is_manual = config
        .active_provider_id
        .as_ref()
        .and_then(|id| config.providers.iter().find(|provider| &provider.id == id))
        .is_some_and(|provider| provider.source != "cc-switch");

    if !active_is_manual
        && sync.current_provider_id.is_some()
        && config
            .providers
            .iter()
            .any(|provider| Some(&provider.id) == sync.current_provider_id.as_ref())
    {
        config.active_provider_id = sync.current_provider_id;
    } else if config.active_provider_id.is_none() {
        config.active_provider_id = config.providers.first().map(|provider| provider.id.clone());
    }

    Ok((imported, updated, removed, skipped))
}

fn read_cc_switch_providers() -> Result<CcSwitchSync, String> {
    let root = cc_switch_root();
    let database = root.join("cc-switch.db");
    if !database.is_file() {
        return Err("cc-switch database was not found".to_string());
    }

    let current_provider_id = read_cc_switch_settings(&root).ok().and_then(|settings| {
        settings
            .get("currentProviderClaude")
            .or_else(|| settings.get("currentProviderClaudeDesktop"))
            .and_then(Value::as_str)
            .map(ToString::to_string)
    });
    let connection = Connection::open(&database).map_err(|error| error.to_string())?;
    let mut statement = connection
        .prepare(
            "select id, app_type, name, settings_config, is_current, website_url, meta, category \
             from providers \
             where app_type in ('claude', 'claude-desktop') \
             order by case when is_current then 0 else 1 end, app_type, coalesce(sort_index, 2147483647), coalesce(created_at, 0) desc",
        )
        .map_err(|error| error.to_string())?;

    let mut rows = statement.query([]).map_err(|error| error.to_string())?;
    let mut providers = Vec::new();
    let mut seen = HashSet::new();
    while let Some(row) = rows.next().map_err(|error| error.to_string())? {
        let id: String = row.get(0).map_err(|error| error.to_string())?;
        let app_type: String = row.get(1).map_err(|error| error.to_string())?;
        let name: String = row.get(2).map_err(|error| error.to_string())?;
        let settings_config: String = row.get(3).map_err(|error| error.to_string())?;
        let is_current: bool = row.get(4).unwrap_or(false);
        let website_url: Option<String> = row.get(5).ok();
        let meta_config: String = row.get(6).unwrap_or_else(|_| "{}".to_string());
        let category: Option<String> = row.get(7).ok();
        let settings: Value = serde_json::from_str(&settings_config).unwrap_or(Value::Null);
        let meta: Value = serde_json::from_str(&meta_config).unwrap_or(Value::Null);
        let env = settings.get("env").and_then(Value::as_object);
        let usage_script = meta.get("usage_script").and_then(Value::as_object);
        let base_url = env
            .and_then(|env| env.get("ANTHROPIC_BASE_URL"))
            .and_then(Value::as_str)
            .or_else(|| settings.get("base_url").and_then(Value::as_str))
            .or_else(|| {
                usage_script
                    .and_then(|meta| meta.get("baseUrl"))
                    .and_then(Value::as_str)
            })
            .or(website_url.as_deref())
            .unwrap_or_default()
            .to_string();
        let api_key = env
            .and_then(|env| env.get("ANTHROPIC_AUTH_TOKEN"))
            .and_then(Value::as_str)
            .or_else(|| {
                env.and_then(|env| env.get("ANTHROPIC_API_KEY"))
                    .and_then(Value::as_str)
            })
            .or_else(|| settings.get("api_key").and_then(Value::as_str))
            .or_else(|| {
                usage_script
                    .and_then(|meta| meta.get("apiKey"))
                    .and_then(Value::as_str)
            })
            .unwrap_or_default()
            .to_string();

        if is_cc_switch_official_provider(&id, &name, category.as_deref(), &base_url, &api_key)
            || (base_url.trim().is_empty() && api_key.trim().is_empty())
        {
            continue;
        }

        let dedupe_key = cc_switch_dedupe_key(&app_type, &base_url, &api_key);
        if !seen.insert(dedupe_key) {
            continue;
        }

        providers.push(ApiProvider {
            id: format!("cc-switch-{id}"),
            protocol: infer_provider_protocol(&name, &base_url),
            model_mappings: Vec::new(),
            name,
            app_type,
            source: "cc-switch".to_string(),
            base_url,
            api_key,
            enabled: true,
        });

        if is_current {
            // Keep cc-switch's current provider even when settings.json is stale.
        }
    }

    Ok(CcSwitchSync {
        providers,
        current_provider_id: current_provider_id.map(|id| format!("cc-switch-{id}")),
    })
}

fn cc_switch_status() -> CcSwitchStatus {
    let root = cc_switch_root();
    let settings_path = root.join("settings.json");
    let database_path = root.join("cc-switch.db");
    let mut status = CcSwitchStatus {
        found: root.is_dir(),
        root: path_string(&root),
        settings_path: settings_path.is_file().then(|| path_string(&settings_path)),
        database_path: database_path.is_file().then(|| path_string(&database_path)),
        provider_count: 0,
        current_provider_id: None,
        last_error: None,
    };

    match read_cc_switch_providers() {
        Ok(sync) => {
            status.provider_count = sync.providers.len();
            status.current_provider_id = sync.current_provider_id;
        }
        Err(error) => status.last_error = Some(error),
    }

    status
}

fn developer_capabilities_status_sync() -> DeveloperCapabilitiesStatus {
    build_developer_capabilities_status("Claude Desktop 内置能力状态已读取。")
}

fn enable_developer_capabilities_sync() -> Result<DeveloperCapabilitiesStatus, String> {
    let workspace = developer_workspace_path();
    for path in developer_config_paths().into_iter().map(|item| item.1) {
        merge_developer_capability_config(&path, &workspace)?;
    }
    Ok(build_developer_capabilities_status(
        "已写入 Claude Desktop 内置能力配置。重启 Claude Desktop 后，浏览器控制、工作区文件访问和插件运行入口会加载。",
    ))
}

fn build_developer_capabilities_status(message: &str) -> DeveloperCapabilitiesStatus {
    let config_paths = developer_config_paths()
        .into_iter()
        .map(|(label, path)| capability_config_path(label, path))
        .collect::<Vec<_>>();
    let browser_mcp_configured = config_paths.iter().any(|item| item.browser_mcp);
    let workspace_mcp_configured = config_paths.iter().any(|item| item.workspace_mcp);
    DeveloperCapabilitiesStatus {
        config_paths,
        browser_mcp_configured,
        workspace_mcp_configured,
        npx_available: command_available(npx_command_name()),
        workspace_path: path_string(&developer_workspace_path()),
        chrome_connector_url: "https://chromewebstore.google.com/".to_string(),
        chrome_help_url:
            "https://support.claude.com/en/articles/11175166-getting-started-with-claude-for-chrome"
                .to_string(),
        message: message.to_string(),
    }
}

fn capability_config_path(label: String, path: PathBuf) -> CapabilityConfigPath {
    let value = read_json_file(&path).unwrap_or_else(|_| json!({}));
    let server = |name: &str| {
        value
            .pointer(&format!("/mcpServers/{name}"))
            .is_some_and(Value::is_object)
    };
    CapabilityConfigPath {
        label,
        exists: path.is_file(),
        writable: path.parent().is_some_and(|parent| parent.exists()) || path.parent().is_some(),
        path: path_string(&path),
        browser_mcp: server("claude-plus-browser"),
        workspace_mcp: server("claude-plus-workspace"),
    }
}

fn developer_config_paths() -> Vec<(String, PathBuf)> {
    let mut paths = Vec::new();
    if cfg!(target_os = "windows") {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            paths.push((
                "Claude Desktop 能力配置".to_string(),
                PathBuf::from(appdata)
                    .join("Claude")
                    .join(CLAUDE_3P_CONFIG_FILE),
            ));
        }
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            let roaming = PathBuf::from(local_app_data)
                .join("Packages")
                .join("Claude_pzs8sxrjxfjjc")
                .join("LocalCache")
                .join("Roaming")
                .join("Claude")
                .join(CLAUDE_3P_CONFIG_FILE);
            paths.push(("Claude MSIX 能力配置".to_string(), roaming));
        }
    }
    if cfg!(target_os = "macos") {
        if let Some(home) = std::env::var_os("HOME") {
            paths.push((
                "Claude Desktop 官方配置".to_string(),
                PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("Claude")
                    .join(CLAUDE_3P_CONFIG_FILE),
            ));
        }
    }
    if let Ok(path) = claude_3p_user_data_dir() {
        paths.push((
            "Claude-3p 能力配置".to_string(),
            path.join(CLAUDE_3P_CONFIG_FILE),
        ));
    }
    paths
}

fn merge_developer_capability_config(path: &Path, workspace: &Path) -> Result<(), String> {
    let mut value = read_json_file(path).unwrap_or_else(|_| json!({}));
    if !value.is_object() {
        value = json!({});
    }
    let Some(root) = value.as_object_mut() else {
        return Err("Unable to create Claude Desktop config object".to_string());
    };
    let mcp_servers = root
        .entry("mcpServers".to_string())
        .or_insert_with(|| json!({}));
    if !mcp_servers.is_object() {
        *mcp_servers = json!({});
    }
    let Some(servers) = mcp_servers.as_object_mut() else {
        return Err("Unable to create capability config object".to_string());
    };
    servers.insert(
        "claude-plus-browser".to_string(),
        json!({
            "command": npx_command_name(),
            "args": ["-y", "@playwright/mcp@latest"]
        }),
    );
    servers.insert(
        "claude-plus-workspace".to_string(),
        json!({
            "command": npx_command_name(),
            "args": ["-y", "@modelcontextprotocol/server-filesystem", path_string(workspace)]
        }),
    );
    write_json_file(path, &value)
}

fn official_plugins_status_sync() -> OfficialPluginsStatus {
    let cli_path = claude_cli_path();
    let marketplace = official_marketplace_info();
    let (mut available, mut installed, cli_message) = match claude_plugin_list_json() {
        Ok(value) => {
            let installed = value
                .get("installed")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(plugin_name_from_value)
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default();
            let available = value
                .get("available")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| official_plugin_entry_from_value(item, &installed))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            (available, installed, None)
        }
        Err(error) => {
            let installed = HashSet::new();
            let available = marketplace
                .marketplace_path
                .as_ref()
                .and_then(|path| read_marketplace_plugins(path, &installed).ok())
                .unwrap_or_default();
            (available, installed, Some(error))
        }
    };

    let marketplace_plugin_count = if available.is_empty() {
        marketplace.plugin_count
    } else {
        available.len()
    };
    let plugins = prioritize_official_plugins(available.clone());
    let featured_plugins = select_featured_plugins(&mut available);
    let mut installed_plugins = installed.drain().collect::<Vec<_>>();
    installed_plugins.sort();
    let message = if let Some(error) = cli_message {
        format!("Claude CLI marketplace fallback used: {error}")
    } else if marketplace.marketplace_configured {
        "Official plugin marketplace is available.".to_string()
    } else {
        "Official plugin marketplace is not configured yet.".to_string()
    };

    OfficialPluginsStatus {
        claude_cli_available: cli_path.is_some(),
        claude_cli_path: cli_path.as_ref().map(|path| path_string(path)),
        marketplace_configured: marketplace.marketplace_configured,
        marketplace_name: OFFICIAL_PLUGIN_MARKETPLACE_NAME.to_string(),
        marketplace_path: marketplace
            .marketplace_path
            .as_ref()
            .map(|path| path_string(path)),
        marketplace_last_updated: marketplace.last_updated,
        marketplace_plugin_count,
        installed_plugins,
        plugins,
        featured_plugins,
        message,
    }
}

fn sync_official_plugin_marketplace_sync() -> Result<OfficialPluginActionResult, String> {
    let initial = official_plugins_status_sync();
    let args = if initial.marketplace_configured {
        vec![
            "plugin".to_string(),
            "marketplace".to_string(),
            "update".to_string(),
            OFFICIAL_PLUGIN_MARKETPLACE_NAME.to_string(),
        ]
    } else {
        vec![
            "plugin".to_string(),
            "marketplace".to_string(),
            "add".to_string(),
            OFFICIAL_PLUGIN_MARKETPLACE_REPO.to_string(),
            "--scope".to_string(),
            "user".to_string(),
        ]
    };
    let output = run_claude_cli(&args)?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let mut ok = output.status.success();
    let mut exit_code = output.status.code();

    let mut combined_stdout = stdout;
    let mut combined_stderr = stderr;
    if ok && !initial.marketplace_configured {
        let update_output = run_claude_cli(&[
            "plugin".to_string(),
            "marketplace".to_string(),
            "update".to_string(),
            OFFICIAL_PLUGIN_MARKETPLACE_NAME.to_string(),
        ])?;
        combined_stdout.push_str(&String::from_utf8_lossy(&update_output.stdout));
        combined_stderr.push_str(&String::from_utf8_lossy(&update_output.stderr));
        if !update_output.status.success() {
            ok = false;
            exit_code = update_output.status.code();
        }
    }

    Ok(OfficialPluginActionResult {
        ok,
        exit_code,
        message: if ok {
            "Official plugin marketplace synced.".to_string()
        } else {
            "Official plugin marketplace sync failed.".to_string()
        },
        stdout: combined_stdout,
        stderr: combined_stderr,
        status: official_plugins_status_sync(),
    })
}

fn install_official_plugin_sync(plugin: String) -> Result<OfficialPluginActionResult, String> {
    let plugin = validate_official_plugin_name(&plugin)?;
    let plugin_spec = format!("{plugin}@{OFFICIAL_PLUGIN_MARKETPLACE_NAME}");
    let output = run_claude_cli(&[
        "plugin".to_string(),
        "install".to_string(),
        plugin_spec,
        "--scope".to_string(),
        "user".to_string(),
    ])?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let ok = output.status.success();
    Ok(OfficialPluginActionResult {
        ok,
        exit_code: output.status.code(),
        message: if ok {
            format!("Official plugin installed: {plugin}")
        } else {
            format!("Official plugin install failed: {plugin}")
        },
        stdout,
        stderr,
        status: official_plugins_status_sync(),
    })
}

struct OfficialMarketplaceInfo {
    marketplace_configured: bool,
    marketplace_path: Option<PathBuf>,
    last_updated: Option<String>,
    plugin_count: usize,
}

fn official_marketplace_info() -> OfficialMarketplaceInfo {
    let known_path = claude_plugins_root().join("known_marketplaces.json");
    let known = read_json_file(&known_path).unwrap_or_else(|_| json!({}));
    let official = known.get(OFFICIAL_PLUGIN_MARKETPLACE_NAME);
    let marketplace_path = official
        .and_then(|value| value.get("installLocation"))
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .or_else(|| {
            let fallback = claude_plugins_root()
                .join("marketplaces")
                .join(OFFICIAL_PLUGIN_MARKETPLACE_NAME);
            fallback.exists().then_some(fallback)
        });
    let last_updated = official
        .and_then(|value| value.get("lastUpdated"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let plugin_count = marketplace_path
        .as_ref()
        .and_then(|path| {
            read_json_file(&path.join(".claude-plugin").join("marketplace.json"))
                .ok()
                .and_then(|value| value.get("plugins").and_then(Value::as_array).map(Vec::len))
        })
        .unwrap_or_default();

    OfficialMarketplaceInfo {
        marketplace_configured: official.is_some() || marketplace_path.is_some(),
        marketplace_path,
        last_updated,
        plugin_count,
    }
}

fn claude_plugin_list_json() -> Result<Value, String> {
    let output = run_claude_cli(&[
        "plugin".to_string(),
        "list".to_string(),
        "--json".to_string(),
        "--available".to_string(),
    ])?;
    if !output.status.success() {
        return Err(format_command_failure(&output));
    }
    serde_json::from_slice::<Value>(&output.stdout).map_err(|error| error.to_string())
}

fn read_marketplace_plugins(
    marketplace_path: &Path,
    installed: &HashSet<String>,
) -> Result<Vec<OfficialPluginEntry>, String> {
    let value = read_json_file(&marketplace_path.join(".claude-plugin").join("marketplace.json"))
        .map_err(|error| error.to_string())?;
    Ok(value
        .get("plugins")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| official_plugin_entry_from_value(item, installed))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default())
}

fn select_featured_plugins(available: &mut Vec<OfficialPluginEntry>) -> Vec<OfficialPluginEntry> {
    let featured_names = [
        "playwright",
        "github",
        "gitlab",
        "linear",
        "asana",
        "context7",
        "firebase",
        "serena",
        "terraform",
        "code-review",
        "security-guidance",
        "pr-review-toolkit",
        "plugin-dev",
        "skill-creator",
        "mcp-server-dev",
        "claude-code-setup",
        "claude-md-management",
        "frontend-design",
        "feature-dev",
        "commit-commands",
        "code-simplifier",
        "typescript-lsp",
        "pyright-lsp",
        "rust-analyzer-lsp",
        "gopls-lsp",
        "clangd-lsp",
        "csharp-lsp",
        "jdtls-lsp",
        "kotlin-lsp",
        "lua-lsp",
        "php-lsp",
        "ruby-lsp",
        "swift-lsp",
        "discord",
        "telegram",
        "fakechat",
    ];
    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    for name in featured_names {
        if let Some(plugin) = available.iter().find(|plugin| plugin.name == name) {
            seen.insert(plugin.name.clone());
            selected.push(plugin.clone());
        }
    }
    if selected.len() < 24 {
        available.sort_by(|left, right| {
            right
                .install_count
                .unwrap_or_default()
                .cmp(&left.install_count.unwrap_or_default())
                .then_with(|| left.name.cmp(&right.name))
        });
        for plugin in available.iter() {
            if selected.len() >= 24 {
                break;
            }
            if seen.insert(plugin.name.clone()) {
                selected.push(plugin.clone());
            }
        }
    }
    selected
}

fn prioritize_official_plugins(mut plugins: Vec<OfficialPluginEntry>) -> Vec<OfficialPluginEntry> {
    let featured_names = [
        "playwright",
        "github",
        "gitlab",
        "linear",
        "asana",
        "context7",
        "firebase",
        "serena",
        "terraform",
        "code-review",
        "security-guidance",
        "pr-review-toolkit",
        "plugin-dev",
        "skill-creator",
        "mcp-server-dev",
        "claude-code-setup",
        "claude-md-management",
        "frontend-design",
        "feature-dev",
        "commit-commands",
        "code-simplifier",
        "typescript-lsp",
        "pyright-lsp",
        "rust-analyzer-lsp",
        "gopls-lsp",
        "clangd-lsp",
        "csharp-lsp",
        "jdtls-lsp",
        "kotlin-lsp",
        "lua-lsp",
        "php-lsp",
        "ruby-lsp",
        "swift-lsp",
        "discord",
        "telegram",
        "fakechat",
    ];
    plugins.sort_by(|left, right| {
        let left_rank = featured_names
            .iter()
            .position(|name| *name == left.name)
            .unwrap_or(usize::MAX);
        let right_rank = featured_names
            .iter()
            .position(|name| *name == right.name)
            .unwrap_or(usize::MAX);
        left_rank
            .cmp(&right_rank)
            .then_with(|| {
                right
                    .install_count
                    .unwrap_or_default()
                    .cmp(&left.install_count.unwrap_or_default())
            })
            .then_with(|| left.name.cmp(&right.name))
    });
    plugins
}

fn official_plugin_entry_from_value(
    value: &Value,
    installed: &HashSet<String>,
) -> Option<OfficialPluginEntry> {
    let name = value.get("name")?.as_str()?.to_string();
    let plugin_id = value
        .get("pluginId")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{name}@{OFFICIAL_PLUGIN_MARKETPLACE_NAME}"));
    Some(OfficialPluginEntry {
        description: value
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        category: value
            .get("category")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        author: author_name(value.get("author")),
        homepage: value
            .get("homepage")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        source: source_summary(value.get("source")),
        install_count: value.get("installCount").and_then(Value::as_u64),
        installed: installed.contains(&name) || installed.contains(&plugin_id),
        name,
        plugin_id,
    })
}

fn plugin_name_from_value(value: &Value) -> Option<String> {
    if let Some(name) = value.as_str() {
        return Some(name.split('@').next().unwrap_or(name).to_string());
    }
    value
        .get("name")
        .and_then(Value::as_str)
        .or_else(|| value.get("pluginId").and_then(Value::as_str))
        .map(|name| name.split('@').next().unwrap_or(name).to_string())
}

fn author_name(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(value)) => Some(value.clone()),
        Some(Value::Object(object)) => object
            .get("name")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        _ => None,
    }
}

fn source_summary(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(source)) => source.clone(),
        Some(Value::Object(object)) => {
            let kind = object
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("source");
            let url = object
                .get("url")
                .and_then(Value::as_str)
                .or_else(|| object.get("repo").and_then(Value::as_str))
                .unwrap_or_default();
            let path = object
                .get("path")
                .and_then(Value::as_str)
                .unwrap_or_default();
            [kind, url, path]
                .into_iter()
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" / ")
        }
        _ => "-".to_string(),
    }
}

fn validate_official_plugin_name(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("Plugin name is required".to_string());
    }
    if value.len() > 120
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err("Plugin name is malformed".to_string());
    }
    Ok(value.to_string())
}

fn run_claude_cli(args: &[String]) -> Result<std::process::Output, String> {
    let path = claude_cli_path().ok_or_else(|| "Claude CLI was not found".to_string())?;
    if cfg!(target_os = "windows") {
        let mut parts = vec![format!("& {}", powershell_single_quoted(&path_string(&path)))];
        parts.extend(args.iter().map(|arg| powershell_single_quoted(arg)));
        let script = format!("{}; exit $LASTEXITCODE", parts.join(" "));
        return run_powershell_script(&script);
    }
    let mut command = Command::new(path);
    command.args(args).stdin(Stdio::null());
    command.output().map_err(|error| error.to_string())
}

fn claude_cli_path() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            let path = PathBuf::from(appdata).join("npm").join("claude.cmd");
            if path.is_file() {
                return Some(path);
            }
        }
        let mut command = Command::new("where");
        hide_child_console(&mut command);
        let output = command
            .arg("claude.cmd")
            .stdin(Stdio::null())
            .output()
            .ok()?;
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .map(PathBuf::from);
        }
        return None;
    }
    for candidate in ["claude", "claude-code"] {
        let mut command = Command::new("which");
        let output = command
            .arg(candidate)
            .stdin(Stdio::null())
            .output()
            .ok()?;
        if output.status.success() {
            if let Some(path) = String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
            {
                return Some(PathBuf::from(path));
            }
        }
    }
    None
}

fn claude_plugins_root() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("plugins")
}

fn developer_workspace_path() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn npx_command_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "npx.cmd"
    } else {
        "npx"
    }
}

fn command_available(command_name: &str) -> bool {
    if cfg!(target_os = "windows") {
        let mut command = Command::new("where");
        hide_child_console(&mut command);
        return command
            .arg(command_name)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
    }
    Command::new("which")
        .arg(command_name)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn read_cc_switch_settings(root: &Path) -> anyhow::Result<Value> {
    let path = root.join("settings.json");
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str::<Value>(&raw)?)
}

fn config_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata).join("Claude++").join("config.json");
        }
    }

    if cfg!(target_os = "macos") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Claude++")
                .join("config.json");
        }
    }

    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude-plus")
        .join("config.json")
}

fn cc_switch_root() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cc-switch")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn normalize_app_type(value: &str) -> String {
    match value {
        "claude-desktop" => "claude-desktop".to_string(),
        _ => "claude".to_string(),
    }
}

fn normalize_provider_protocol(value: &str) -> String {
    let value = value.trim().to_ascii_lowercase();
    if value.contains("openai") || value.contains("codex") {
        PROVIDER_PROTOCOL_OPENAI.to_string()
    } else {
        PROVIDER_PROTOCOL_ANTHROPIC.to_string()
    }
}

fn normalize_model_mapping(mapping: ModelMapping) -> Option<ModelMapping> {
    let claude_route = mapping.claude_route.trim().to_string();
    let target_model = mapping.target_model.trim().to_string();
    if claude_route.is_empty() || target_model.is_empty() {
        return None;
    }
    let label = if mapping.label.trim().is_empty() {
        format!("{} via {}", model_label(&target_model), claude_route)
    } else {
        mapping.label.trim().to_string()
    };
    Some(ModelMapping {
        claude_route,
        target_model,
        label,
        enabled: mapping.enabled,
    })
}

fn infer_provider_protocol(name: &str, base_url: &str) -> String {
    let lower = format!("{name} {base_url}").to_ascii_lowercase();
    if lower.contains("/anthropic") || lower.contains("api.anthropic.com") {
        return PROVIDER_PROTOCOL_ANTHROPIC.to_string();
    }
    if lower.contains("openai")
        || lower.contains("codex")
        || lower.contains("moonshot")
        || lower.contains("kimi")
        || lower.contains("siliconflow")
        || lower.contains("walkcoding")
        || lower.contains("walkai")
        || lower.trim_end_matches('/').ends_with("/v1")
    {
        return PROVIDER_PROTOCOL_OPENAI.to_string();
    }
    PROVIDER_PROTOCOL_ANTHROPIC.to_string()
}

fn effective_provider_protocol(provider: &ApiProvider) -> String {
    if provider.protocol.trim().is_empty() {
        infer_provider_protocol(&provider.name, &provider.base_url)
    } else {
        normalize_provider_protocol(&provider.protocol)
    }
}

fn provider_requires_gateway_adapter(provider: &ApiProvider) -> bool {
    effective_provider_protocol(provider) == PROVIDER_PROTOCOL_OPENAI
}

fn config_uses_local_gateway(config: &AppConfig) -> bool {
    config.gateway.enabled
}

fn is_cc_switch_official_provider(
    id: &str,
    name: &str,
    category: Option<&str>,
    base_url: &str,
    api_key: &str,
) -> bool {
    if category.is_some_and(|value| value.eq_ignore_ascii_case("official")) {
        return true;
    }

    let id = id.to_ascii_lowercase();
    let name = name.to_ascii_lowercase();
    let base_url = base_url.to_ascii_lowercase();
    let has_key = !api_key.trim().is_empty();

    !has_key
        && (id.ends_with("-official")
            || name.contains("official")
            || base_url.contains("claude.ai/download")
            || base_url.contains("chatgpt.com/codex")
            || base_url.contains("ai.google.dev"))
}

fn cc_switch_dedupe_key(app_type: &str, base_url: &str, api_key: &str) -> String {
    format!(
        "{}|{}|{}",
        normalize_app_type(app_type),
        base_url.trim().trim_end_matches('/').to_ascii_lowercase(),
        api_key.trim()
    )
}

fn mask_secret(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let prefix: String = value.chars().take(6).collect();
    let suffix: String = value
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{prefix}...{suffix}")
}

fn unix_millis() -> u128 {
    now_millis()
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn path_string(path: &Path) -> String {
    path.display().to_string()
}

fn main() {
    if run_headless_command() {
        return;
    }

    tauri::Builder::default()
        .setup(|_| {
            if let Ok(mut config) = read_config() {
                if apply_cc_switch_sync(&mut config).is_ok() {
                    if let Err(error) = write_config(&config) {
                        eprintln!("[Claude++] cc-switch startup sync skipped: {error}");
                    }
                }
                if config_uses_local_gateway(&config) {
                    if let Err(error) = ensure_gateway_runtime(&config) {
                        eprintln!("[Claude++] gateway auto-start skipped: {error}");
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            detect_install,
            developer_capabilities_status,
            delete_api_provider,
            discover_provider_models,
            disable_chinese_localization,
            enable_developer_capabilities,
            enable_chinese_localization,
            enable_virtual_machine_platform,
            gateway_status,
            history_scan_status,
            install_claude_modern,
            install_official_plugin,
            launch_claude_desktop,
            launch_claude_desktop_current_provider,
            launch_claude_desktop_with_provider,
            open_external_url,
            official_plugins_status,
            patch_status,
            patch_stage_only,
            read_app_state,
            relaunch_as_admin,
            repair_history,
            save_api_provider,
            save_gateway_options,
            save_sandbox_options,
            save_user_script,
            set_active_provider,
            start_gateway,
            stop_gateway,
            sync_cc_switch_config,
            sync_official_plugin_marketplace,
            system_readiness_status,
            test_active_provider,
            test_provider
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Claude++ desktop app");
}

fn run_headless_command() -> bool {
    let args = std::env::args().collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--claude-plus-enable-zh-cn") {
        match set_chinese_localization_sync(true) {
            Ok(result) => {
                match serde_json::to_string_pretty(&result) {
                    Ok(json) => println!("{json}"),
                    Err(error) => eprintln!("Failed to encode localization result: {error}"),
                }
                let _ = std::io::stdout().flush();
                return true;
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }
    if args.iter().any(|arg| arg == "--claude-plus-disable-zh-cn") {
        match set_chinese_localization_sync(false) {
            Ok(result) => {
                match serde_json::to_string_pretty(&result) {
                    Ok(json) => println!("{json}"),
                    Err(error) => eprintln!("Failed to encode localization result: {error}"),
                }
                let _ = std::io::stdout().flush();
                return true;
            }
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }
    if !args.iter().any(|arg| arg == "--claude-plus-inject-current") {
        return false;
    }

    match launch_claude_desktop_current_provider() {
        Ok(result) => {
            match serde_json::to_string_pretty(&result) {
                Ok(json) => println!("{json}"),
                Err(error) => eprintln!("Failed to encode launch result: {error}"),
            }
            let _ = std::io::stdout().flush();
            if args.iter().any(|arg| arg == "--keep-gateway") {
                loop {
                    std::thread::sleep(Duration::from_secs(60));
                }
            }
            true
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
