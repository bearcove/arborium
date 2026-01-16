import { describe, it, expect } from "vitest";
import { spansToHtml } from "./utils.js";
import type { Utf16Span } from "./types.js";

// Helper to get UTF-16 code unit offsets for a substring
// This is what JavaScript's indexOf and slice use natively
function getUtf16Offsets(source: string, substring: string): { start: number; end: number } {
  const start = source.indexOf(substring);
  if (start === -1) throw new Error(`Substring "${substring}" not found in "${source}"`);
  return { start, end: start + substring.length };
}

describe("spansToHtml", () => {
  it("handles ASCII text correctly", () => {
    const source = "let x = 42;";
    const spans: Utf16Span[] = [
      { ...getUtf16Offsets(source, "let"), capture: "keyword" },
      { ...getUtf16Offsets(source, "42"), capture: "number" },
    ];

    const html = spansToHtml(source, spans);
    expect(html).toBe("<a-k>let</a-k> x = <a-n>42</a-n>;");
  });

  it("handles emoji correctly", () => {
    const source = "hello🌍world";
    const spans: Utf16Span[] = [
      { ...getUtf16Offsets(source, "hello"), capture: "string" },
      { ...getUtf16Offsets(source, "world"), capture: "keyword" },
    ];

    const html = spansToHtml(source, spans);
    expect(html).toBe("<a-s>hello</a-s>🌍<a-k>world</a-k>");
  });

  it("handles Chinese characters correctly", () => {
    const source = "let 变量 = 1";
    const spans: Utf16Span[] = [
      { ...getUtf16Offsets(source, "let"), capture: "keyword" },
      { ...getUtf16Offsets(source, "变量"), capture: "variable" },
      { ...getUtf16Offsets(source, "1"), capture: "number" },
    ];

    const html = spansToHtml(source, spans);
    expect(html).toBe("<a-k>let</a-k> <a-v>变量</a-v> = <a-n>1</a-n>");
  });

  it("handles multiple emoji in sequence", () => {
    const source = "a🎉🎊b";
    const spans: Utf16Span[] = [
      { ...getUtf16Offsets(source, "a"), capture: "variable" },
      { ...getUtf16Offsets(source, "b"), capture: "variable" },
    ];

    const html = spansToHtml(source, spans);
    expect(html).toBe("<a-v>a</a-v>🎉🎊<a-v>b</a-v>");
  });

  it("handles overlapping spans by skipping later ones", () => {
    const source = "hello";
    const spans: Utf16Span[] = [
      { start: 0, end: 5, capture: "string" },
      { start: 2, end: 4, capture: "keyword" }, // overlaps, should be skipped
    ];

    const html = spansToHtml(source, spans);
    expect(html).toBe("<a-s>hello</a-s>");
  });

  it("handles empty spans array", () => {
    const source = "hello world";
    const html = spansToHtml(source, []);
    expect(html).toBe("hello world");
  });

  it("escapes HTML special characters", () => {
    const source = "<div>&</div>";
    const spans: Utf16Span[] = [{ ...getUtf16Offsets(source, "<div>"), capture: "tag" }];

    const html = spansToHtml(source, spans);
    expect(html).toBe("<a-tg>&lt;div&gt;</a-tg>&amp;&lt;/div&gt;");
  });

  it("handles 2-byte UTF-8 characters (Latin extended)", () => {
    const source = "café";
    const spans: Utf16Span[] = [{ ...getUtf16Offsets(source, "café"), capture: "string" }];

    const html = spansToHtml(source, spans);
    expect(html).toBe("<a-s>café</a-s>");
  });

  it("handles mixed content with µ and á (the cpp sample case)", () => {
    // This is the actual case that was failing - cpp sample has these chars
    const source = 'fmt::format("{}", std::chrono::microseconds(42)), "42µs"';
    const spans: Utf16Span[] = [{ ...getUtf16Offsets(source, '"42µs"'), capture: "string" }];

    const html = spansToHtml(source, spans);
    expect(html).toContain("<a-s>&quot;42µs&quot;</a-s>");
  });
});
