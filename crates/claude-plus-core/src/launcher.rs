use crate::install::ClaudeInstall;
use anyhow::{Context, Result};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};

#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub debug_port: u16,
    pub extra_args: Vec<String>,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            debug_port: 49321,
            extra_args: Vec::new(),
        }
    }
}

pub fn launch_claude(install: &ClaudeInstall, options: &LaunchOptions) -> Result<Child> {
    let mut command = Command::new(&install.executable);
    hide_child_console(&mut command);
    command
        .current_dir(&install.working_dir)
        .arg(format!("--remote-debugging-port={}", options.debug_port))
        .args(&options.extra_args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    command.spawn().with_context(|| {
        format!(
            "failed to launch Claude Desktop from {}",
            install.executable.display()
        )
    })
}

fn hide_child_console(command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        command.creation_flags(0x08000000);
    }
}
