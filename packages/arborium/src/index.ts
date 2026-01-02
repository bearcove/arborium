/**
 * @arborium/arborium - High-performance syntax highlighting
 *
 * ESM entry point for programmatic usage.
 */

export { loadGrammar, highlight, spansToHtml, getConfig } from './loader.js';
export { detectLanguage, extractLanguageFromClass, normalizeLanguage } from './detect.js';
export { pluginVersion, availableLanguages, highlights } from './plugins-manifest.js';
export type {
  Grammar,
  Session,
  Span,
  Injection,
  ParseResult,
  Highlight,
  ArboriumConfig,
} from './types.js';
