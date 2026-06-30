// Vendors the exact self-hosted web fonts the site uses from the Fontsource
// npm packages into `assets/fonts/`, which Trunk then serves (see
// `assets/fonts.css` + the `copy-dir` link in `index.html`). The packages are
// dev dependencies so Renovate tracks upstream font updates; re-run this script
// after a bump to refresh the committed woff2 files:
//
//   npm run vendor:fonts
//
// Only the weights/styles/subsets actually referenced by `assets/fonts.css` are
// copied, keeping the served payload minimal. Keep the two in sync.

import { copyFileSync, existsSync, mkdirSync, readdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const frontendRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const dest = join(frontendRoot, "assets", "fonts");
const fontsourceRoot = join(frontendRoot, "node_modules", "@fontsource");

// "latin" covers ASCII + Latin-1 (German umlauts/ß and the common Western
// accents). The heavier "latin-ext" subset is intentionally not vendored: the
// few extended glyphs that may appear in repo names fall back to the system
// font via the @font-face unicode-range, which keeps the shipped payload
// minimal without producing tofu.
const SUBSETS = ["latin"];

// pkg, weights, styles — mirrors the @font-face blocks in assets/fonts.css.
const SPEC = [
  { pkg: "space-grotesk", weights: [400, 500, 600, 700], styles: ["normal"] },
  { pkg: "jetbrains-mono", weights: [400, 500, 700], styles: ["normal"] },
];

rmSync(dest, { recursive: true, force: true });
mkdirSync(dest, { recursive: true });

let copied = 0;
for (const { pkg, weights, styles } of SPEC) {
  const filesDir = join(fontsourceRoot, pkg, "files");
  if (!existsSync(filesDir)) {
    throw new Error(`@fontsource/${pkg} is not installed — run \`npm install\` first.`);
  }
  for (const subset of SUBSETS) {
    for (const weight of weights) {
      for (const style of styles) {
        const file = `${pkg}-${subset}-${weight}-${style}.woff2`;
        const from = join(filesDir, file);
        if (!existsSync(from)) {
          throw new Error(`expected font file missing: ${file}`);
        }
        copyFileSync(from, join(dest, file));
        copied += 1;
      }
    }
  }
}

console.log(`vendor-fonts: copied ${copied} woff2 file(s) into ${join("assets", "fonts")}`);
console.log(`              (${readdirSync(dest).length} file(s) now present)`);
