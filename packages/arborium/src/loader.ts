import { createWasiImports, grammarTypesImport } from './wasi-shims.js';
import type { Grammar, ParseResult, Span, ArboriumConfig } from './types.js';

/** CDN base URLs */
const CDN_URLS: Record<string, string> = {
  jsdelivr: 'https://cdn.jsdelivr.net/npm',
  unpkg: 'https://unpkg.com',
};

/** Cache of loaded grammars */
const grammarCache = new Map<string, Grammar>();

/** Cache of in-flight loads */
const loadingCache = new Map<string, Promise<Grammar>>();

/** Default configuration */
const defaultConfig: Required<ArboriumConfig> = {
  manual: false,
  theme: 'tokyo-night',
  selector: 'pre code, code[data-lang]',
  cdn: 'jsdelivr',
  version: 'latest',
};

/** Get the CDN base URL */
function getCdnUrl(cdn: string): string {
  // If it's a known CDN name, use the preset
  if (cdn in CDN_URLS) {
    return CDN_URLS[cdn];
  }
  // Otherwise treat it as a custom base URL
  return cdn;
}

/** Get the full URL for a grammar package file */
function getGrammarUrl(
  language: string,
  file: string,
  config: ArboriumConfig
): string {
  const cdn = config.cdn || defaultConfig.cdn;
  const version = config.version || defaultConfig.version;
  const baseUrl = getCdnUrl(cdn);
  const versionSuffix = version === 'latest' ? '' : `@${version}`;
  return `${baseUrl}/@arborium/${language}${versionSuffix}/${file}`;
}

/** Wrapper around a loaded grammar plugin */
class GrammarImpl implements Grammar {
  private plugin: GrammarPlugin;
  private session: number | null = null;

  constructor(plugin: GrammarPlugin) {
    this.plugin = plugin;
  }

  languageId(): string {
    return this.plugin.languageId();
  }

  injectionLanguages(): string[] {
    return this.plugin.injectionLanguages();
  }

  parse(source: string): ParseResult {
    // Create a session, parse, then free it
    const session = this.plugin.createSession();
    try {
      this.plugin.setText(session, source);
      return this.plugin.parse(session);
    } finally {
      this.plugin.freeSession(session);
    }
  }

  highlight(source: string): string {
    const result = this.parse(source);
    return spansToHtml(source, result.spans);
  }

  dispose(): void {
    if (this.session !== null) {
      this.plugin.freeSession(this.session);
      this.session = null;
    }
  }
}

/** Plugin interface as exported by jco */
interface GrammarPlugin {
  languageId(): string;
  injectionLanguages(): string[];
  createSession(): number;
  freeSession(session: number): void;
  setText(session: number, text: string): void;
  parse(session: number): ParseResult;
}

/** Load a grammar from CDN */
export async function loadGrammar(
  language: string,
  config: ArboriumConfig = {}
): Promise<Grammar> {
  // Check cache first
  const cached = grammarCache.get(language);
  if (cached) {
    return cached;
  }

  // Check if already loading
  const loading = loadingCache.get(language);
  if (loading) {
    return loading;
  }

  // Start loading
  const loadPromise = doLoadGrammar(language, config);
  loadingCache.set(language, loadPromise);

  try {
    const grammar = await loadPromise;
    grammarCache.set(language, grammar);
    return grammar;
  } finally {
    loadingCache.delete(language);
  }
}

async function doLoadGrammar(
  language: string,
  config: ArboriumConfig
): Promise<Grammar> {
  // Fetch the grammar.js module
  const jsUrl = getGrammarUrl(language, 'grammar.js', config);
  const wasmUrl = getGrammarUrl(language, 'grammar.core.wasm', config);

  // Dynamic import the JS module
  const module = await import(/* @vite-ignore */ jsUrl);

  // Create a getCoreModule function that fetches the WASM
  const getCoreModule = async (path: string): Promise<WebAssembly.Module> => {
    // The path will be something like "grammar.core.wasm"
    // We need to fetch it from CDN
    const url = path.includes('://') ? path : wasmUrl;
    const response = await fetch(url);
    const bytes = await response.arrayBuffer();
    return WebAssembly.compile(bytes);
  };

  // Create WASI imports
  const wasiImports = createWasiImports();
  const imports = {
    ...wasiImports,
    ...grammarTypesImport,
  };

  // Instantiate the component
  const instance = await module.instantiate(getCoreModule, imports);

  // Get the plugin interface
  const plugin = instance.plugin as GrammarPlugin;

  return new GrammarImpl(plugin);
}

/** Convert spans to HTML with custom elements */
export function spansToHtml(source: string, spans: Span[]): string {
  if (spans.length === 0) {
    return escapeHtml(source);
  }

  // Sort spans by start position
  const sorted = [...spans].sort((a, b) => a.start - b.start);

  const parts: string[] = [];
  let pos = 0;

  for (const span of sorted) {
    // Add unhighlighted text before this span
    if (span.start > pos) {
      parts.push(escapeHtml(source.slice(pos, span.start)));
    }

    // Add highlighted span
    const text = source.slice(span.start, span.end);
    const tag = captureToTag(span.capture);
    parts.push(`<${tag}>${escapeHtml(text)}</${tag}>`);

    pos = span.end;
  }

  // Add remaining text
  if (pos < source.length) {
    parts.push(escapeHtml(source.slice(pos)));
  }

  return parts.join('');
}

/** Map capture names to custom element tags */
function captureToTag(capture: string): string {
  // Use short custom element names: a-k for keyword, a-s for string, etc.
  const shortNames: Record<string, string> = {
    keyword: 'a-k',
    string: 'a-s',
    comment: 'a-c',
    function: 'a-f',
    'function.call': 'a-f',
    'function.method': 'a-f',
    type: 'a-t',
    variable: 'a-v',
    'variable.parameter': 'a-v',
    'variable.builtin': 'a-vb',
    number: 'a-n',
    operator: 'a-o',
    punctuation: 'a-p',
    'punctuation.bracket': 'a-p',
    'punctuation.delimiter': 'a-p',
    constant: 'a-ct',
    'constant.builtin': 'a-cb',
    property: 'a-pr',
    attribute: 'a-at',
    tag: 'a-tg',
    namespace: 'a-ns',
    label: 'a-lb',
    escape: 'a-e',
    embedded: 'a-em',
  };

  // Check for exact match or prefix match
  if (capture in shortNames) {
    return shortNames[capture];
  }

  // Try prefix match
  for (const [prefix, tag] of Object.entries(shortNames)) {
    if (capture.startsWith(prefix + '.')) {
      return tag;
    }
  }

  // Fallback: use a-x with data attribute for unknown captures
  return 'a-x';
}

/** Escape HTML special characters */
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

/** Highlight code with a loaded grammar */
export async function highlight(
  language: string,
  source: string,
  config: ArboriumConfig = {}
): Promise<string> {
  const grammar = await loadGrammar(language, config);
  return grammar.highlight(source);
}

/** Get the default config merged with user config */
export function getConfig(userConfig?: ArboriumConfig): Required<ArboriumConfig> {
  return { ...defaultConfig, ...userConfig };
}

export { defaultConfig };
