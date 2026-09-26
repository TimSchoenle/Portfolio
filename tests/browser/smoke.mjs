// Browser smoke test: loads the running site in headless Chromium and checks what only a real
// browser can see — that the wasm client hydrates without errors, in the language the server
// rendered, under the Content-Security-Policy the server sent.
//
// Usage: node smoke.mjs [base-url]   (default http://localhost:8080)
//
// A CSP refusal, a hydration panic or a failed module load all surface as a console error or an
// uncaught page error, so any of those fails the run.

import { chromium } from "playwright";

const base = process.argv[2] ?? "http://localhost:8080";
const failures = [];
const fail = (message) => {
  failures.push(message);
  console.error(`FAIL: ${message}`);
};

/**
 * Opens `path` in a fresh context with `locale` (which sets `Accept-Language`), waits for the
 * client to hydrate, and returns the page with every console error and page error it produced.
 */
async function open(browser, path, locale) {
  const context = await browser.newContext({ locale });
  const page = await context.newPage();
  const errors = [];
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto(`${base}${path}`, { waitUntil: "networkidle" });
  // Hydration is done once the client has attached its listeners; the palette shortcut is
  // one of them, so opening the palette proves the client is live.
  await page.keyboard.press("Control+k");
  const palette = await page
    .waitForSelector(".cmdk-modal", { timeout: 15_000 })
    .then(() => true)
    .catch(() => false);
  if (palette) await page.keyboard.press("Escape");
  return { context, page, errors, palette };
}

async function check(browser, label, path, locale, expectedLang) {
  const { context, page, errors, palette } = await open(browser, path, locale);
  try {
    if (!palette) fail(`${label}: the client never became interactive (⌘K opened nothing)`);
    for (const error of errors) fail(`${label}: ${error}`);
    // The masthead mirrors the client's active language onto <html lang>, so a client that
    // hydrated in a different language than the server rendered shows up here.
    const lang = await page.evaluate(() => document.documentElement.lang);
    if (lang !== expectedLang) {
      fail(`${label}: hydrated as '${lang}', the server rendered '${expectedLang}'`);
    } else {
      console.log(`ok: ${label} hydrates as '${lang}' with no console errors`);
    }
  } finally {
    await context.close();
  }
}

const browser = await chromium.launch();
try {
  await check(browser, "English /", "/", "en-US", "en");
  // Twice in fresh contexts (no cookie): the second request is answered from the server's
  // page memo, whose render never ran — the case that used to hydrate a German page in English.
  await check(browser, "German / (first)", "/", "de-DE", "de");
  await check(browser, "German / (cached)", "/", "de-DE", "de");
  await check(browser, "?lang=de", "/?lang=de", "en-US", "de");
  await check(browser, "/licenses", "/licenses", "en-US", "en");
  await check(browser, "/legal/imprint", "/legal/imprint", "de-DE", "de");
} finally {
  await browser.close();
}

if (failures.length > 0) {
  console.error(`${failures.length} browser check(s) failed`);
  process.exit(1);
}
console.log("All browser smoke tests passed.");
