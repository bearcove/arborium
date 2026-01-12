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

  it("handles emoji with UTF-8 offsets", () => {
    // "hello🌍world" - emoji is 4 bytes in UTF-8 (bytes 5-9)
    const source = "hello🌍world";
    const spans: Span[] = [
      { start: 0, end: 5, capture: "string" }, // "hello" (bytes 0-5)
      { start: 9, end: 14, capture: "keyword" }, // "world" (bytes 9-14)
    ];

    const html = spansToHtml(source, spans);

    expect(html).toBe("<a-s>hello</a-s>🌍<a-k>world</a-k>");
  });

  it("handles Chinese characters with UTF-8 offsets", () => {
    // Chinese chars are 3 bytes each in UTF-8
    const source = "let 变量 = 1";
    // "let"=0-3, " "=3, "变"=4-7 (3 bytes), "量"=7-10 (3 bytes), " = "=10-13, "1"=13
    const spans: Span[] = [
      { start: 0, end: 3, capture: "keyword" }, // "let"
      { start: 4, end: 10, capture: "variable" }, // "变量" (6 bytes total)
      { start: 13, end: 14, capture: "number" }, // "1"
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
    // Each emoji is 4 UTF-8 bytes
    const source = "a🎉🎊b";
    // a=0-1, 🎉=1-5 (4 bytes), 🎊=5-9 (4 bytes), b=9-10
    const spans: Span[] = [
      { start: 0, end: 1, capture: "variable" }, // "a"
      { start: 9, end: 10, capture: "variable" }, // "b"
    ];

    const html = spansToHtml(source, spans);

    expect(html).toBe("<a-v>a</a-v>🎉🎊<a-v>b</a-v>");
  });

  it("converts UTF-8 offsets to UTF-16 for String.slice()", () => {
    // Grammar outputs UTF-8 byte offsets, but JS needs UTF-16 code unit indices
    const source = "hello🌍world";

    // These are UTF-8 byte offsets (what tree-sitter outputs)
    const helloSpan: Span = { start: 0, end: 5, capture: "string" }; // bytes 0-5
    const worldSpan: Span = { start: 9, end: 14, capture: "keyword" }; // bytes 9-14 (after 4-byte emoji)

    // UTF-8 offsets don't work directly with String.slice()
    expect(source.slice(helloSpan.start, helloSpan.end)).toBe("hello"); // happens to work (ASCII)
    expect(source.slice(worldSpan.start, worldSpan.end)).not.toBe("world"); // would fail!

    // But spansToHtml converts them correctly
    const html = spansToHtml(source, [helloSpan, worldSpan]);
    expect(html).toBe("<a-s>hello</a-s>🌍<a-k>world</a-k>");
  });
});
