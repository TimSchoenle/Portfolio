// Serves the site locally with SSR + hydration and live reload:
//
//   node apps/web/scripts/dev.mjs [port]     (default 8080; `just dev` runs this)
//
// Written for Node rather than a shell because Node is already a prerequisite and behaves the
// same on every platform, where a POSIX shell is not guaranteed on Windows. Uses only Node's
// standard library, so it runs before `npm ci` has.
//
// Before starting `dx serve`, it prepares what the web build embeds or serves:
//   - the resume PDFs, their fingerprints and the social card, generated once into
//     `apps/web/generated` (delete that directory to regenerate them);
//   - the Tailwind stylesheet, built once and then rebuilt by a watcher on every change.
// The third-party license inventory is left to `just licenses`; without it `/licenses` renders
// its empty state.
//
// The server refuses to start without legal documents, so `PORTFOLIO_CONFIG` points it at the
// templates in `legal/`.

import { spawn, spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const webRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = join(webRoot, "..", "..");
const port = process.argv[2] ?? "8080";

if (!/^\d+$/.test(port)) {
  fail(`port must be a number, got "${port}"`);
}

/** Prints `message` and exits non-zero. */
function fail(message) {
  console.error(`dev: ${message}`);
  process.exit(1);
}

/**
 * Runs `command` to completion with inherited output, exiting when it fails. `shell` is needed
 * only for npm, which is a `.cmd` script on Windows that Node refuses to spawn without one.
 */
function run(command, args, { cwd, shell = false } = {}) {
  console.log(`dev: ${command} ${args.join(" ")}`);
  const result = spawnSync(command, args, { cwd, stdio: "inherit", shell });
  if (result.error) {
    fail(`could not start ${command}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    fail(`${command} exited with status ${result.status}`);
  }
}

/** Arguments that run the Tailwind CLI with Node directly, so the watcher can be stopped. */
function tailwindArgs(...extra) {
  const cliRoot = join(webRoot, "node_modules", "@tailwindcss", "cli");
  const { bin } = JSON.parse(readFileSync(join(cliRoot, "package.json"), "utf8"));
  const entry = typeof bin === "string" ? bin : bin.tailwindcss;
  return [join(cliRoot, entry), "-i", "assets/input.css", "-o", "assets/tailwind.css", ...extra];
}

if (!existsSync(join(webRoot, "generated", "resume-fingerprint.json"))) {
  run("cargo", ["run", "--profile", "tools", "-p", "resume-generator", "--", "apps/web/generated"], {
    cwd: repoRoot,
  });
}

if (!existsSync(join(webRoot, "node_modules"))) {
  run("npm", ["ci"], { cwd: webRoot, shell: process.platform === "win32" });
}

run(process.execPath, tailwindArgs("--minify"), { cwd: webRoot });

// `--watch=always` keeps the watcher alive with no terminal attached to its stdin.
const watcher = spawn(process.execPath, tailwindArgs("--watch=always"), {
  cwd: webRoot,
  stdio: "ignore",
});

const server = spawn("dx", ["serve", "--platform", "web", "--port", port], {
  cwd: webRoot,
  stdio: "inherit",
  env: { ...process.env, PORTFOLIO_CONFIG: join(repoRoot, "legal") },
});

const stop = () => {
  watcher.kill();
  server.kill();
};
for (const signal of ["SIGINT", "SIGTERM"]) {
  process.on(signal, stop);
}

server.on("error", (error) => {
  watcher.kill();
  fail(`could not start dx (install it with \`cargo install --locked dioxus-cli\`): ${error.message}`);
});
server.on("exit", (code) => {
  watcher.kill();
  process.exit(code ?? 1);
});
