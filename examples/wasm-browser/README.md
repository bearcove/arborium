# WASM Browser Example

This example demonstrates using arborium for syntax highlighting in a web browser via WebAssembly.

## Building

```bash
# From repository root
cd examples/wasm-browser

# Build the WASM module (includes Rust language support)
wasm-pack build --target web --release

# Serve the example
python3 -m http.server 8080
# or
npx serve .
```

Then open http://localhost:8080 - you should see Rust code highlighted with syntax colors!

**Note**: This example includes the `lang-rust` feature. You can add other languages by updating `Cargo.toml`:
```toml
arborium = { path = "../../crates/arborium", features = ["lang-rust", "lang-javascript", "lang-python"] }
```

## Verification

The build includes a WASM inspection tool that verifies no `env` imports exist:

```bash
cargo run --bin inspect-wasm
```

Expected output:
```
✅ No 'env' module imports found!
```

## How It Works

`★ Insight ─────────────────────────────────────`
- **wasm-bindgen**: Generates JavaScript glue code that bridges Rust/WASM and JavaScript
- **Macro replacement**: The `fprintf` macro is evaluated at C preprocessing time, before compilation, so the linker never sees a reference to an external fprintf function
- **Zero runtime cost**: `(void)(stream)` evaluates the stream parameter (avoiding warnings) but generates no code, and the macro evaluates to 0 (matching fprintf's return type)
`─────────────────────────────────────────────────`

## Integration with CI

This example can be built in CI to ensure WASM compatibility:

```bash
# In CI
cd examples/wasm-browser
wasm-pack build --target web --release
cargo run --bin inspect-wasm | grep "✅ No 'env' module imports"
```
