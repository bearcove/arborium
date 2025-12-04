/**
 * @arborium/arborium IIFE bundle
 *
 * Drop-in auto-highlighter that runs on page load.
 * Configuration via data attributes or window.Arborium object.
 */

import { loadGrammar, highlight, getConfig, defaultConfig } from './loader.js';
import {
  detectLanguage,
  extractLanguageFromClass,
  normalizeLanguage,
} from './detect.js';
import type { ArboriumConfig } from './types.js';

// Capture current script immediately (before any async operations)
const currentScript = document.currentScript as HTMLScriptElement | null;

/** Parse configuration from script data attributes */
function getConfigFromScript(): Partial<ArboriumConfig> {
  if (!currentScript) return {};

  const config: Partial<ArboriumConfig> = {};

  if (currentScript.hasAttribute('data-manual')) {
    config.manual = true;
  }

  const theme = currentScript.getAttribute('data-theme');
  if (theme) config.theme = theme;

  const selector = currentScript.getAttribute('data-selector');
  if (selector) config.selector = selector;

  const cdn = currentScript.getAttribute('data-cdn');
  if (cdn) config.cdn = cdn;

  const version = currentScript.getAttribute('data-version');
  if (version) config.version = version;

  return config;
}

/** Get merged configuration from all sources */
function getMergedConfig(): Required<ArboriumConfig> {
  // Priority: data attributes > window.Arborium > defaults
  const windowConfig = window.Arborium || {};
  const scriptConfig = getConfigFromScript();
  return getConfig({ ...windowConfig, ...scriptConfig });
}

/** Find all code blocks that need highlighting */
function findCodeBlocks(selector: string): HTMLElement[] {
  return Array.from(document.querySelectorAll(selector));
}

/** Get the language for a code block */
function getLanguageForBlock(block: HTMLElement): string | null {
  // Check data-lang attribute
  const dataLang = block.getAttribute('data-lang');
  if (dataLang) return normalizeLanguage(dataLang);

  // Check class="language-*"
  const className = block.className;
  const classLang = extractLanguageFromClass(className);
  if (classLang) return normalizeLanguage(classLang);

  // Check parent element (often <pre> wraps <code>)
  const parent = block.parentElement;
  if (parent) {
    const parentDataLang = parent.getAttribute('data-lang');
    if (parentDataLang) return normalizeLanguage(parentDataLang);

    const parentClassLang = extractLanguageFromClass(parent.className);
    if (parentClassLang) return normalizeLanguage(parentClassLang);
  }

  // Try auto-detection
  const source = block.textContent || '';
  return detectLanguage(source);
}

/** Inject theme CSS if not already present */
function injectThemeCSS(theme: string): void {
  const themeId = `arborium-theme-${theme}`;
  if (document.getElementById(themeId)) return;

  // Get the base URL for CSS
  const config = getMergedConfig();
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
  const cssUrl = `${baseUrl}/@arborium/arborium${versionSuffix}/dist/themes/${theme}.css`;

  const link = document.createElement('link');
  link.id = themeId;
  link.rel = 'stylesheet';
  link.href = cssUrl;
  document.head.appendChild(link);
}

/** Highlight a single code block */
async function highlightBlock(
  block: HTMLElement,
  language: string,
  config: ArboriumConfig
): Promise<void> {
  const source = block.textContent || '';
  if (!source.trim()) return;

  try {
    const html = await highlight(language, source, config);
    block.innerHTML = html;
    block.setAttribute('data-highlighted', 'true');
    block.setAttribute('data-lang', language);
  } catch (err) {
    console.warn(`[arborium] Failed to highlight ${language}:`, err);
    // Don't modify the block on error
  }
}

/** Main auto-highlight function */
async function autoHighlight(): Promise<void> {
  const config = getMergedConfig();

  // Inject theme CSS
  injectThemeCSS(config.theme);

  // Find all code blocks
  const blocks = findCodeBlocks(config.selector);
  if (blocks.length === 0) return;

  // Group blocks by language
  const blocksByLanguage = new Map<string, HTMLElement[]>();
  const unknownBlocks: HTMLElement[] = [];

  for (const block of blocks) {
    // Skip already highlighted blocks
    if (block.hasAttribute('data-highlighted')) continue;

    const language = getLanguageForBlock(block);
    if (language) {
      const existing = blocksByLanguage.get(language) || [];
      existing.push(block);
      blocksByLanguage.set(language, existing);
    } else {
      unknownBlocks.push(block);
    }
  }

  // Load grammars in parallel for all detected languages
  const languages = Array.from(blocksByLanguage.keys());
  const loadPromises = languages.map((lang) =>
    loadGrammar(lang, config).catch((err) => {
      console.warn(`[arborium] Failed to load grammar for ${lang}:`, err);
      return null;
    })
  );

  // Wait for all grammars to load
  const grammars = await Promise.all(loadPromises);

  // Highlight blocks for each loaded grammar
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

  // Wait for all highlighting to complete
  await Promise.all(highlightPromises);

  // Log summary
  const total = blocks.length;
  const highlighted = blocks.filter((b) =>
    b.hasAttribute('data-highlighted')
  ).length;
  const skipped = unknownBlocks.length;

  if (highlighted > 0 || skipped > 0) {
    console.debug(
      `[arborium] Highlighted ${highlighted}/${total} blocks` +
        (skipped > 0 ? ` (${skipped} unknown language)` : '')
    );
  }
}

/** Public API for manual highlighting */
export async function highlightAll(config?: ArboriumConfig): Promise<void> {
  const mergedConfig = getConfig({ ...getMergedConfig(), ...config });
  await autoHighlight();
}

/** Public API for highlighting a specific element */
export async function highlightElement(
  element: HTMLElement,
  language?: string,
  config?: ArboriumConfig
): Promise<void> {
  const mergedConfig = getConfig({ ...getMergedConfig(), ...config });
  const lang = language || getLanguageForBlock(element);

  if (!lang) {
    console.warn('[arborium] Could not detect language for element');
    return;
  }

  await highlightBlock(element, lang, mergedConfig);
}

// Expose public API on window
(window as any).arborium = {
  highlightAll,
  highlightElement,
  loadGrammar,
  highlight,
  detectLanguage,
  config: getMergedConfig(),
};

// Auto-highlight on DOMContentLoaded (unless manual mode)
const config = getMergedConfig();
if (!config.manual) {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => autoHighlight());
  } else {
    // DOM already loaded
    autoHighlight();
  }
}
