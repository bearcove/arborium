/* tslint:disable */
/* eslint-disable */

export class HighlightConfig {
  free(): void;
  [Symbol.dispose](): void;
  setMaxInjectionDepth(depth: number): void;
  /**
   * Set HTML format to class names: `<span class="keyword">`, etc.
   */
  setHtmlFormatClassNames(): void;
  /**
   * Set HTML format to custom elements (default): `<a-k>`, `<a-f>`, etc.
   */
  setHtmlFormatCustomElements(): void;
  /**
   * Set HTML format to class names with custom prefix.
   */
  setHtmlFormatClassNamesWithPrefix(prefix: string): void;
  constructor();
  /**
   * Set HTML format to custom elements with custom prefix.
   */
  setHtmlFormatCustomElementsWithPrefix(prefix: string): void;
}

/**
 * Highlight source code, resolving injections recursively.
 *
 * This uses the shared `AsyncHighlighter` from `arborium_highlight`,
 * ensuring the same injection handling logic as Rust native.
 */
export function highlight(language: string, source: string): Promise<string>;

/**
 * Highlight with custom configuration.
 */
export function highlightWithConfig(language: string, source: string, config: HighlightConfig): Promise<string>;

/**
 * Check if a language is available for highlighting.
 */
export function isLanguageAvailable(language: string): boolean;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_highlightconfig_free: (a: number, b: number) => void;
  readonly highlight: (a: number, b: number, c: number, d: number) => any;
  readonly highlightWithConfig: (a: number, b: number, c: number, d: number, e: number) => any;
  readonly highlightconfig_new: () => number;
  readonly highlightconfig_setHtmlFormatClassNames: (a: number) => void;
  readonly highlightconfig_setHtmlFormatClassNamesWithPrefix: (a: number, b: number, c: number) => void;
  readonly highlightconfig_setHtmlFormatCustomElements: (a: number) => void;
  readonly highlightconfig_setHtmlFormatCustomElementsWithPrefix: (a: number, b: number, c: number) => void;
  readonly highlightconfig_setMaxInjectionDepth: (a: number, b: number) => void;
  readonly isLanguageAvailable: (a: number, b: number) => number;
  readonly wasm_bindgen__convert__closures_____invoke__h9efbcc4b940797ad: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h7d0fb7af338d42f6: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h9e9efe1fd5253216: (a: number, b: number, c: any, d: any) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
