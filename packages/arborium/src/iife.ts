/**
 * @arborium/arborium IIFE bundle
 *
 * Drop-in auto-highlighter that runs on page load.
 * Configuration via data attributes or window.Arborium object.
 *
 * ## Config Waterfall (highest to lowest priority)
 *
 * 1. Function-level configOverrides (passed to highlight(), highlightAll(), etc.)
 * 2. globalConfig (set once at load, can be changed via setConfig())
 * 3. data-* attributes on the script tag
 * 4. window.Arborium object (set before script loads)
 * 5. Auto-detected theme (rustdoc or system preference)
 * 6. defaultConfig (built into loader.ts)
 *
 * The config is merged ONCE at script load time (levels 3-6), then stored in
 * globalConfig. All functions use getConfig(overrides) to access it, allowing
 * per-call overrides (level 1) to take precedence.
 */

import { loadGrammar, highlight, getConfig, setConfig } from "./loader.js";
import { detectLanguage, extractLanguageFromClass, normalizeLanguage } from "./detect.js";
import type { ArboriumConfig } from "./types.js";

// =============================================================================
// SECTION 1: Script Tag Capture (must happen synchronously at load time)
// =============================================================================

const currentScript = document.currentScript as HTMLScriptElement | null;

// =============================================================================
// SECTION 2: Config Source Readers (read from data-*, window.Arborium, env)
// =============================================================================

/** Parse query parameters from script src URL (for local testing) */
function getQueryParams(): URLSearchParams {
  if (!currentScript?.src) return new URLSearchParams();
  try {
    const url = new URL(currentScript.src);
    return url.searchParams;
  } catch {
    return new URLSearchParams();
  }
}

/** Parse configuration from script data attributes and query params */
function getConfigFromScript(): Partial<ArboriumConfig> {
  if (!currentScript) return {};

  const config: Partial<ArboriumConfig> = {};
  const params = getQueryParams();

  // Data attributes
  if (currentScript.hasAttribute("data-manual")) {
    config.manual = true;
  }

  const theme = currentScript.getAttribute("data-theme");
  if (theme) config.theme = theme;

  const selector = currentScript.getAttribute("data-selector");
  if (selector) config.selector = selector;

  const cdn = currentScript.getAttribute("data-cdn");
  if (cdn) config.cdn = cdn;

  const version = currentScript.getAttribute("data-version");
  if (version) config.version = version;

  // Query parameters (for local testing)
  const pluginsUrl = params.get("pluginsUrl");
  if (pluginsUrl) config.pluginsUrl = pluginsUrl;

  const hostUrl = params.get("hostUrl");
  if (hostUrl) config.hostUrl = hostUrl;

  return config;
}

/** Detect if we're running on a rustdoc-generated page */
function isRustdocEnvironment(): boolean {
  const generator = document.querySelector('meta[name="generator"]');
  return generator?.getAttribute("content") === "rustdoc";
}

/** Map rustdoc theme names to Arborium theme IDs */
function mapRustdocTheme(value?: string): string | null {
  if (!value) return null;

  const themeMap: Record<string, string> = {
    light: "rustdoc-light",
    dark: "rustdoc-dark",
    ayu: "rustdoc-ayu",
  };

  return themeMap[value] ?? null;
}

/** Detect the current theme from rustdoc or system preference */
function getAutoTheme(): string {
  if (isRustdocEnvironment()) {
    const rustdocTheme = mapRustdocTheme(document.documentElement.dataset.theme);
    if (rustdocTheme) {
      return rustdocTheme;
    }
  }

  // Fall back to system preference
  const isLight = window.matchMedia("(prefers-color-scheme: light)").matches;
  return isLight ? "github-light" : "one-dark";
}

// =============================================================================
// SECTION 3: Config Initialization (called ONCE at script load)
// =============================================================================

/**
 * Merge configuration from all IIFE-specific sources into globalConfig.
 * Called ONCE at script load time. After this, use getConfig(overrides).
 */
function initializeConfig(): void {
  // Priority: data attributes > window.Arborium > auto-detect > defaults
  const windowConfig = window.Arborium || {};
  const scriptConfig = getConfigFromScript();
  const merged = { ...windowConfig, ...scriptConfig };

  // Auto-detect theme if not explicitly set
  if (!merged.theme) {
    merged.theme = getAutoTheme();
  }

  // Apply to global config (this is the ONLY setConfig call during init)
  setConfig(merged);
}

// =============================================================================
// SECTION 4: CSS Injection
// =============================================================================

let currentThemeId: string | null = null;

/** Get the CSS base URL for a given config */
function getCssBaseUrl(config: Required<ArboriumConfig>): string {
  if (config.hostUrl) {
    return `${config.hostUrl}/themes`;
  }

  const cdn = config.cdn;
  const version = config.version;

  let baseUrl: string;
  if (cdn === "jsdelivr") {
    baseUrl = "https://cdn.jsdelivr.net/npm";
  } else if (cdn === "unpkg") {
    baseUrl = "https://unpkg.com";
  } else {
    baseUrl = cdn;
  }

  const versionSuffix = version === "latest" ? "" : `@${version}`;
  return `${baseUrl}/@arborium/arborium${versionSuffix}/dist/themes`;
}

/** Inject base CSS (only once) */
function injectBaseCSS(config: Required<ArboriumConfig>): void {
  const baseId = "arborium-base";
  if (document.getElementById(baseId)) return;

  const cssUrl = `${getCssBaseUrl(config)}/base-rustdoc.css`;
  console.debug(`[arborium] Loading base CSS: ${cssUrl}`);

  const link = document.createElement("link");
  link.id = baseId;
  link.rel = "stylesheet";
  link.href = cssUrl;
  document.head.appendChild(link);
}

/** Inject theme CSS, removing any previously loaded theme */
function injectThemeCSS(config: Required<ArboriumConfig>): void {
  const theme = config.theme;

  // Remove old theme if different
  if (currentThemeId && currentThemeId !== theme) {
    const oldLink = document.getElementById(`arborium-theme-${currentThemeId}`);
    if (oldLink) {
      oldLink.remove();
      console.debug(`[arborium] Removed theme: ${currentThemeId}`);
    }
  }

  const themeId = `arborium-theme-${theme}`;
  if (document.getElementById(themeId)) {
    currentThemeId = theme;
    return;
  }

  const cssUrl = `${getCssBaseUrl(config)}/${theme}.css`;
  console.debug(`[arborium] Loading theme: ${cssUrl}`);

  const link = document.createElement("link");
  link.id = themeId;
  link.rel = "stylesheet";
  link.href = cssUrl;
  document.head.appendChild(link);

  currentThemeId = theme;
}

// =============================================================================
// SECTION 5: Code Block Detection
// =============================================================================

/** Find all code blocks matching the selector */
function findCodeBlocks(selector: string): HTMLElement[] {
  return Array.from(document.querySelectorAll(selector));
}

/** Check if a code block already has syntax highlighting or semantic markup */
function hasExistingHighlighting(block: HTMLElement): boolean {
  // Check for common highlighting library markers
  const highlightClasses = ["hljs", "highlighted", "prism-code", "shiki"];
  for (const cls of highlightClasses) {
    if (block.classList.contains(cls)) return true;
  }

  // Check if there are spans with syntax highlighting classes inside
  const spans = block.querySelectorAll("span[class]");
  if (spans.length > 0) {
    const classedSpans = Array.from(spans).filter((s) => s.className && s.className.length > 0);
    if (classedSpans.length >= 3) return true;
  }

  // Check for semantic markup (e.g., docs.rs uses <a> tags for type/function links)
  const links = block.querySelectorAll("a");
  if (links.length >= 2) return true;

  return false;
}

/** Get the language for a code block */
function getLanguageForBlock(block: HTMLElement): string | null {
  // Check data-lang attribute
  const dataLang = block.getAttribute("data-lang");
  if (dataLang) return normalizeLanguage(dataLang);

  // Check class="language-*"
  const className = block.className;
  const classLang = extractLanguageFromClass(className);
  if (classLang) return normalizeLanguage(classLang);

  // Check parent element (often <pre> wraps <code>)
  const parent = block.parentElement;
  if (parent) {
    const parentDataLang = parent.getAttribute("data-lang");
    if (parentDataLang) return normalizeLanguage(parentDataLang);

    const parentClassLang = extractLanguageFromClass(parent.className);
    if (parentClassLang) return normalizeLanguage(parentClassLang);
  }

  // Try auto-detection
  const source = block.textContent || "";
  return detectLanguage(source);
}

// =============================================================================
// SECTION 6: Highlighting Logic
// =============================================================================

/** Highlight a single code block */
async function highlightBlock(
  block: HTMLElement,
  language: string,
  config: Partial<ArboriumConfig>,
): Promise<void> {
  const source = block.textContent || "";
  if (!source.trim()) return;

  try {
    const html = await highlight(language, source, config);
    block.innerHTML = html;
    block.setAttribute("data-highlighted", "true");
    block.setAttribute("data-lang", language);
  } catch (err) {
    console.warn(`[arborium] Failed to highlight ${language}:`, err);
  }
}

/** Auto-highlight all code blocks on the page */
async function autoHighlight(configOverrides?: Partial<ArboriumConfig>): Promise<void> {
  const config = getConfig(configOverrides);

  // Inject CSS
  injectBaseCSS(config);
  injectThemeCSS(config);

  // Find all code blocks
  const blocks = findCodeBlocks(config.selector);
  if (blocks.length === 0) return;

  // Group blocks by language
  const blocksByLanguage = new Map<string, HTMLElement[]>();
  const unknownBlocks: HTMLElement[] = [];

  for (const block of blocks) {
    if (block.hasAttribute("data-highlighted")) continue;
    if (hasExistingHighlighting(block)) continue;

    const language = getLanguageForBlock(block);
    if (language) {
      const existing = blocksByLanguage.get(language) || [];
      existing.push(block);
      blocksByLanguage.set(language, existing);
    } else {
      unknownBlocks.push(block);
    }
  }

  // Load grammars in parallel
  const languages = Array.from(blocksByLanguage.keys());
  const loadPromises = languages.map((lang) =>
    loadGrammar(lang, config).catch((err) => {
      console.warn(`[arborium] Failed to load grammar for ${lang}:`, err);
      return null;
    }),
  );

  const grammars = await Promise.all(loadPromises);

  // Highlight blocks
  const highlightPromises: Promise<void>[] = [];
  for (let i = 0; i < languages.length; i++) {
    const language = languages[i];
    const grammar = grammars[i];
    if (!grammar) continue;

    const languageBlocks = blocksByLanguage.get(language) || [];
    for (const block of languageBlocks) {
      highlightPromises.push(highlightBlock(block, language, config));
    }
  }

  await Promise.all(highlightPromises);

  // Log summary
  const total = blocks.length;
  const highlighted = blocks.filter((b) => b.hasAttribute("data-highlighted")).length;
  const skipped = unknownBlocks.length;

  if (highlighted > 0 || skipped > 0) {
    console.debug(
      `[arborium] Highlighted ${highlighted}/${total} blocks` +
        (skipped > 0 ? ` (${skipped} unknown language)` : ""),
    );
  }
}

// =============================================================================
// SECTION 7: Theme Change Watching
// =============================================================================

/** Handle theme changes (rustdoc or system preference) */
async function onThemeChange(): Promise<void> {
  const newTheme = getAutoTheme();

  if (currentThemeId !== newTheme) {
    setConfig({ theme: newTheme });
    injectThemeCSS(getConfig());
    console.debug(`[arborium] Theme changed to: ${newTheme}`);
  }
}

/** Set up watchers for theme changes */
function watchThemeChanges(): void {
  // Watch for rustdoc theme attribute changes
  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      if (mutation.attributeName === "data-theme" && isRustdocEnvironment()) {
        onThemeChange();
        break;
      }
    }
  });
  observer.observe(document.documentElement, { attributes: true });

  // Watch for system color scheme changes
  window
    .matchMedia("(prefers-color-scheme: light)")
    .addEventListener("change", () => onThemeChange());
}

// =============================================================================
// SECTION 8: Public API
// =============================================================================

/** Highlight all code blocks on the page */
export async function highlightAll(configOverrides?: Partial<ArboriumConfig>): Promise<void> {
  await autoHighlight(configOverrides);
}

/** Highlight a specific element */
export async function highlightElement(
  element: HTMLElement,
  language?: string,
  configOverrides?: Partial<ArboriumConfig>,
): Promise<void> {
  const config = getConfig(configOverrides);
  const lang = language || getLanguageForBlock(element);

  if (!lang) {
    console.warn("[arborium] Could not detect language for element");
    return;
  }

  await highlightBlock(element, lang, config);
}

// =============================================================================
// SECTION 9: Initialization (runs at script load)
// =============================================================================

// Step 1: Merge config from all IIFE-specific sources ONCE
initializeConfig();

// Step 2: Expose public API on window
(window as any).arborium = {
  // Highlighting functions
  highlightAll,
  highlightElement,
  loadGrammar,
  highlight,
  detectLanguage,

  // Config access (getter returns current config, setter merges into it)
  get config() {
    return getConfig();
  },
  set config(overrides: Partial<ArboriumConfig>) {
    setConfig(overrides);
  },
  getConfig,
  setConfig,
};

// Step 3: Auto-highlight on DOMContentLoaded (unless manual mode)
const initialConfig = getConfig();
if (!initialConfig.manual) {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => {
      autoHighlight();
      watchThemeChanges();
    });
  } else {
    autoHighlight();
    watchThemeChanges();
  }
}
