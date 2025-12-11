/**
 * Arborium loader - loads grammar plugins and highlights code.
 *
 * Architecture:
 * 1. Fetch plugins.json from arborium.bearcove.eu to get grammar CDN URLs
 * 2. Load grammar WIT components on demand from @arborium/<lang> packages
 * 3. Parse and highlight using the grammar's tree-sitter parser
 */

import { createWasiImports, grammarTypesImport } from './wasi-shims.js';
import type { ParseResult, ArboriumConfig, Grammar, Span, Injection } from './types.js';

// Default config
export const defaultConfig: Required<ArboriumConfig> = {
  manual: false,
  theme: 'tokyo-night',
  selector: 'pre code',
  cdn: 'jsdelivr',
  version: 'latest',
  pluginsUrl: 'https://arborium.bearcove.eu/plugins.json',
  hostUrl: '', // Empty means use CDN based on version
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

// Plugin manifest from plugins.json
interface PluginEntry {
  language: string;
  package: string;
  version: string;
  cdn_js: string;
  cdn_wasm: string;
  local_js: string;
  local_wasm: string;
}

interface PluginsManifest {
  dev_mode: boolean;
  generated_at: string;
  entries: PluginEntry[];
}

let pluginsManifest: PluginsManifest | null = null;

// Languages we know are available (from plugins.json)
let availableLanguages: Set<string> = new Set();

/** WIT Result type as returned by jco-generated code */
type WitResult<T, E> =
  | { tag: 'ok'; val: T }
  | { tag: 'err'; val: E };

/** Plugin interface as exported by jco-generated WIT components */
interface JcoPlugin {
  languageId(): string;
  injectionLanguages(): string[];
  createSession(): number;
  freeSession(session: number): void;
  setText(session: number, text: string): void;
  parse(session: number): WitResult<ParseResult, { message: string }>;
}

/** A loaded grammar plugin (WIT component) */
interface GrammarPlugin {
  languageId: string;
  injectionLanguages: string[];
  parse: (text: string) => ParseResult;
}

/** Load the plugins manifest */
async function loadPluginsManifest(): Promise<void> {
  if (pluginsManifest) return;

  console.debug(`[arborium] Loading plugins manifest: ${config.pluginsUrl}`);
  const response = await fetch(config.pluginsUrl);
  if (!response.ok) {
    throw new Error(`Failed to load plugins.json: ${response.status}`);
  }
  pluginsManifest = await response.json();

  // Populate available languages
  availableLanguages = new Set(pluginsManifest!.entries.map(e => e.language));
  console.debug(`[arborium] Available languages: ${Array.from(availableLanguages).join(', ')}`);
}

/** Load a grammar plugin */
async function loadGrammarPlugin(language: string): Promise<GrammarPlugin | null> {
  // Check cache
  const cached = grammarCache.get(language);
  if (cached) {
    console.debug(`[arborium] Grammar '${language}' found in cache`);
    return cached;
  }

  // Ensure manifest is loaded
  await loadPluginsManifest();

  // Find language entry
  const entry = pluginsManifest?.entries.find(e => e.language === language);
  if (!entry) {
    console.debug(`[arborium] Grammar '${language}' not found in manifest`);
    return null;
  }

  try {
    // Always use CDN URLs for grammars (they're published packages)
    // hostUrl only affects the host module loading
    const jsUrl = entry.cdn_js;
    const baseUrl = jsUrl.substring(0, jsUrl.lastIndexOf('/'));

    console.debug(`[arborium] Loading grammar '${language}' from ${jsUrl}`);
    // Dynamically import the JS module
    const module = await import(/* @vite-ignore */ jsUrl);

    // Create a getCoreModule function that fetches WASM files by path
    const getCoreModule = async (path: string): Promise<WebAssembly.Module> => {
      const wasmUrl = `${baseUrl}/${path}`;
      const response = await fetch(wasmUrl);
      if (!response.ok) {
        throw new Error(`Failed to fetch WASM ${path}: ${response.status}`);
      }
      const bytes = await response.arrayBuffer();
      return WebAssembly.compile(bytes);
    };

    // Create WASI imports
    const wasiImports = createWasiImports();
    const imports = {
      ...wasiImports,
      ...grammarTypesImport,
    };

    // Instantiate the jco-generated component
    const instance = await module.instantiate(getCoreModule, imports);

    // Get the plugin interface
    const jcoPlugin = (instance.plugin || instance['arborium:grammar/plugin@0.1.0']) as JcoPlugin;
    if (!jcoPlugin) {
      console.error(`Grammar '${language}' missing plugin interface`);
      return null;
    }

    // Wrap as GrammarPlugin with session-based parsing
    const plugin: GrammarPlugin = {
      languageId: language,
      injectionLanguages: jcoPlugin.injectionLanguages?.() ?? [],
      parse: (text: string) => {
        const session = jcoPlugin.createSession();
        try {
          jcoPlugin.setText(session, text);
          const result = jcoPlugin.parse(session);

          // Handle various result shapes from jco
          // Some versions return { tag: 'ok', val: ParseResult }
          // Others might return ParseResult directly
          if (result.tag === 'err') {
            const err = result.val as { message?: string };
            console.error(`[arborium] Parse error: ${err?.message}`);
            return { spans: [], injections: [] };
          }

          // Extract the actual value - could be result.val or result itself
          const val = result.tag === 'ok' ? result.val : result;
          if (!val || typeof val !== 'object') {
            console.error(`[arborium] Unexpected parse result:`, result);
            return { spans: [], injections: [] };
          }

          // Access spans/injections with type coercion
          const parsed = val as { spans?: Span[]; injections?: Injection[] };
          return {
            spans: parsed.spans || [],
            injections: parsed.injections || [],
          };
        } finally {
          jcoPlugin.freeSession(session);
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
      return availableLanguages.has(language) || grammarCache.has(language);
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
  if (cdn === 'jsdelivr') {
    baseUrl = 'https://cdn.jsdelivr.net/npm';
  } else if (cdn === 'unpkg') {
    baseUrl = 'https://unpkg.com';
  } else {
    baseUrl = cdn;
  }
  const versionSuffix = version === 'latest' ? '' : `@${version}`;
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
      console.error('[arborium] Failed to load host:', e);
      return null;
    }
  })();

  return hostLoadPromise;
}

/** Highlight source code */
export async function highlight(language: string, source: string, _config?: ArboriumConfig): Promise<string> {
  // Try to use the Rust host (handles injections properly)
  const host = await loadHost();
  if (host) {
    try {
      return host.highlight(language, source);
    } catch (e) {
      console.warn('Host highlight failed, falling back to JS:', e);
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
export async function loadGrammar(language: string, _config?: ArboriumConfig): Promise<Grammar | null> {
  const plugin = await loadGrammarPlugin(language);
  if (!plugin) return null;

  return {
    languageId: () => plugin.languageId,
    injectionLanguages: () => plugin.injectionLanguages,
    highlight: async (source: string) => {
      const result = plugin.parse(source);
      return spansToHtml(source, result.spans);
    },
    parse: (source: string) => plugin.parse(source),
    dispose: () => {
      // No-op for now, plugins are cached
    },
  };
}

/** Convert spans to HTML */
export function spansToHtml(source: string, spans: Span[]): string {
  // Sort spans by start position
  const sorted = [...spans].sort((a, b) => a.start - b.start);

  let html = '';
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
  if (capture.startsWith('keyword') || capture === 'include' || capture === 'conditional') {
    return 'k';
  }
  if (capture.startsWith('function') || capture.startsWith('method')) {
    return 'f';
  }
  if (capture.startsWith('string') || capture === 'character') {
    return 's';
  }
  if (capture.startsWith('comment')) {
    return 'c';
  }
  if (capture.startsWith('type')) {
    return 't';
  }
  if (capture.startsWith('variable')) {
    return 'v';
  }
  if (capture.startsWith('number') || capture === 'float') {
    return 'n';
  }
  if (capture.startsWith('operator')) {
    return 'o';
  }
  if (capture.startsWith('punctuation')) {
    return 'p';
  }
  if (capture.startsWith('tag')) {
    return 'tg';
  }
  if (capture.startsWith('attribute')) {
    return 'at';
  }
  return null;
}

/** Escape HTML special characters */
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
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
  await loadPluginsManifest();
  return availableLanguages.has(language);
}

/** Get list of available languages */
export async function getAvailableLanguages(): Promise<string[]> {
  await loadPluginsManifest();
  return Array.from(availableLanguages);
}
