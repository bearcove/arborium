/** A span of highlighted text */
export interface Span {
  start: number;
  end: number;
  /** The capture name (e.g., "keyword", "string", "comment") */
  capture: string;
}

/** A language injection (e.g., JS inside HTML) */
export interface Injection {
  start: number;
  end: number;
  language: string;
  includeChildren: boolean;
}

/** Result of parsing source code */
export interface ParseResult {
  spans: Span[];
  injections: Injection[];
}

/**
 * A parsing session for incremental highlighting.
 *
 * Sessions allow you to reuse the parser state between parses, which is more
 * efficient than creating a new session for each parse. This is useful for
 * editors where text changes frequently.
 *
 * Usage pattern:
 *   - Call `setText(newText)` whenever the text changes.
 *   - Then call `parse()` to parse the current text and get results.
 *
 * Example:
 * ```ts
 * const session = grammar.createSession();
 * session.setText("let x = 1;");
 * let result = session.parse();
 * // ... user edits text ...
 * session.setText("let x = 42;");
 * result = session.parse();
 * session.free();
 * ```
 */
export interface Session {
  /** Set the text to parse */
  setText(text: string): void;
  /** Parse the current text and return spans/injections */
  parse(): ParseResult;
  /** Cancel any in-progress parsing */
  cancel(): void;
  /**
   * Free the session resources. Must be called when done to prevent memory leaks.
   * Failure to call free() will result in WASM memory not being released.
   */
  free(): void;
}

/** A loaded grammar plugin */
export interface Grammar {
  /** The language identifier */
  languageId(): string;
  /** Languages this grammar may inject */
  injectionLanguages(): string[];
  /** Highlight source code, returning HTML string */
  highlight(source: string): string | Promise<string>;
  /** Parse source code, returning raw spans (creates a one-shot session internally) */
  parse(source: string): ParseResult;
  /** Create a session for incremental parsing */
  createSession(): Session;
  /** Dispose of resources */
  dispose(): void;
}

/** A highlighting tag */
export interface Highlight {
  /** Long name, used in `Span` */
  name: string;
  /** Short name, used in HTML tags and CSS variables */
  tag: string;
  /** Parent tag for fallback */
  parentTag?: string;
}

type MaybePromise<T> = T | Promise<T>;

interface ResolveArgs {
  /** Language to load the grammar plugin for */
  language: string;
  /** Base URL derived from language, CDN and version */
  baseUrl: string;
  /** Relative path in the module to load */
  path: string;
}

/** Configuration for the arborium runtime */
export interface ArboriumConfig {
  /** Disable auto-highlighting on page load */
  manual?: boolean;
  /** Theme to use: "tokyo-night" | "github-light" | custom */
  theme?: string;
  /** CSS selector for code blocks */
  selector?: string;
  /** CDN to use: "jsdelivr" | "unpkg" | custom base URL */
  cdn?: string;
  /** Package version to load (default: "1" for latest 1.x.x) */
  version?: string;
  /** URL to plugins.json manifest - overrides bundled manifest (for local testing) */
  pluginsUrl?: string;
  /** Base URL for the Rust host module (for local testing) */
  hostUrl?: string;
  /** Custom grammar resolution for JS */
  resolveJs?(args: ResolveArgs): MaybePromise<unknown>;
  /** Custom grammar resolution for WASM */
  resolveWasm?(args: ResolveArgs): MaybePromise<Response | BufferSource | WebAssembly.Module>;
}

/** Global config set before script loads */
declare global {
  interface Window {
    Arborium?: ArboriumConfig;
  }
}
