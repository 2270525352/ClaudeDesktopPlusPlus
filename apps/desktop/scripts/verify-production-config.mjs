import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const appRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const configPath = join(appRoot, "src-tauri", "tauri.conf.json");
const config = JSON.parse(readFileSync(configPath, "utf8"));
const serialized = JSON.stringify(config);

if (config.build?.devUrl || /(?:127\.0\.0\.1|localhost):5177/i.test(serialized)) {
  console.error(
    "Production Tauri config must not depend on the localhost development server.",
  );
  process.exit(1);
}

if (!config.build?.frontendDist) {
  console.error("Production Tauri config must define an embedded frontendDist.");
  process.exit(1);
}

console.log(`Production UI will be embedded from ${config.build.frontendDist}`);
