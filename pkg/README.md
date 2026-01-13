# arborium-host

WASM host runtime for arborium in browsers.

## Purpose

Provides the browser-side runtime for loading and executing arborium grammar plugins.
Uses wasm-bindgen for JavaScript interop.

## How It Works

The host expects these functions on `window.arboriumHost`:

```javascript
window.arboriumHost = {
    // Check if a language is available (sync)
    isLanguageAvailable(language) { ... },

    // Load a grammar plugin, returns a handle (async)
    async loadGrammar(language) { ... },
};
```

Grammar plugins are WIT components loaded on-demand from a CDN.

This crate implements `GrammarProvider` to integrate with `arborium-highlight`,
ensuring browser and native Rust use the same highlighting logic.
---

Part of the [arborium](https://github.com/bearcove/arborium) project. See [arborium.dev](https://arborium.dev) for more information.
