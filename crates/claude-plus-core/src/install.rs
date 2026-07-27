#[cfg(target_os = "windows")]
use serde::Deserialize;
use std::env;
use std::path::{Path, PathBuf};
#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::process::{Command, Stdio};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;
#[cfg(target_os = "windows")]
const CLAUDE_APP_USER_MODEL_ID: &str = "Claude_pzs8sxrjxfjjc!Claude";

#[derive(Debug, Clone)]
pub struct ClaudeInstall {
    pub executable: PathBuf,
    pub working_dir: PathBuf,
    pub source: &'static str,
    pub app_user_model_id: Option<String>,
}

impl ClaudeInstall {
    fn from_executable(executable: PathBuf, source: &'static str) -> Option<Self> {
        if !executable.is_file() {
            return None;
        }

        let working_dir = executable
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        Some(Self {
            executable,
            working_dir,
            source,
            app_user_model_id: None,
        })
    }

    #[cfg(target_os = "windows")]
    fn from_appx_executable(
        executable: PathBuf,
        app_user_model_id: String,
        source: &'static str,
    ) -> Option<Self> {
        let mut install = Self::from_executable(executable, source)?;
        install.app_user_model_id = Some(app_user_model_id);
        Some(install)
    }
}

pub fn detect_claude_install() -> Option<ClaudeInstall> {
    if let Some(override_path) = env::var_os("CLAUDE_DESKTOP_PATH").map(PathBuf::from) {
        if let Some(install) = ClaudeInstall::from_executable(override_path, "CLAUDE_DESKTOP_PATH")
        {
            return Some(install);
        }
    }
    if let Some(override_path) = claude_install_override_file()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|path| PathBuf::from(path.trim()))
    {
        if let Some(install) =
            ClaudeInstall::from_executable(override_path, "Claude++/claude-desktop-path.txt")
        {
            return Some(install);
        }
    }

    #[cfg(target_os = "windows")]
    {
        detect_windows_claude_install()
    }

    #[cfg(target_os = "macos")]
    {
        detect_macos_claude_install()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        None
    }
}

pub fn claude_install_override_file() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        return env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|path| path.join("Claude++").join("claude-desktop-path.txt"));
    }

    #[cfg(target_os = "macos")]
    {
        return env::var_os("HOME").map(PathBuf::from).map(|path| {
            path.join("Library")
                .join("Application Support")
                .join("Claude++")
                .join("claude-desktop-path.txt")
        });
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        None
    }
}

#[cfg(target_os = "windows")]
fn detect_windows_claude_install() -> Option<ClaudeInstall> {
    let local_app_data = env::var_os("LOCALAPPDATA").map(PathBuf::from)?;
    let program_files = env::var_os("ProgramFiles").map(PathBuf::from);
    let program_files_x86 = env::var_os("ProgramFiles(x86)").map(PathBuf::from);
    let anthropic_root = local_app_data.join("AnthropicClaude");
    let anthropic_stub = anthropic_root.join("claude.exe");
    let anthropic_app_executable = latest_squirrel_app_executable(&anthropic_root);

    let app_execution_alias = local_app_data
        .join("Microsoft")
        .join("WindowsApps")
        .join("Claude.exe");
    if let Some(install) = ClaudeInstall::from_appx_executable(
        app_execution_alias,
        CLAUDE_APP_USER_MODEL_ID.to_string(),
        "LOCALAPPDATA/Microsoft/WindowsApps/Claude.exe",
    ) {
        return Some(install);
    }

    if let Some(program_files) = program_files.as_ref() {
        if let Some((appx_executable, app_user_model_id)) =
            latest_appx_claude_executable(program_files)
        {
            if let Some(install) = ClaudeInstall::from_appx_executable(
                appx_executable,
                app_user_model_id,
                "ProgramFiles/WindowsApps/Claude",
            ) {
                return Some(install);
            }
        }
    }

    if let Some(app_executable) = anthropic_app_executable.as_ref() {
        if let Some(install) = ClaudeInstall::from_executable(
            app_executable.clone(),
            "LOCALAPPDATA/AnthropicClaude/app-*",
        ) {
            return Some(install);
        }
    }

    if let Some(install) =
        ClaudeInstall::from_executable(anthropic_stub, "LOCALAPPDATA/AnthropicClaude")
    {
        return Some(install);
    }

    let mut candidates = Vec::new();
    candidates.push((
        local_app_data
            .join("Programs")
            .join("Claude")
            .join("Claude.exe"),
        "LOCALAPPDATA/Programs/Claude",
    ));
    candidates.push((
        local_app_data.join("Claude").join("Claude.exe"),
        "LOCALAPPDATA/Claude",
    ));

    if let Some(program_files) = program_files {
        candidates.push((
            program_files.join("Claude").join("Claude.exe"),
            "ProgramFiles/Claude",
        ));
    }

    if let Some(program_files_x86) = program_files_x86 {
        candidates.push((
            program_files_x86.join("Claude").join("Claude.exe"),
            "ProgramFiles(x86)/Claude",
        ));
    }

    candidates
        .into_iter()
        .find_map(|(path, source)| ClaudeInstall::from_executable(path, source))
        .or_else(discover_windows_claude_with_powershell)
}

#[cfg(target_os = "windows")]
fn latest_squirrel_app_executable(root: &Path) -> Option<PathBuf> {
    let mut app_dirs = std::fs::read_dir(root)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let file_type = entry.file_type().ok()?;
            if !file_type.is_dir() {
                return None;
            }

            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("app-") {
                return None;
            }

            let modified = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok();
            Some((entry.path(), modified))
        })
        .collect::<Vec<_>>();

    app_dirs.sort_by(|left, right| left.1.cmp(&right.1));
    app_dirs
        .into_iter()
        .rev()
        .map(|(path, _)| path.join("claude.exe"))
        .find(|path| path.is_file())
}

#[cfg(target_os = "windows")]
fn latest_appx_claude_executable(program_files: &Path) -> Option<(PathBuf, String)> {
    let windows_apps = program_files.join("WindowsApps");
    let mut app_dirs = std::fs::read_dir(windows_apps)
        .ok()?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let file_type = entry.file_type().ok()?;
            if !file_type.is_dir() {
                return None;
            }

            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.to_ascii_lowercase().contains("claude") {
                return None;
            }

            let executable = [
                entry.path().join("app").join("claude.exe"),
                entry.path().join("Claude.exe"),
                entry.path().join("claude.exe"),
            ]
            .into_iter()
            .find(|path| path.is_file())?;

            let modified = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok();
            let package_family = name.split_once("__").map(|(_, suffix)| {
                let package_name = name.split('_').next().unwrap_or("Claude");
                format!("{package_name}_{suffix}!Claude")
            })?;
            Some((executable, package_family, modified))
        })
        .collect::<Vec<_>>();

    app_dirs.sort_by(|left, right| left.2.cmp(&right.2));
    app_dirs
        .into_iter()
        .rev()
        .map(|(path, app_user_model_id, _)| (path, app_user_model_id))
        .next()
}

#[cfg(target_os = "windows")]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowsClaudeDiscovery {
    executable: String,
    app_user_model_id: Option<String>,
}

#[cfg(target_os = "windows")]
fn discover_windows_claude_with_powershell() -> Option<ClaudeInstall> {
    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)

function Emit-ClaudeInstall([string]$Executable, [string]$AppUserModelId) {
  if ([string]::IsNullOrWhiteSpace($Executable) -or -not (Test-Path -LiteralPath $Executable -PathType Leaf)) {
    return
  }
  [PSCustomObject]@{
    executable = $Executable
    appUserModelId = if ([string]::IsNullOrWhiteSpace($AppUserModelId)) { $null } else { $AppUserModelId }
  } | ConvertTo-Json -Compress
  exit 0
}

$packages = Get-AppxPackage |
  Where-Object {
    $_.Name -match 'Claude' -or
    $_.PackageFamilyName -match '^Claude_' -or
    $_.PackageFullName -match 'Claude'
  } |
  Sort-Object Version -Descending
foreach ($package in $packages) {
  $manifest = Get-AppxPackageManifest -Package $package.PackageFullName
  $applicationId = $manifest.Package.Applications.Application |
    Select-Object -First 1 -ExpandProperty Id
  $aumid = if ($applicationId) { "$($package.PackageFamilyName)!$applicationId" } else { $null }
  $candidates = @(
    (Join-Path $package.InstallLocation 'app\claude.exe'),
    (Join-Path $package.InstallLocation 'Claude.exe'),
    (Join-Path $package.InstallLocation 'claude.exe')
  )
  foreach ($candidate in $candidates) {
    Emit-ClaudeInstall $candidate $aumid
  }
}

$uninstallRoots = @(
  'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
  'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
  'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*'
)
foreach ($entry in (Get-ItemProperty $uninstallRoots | Where-Object { $_.DisplayName -match '^Claude(\s|$)' })) {
  $displayIcon = if ($entry.DisplayIcon) { $entry.DisplayIcon.Trim('"') -replace ',\d+$', '' } else { $null }
  $candidates = @(
    $displayIcon,
    (Join-Path $entry.InstallLocation 'Claude.exe'),
    (Join-Path $entry.InstallLocation 'claude.exe'),
    (Join-Path $entry.InstallLocation 'app\claude.exe')
  )
  foreach ($candidate in $candidates) {
    Emit-ClaudeInstall $candidate $null
  }
}

$shortcutRoots = @(
  [Environment]::GetFolderPath('Programs'),
  [Environment]::GetFolderPath('CommonPrograms'),
  [Environment]::GetFolderPath('Desktop'),
  [Environment]::GetFolderPath('CommonDesktopDirectory')
) | Where-Object { $_ -and (Test-Path -LiteralPath $_) }
$shell = New-Object -ComObject WScript.Shell
foreach ($shortcut in (Get-ChildItem -LiteralPath $shortcutRoots -Filter '*.lnk' -Recurse |
  Where-Object { $_.BaseName -match '^Claude(\+\+)?$' })) {
  $target = $shell.CreateShortcut($shortcut.FullName).TargetPath
  if ($target -and $target -notmatch 'claude-plus-desktop') {
    Emit-ClaudeInstall $target $null
  }
}
"#;

    let mut command = Command::new("powershell.exe");
    command
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW);
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let discovery = stdout
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str::<WindowsClaudeDiscovery>(line.trim()).ok())?;
    let executable = PathBuf::from(discovery.executable);
    if let Some(app_user_model_id) = discovery.app_user_model_id {
        ClaudeInstall::from_appx_executable(
            executable,
            app_user_model_id,
            "PowerShell/Get-AppxPackage",
        )
    } else {
        ClaudeInstall::from_executable(executable, "PowerShell/RegistryOrShortcut")
    }
}

#[cfg(target_os = "macos")]
fn detect_macos_claude_install() -> Option<ClaudeInstall> {
    let mut candidates = vec![
        (
            PathBuf::from("/Applications/Claude.app/Contents/MacOS/Claude"),
            "/Applications/Claude.app",
        ),
        (
            PathBuf::from("/Applications/Claude Desktop.app/Contents/MacOS/Claude"),
            "/Applications/Claude Desktop.app",
        ),
        (
            PathBuf::from("/Applications/Setapp/Claude.app/Contents/MacOS/Claude"),
            "/Applications/Setapp/Claude.app",
        ),
    ];

    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        candidates.push((
            home.join("Applications")
                .join("Claude.app")
                .join("Contents")
                .join("MacOS")
                .join("Claude"),
            "~/Applications/Claude.app",
        ));
        candidates.push((
            home.join("Applications")
                .join("Claude Desktop.app")
                .join("Contents")
                .join("MacOS")
                .join("Claude"),
            "~/Applications/Claude Desktop.app",
        ));
    }

    candidates
        .into_iter()
        .find_map(|(path, source)| ClaudeInstall::from_executable(path, source))
        .or_else(discover_macos_claude_with_spotlight)
}

#[cfg(target_os = "macos")]
fn discover_macos_claude_with_spotlight() -> Option<ClaudeInstall> {
    let output = Command::new("mdfind")
        .args([
            "kMDItemContentType == 'com.apple.application-bundle' && (kMDItemFSName == 'Claude.app' || kMDItemFSName == 'Claude Desktop.app')",
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .flat_map(|bundle| {
            let contents = PathBuf::from(bundle).join("Contents").join("MacOS");
            [contents.join("Claude"), contents.join("claude")]
        })
        .find_map(|path| ClaudeInstall::from_executable(path, "Spotlight/Claude.app"))
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_root(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("claude-plus-{name}-{suffix}"))
    }

    #[test]
    fn detects_squirrel_version_directory() {
        let root = temporary_root("squirrel");
        let executable = root.join("app-1.2.3").join("claude.exe");
        fs::create_dir_all(executable.parent().expect("executable parent")).unwrap();
        fs::write(&executable, b"test").unwrap();

        assert_eq!(latest_squirrel_app_executable(&root), Some(executable));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn detects_appx_package_with_prefixed_package_name() {
        let program_files = temporary_root("appx");
        let executable = program_files
            .join("WindowsApps")
            .join("AnthropicPBC.Claude_1.2.3.0_x64__pzs8sxrjxfjjc")
            .join("app")
            .join("claude.exe");
        fs::create_dir_all(executable.parent().expect("executable parent")).unwrap();
        fs::write(&executable, b"test").unwrap();

        let detected =
            latest_appx_claude_executable(&program_files).expect("appx install should be found");
        assert_eq!(detected.0, executable);
        assert_eq!(detected.1, "AnthropicPBC.Claude_pzs8sxrjxfjjc!Claude");

        fs::remove_dir_all(program_files).unwrap();
    }
}
