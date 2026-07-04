use anyhow::{bail, Context, Result};
use claude_plus_core::asar_patch::stage_preload_patch;
use claude_plus_core::cdp::{
    inject_script, is_local_port_open, wait_for_injectable_target, wait_for_version,
};
use claude_plus_core::install::detect_claude_install;
use claude_plus_core::launcher::{launch_claude, LaunchOptions};
use std::process::Child;
use std::time::Duration;

const INJECT_SCRIPT: &str = include_str!("../../../assets/inject/renderer-inject.js");

fn main() -> Result<()> {
    let args = Args::parse()?;

    match args.command {
        CommandMode::Launch(launch_args) => run_launch(launch_args),
        CommandMode::Patch(patch_args) => run_patch(patch_args),
    }
}

fn run_launch(args: LaunchArgs) -> Result<()> {
    let install = detect_claude_install().context("Claude Desktop install was not found")?;

    println!("Claude++ launcher");
    println!("  install: {}", install.executable.display());
    println!("  source:  {}", install.source);
    println!("  port:    {}", args.port);

    let mut child = if !args.attach_only {
        if is_local_port_open(args.port) {
            bail!(
                "port {} is already in use; close the existing app or use --attach-only",
                args.port
            );
        }

        let child = launch_claude(
            &install,
            &LaunchOptions {
                debug_port: args.port,
                extra_args: args.extra_args,
            },
        )?;
        println!("  status:  launched Claude Desktop");
        Some(child)
    } else {
        println!("  status:  attach-only mode");
        None
    };

    let wait_timeout = Duration::from_millis(args.wait_ms);
    let version = wait_for_version(args.port, wait_timeout).map_err(|error| {
        diagnose_version_wait_failure(error, args.port, args.attach_only, child.as_mut())
    })?;
    println!("  browser: {}", version.browser);
    println!("  cdp:     {}", version.protocol_version);

    let target = wait_for_injectable_target(args.port, wait_timeout)?;
    println!("  target:  {} {}", target.title, target.url);

    if args.inject {
        let websocket_url = target
            .websocket_debugger_url
            .as_deref()
            .context("selected target does not expose a websocket URL")?;
        inject_script(websocket_url, INJECT_SCRIPT)?;
        println!("  inject:  ok");
    } else {
        println!("  inject:  skipped; pass --inject to inject renderer script");
    }

    Ok(())
}

fn run_patch(args: PatchArgs) -> Result<()> {
    if !args.stage_only {
        bail!("patch currently requires --stage-only");
    }

    let install = detect_claude_install().context("Claude Desktop install was not found")?;

    println!("Claude++ patch staging");
    println!("  install:       {}", install.executable.display());
    println!("  source:        {}", install.source);
    println!("  mode:          stage-only");

    let staged = stage_preload_patch(&install, INJECT_SCRIPT)?;

    println!("  source asar:   {}", staged.source_asar.display());
    println!("  stage dir:     {}", staged.stage_dir.display());
    println!("  original copy: {}", staged.original_copy.display());
    println!("  extracted:     {}", staged.extract_dir.display());
    println!("  patched file:  {}", staged.patched_preload.display());
    println!("  staged asar:   {}", staged.staged_asar.display());
    println!("  verify:        ok");
    if let Some(source_unpacked) = staged.source_unpacked.as_ref() {
        println!("  source unpacked: {}", source_unpacked.display());
    }
    if let Some(original_unpacked) = staged.original_unpacked_copy.as_ref() {
        println!("  copied unpacked: {}", original_unpacked.display());
    }
    if let Some(staged_unpacked) = staged.staged_unpacked.as_ref() {
        println!("  staged unpacked: {}", staged_unpacked.display());
    }
    println!("  install:       skipped");

    Ok(())
}

fn diagnose_version_wait_failure(
    error: anyhow::Error,
    port: u16,
    attach_only: bool,
    child: Option<&mut Child>,
) -> anyhow::Error {
    let endpoint = format!("http://127.0.0.1:{port}/json/version");

    if attach_only {
        return error.context(format!(
            "CDP endpoint was not reachable at {endpoint}; start Claude with a remote debugging port first"
        ));
    }

    if let Some(child) = child {
        match child.try_wait() {
            Ok(Some(status)) => {
                return error.context(format!(
                    "Claude Desktop exited before CDP became available at {endpoint} (exit status: {status}); current Claude Desktop builds may reject --remote-debugging-port without signed CLAUDE_CDP_AUTH"
                ));
            }
            Ok(None) => {}
            Err(wait_error) => {
                return error.context(format!(
                    "CDP endpoint was not reachable at {endpoint}, and Claude process status could not be checked: {wait_error}"
                ));
            }
        }
    }

    error.context(format!(
        "CDP endpoint was not reachable at {endpoint}; ensure Claude was fully quit before launch, or this Claude Desktop build may be guarding remote debugging"
    ))
}

#[derive(Debug)]
struct Args {
    command: CommandMode,
}

#[derive(Debug)]
enum CommandMode {
    Launch(LaunchArgs),
    Patch(PatchArgs),
}

#[derive(Debug)]
struct LaunchArgs {
    port: u16,
    wait_ms: u64,
    inject: bool,
    attach_only: bool,
    extra_args: Vec<String>,
}

#[derive(Debug)]
struct PatchArgs {
    stage_only: bool,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut args = std::env::args().skip(1);
        let Some(first) = args.next() else {
            return Ok(Self {
                command: CommandMode::Launch(LaunchArgs::parse(std::iter::empty::<String>())?),
            });
        };

        if first == "patch" {
            return Ok(Self {
                command: CommandMode::Patch(PatchArgs::parse(args)?),
            });
        }

        if first == "--help" || first == "-h" {
            print_help();
            std::process::exit(0);
        }

        let launch_args = LaunchArgs::parse(std::iter::once(first).chain(args))?;
        Ok(Self {
            command: CommandMode::Launch(launch_args),
        })
    }
}

impl LaunchArgs {
    fn parse<I>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = String>,
    {
        let mut port = 49321;
        let mut wait_ms = 15_000;
        let mut inject = false;
        let mut attach_only = false;
        let mut extra_args = Vec::new();

        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--port" => {
                    let value = args.next().context("--port requires a value")?;
                    port = value.parse().context("--port must be a valid u16")?;
                }
                "--wait-ms" => {
                    let value = args.next().context("--wait-ms requires a value")?;
                    wait_ms = value.parse().context("--wait-ms must be a valid integer")?;
                }
                "--inject" => inject = true,
                "--attach-only" => attach_only = true,
                "--" => {
                    extra_args.extend(args);
                    break;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                "patch" => bail!("patch must be the first argument"),
                other if other.starts_with('-') => bail!("unknown argument: {other}"),
                other => extra_args.push(other.to_string()),
            }
        }

        Ok(Self {
            port,
            wait_ms,
            inject,
            attach_only,
            extra_args,
        })
    }
}

impl PatchArgs {
    fn parse<I>(args: I) -> Result<Self>
    where
        I: IntoIterator<Item = String>,
    {
        let mut stage_only = false;

        for arg in args {
            match arg.as_str() {
                "--stage-only" => stage_only = true,
                "--help" | "-h" => {
                    print_patch_help();
                    std::process::exit(0);
                }
                other => bail!("unknown patch argument: {other}"),
            }
        }

        Ok(Self { stage_only })
    }
}

fn print_help() {
    println!(
        r#"Claude++ launcher

Usage:
  claude-plus-launcher [--inject] [--attach-only] [--port 49321] [--wait-ms 15000] [-- <extra Claude args>]
  claude-plus-launcher patch --stage-only

Options:
  --inject       Inject the bundled renderer script into the selected Claude page.
  --attach-only  Do not launch Claude; connect to an already running debug port.
  --port         Chromium remote debugging port. Defaults to 49321.
  --wait-ms      Time to wait for CDP targets. Defaults to 15000.
"#
    );
}

fn print_patch_help() {
    println!(
        r#"Claude++ patch staging

Usage:
  claude-plus-launcher patch --stage-only

Options:
  --stage-only  Copy, extract, patch, repack, and verify app.asar in a temp staging directory.
"#
    );
}
