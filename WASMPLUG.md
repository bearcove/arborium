# WASM Plugin Plan

Actionable roadmap to implement the WASM-based Tree-sitter grammar plugin architecture described in issue #1. Ordered to let someone new pick up work and make continuous progress.

## Outcomes
- Browser JS stays a minimal loader; Rust host (`arborium.wasm`) orchestrates everything.
- Host and grammar plugins are WASM components talking via WIT; facet-postcard works Rust↔Rust (no serde).
- Incremental parsing, apply-edit, cancellation, and language injection proven with a multi-language demo.
- CI builds host + plugins, runs tests, and ships browser-loadable artifacts.

### Architecture (Rust ↔ Rust, JS only loads)

```
Browser JS (loader / fetcher only)
      │  (imports jco-generated ES modules)
      ▼
   arborium.wasm           ← Rust host component
      │  • load grammar plugins
      │  • resolve injections recursively
      │  • merge spans, handle cancellation
      │  • facet-postcard de/serialization (no serde)
      ├───────────────┬───────────────┬───────────────┐
      ▼               ▼               ▼               ▼
arborium-grammar-rust.wasm   arborium-grammar-html.wasm   arborium-grammar-javascript.wasm   ...
(one grammar per plugin; Tree-sitter runtime + queries inside each)
```

### Build/Ship Flow

```
cargo component build (host) ─┐
                              ├─> jco transpile → dist/host/{arborium.js, arborium.core.wasm}
cargo component build (lang) ─┘
                              ├─> jco transpile → dist/plugins/<lang>/{grammar.js, grammar.core.wasm}
copy queries (.scm) ──────────┘
```

## Phase 0 – Prep (1–2 days)
- [ ] Confirm minimal “first three” grammars for end-to-end bring-up (suggest: `rust`, `html`, `javascript`).
- [ ] Decide repo locations for: shared types crate, plugin template, host demo, build outputs (`dist/plugins/<lang>/`).
- [ ] Install toolchain: `rustup target add wasm32-wasip2`, `cargo component`, `jco`, Node/Yarn/npm (pick one), headless Chrome/Playwright for smoke tests.

## Phase 1 – WIT Interface & Shared Types
- [ ] Create WIT file `wit/grammar.wit` (or `crates/arborium-wire/grammar.wit`) defining the component interface from issue #1.
- [ ] Add crate `crates/arborium-wire` (aka `arborium-protocol`, no_std ok) exposing:
  - `ParseResult`, `Span`, `Injection`, `ParseError` structs.
  - `WIRE_VERSION` and version bump/reject helpers.
  - facet-postcard ser/de (no serde) re-exported for consumers.
- [ ] Workspace dependency on `facet-postcard` added once (avoid per-crate drift).
- [ ] Test: round-trip a sample `ParseResult` with facet-postcard to lock the schema.

## Phase 2 – Plugin Runtime Crate
- [ ] Create `crates/arborium-plugin-runtime` to be reused by each grammar crate:
  - Session map (create/free) and grammar-specific parser wrapper.
  - Implements WIT exports: `set-text`, `apply-edit`, `parse`, `cancel`.
  - Tree-sitter query execution to turn captures into `Span`/`Injection`.
  - Cancellation flag checked in parse loop.
- [ ] Integrate existing query files (`highlights.scm`, `injections.scm`, optional `locals.scm`).
- [ ] Provide a small Rust smoke test to parse text, apply edit, and verify cancellation short-circuits.

## Phase 3 – Build/Transpile Pipeline
- [ ] Add workspace `cargo-component` support.
- [ ] `xtask plugins build [--lang <lang>]` should:
  1. Build each grammar as WASM component (`wasm32-wasip2`) using the runtime crate.
  2. Run `jco transpile … --instantiation async --out-dir dist/plugins/<lang>/`.
  3. Copy `.scm` query files alongside outputs.
  4. Enforce size budget (start at 1.5 MB for `grammar.core.wasm`).
- [ ] `xtask plugins clean` removes `dist/plugins`.
- [ ] Grammar crates opt-in to plugin build via feature flag or separate target (non-plugin build still works).

## Phase 4 – Host WASM (Rust)
- [ ] Create `crates/arborium-host` targeting `wasm32-wasip2`:
  - Imports grammar-plugin WIT, manages dynamic loading of grammar components.
  - Orchestrates sessions, edit vs set_text paths, cancellation propagation.
  - Resolves injections recursively; merges/normalizes overlapping spans.
  - facet-postcard deserialization remains in Rust (serde-free).
- [ ] Define host WIT world exposed to the browser loader (minimal functions to start/stop sessions, set text, apply edits, fetch spans).
- [ ] Add memoization of loaded plugins inside the host to avoid repeated instantiation.

## Phase 5 – Injection, Demo & Perf
- [ ] Map query captures → `Injection` records in plugins.
- [ ] Host recursively spawns child grammars for injected ranges; parent cancellation cancels children.
- [ ] Encode plugin dependency hints (e.g., `svelte` → `javascript`, `css`) so host auto-loads required child grammars when injections reference them.
- [ ] Update demo: browser loads `arborium.wasm` + selected grammar components; include HTML → JS → SQL and Svelte → JS/CSS fixtures.
- [ ] Add perf checks (parse time pre/post edit) and cache-hit verification.

## Phase 6 – Tests & CI
- [ ] Unit: protocol round-trip, session lifecycle, apply_edit correctness, cancellation cooperation.
- [ ] Integration: headless browser via `chromiumoxide` driving the demo (JS only loads WASMs); validate spans/injections render.
- [ ] Size/validate: `wasm-tools validate` each `grammar.core.wasm` and host; budget guard.
- [ ] CI matrix:
  - Build shared crates.
  - `xtask plugins build --lang rust html javascript`.
  - Host unit + chromiumoxide smoke.
  - Upload `dist/plugins` and host artifacts on main/PRs.

## Phase 7 – Documentation & DX
- [ ] Add “How to add a new grammar” doc using the template crate (copy grammar repo, wire queries, run `xtask plugins build --lang <new>`).
- [ ] Update `README`/`docs` with architecture diagram, build commands, and troubleshooting (version mismatch, missing queries, large wasm).
- [ ] Provide a minimal demo page served from `demo/` that loads plugins dynamically and prints spans/injections for inspection.

## Phase 8 – Rollout
- [ ] Migrate prioritized grammars one by one via the runtime + build pipeline.
- [ ] Track progress in issue #1 with per-grammar checklist (built, queries copied, smoke passing).
- [ ] Finish phase when first three grammars pass browser smoke and CI is green.

## Resolved Decisions
- Dynamic loading: browser JS fetches plugin bytes and passes them into the host via a `load_plugin(lang, bytes)` import; host instantiates components from those bytes. Keep per-plugin artifacts separate to preserve dynamic loading; no pre-linking.
- Query files: embed `.scm` into each grammar plugin at build time (avoids extra fetches and keeps plugin self-contained).
- Plugin granularity: keep TypeScript as its own plugin but bundle the necessary JS query captures inside it (no runtime dependency on a separate JS plugin).
- Injection dependencies: plugins declare the language ids they may inject (e.g., Svelte lists `javascript` and `css`); host resolves and loads those child plugins automatically when injections reference them.
- Serialization: use `facet-postcard` everywhere (no serde on WASM path); host and plugins share the same versioned format.

## Remaining Unknowns / Risks
- Wasip2 + jco interplay: confirm host can instantiate other components from raw bytes under wasip2, and that generated imports line up; prototype early.
- ABI drift: define version check between host and plugins (e.g., WIRE_VERSION plus a HOST_API_VERSION) and a graceful rejection path.
- Memory limits: per-session memory growth under deep injection chains; decide on caps or eviction strategy.
- Tree-sitter query compatibility: some grammars may need upstream query tweaks for component build; track outliers.
- Build size/time: multi-grammar component builds may be slow; consider per-lang matrix and artifact caching in CI.
- CSP / loader: if served with strict CSP, ensure dynamic `import()` of ES modules and fetch of wasm binaries are allowed; document required headers.
- Error surface: standardize error codes/messages from host to JS loader (plugin missing, version mismatch, parse failure, cancellation).
- Caching policy: eviction strategy for instantiated plugins and parsed trees in the host; avoid unbounded growth in long-lived pages.
## Definition of Done
- `xtask plugins build` succeeds and emits usable artifacts for the initial grammar set.
- Browser demo loads plugins, supports edits, shows spans and injection-derived sub-parses.
- CI gates on wire-version tests, size guardrails, wasm validation, and browser smoke.
- Clear documentation enabling others to add grammars without maintainer assistance.
