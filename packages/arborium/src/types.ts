// ============================================================================
// UTF-8 types (internal, for Rust host)
// ============================================================================

/**
 * A span of highlighted text with UTF-8 byte offsets.
 *
 * This is the native format from tree-sitter. Use this when working with
 * Rust code that needs to slice strings with `&source[start..end]`.
 *
 * For JavaScript string operations, use {@link Utf16Span} instead.
 *
 * @internal
 */
export interface Utf8Span {
  /** UTF-8 byte offset where the span starts (inclusive) */
  start: number;
  /** UTF-8 byte offset where the span ends (exclusive) */
  end: number;
  /** The capture name (e.g., "keyword", "string", "comment") */
  capture: string;
}

/**
 * A language injection with UTF-8 byte offsets.
 *
 * @internal
 */
export interface Utf8Injection {
  /** UTF-8 byte offset where the injection starts (inclusive) */
  start: number;
  /** UTF-8 byte offset where the injection ends (exclusive) */
  end: number;
  language: string;
  includeChildren: boolean;
}

/**
 * Result of parsing source code, with UTF-8 byte offsets.
 *
 * @internal
 */
export interface Utf8ParseResult {
  spans: Utf8Span[];
  injections: Utf8Injection[];
}

// ============================================================================
// UTF-16 types (public API, for JavaScript)
// ============================================================================

/**
 * A span of highlighted text with UTF-16 code unit indices.
 *
 * The `start` and `end` fields are compatible with JavaScript string APIs
 * like `String.prototype.slice()`, `substring()`, and DOM `Range`.
 *
 * @example
 * ```ts
 * const result = grammar.parse(source);
 * for (const span of result.spans) {
 *   const text = source.slice(span.start, span.end);
 *   console.log(`${span.capture}: ${text}`);
 * }
 * ```
 */
export interface Utf16Span {
  /** UTF-16 code unit index where the span starts (inclusive) */
  start: number;
  /** UTF-16 code unit index where the span ends (exclusive) */
  end: number;
  /** The capture name (e.g., "keyword", "string", "comment") */
  capture: string;
}

/**
 * A language injection (e.g., JS inside HTML) with UTF-16 code unit indices.
 */
export interface Utf16Injection {
  /** UTF-16 code unit index where the injection starts (inclusive) */
  start: number;
  /** UTF-16 code unit index where the injection ends (exclusive) */
  end: number;
  language: string;
  includeChildren: boolean;
}

/**
 * Result of parsing source code, with UTF-16 code unit indices.
 *
 * This is the format returned by the public API (`grammar.parse()`,
 * `session.parse()`). Offsets are compatible with JavaScript string operations.
 */
export interface Utf16ParseResult {
  spans: Utf16Span[];
  injections: Utf16Injection[];
}

// ============================================================================
// Legacy type aliases (for backwards compatibility)
// ============================================================================

/**
 * @deprecated Use {@link Utf16Span} for JavaScript or {@link Utf8Span} for Rust interop.
 */
export type Span = Utf16Span;

/**
 * @deprecated Use {@link Utf16Injection} for JavaScript or {@link Utf8Injection} for Rust interop.
 */
export type Injection = Utf16Injection;

/**
 * @deprecated Use {@link Utf16ParseResult} for JavaScript or {@link Utf8ParseResult} for Rust interop.
 */
export type ParseResult = Utf16ParseResult;

// ============================================================================
// Session and Grammar interfaces
// ============================================================================

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
  /** Parse the current text and return spans/injections with UTF-16 offsets */
  parse(): Utf16ParseResult;
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
  /** Parse source code, returning spans with UTF-16 offsets (creates a one-shot session internally) */
  parse(source: string): Utf16ParseResult;
  /** Create a session for incremental parsing */
  createSession(): Session;
  /** Dispose of resources */
  dispose(): void;
}

// ============================================================================
// Other types
// ============================================================================

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
