// Test language injection - HTML with embedded JavaScript
// Run with: node component-demo/test-injection.mjs

import { ArboriumHost } from './host.mjs';

const host = new ArboriumHost();

console.log('=== Language Injection Test ===\n');

// HTML with embedded JavaScript
const htmlCode = `<!DOCTYPE html>
<html lang="en">
<head>
  <title>Injection Test</title>
  <script>
    function greet(name) {
      const message = "Hello, " + name + "!";
      console.log(message);
    }
  </script>
</head>
<body>
  <h1>Test Page</h1>
  <button onclick="greet('World')">Click me</button>
</body>
</html>`;

console.log('Input HTML:');
console.log('---');
console.log(htmlCode);
console.log('---\n');

// Parse HTML
const session = await host.createSession('html');
host.setText(session, htmlCode);
const result = await host.parse(session);

console.log(`Total spans: ${result.spans.length}`);
console.log(`Injections detected: ${result.injections.length}`);

if (result.injections.length > 0) {
  console.log('\nInjection details:');
  for (const inj of result.injections) {
    const text = htmlCode.slice(inj.start, inj.end);
    console.log(`  - ${inj.language}: [${inj.start}-${inj.end}]`);
    console.log(`    Content: "${text.slice(0, 50)}${text.length > 50 ? '...' : ''}"`);
  }
}

// Group spans by capture
const byCapture = {};
for (const span of result.spans) {
  const key = span._injectedLanguage
    ? `${span._injectedLanguage}:${span.capture}`
    : span.capture;
  byCapture[key] = (byCapture[key] || 0) + 1;
}

console.log('\nSpans by capture (language:capture for injected):');
for (const [capture, count] of Object.entries(byCapture).sort()) {
  console.log(`  ${capture}: ${count}`);
}

// Show some interesting spans
console.log('\nSample injected JavaScript spans:');
const jsSpans = result.spans.filter(s => s._injectedLanguage === 'javascript');
for (const span of jsSpans.slice(0, 10)) {
  const text = htmlCode.slice(span.start, span.end);
  console.log(`  ${span.capture}: "${text}"`);
}

host.freeSession(session);
console.log('\n=== Test Complete ===');
