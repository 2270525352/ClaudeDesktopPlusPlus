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
const child = spawn(process.execPath, [tauriCli, ...args], {
  cwd: appRoot,
  env,
  shell: false,
  stdio: "inherit",
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }
  if (code === 0 && process.platform === "win32" && command === "build" && llvmMingwBin) {
    copyWindowsRuntimeDlls(llvmMingwBin);
  }
  process.exit(code ?? 1);
});

function hasTargetArg(values) {
  return values.some((value) => value === "--target" || value === "-t" || value.startsWith("--target="));
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

function copyWindowsRuntimeDlls(bin) {
  const releaseDir = join(appRoot, "src-tauri", "target", "x86_64-pc-windows-gnullvm", "release");
  if (!existsSync(releaseDir)) {
    return;
  }

  for (const fileName of ["libunwind.dll"]) {
    const source = join(bin, fileName);
    if (existsSync(source)) {
      copyFileSync(source, join(releaseDir, fileName));
    }
  }
}
