/**
 * Arborium loader - loads grammar plugins and highlights code.
 *
 * Architecture:
 * 1. Grammar registry is bundled at build time (no network request needed in production)
 *    - Can be overridden via pluginsUrl config for local development
 * 2. Load grammar wasm-bindgen modules on demand from @arborium/<lang> packages
 * 3. Parse and highlight using the grammar's tree-sitter parser
 */

import type { ParseResult, ArboriumConfig, Grammar, Session, Span } from "./types.js";
import { availableLanguages, pluginVersion } from "./plugins-manifest.js";

// Default config
export const defaultConfig: Required<ArboriumConfig> = {
  manual: false,
  theme: "one-dark",
  selector: "pre code",
  cdn: "jsdelivr",
  version: pluginVersion, // Precise version from manifest
  pluginsUrl: "", // Empty means use bundled manifest
  hostUrl: "", // Empty means use CDN based on version
  resolveJs: ({ baseUrl, path }) => import(/* @vite-ignore */ `${baseUrl}/${path}`),
  resolveWasm: ({ baseUrl, path }) => fetch(`${baseUrl}/${path}`),
};

// Rust host module (loaded on demand)
interface HostModule {
  highlight: (language: string, source: string) => string;
  isLanguageAvailable: (language: string) => boolean;
}
let hostModule: HostModule | null = null;
let hostLoadPromise: Promise<HostModule | null> | null = null;

// Merged config
let config: Required<ArboriumConfig> = { ...defaultConfig };

// Grammar plugins cache
const grammarCache = new Map<string, GrammarPlugin>();

// Languages we know are available (bundled at build time)
const knownLanguages: Set<string> = new Set(availableLanguages);

// For local development: can override with pluginsUrl to load from dev server
interface LocalManifest {
  entries: Array<{
    language: string;
    local_js: string;
    local_wasm: string;
  }>;
}
let localManifest: LocalManifest | null = null;
let localManifestPromise: Promise<void> | null = null;

/** Load local manifest if pluginsUrl is configured (for dev server) */
async function ensureLocalManifest(): Promise<void> {
  if (!config.pluginsUrl) {
    return;
  }

  if (localManifestPromise) {
    return localManifestPromise;
  }

  localManifestPromise = (async () => {
    console.debug(`[arborium] Loading local plugins manifest from: ${config.pluginsUrl}`);
    const response = await fetch(config.pluginsUrl);
    if (!response.ok) {
      throw new Error(`Failed to load plugins.json: ${response.status}`);
    }
    localManifest = await response.json();
    console.debug(`[arborium] Loaded local manifest with ${localManifest?.entries.length} entries`);
  })();

  return localManifestPromise;
}

/** Get the CDN base URL for a grammar */
function getGrammarBaseUrl(language: string): string {
  // If we have a local manifest (dev mode), use the local path
  if (localManifest) {
    const entry = localManifest.entries.find((e) => e.language === language);
    if (entry) {
      // Extract base URL from local_js path (e.g., "/langs/group-hazel/python/npm/grammar.js" -> "/langs/group-hazel/python/npm")
      return entry.local_js.substring(0, entry.local_js.lastIndexOf("/"));
    }
  }

  // Production: derive from language name using precise version
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
  return `${baseUrl}/@arborium/${language}@${version}`;
}

type MaybePromise<T> = Promise<T> | T;

// See https://github.com/wasm-bindgen/wasm-bindgen/blob/dda4821ee2fbcaa7adc58bc8c385ed8d3627a272/crates/cli-support/src/js/mod.rs#L860
/** Source of the WASM module for wasm-bindgen */
type WbgInitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

/** wasm-bindgen plugin module interface */
interface WasmBindgenPlugin {
  default: (
    module_or_path?: { module_or_path: MaybePromise<WbgInitInput> } | undefined
    // deprecated: | MaybePromise<WbgInitInput>,
  ) => Promise<void>;
  language_id: () => string;
  injection_languages: () => string[];
  create_session: () => number;
  free_session: (session: number) => void;
  set_text: (session: number, text: string) => void;
  parse: (session: number) => ParseResult;
  cancel: (session: number) => void;
}

/** A loaded grammar plugin */
interface GrammarPlugin {
  languageId: string;
  injectionLanguages: string[];
  module: WasmBindgenPlugin;
  parse: (text: string) => ParseResult;
}

/** Load a grammar plugin */
async function loadGrammarPlugin(language: string): Promise<GrammarPlugin | null> {
  // Check cache
  const cached = grammarCache.get(language);
  if (cached) {
    console.debug(`[arborium] Grammar '${language}' found in cache`);
    return cached;
  }

  // Load local manifest if in dev mode
  await ensureLocalManifest();

  // Check if language is known
  if (
    !knownLanguages.has(language) &&
    !localManifest?.entries.some((e) => e.language === language)
  ) {
    console.debug(`[arborium] Grammar '${language}' not available`);
    return null;
  }

  try {
    const baseUrl = getGrammarBaseUrl(language);
    const detail = config.resolveJs === defaultConfig.resolveJs ? ` from ${baseUrl}/grammar.js` : "";
    console.debug(`[arborium] Loading grammar '${language}'${detail}`);

    const module = (await config.resolveJs({ language, baseUrl, path: "grammar.js" })) as WasmBindgenPlugin;
    const wasm = await config.resolveWasm({ language, baseUrl, path: "grammar_bg.wasm" });

    // Initialize the WASM module
    await module.default({ module_or_path: wasm });

    // Verify it loaded correctly
    const loadedId = module.language_id();
    if (loadedId !== language) {
      console.warn(`[arborium] Language ID mismatch: expected '${language}', got '${loadedId}'`);
    }

    // Get injection languages
    const injectionLanguages = module.injection_languages();

    // Wrap as GrammarPlugin with session-based parsing
    const plugin: GrammarPlugin = {
      languageId: language,
      injectionLanguages,
      module,
      parse: (text: string) => {
        const session = module.create_session();
        try {
          module.set_text(session, text);
          // wasm-bindgen returns ParseResult directly (or throws on error)
          const result = module.parse(session);
          return {
            spans: result.spans || [],
            injections: result.injections || [],
          };
        } catch (e) {
          console.error(`[arborium] Parse error:`, e);
          return { spans: [], injections: [] };
        } finally {
          module.free_session(session);
        }
      },
    };

    grammarCache.set(language, plugin);
    console.debug(`[arborium] Grammar '${language}' loaded successfully`);
    return plugin;
  } catch (e) {
    console.error(`[arborium] Failed to load grammar '${language}':`, e);
    return null;
  }
}

// Handle to plugin mapping for the host interface
const handleToPlugin = new Map<number, GrammarPlugin>();
let nextHandle = 1;

/** Setup window.arboriumHost for the Rust host to call into */
function setupHostInterface(): void {
  (window as any).arboriumHost = {
    /** Check if a language is available (sync) */
    isLanguageAvailable(language: string): boolean {
      return knownLanguages.has(language) || grammarCache.has(language);
    },

    /** Load a grammar and return a handle (async) */
    async loadGrammar(language: string): Promise<number> {
      const plugin = await loadGrammarPlugin(language);
      if (!plugin) return 0; // 0 = not found

      // Check if we already have a handle
      for (const [handle, p] of handleToPlugin) {
        if (p === plugin) return handle;
      }

      // Create new handle
      const handle = nextHandle++;
      handleToPlugin.set(handle, plugin);
      return handle;
    },

    /** Parse text using a grammar handle (sync) */
    parse(handle: number, text: string): ParseResult {
      const plugin = handleToPlugin.get(handle);
      if (!plugin) return { spans: [], injections: [] };
      return plugin.parse(text);
    },
  };
}

/** Get the host URL based on config */
function getHostUrl(): string {
  if (config.hostUrl) {
    return config.hostUrl;
  }
  // Use CDN
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
  return `${baseUrl}/@arborium/arborium${versionSuffix}/dist`;
}

/** Load the Rust host module */
async function loadHost(): Promise<HostModule | null> {
  if (hostModule) return hostModule;
  if (hostLoadPromise) return hostLoadPromise;

  hostLoadPromise = (async () => {
    // Setup the interface the host imports
    setupHostInterface();

    const hostUrl = getHostUrl();
    const jsUrl = `${hostUrl}/arborium_host.js`;
    const wasmUrl = `${hostUrl}/arborium_host_bg.wasm`;

    console.debug(`[arborium] Loading host from ${jsUrl}`);
    try {
      const module = await import(/* @vite-ignore */ jsUrl);
      await module.default(wasmUrl);

      hostModule = {
        highlight: module.highlight,
        isLanguageAvailable: module.isLanguageAvailable,
      };
      console.debug(`[arborium] Host loaded successfully`);
      return hostModule;
    } catch (e) {
      console.error("[arborium] Failed to load host:", e);
      return null;
    }
  })();

  return hostLoadPromise;
}

/** Highlight source code */
export async function highlight(
  language: string,
  source: string,
  _config?: ArboriumConfig,
): Promise<string> {
  // Try to use the Rust host (handles injections properly)
  const host = await loadHost();
  if (host) {
    try {
      return host.highlight(language, source);
    } catch (e) {
      console.warn("Host highlight failed, falling back to JS:", e);
    }
  }

  // Fallback to JS-only highlighting (no injection support)
  const plugin = await loadGrammarPlugin(language);
  if (!plugin) {
    return escapeHtml(source);
  }

  const result = plugin.parse(source);
  return spansToHtml(source, result.spans);
}

/** Load a grammar for direct use */
export async function loadGrammar(
  language: string,
  _config?: ArboriumConfig,
): Promise<Grammar | null> {
  const plugin = await loadGrammarPlugin(language);
  if (!plugin) return null;

  const { module } = plugin;

  return {
    languageId: () => plugin.languageId,
    injectionLanguages: () => plugin.injectionLanguages,
    highlight: async (source: string) => {
      const result = plugin.parse(source);
      return spansToHtml(source, result.spans);
    },
    parse: (source: string) => plugin.parse(source),
    createSession: (): Session => {
      const handle = module.create_session();
      return {
        setText: (text: string) => module.set_text(handle, text),
        parse: () => {
          try {
            const result = module.parse(handle);
            return {
              spans: result.spans || [],
              injections: result.injections || [],
            };
          } catch (e) {
            console.error(`[arborium] Session parse error:`, e);
            return { spans: [], injections: [] };
          }
        },
        cancel: () => module.cancel(handle),
        free: () => module.free_session(handle),
      };
    },
    dispose: () => {
      // No-op for now, plugins are cached
    },
  };
}

/** Convert spans to HTML */
export function spansToHtml(source: string, spans: Span[]): string {
  // Sort spans by start position
  const sorted = [...spans].sort((a, b) => a.start - b.start);

  let html = "";
  let pos = 0;

  for (const span of sorted) {
    // Skip overlapping spans
    if (span.start < pos) continue;

    // Add text before span
    if (span.start > pos) {
      html += escapeHtml(source.slice(pos, span.start));
    }

    // Get tag for capture
    const tag = getTagForCapture(span.capture);
    const text = escapeHtml(source.slice(span.start, span.end));

    if (tag) {
      html += `<a-${tag}>${text}</a-${tag}>`;
    } else {
      html += text;
    }

    pos = span.end;
  }

  // Add remaining text
  if (pos < source.length) {
    html += escapeHtml(source.slice(pos));
  }

  return html;
}

/** Get the short tag for a capture name */
function getTagForCapture(capture: string): string | null {
  if (capture.startsWith("keyword") || capture === "include" || capture === "conditional") {
    return "k";
  }
  if (capture.startsWith("function") || capture.startsWith("method")) {
    return "f";
  }
  if (capture.startsWith("string") || capture === "character") {
    return "s";
  }
  if (capture.startsWith("comment")) {
    return "c";
  }
  if (capture.startsWith("type")) {
    return "t";
  }
  if (capture.startsWith("variable")) {
    return "v";
  }
  if (capture.startsWith("number") || capture === "float") {
    return "n";
  }
  if (capture.startsWith("operator")) {
    return "o";
  }
  if (capture.startsWith("punctuation")) {
    return "p";
  }
  if (capture.startsWith("tag")) {
    return "tg";
  }
  if (capture.startsWith("attribute")) {
    return "at";
  }
  return null;
}

/** Escape HTML special characters */
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/** Get current config, optionally merging with overrides */
export function getConfig(overrides?: Partial<ArboriumConfig>): Required<ArboriumConfig> {
  if (overrides) {
    return { ...config, ...overrides };
  }
  return { ...config };
}

/** Set/merge config */
export function setConfig(newConfig: Partial<ArboriumConfig>): void {
  config = { ...config, ...newConfig };
}

/** Check if a language is available */
export async function isLanguageAvailable(language: string): Promise<boolean> {
  await ensureLocalManifest();
  return (
    knownLanguages.has(language) ||
    (localManifest?.entries.some((e) => e.language === language) ?? false)
  );
}

/** Get list of available languages */
export async function getAvailableLanguages(): Promise<string[]> {
  await ensureLocalManifest();
  // In dev mode, use local manifest if available
  if (localManifest) {
    return localManifest.entries.map((e) => e.language);
  }
  return Array.from(knownLanguages);
}
