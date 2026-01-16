/**
 * @arborium/arborium - High-performance syntax highlighting
 *
 * ESM entry point for programmatic usage.
 */

export { detectLanguage, extractLanguageFromClass, normalizeLanguage } from "./detect.js";
export {
  getConfig,
  highlight,
  loadGrammar,
  setConfig,
  getAvailableLanguages,
  isLanguageAvailable,
} from "./loader.js";
export { availableLanguages, highlights, pluginVersion } from "./plugins-manifest.js";
export type {
  ArboriumConfig,
  Grammar,
  Highlight,
  Injection,
  ParseResult,
  ResolveArgs,
  Session,
  Span,
  Utf8Injection,
  Utf8ParseResult,
  Utf8Span,
  Utf16Injection,
  Utf16ParseResult,
  Utf16Span,
} from "./types.js";
