// Arborium Host - Orchestrates grammar plugins
// This is a JavaScript implementation of the host for browser/Node.js use.
// A future WASM host component could provide the same interface.

import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const distDir = join(__dirname, '..', 'dist', 'plugins');

// WASI stubs for plugins
const wasiStubs = {
  'wasi:cli/environment': {
    getEnvironment() { return []; },
  },
  'wasi:cli/exit': {
    exit(status) {
      if (status.tag === 'err') throw new Error('WASI exit');
    },
  },
  'wasi:cli/stderr': {
    getStderr() {
      return { blockingWriteAndFlush() {}, write() { return { tag: 'ok', val: 0n }; } };
    },
  },
  'wasi:io/error': {},
  'wasi:io/streams': {},
  'wasi:random/insecure-seed': {
    insecureSeed() { return [BigInt(Date.now()), BigInt(Math.floor(Math.random() * 0xFFFFFFFF))]; },
  },
};

/**
 * A loaded grammar plugin
 */
class GrammarPlugin {
  constructor(languageId, plugin) {
    this.languageId = languageId;
    this.plugin = plugin;
  }

  createSession() {
    return this.plugin.createSession();
  }

  freeSession(session) {
    this.plugin.freeSession(session);
  }

  setText(session, text) {
    this.plugin.setText(session, text);
  }

  applyEdit(session, text, edit) {
    this.plugin.applyEdit(session, text, edit);
  }

  parse(session) {
    return this.plugin.parse(session);
  }

  cancel(session) {
    this.plugin.cancel(session);
  }

  injectionLanguages() {
    return this.plugin.injectionLanguages();
  }
}

/**
 * Arborium Host - manages grammar plugins and orchestrates parsing
 */
export class ArboriumHost {
  constructor() {
    /** @type {Map<string, GrammarPlugin>} */
    this.plugins = new Map();
    /** @type {Map<number, HostSession>} */
    this.sessions = new Map();
    this.nextSessionId = 1;
  }

  /**
   * Load a grammar plugin from the dist directory
   * @param {string} languageId - e.g., "rust", "html", "javascript"
   */
  async loadPlugin(languageId) {
    if (this.plugins.has(languageId)) {
      return this.plugins.get(languageId);
    }

    const pluginDir = join(distDir, languageId);
    const { instantiate } = await import(join(pluginDir, 'grammar.js'));

    const getCoreModule = async (path) => {
      const bytes = await readFile(join(pluginDir, path));
      return WebAssembly.compile(bytes);
    };

    const component = await instantiate(getCoreModule, wasiStubs);
    const plugin = new GrammarPlugin(languageId, component.plugin);
    this.plugins.set(languageId, plugin);

    console.log(`Loaded plugin: ${languageId}`);
    return plugin;
  }

  /**
   * Create a new parsing session for a language
   * @param {string} languageId
   * @returns {number} session handle
   */
  async createSession(languageId) {
    const plugin = await this.loadPlugin(languageId);
    const pluginSession = plugin.createSession();

    const sessionId = this.nextSessionId++;
    this.sessions.set(sessionId, {
      languageId,
      pluginSession,
      text: '',
      childSessions: new Map(), // injection language -> child session info
    });

    return sessionId;
  }

  /**
   * Free a session and all its child sessions
   * @param {number} sessionId
   */
  freeSession(sessionId) {
    const session = this.sessions.get(sessionId);
    if (!session) return;

    // Free child sessions first
    for (const child of session.childSessions.values()) {
      const childPlugin = this.plugins.get(child.languageId);
      if (childPlugin) {
        childPlugin.freeSession(child.pluginSession);
      }
    }

    // Free main session
    const plugin = this.plugins.get(session.languageId);
    if (plugin) {
      plugin.freeSession(session.pluginSession);
    }

    this.sessions.delete(sessionId);
  }

  /**
   * Set text for a session
   * @param {number} sessionId
   * @param {string} text
   */
  setText(sessionId, text) {
    const session = this.sessions.get(sessionId);
    if (!session) throw new Error(`Invalid session: ${sessionId}`);

    session.text = text;
    const plugin = this.plugins.get(session.languageId);
    plugin.setText(session.pluginSession, text);
  }

  /**
   * Apply an incremental edit
   * @param {number} sessionId
   * @param {string} newText
   * @param {object} edit
   */
  applyEdit(sessionId, newText, edit) {
    const session = this.sessions.get(sessionId);
    if (!session) throw new Error(`Invalid session: ${sessionId}`);

    session.text = newText;
    const plugin = this.plugins.get(session.languageId);
    plugin.applyEdit(session.pluginSession, newText, edit);
  }

  /**
   * Parse and return all spans, recursively resolving injections
   * @param {number} sessionId
   * @param {object} options
   * @param {number} [options.maxDepth=5] - Maximum injection depth
   * @returns {Promise<{spans: Array, injections: Array}>}
   */
  async parse(sessionId, options = {}) {
    const { maxDepth = 5 } = options;
    const session = this.sessions.get(sessionId);
    if (!session) throw new Error(`Invalid session: ${sessionId}`);

    const plugin = this.plugins.get(session.languageId);
    const result = plugin.parse(session.pluginSession);

    // Collect all spans (will include injected language spans)
    const allSpans = [...result.spans];

    // Recursively parse injections
    if (maxDepth > 0 && result.injections.length > 0) {
      for (const injection of result.injections) {
        try {
          // Load the injection plugin if needed
          const injectionPlugin = await this.loadPlugin(injection.language);

          // Get or create child session for this injection
          let childInfo = session.childSessions.get(injection.language);
          if (!childInfo) {
            childInfo = {
              languageId: injection.language,
              pluginSession: injectionPlugin.createSession(),
            };
            session.childSessions.set(injection.language, childInfo);
          }

          // Extract the injected text
          const injectedText = session.text.slice(injection.start, injection.end);

          // Parse the injection
          injectionPlugin.setText(childInfo.pluginSession, injectedText);
          const childResult = injectionPlugin.parse(childInfo.pluginSession);

          // Offset child spans to match parent coordinates
          for (const span of childResult.spans) {
            allSpans.push({
              start: span.start + injection.start,
              end: span.end + injection.start,
              capture: span.capture,
              // Mark as from injection for debugging
              _injectedFrom: session.languageId,
              _injectedLanguage: injection.language,
            });
          }

          // TODO: Recursively handle nested injections (e.g., HTML → JS → template strings)
        } catch (err) {
          console.warn(`Failed to parse injection for ${injection.language}:`, err.message);
        }
      }
    }

    // Sort spans by position
    allSpans.sort((a, b) => a.start - b.start || a.end - b.end);

    return {
      spans: allSpans,
      injections: result.injections,
    };
  }

  /**
   * Cancel an in-progress parse
   * @param {number} sessionId
   */
  cancel(sessionId) {
    const session = this.sessions.get(sessionId);
    if (!session) return;

    // Cancel main session
    const plugin = this.plugins.get(session.languageId);
    if (plugin) {
      plugin.cancel(session.pluginSession);
    }

    // Cancel child sessions
    for (const child of session.childSessions.values()) {
      const childPlugin = this.plugins.get(child.languageId);
      if (childPlugin) {
        childPlugin.cancel(child.pluginSession);
      }
    }
  }

  /**
   * Get list of loaded plugins
   * @returns {string[]}
   */
  getLoadedPlugins() {
    return Array.from(this.plugins.keys());
  }
}

// Demo/test if run directly
if (import.meta.url === `file://${process.argv[1]}`) {
  const host = new ArboriumHost();

  console.log('=== Arborium Host Demo ===\n');

  // Load and test Rust plugin
  const session = await host.createSession('rust');
  console.log('Created session:', session);

  const code = `fn main() {
    let msg = "Hello";
    println!("{}", msg);
}`;

  host.setText(session, code);
  const result = await host.parse(session);

  console.log(`\nParsed ${result.spans.length} spans:`);
  const byCapture = {};
  for (const span of result.spans) {
    byCapture[span.capture] = (byCapture[span.capture] || 0) + 1;
  }
  console.log(byCapture);

  if (result.injections.length > 0) {
    console.log('\nInjections:', result.injections);
  }

  host.freeSession(session);
  console.log('\nSession freed. Loaded plugins:', host.getLoadedPlugins());
}
