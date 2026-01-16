import { describe, it, expect } from "vitest";
import { escapeHtml } from "./utils.js";

describe("escapeHtml", () => {
  it("escapes HTML special characters", () => {
    expect(escapeHtml("<div>")).toBe("&lt;div&gt;");
    expect(escapeHtml("a & b")).toBe("a &amp; b");
    expect(escapeHtml('"quoted"')).toBe("&quot;quoted&quot;");
  });

  it("handles empty string", () => {
    expect(escapeHtml("")).toBe("");
  });

  it("passes through safe text unchanged", () => {
    expect(escapeHtml("hello world")).toBe("hello world");
  });

  it("handles non-ASCII characters", () => {
    expect(escapeHtml("café")).toBe("café");
    expect(escapeHtml("变量")).toBe("变量");
    expect(escapeHtml("🎉")).toBe("🎉");
  });
});

// Note: Integration tests for highlight() require the Rust host WASM module
// which is only available after building. These tests verify the utility
// functions that are always available.
//
// The highlight() function is tested end-to-end via:
// 1. Rust unit tests in arborium-highlight (spans_to_html tests)
// 2. The demo/playground which exercises the full pipeline
// 3. CI integration tests that build and run the full stack
