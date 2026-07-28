import { copyFileSync, existsSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";

const appRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const command = process.argv[2];
const args = process.argv.slice(2);
const env = { ...process.env };
let llvmMingwBin;

if (!command) {
  console.error("Usage: node scripts/tauri-run.mjs <dev|build|info|...>");
  process.exit(1);
}

if (command === "dev" && !hasConfigArg(args)) {
  args.push(
    "--config",
    join(appRoot, "src-tauri", "tauri.dev.conf.json"),
  );
}

if (process.platform === "win32") {
  env.RUSTUP_TOOLCHAIN ||= "stable-x86_64-pc-windows-gnullvm";

  llvmMingwBin = findLlvmMingwBin();
  if (llvmMingwBin) {
    env.Path = `${llvmMingwBin};${env.Path || ""}`;
  }

  if ((command === "dev" || command === "build") && !hasTargetArg(args)) {
    args.push("--target", "x86_64-pc-windows-gnullvm");
  }
}

const tauriCli = join(appRoot, "node_modules", "@tauri-apps", "cli", "tauri.js");
const target = readTarget(args);
const isLlvmMingwTarget =
  process.platform === "win32" &&
  target === "x86_64-pc-windows-gnullvm" &&
  Boolean(llvmMingwBin);
let exitCode;

if (isLlvmMingwTarget && command === "build" && !args.includes("--no-bundle")) {
  exitCode = await runTauri([...args, "--no-bundle"]);
  if (exitCode === 0) {
    copyWindowsRuntimeDlls(llvmMingwBin, target);
    exitCode = await runTauri([
      "bundle",
      "--target",
      target,
      "--config",
      join(appRoot, "src-tauri", "tauri.gnullvm.conf.json"),
    ]);
  }
} else {
  if (isLlvmMingwTarget && command === "bundle") {
    copyWindowsRuntimeDlls(llvmMingwBin, target);
    args.push(
      "--config",
      join(appRoot, "src-tauri", "tauri.gnullvm.conf.json"),
    );
  }

  exitCode = await runTauri(args);
  if (exitCode === 0 && isLlvmMingwTarget && command === "build") {
    copyWindowsRuntimeDlls(llvmMingwBin, target);
  }
}

process.exit(exitCode);

function hasTargetArg(values) {
  return values.some((value) => value === "--target" || value === "-t" || value.startsWith("--target="));
}

function hasConfigArg(values) {
  return values.some((value) => value === "--config" || value === "-c" || value.startsWith("--config="));
}

function readTarget(values) {
  for (let index = 0; index < values.length; index += 1) {
    const value = values[index];
    if (value === "--target" || value === "-t") {
      return values[index + 1];
    }
    if (value.startsWith("--target=")) {
      return value.slice("--target=".length);
    }
  }
  return undefined;
}

function runTauri(cliArgs) {
  return new Promise((resolveExit) => {
    const child = spawn(process.execPath, [tauriCli, ...cliArgs], {
      cwd: appRoot,
      env,
      shell: false,
      stdio: "inherit",
    });

    child.on("error", () => resolveExit(1));
    child.on("exit", (code, signal) => {
      if (signal) {
        process.kill(process.pid, signal);
        resolveExit(1);
        return;
      }
      resolveExit(code ?? 1);
    });
  });
}

function findLlvmMingwBin() {
  const home = process.env.USERPROFILE;
  if (!home) {
    return undefined;
  }

  const toolsRoot = join(home, ".local", "tools");
  if (!existsSync(toolsRoot)) {
    return undefined;
  }

  const candidates = readdirSync(toolsRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .filter((name) => name.startsWith("llvm-mingw-") && name.includes("-ucrt-x86_64"))
    .sort()
    .reverse();

  for (const name of candidates) {
    const bin = join(toolsRoot, name, "bin");
    if (existsSync(join(bin, "x86_64-w64-mingw32-clang.exe"))) {
      return bin;
    }
  }

  return undefined;
}

function copyWindowsRuntimeDlls(bin, targetTriple) {
  const releaseDir = join(appRoot, "src-tauri", "target", targetTriple, "release");
  if (!existsSync(releaseDir)) {
    throw new Error(`Release directory not found: ${releaseDir}`);
  }

  for (const fileName of ["libunwind.dll"]) {
    const source = join(bin, fileName);
    if (existsSync(source)) {
      copyFileSync(source, join(releaseDir, fileName));
    }
  }

  for (const fileName of ["WebView2Loader.dll", "libunwind.dll"]) {
    if (!existsSync(join(releaseDir, fileName))) {
      throw new Error(`Required Windows runtime is missing: ${fileName}`);
    }
  }
}
