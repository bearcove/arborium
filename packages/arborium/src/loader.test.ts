import { describe, it, expect } from "vitest";
import { spansToHtml } from "./utils.js";
import type { Span } from "./types.js";

describe("spansToHtml", () => {
  it("handles ASCII text correctly", () => {
    const source = "let x = 42;";
    const spans: Span[] = [
      { start: 0, end: 3, capture: "keyword" },
      { start: 8, end: 10, capture: "number" },
    ];

    const html = spansToHtml(source, spans);

    expect(html).toBe("<a-k>let</a-k> x = <a-n>42</a-n>;");
  });

  it("handles emoji with UTF-16 offsets", () => {
    // "hello🌍world" - emoji is at UTF-16 indices 5-7 (2 code units)
    const source = "hello🌍world";
    const spans: Span[] = [
      { start: 0, end: 5, capture: "string" }, // "hello"
      { start: 7, end: 12, capture: "keyword" }, // "world"
    ];

    const html = spansToHtml(source, spans);

    expect(html).toBe("<a-s>hello</a-s>🌍<a-k>world</a-k>");
  });

  it("handles Chinese characters with UTF-16 offsets", () => {
    // Chinese chars are 1 UTF-16 code unit each (BMP)
    const source = "let 变量 = 1";
    const spans: Span[] = [
      { start: 0, end: 3, capture: "keyword" }, // "let"
      { start: 4, end: 6, capture: "variable" }, // "变量"
      { start: 9, end: 10, capture: "number" }, // "1"
    ];

    const html = spansToHtml(source, spans);

    expect(html).toBe("<a-k>let</a-k> <a-v>变量</a-v> = <a-n>1</a-n>");
  });

  it("handles mixed emoji and text", () => {
    // "fn 🦀() {}" - 🦀 is at UTF-16 indices 3-5
    const source = "fn 🦀() {}";
    const spans: Span[] = [
      { start: 0, end: 2, capture: "keyword" }, // "fn"
    ];

    const html = spansToHtml(source, spans);

    expect(html).toBe("<a-k>fn</a-k> 🦀() {}");
  });

  it("handles overlapping spans by skipping later ones", () => {
    const source = "hello";
    const spans: Span[] = [
      { start: 0, end: 5, capture: "string" },
      { start: 2, end: 4, capture: "keyword" }, // overlaps, should be skipped
    ];

    const html = spansToHtml(source, spans);

    expect(html).toBe("<a-s>hello</a-s>");
  });

  it("handles empty spans array", () => {
    const source = "hello world";
    const spans: Span[] = [];

    const html = spansToHtml(source, spans);

    expect(html).toBe("hello world");
  });

  it("escapes HTML special characters", () => {
    const source = "<div>&</div>";
    const spans: Span[] = [
      { start: 0, end: 5, capture: "tag" }, // "<div>"
    ];

    const html = spansToHtml(source, spans);

    expect(html).toBe("<a-tg>&lt;div&gt;</a-tg>&amp;&lt;/div&gt;");
  });

  it("handles multiple emoji in sequence", () => {
    // Each emoji is 2 UTF-16 code units
    const source = "a🎉🎊b";
    // a=0, 🎉=1-2, 🎊=3-4, b=5
    const spans: Span[] = [
      { start: 0, end: 1, capture: "variable" }, // "a"
      { start: 5, end: 6, capture: "variable" }, // "b"
    ];

    const html = spansToHtml(source, spans);

    expect(html).toBe("<a-v>a</a-v>🎉🎊<a-v>b</a-v>");
  });

  it("verifies spans work with String.slice()", () => {
    // This is the core issue from #94 - spans should work with JS string APIs
    const source = "hello🌍world";

    // These are UTF-16 offsets that should work with slice()
    const helloSpan: Span = { start: 0, end: 5, capture: "string" };
    const worldSpan: Span = { start: 7, end: 12, capture: "keyword" };

    // Verify the offsets work correctly with String.slice()
    expect(source.slice(helloSpan.start, helloSpan.end)).toBe("hello");
    expect(source.slice(worldSpan.start, worldSpan.end)).toBe("world");

    // And verify spansToHtml produces correct output
    const html = spansToHtml(source, [helloSpan, worldSpan]);
    expect(html).toBe("<a-s>hello</a-s>🌍<a-k>world</a-k>");
  });
});
