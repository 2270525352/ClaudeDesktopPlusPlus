use std::env;
use std::path::{Path, PathBuf};

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
    fn from_appx_executable(executable: PathBuf, app_user_model_id: String) -> Option<Self> {
        let mut install = Self::from_executable(executable, "ProgramFiles/WindowsApps/Claude")?;
        install.app_user_model_id = Some(app_user_model_id);
        Some(install)
    }
}

pub fn detect_claude_install() -> Option<ClaudeInstall> {
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

#[cfg(target_os = "windows")]
fn detect_windows_claude_install() -> Option<ClaudeInstall> {
    let local_app_data = env::var_os("LOCALAPPDATA").map(PathBuf::from)?;
    let program_files = env::var_os("ProgramFiles").map(PathBuf::from);
    let program_files_x86 = env::var_os("ProgramFiles(x86)").map(PathBuf::from);
    let anthropic_root = local_app_data.join("AnthropicClaude");
    let anthropic_stub = anthropic_root.join("claude.exe");
    let anthropic_app_executable = latest_squirrel_app_executable(&anthropic_root);

    if let Some(program_files) = program_files.as_ref() {
        if let Some((appx_executable, app_user_model_id)) =
            latest_appx_claude_executable(program_files)
        {
            if let Some(install) =
                ClaudeInstall::from_appx_executable(appx_executable, app_user_model_id)
            {
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
            if !name.starts_with("Claude_") {
                return None;
            }

            let executable = entry.path().join("app").join("claude.exe");
            if !executable.is_file() {
                return None;
            }

            let modified = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok();
            let package_family = name
                .split_once("__")
                .map(|(_, suffix)| format!("Claude_{suffix}!Claude"))?;
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

#[cfg(target_os = "macos")]
fn detect_macos_claude_install() -> Option<ClaudeInstall> {
    let mut candidates = vec![(
        PathBuf::from("/Applications/Claude.app/Contents/MacOS/Claude"),
        "/Applications/Claude.app",
    )];

    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        candidates.push((
            home.join("Applications")
                .join("Claude.app")
                .join("Contents")
                .join("MacOS")
                .join("Claude"),
            "~/Applications/Claude.app",
        ));
    }

    candidates
        .into_iter()
        .find_map(|(path, source)| ClaudeInstall::from_executable(path, source))
}
