import { copyFileSync, existsSync, mkdirSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const appRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const targetTriple = process.env.CLAUDE_PLUS_WINDOWS_TARGET || "x86_64-pc-windows-gnullvm";
const releaseDir = join(appRoot, "src-tauri", "target", targetTriple, "release");

if (process.platform !== "win32" && process.env.TAURI_ENV_PLATFORM !== "windows") {
  process.exit(0);
}

mkdirSync(releaseDir, { recursive: true });

const llvmMingwBin = findLlvmMingwBin();
if (!llvmMingwBin) {
  throw new Error("llvm-mingw runtime directory was not found under %USERPROFILE%\\.local\\tools");
}

copyRuntimeDll("libunwind.dll", llvmMingwBin);

function copyRuntimeDll(fileName, sourceDir) {
  const source = join(sourceDir, fileName);
  if (!existsSync(source)) {
    throw new Error(`Missing runtime DLL: ${source}`);
  }
  copyFileSync(source, join(releaseDir, fileName));
}

function findLlvmMingwBin() {
  const home = process.env.USERPROFILE;
  if (!home) return undefined;

  const toolsRoot = join(home, ".local", "tools");
  if (!existsSync(toolsRoot)) return undefined;

  const candidates = readdirSync(toolsRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .filter((name) => name.startsWith("llvm-mingw-") && name.includes("-ucrt-x86_64"))
    .sort()
    .reverse();

  for (const name of candidates) {
    const rootBin = join(toolsRoot, name, "bin");
    const targetBin = join(toolsRoot, name, "x86_64-w64-mingw32", "bin");
    if (existsSync(join(targetBin, "libunwind.dll"))) return targetBin;
    if (existsSync(join(rootBin, "libunwind.dll"))) return rootBin;
  }

  return undefined;
}
