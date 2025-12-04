/**
 * @arborium/arborium - High-performance syntax highlighting
 *
 * ESM entry point for programmatic usage.
 */

export { loadGrammar, highlight, spansToHtml, getConfig } from './loader.js';
export { detectLanguage, extractLanguageFromClass, normalizeLanguage } from './detect.js';
export type {
  Grammar,
  Span,
  Injection,
  ParseResult,
  ArboriumConfig,
} from './types.js';
