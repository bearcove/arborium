// Test script for the Rust grammar WASM component plugin
// Run with: node component-demo/test-plugin.mjs

import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const pluginDir = join(__dirname, '..', 'dist', 'plugins', 'rust');

// Import the generated grammar module
const { instantiate } = await import(join(pluginDir, 'grammar.js'));

// Helper to load core WASM modules
async function getCoreModule(path) {
  const fullPath = join(pluginDir, path);
  const bytes = await readFile(fullPath);
  return WebAssembly.compile(bytes);
}

// Stub implementations for WASI imports
// Note: jco uses the short key format without version for destructuring
const wasiStubs = {
  'wasi:cli/environment': {
    getEnvironment() { return []; },
  },
  'wasi:cli/exit': {
    exit(status) {
      if (status.tag === 'err') {
        throw new Error(`WASI exit with error`);
      }
    },
  },
  'wasi:cli/stderr': {
    getStderr() {
      return {
        blockingWriteAndFlush() {},
        write() { return { tag: 'ok', val: 0n }; },
      };
    },
  },
  'wasi:io/error': {},
  'wasi:io/streams': {},
  'wasi:random/insecure-seed': {
    insecureSeed() { return [BigInt(Date.now()), BigInt(Math.floor(Math.random() * 0xFFFFFFFF))]; },
  },
};

console.log('Loading Rust grammar plugin...\n');

try {
  // Instantiate the component
  const component = await instantiate(getCoreModule, wasiStubs);
  const plugin = component.plugin;

  // Test the plugin
  console.log('Language ID:', plugin.languageId());
  console.log('Injection languages:', plugin.injectionLanguages());
  console.log();

  // Create a session and parse some Rust code
  const session = plugin.createSession();
  console.log('Created session:', session);

  const rustCode = `fn main() {
    let message = "Hello, World!";
    println!("{}", message);
}`;

  console.log('\nParsing Rust code:');
  console.log('---');
  console.log(rustCode);
  console.log('---\n');

  plugin.setText(session, rustCode);
  const result = plugin.parse(session);

  console.log(`Found ${result.spans.length} spans:`);

  // Group spans by capture name for cleaner output
  const spansByCapture = {};
  for (const span of result.spans) {
    if (!spansByCapture[span.capture]) {
      spansByCapture[span.capture] = [];
    }
    const text = rustCode.slice(span.start, span.end);
    spansByCapture[span.capture].push({ text, start: span.start, end: span.end });
  }

  for (const [capture, spans] of Object.entries(spansByCapture).sort()) {
    console.log(`\n  ${capture}:`);
    for (const { text, start, end } of spans.slice(0, 5)) {
      console.log(`    "${text}" (${start}-${end})`);
    }
    if (spans.length > 5) {
      console.log(`    ... and ${spans.length - 5} more`);
    }
  }

  if (result.injections.length > 0) {
    console.log('\nInjections:', result.injections);
  }

  // Test incremental edit
  console.log('\n\nTesting incremental edit...');
  const newCode = `fn main() {
    let message = "Hello, Rust!";
    println!("{}", message);
}`;

  plugin.applyEdit(session, newCode, {
    startByte: 34,
    oldEndByte: 48,
    newEndByte: 47,
    startRow: 1,
    startCol: 19,
    oldEndRow: 1,
    oldEndCol: 33,
    newEndRow: 1,
    newEndCol: 32,
  });

  const result2 = plugin.parse(session);
  console.log(`After edit: ${result2.spans.length} spans`);

  // Clean up
  plugin.freeSession(session);
  console.log('\nSession freed. Test complete!');

} catch (error) {
  console.error('Error:', error);
  process.exit(1);
}
