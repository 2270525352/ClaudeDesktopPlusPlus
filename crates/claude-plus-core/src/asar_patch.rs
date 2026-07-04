use crate::install::ClaudeInstall;
use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const MAIN_VIEW_PRELOAD: &str = ".vite/build/mainView.js";
const PATCH_MARKER: &str = "CLAUDE_PLUS_PRELOAD_PATCH_V1";
const NATIVE_UNPACK_GLOB: &str = "*.{node,dll,dylib,so}";

#[derive(Debug, Clone)]
pub struct StagedPatch {
    pub source_asar: PathBuf,
    pub source_unpacked: Option<PathBuf>,
    pub stage_dir: PathBuf,
    pub original_copy: PathBuf,
    pub original_unpacked_copy: Option<PathBuf>,
    pub extract_dir: PathBuf,
    pub staged_asar: PathBuf,
    pub staged_unpacked: Option<PathBuf>,
    pub patched_preload: PathBuf,
}

pub fn stage_preload_patch(install: &ClaudeInstall, inject_script: &str) -> Result<StagedPatch> {
    let source_asar = find_app_asar(install).with_context(|| {
        format!(
            "failed to find app.asar near Claude install at {}",
            install.executable.display()
        )
    })?;

    let stage_dir = make_stage_dir()?;
    let original_copy = stage_dir.join("original.app.asar");
    let original_unpacked_copy = unpacked_dir_for_asar(&original_copy);
    let extract_dir = stage_dir.join("extracted");
    let staged_asar = stage_dir.join("staged.app.asar");
    let staged_unpacked = unpacked_dir_for_asar(&staged_asar);
    let source_unpacked = unpacked_dir_for_asar(&source_asar);

    fs::copy(&source_asar, &original_copy).with_context(|| {
        format!(
            "failed to copy {} to {}",
            source_asar.display(),
            original_copy.display()
        )
    })?;

    let copied_original_unpacked = if source_unpacked.is_dir() {
        copy_dir_all(&source_unpacked, &original_unpacked_copy).with_context(|| {
            format!(
                "failed to copy {} to {}",
                source_unpacked.display(),
                original_unpacked_copy.display()
            )
        })?;
        Some(original_unpacked_copy)
    } else {
        None
    };

    run_asar([
        "extract".as_ref(),
        original_copy.as_os_str(),
        extract_dir.as_os_str(),
    ])
    .context("failed to extract staged app.asar copy")?;

    let patched_preload = extract_dir.join(path_from_asar_name(MAIN_VIEW_PRELOAD));
    patch_main_view_preload(&patched_preload, inject_script)?;

    pack_staged_asar(&extract_dir, &staged_asar).context("failed to pack staged app.asar")?;

    verify_staged_asar(&staged_asar)?;

    Ok(StagedPatch {
        source_asar,
        source_unpacked: source_unpacked.is_dir().then_some(source_unpacked),
        stage_dir,
        original_copy,
        original_unpacked_copy: copied_original_unpacked,
        extract_dir,
        staged_asar,
        staged_unpacked: staged_unpacked.is_dir().then_some(staged_unpacked),
        patched_preload,
    })
}

pub fn find_app_asar(install: &ClaudeInstall) -> Option<PathBuf> {
    candidate_app_asar_paths(install)
        .into_iter()
        .find(|path| path.is_file())
}

pub fn candidate_app_asar_paths(install: &ClaudeInstall) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    paths.push(install.working_dir.join("resources").join("app.asar"));
    paths.push(install.working_dir.join("Resources").join("app.asar"));

    if let Some(parent) = install.working_dir.parent() {
        paths.push(parent.join("resources").join("app.asar"));
        paths.push(parent.join("Resources").join("app.asar"));
    }

    if let Some(parent) = install.executable.parent() {
        paths.push(parent.join("resources").join("app.asar"));
        paths.push(parent.join("Resources").join("app.asar"));

        if let Some(grandparent) = parent.parent() {
            paths.push(grandparent.join("resources").join("app.asar"));
            paths.push(grandparent.join("Resources").join("app.asar"));
        }
    }

    dedupe_paths(paths)
}

fn patch_main_view_preload(preload_path: &Path, inject_script: &str) -> Result<()> {
    let mut preload = fs::read_to_string(preload_path)
        .with_context(|| format!("failed to read {}", preload_path.display()))?;

    if preload.contains(PATCH_MARKER) {
        return Ok(());
    }

    preload.push_str(&build_preload_patch(inject_script)?);
    fs::write(preload_path, preload)
        .with_context(|| format!("failed to write {}", preload_path.display()))?;
    Ok(())
}

fn pack_staged_asar(extract_dir: &Path, staged_asar: &Path) -> Result<()> {
    run_asar([
        OsStr::new("pack"),
        OsStr::new("--unpack"),
        OsStr::new(NATIVE_UNPACK_GLOB),
        extract_dir.as_os_str(),
        staged_asar.as_os_str(),
    ])
}

fn build_preload_patch(inject_script: &str) -> Result<String> {
    let script_json =
        serde_json::to_string(inject_script).context("failed to encode inject script")?;

    Ok(format!(
        r#"

;(() => {{
  const marker = "{PATCH_MARKER}";
  if (globalThis[marker]) {{
    return;
  }}
  globalThis[marker] = true;
  const source = {script_json};
  const run = () => {{
    try {{
      (0, eval)(source);
    }} catch (error) {{
      console.error("[Claude++] preload inject failed", error);
    }}
  }};
  if (document.readyState === "loading") {{
    document.addEventListener("DOMContentLoaded", run, {{ once: true }});
  }} else {{
    run();
  }}
}})();
"#
    ))
}

fn verify_staged_asar(staged_asar: &Path) -> Result<()> {
    let verify_dir = staged_asar
        .parent()
        .context("staged app.asar has no parent directory")?
        .join("verify-extracted");

    run_asar([
        "extract".as_ref(),
        staged_asar.as_os_str(),
        verify_dir.as_os_str(),
    ])
    .context("failed to verify-extract staged app.asar")?;

    let preload_path = verify_dir.join(path_from_asar_name(MAIN_VIEW_PRELOAD));
    let preload = fs::read_to_string(&preload_path)
        .with_context(|| format!("failed to read {}", preload_path.display()))?;
    if !preload.contains(PATCH_MARKER) {
        bail!(
            "staged app.asar verification failed: {} does not contain {}",
            MAIN_VIEW_PRELOAD,
            PATCH_MARKER
        );
    }

    Ok(())
}

fn copy_dir_all(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;

    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read entry in {}", source.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to read file type for {}", entry.path().display()))?;
        let destination_path = destination.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &destination_path)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &destination_path).with_context(|| {
                format!("failed to copy file to {}", destination_path.display())
            })?;
        }
    }

    Ok(())
}

fn run_asar<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let output = run_asar_output(args)?;
    if output.status.success() {
        return Ok(());
    }

    bail!(
        "asar command failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_asar_output<I, S>(args: I) -> Result<std::process::Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut command = Command::new(npx_command());
    hide_child_console(&mut command);
    command.arg("--yes").arg("@electron/asar");
    command.args(args);

    command
        .output()
        .context("failed to run npx @electron/asar; install Node.js/npm or ensure npx is on PATH")
}

fn hide_child_console(command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        command.creation_flags(0x08000000);
    }
}

fn npx_command() -> &'static str {
    if cfg!(target_os = "windows") {
        "npx.cmd"
    } else {
        "npx"
    }
}

fn make_stage_dir() -> Result<PathBuf> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX_EPOCH")?
        .as_millis();
    let path =
        std::env::temp_dir().join(format!("claude-plus-stage-{}-{stamp}", std::process::id()));
    fs::create_dir_all(&path)
        .with_context(|| format!("failed to create stage directory {}", path.display()))?;
    Ok(path)
}

fn path_from_asar_name(name: &str) -> PathBuf {
    name.split('/').collect()
}

fn unpacked_dir_for_asar(asar_path: &Path) -> PathBuf {
    asar_path.with_extension("asar.unpacked")
}

fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for path in paths {
        if seen.insert(path.clone()) {
            deduped.push(path);
        }
    }

    deduped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preload_patch_contains_marker_and_script() {
        let patch = build_preload_patch("window.__CLAUDE_PLUS_TEST__ = true;").unwrap();

        assert!(patch.contains(PATCH_MARKER));
        assert!(patch.contains("__CLAUDE_PLUS_TEST__"));
    }

    #[test]
    fn candidates_cover_windows_and_macos_layouts() {
        let install = ClaudeInstall {
            executable: PathBuf::from(r"C:\Users\me\AppData\Local\AnthropicClaude\claude.exe"),
            working_dir: PathBuf::from(r"C:\Users\me\AppData\Local\AnthropicClaude\app-1.2.3"),
            source: "test",
        };
        let candidates = candidate_app_asar_paths(&install);
        assert!(candidates
            .iter()
            .any(|path| path.ends_with(Path::new(r"app-1.2.3\resources\app.asar"))));

        let install = ClaudeInstall {
            executable: PathBuf::from("/Applications/Claude.app/Contents/MacOS/Claude"),
            working_dir: PathBuf::from("/Applications/Claude.app/Contents/MacOS"),
            source: "test",
        };
        let candidates = candidate_app_asar_paths(&install);
        assert!(candidates
            .iter()
            .any(|path| path.ends_with(Path::new("Contents/Resources/app.asar"))));
    }

    #[test]
    fn unpacked_dir_keeps_asar_basename() {
        assert_eq!(
            unpacked_dir_for_asar(Path::new("app.asar")),
            PathBuf::from("app.asar.unpacked")
        );
        assert_eq!(
            unpacked_dir_for_asar(Path::new("original.app.asar")),
            PathBuf::from("original.app.asar.unpacked")
        );
    }
}
