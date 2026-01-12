import { describe, it, expect } from "vitest";
import { spansToHtml, utf8ByteLength, utf8OffsetToUtf16 } from "./utils.js";
import type { Span } from "./types.js";

// Helper to get UTF-8 byte offsets for a substring
function getUtf8Offsets(source: string, substring: string): { start: number; end: number } {
  const encoder = new TextEncoder();
  const idx = source.indexOf(substring);
  if (idx === -1) throw new Error(`Substring "${substring}" not found in "${source}"`);

  const beforeBytes = encoder.encode(source.slice(0, idx));
  const substringBytes = encoder.encode(substring);
  return { start: beforeBytes.length, end: beforeBytes.length + substringBytes.length };
}

describe("utf8ByteLength", () => {
  it("returns correct length for ASCII", () => {
    expect(utf8ByteLength("hello")).toBe(5);
  });

  it("returns correct length for 2-byte chars (Latin extended)", () => {
    expect(utf8ByteLength("é")).toBe(2);
    expect(utf8ByteLength("café")).toBe(5); // c=1, a=1, f=1, é=2
  });

  it("returns correct length for 3-byte chars (CJK)", () => {
    expect(utf8ByteLength("中")).toBe(3);
    expect(utf8ByteLength("中文")).toBe(6);
  });

  it("returns correct length for 4-byte chars (emoji)", () => {
    expect(utf8ByteLength("🌍")).toBe(4);
    expect(utf8ByteLength("🦀")).toBe(4);
    expect(utf8ByteLength("a🌍b")).toBe(6); // 1 + 4 + 1
  });
});

describe("utf8OffsetToUtf16", () => {
  it("returns same offset for ASCII", () => {
    const source = "hello";
    expect(utf8OffsetToUtf16(source, 0)).toBe(0);
    expect(utf8OffsetToUtf16(source, 3)).toBe(3);
    expect(utf8OffsetToUtf16(source, 5)).toBe(5);
  });

  it("converts correctly with 4-byte emoji", () => {
    const source = "hello🌍world";
    // UTF-8: hello(5) + 🌍(4) + world(5) = 14 bytes
    // UTF-16: hello(5) + 🌍(2) + world(5) = 12 code units

    expect(utf8OffsetToUtf16(source, 0)).toBe(0);   // start of "hello"
    expect(utf8OffsetToUtf16(source, 5)).toBe(5);   // end of "hello" / start of emoji
    expect(utf8OffsetToUtf16(source, 9)).toBe(7);   // end of emoji / start of "world"
    expect(utf8OffsetToUtf16(source, 14)).toBe(12); // end of string
  });

  it("converts correctly with 3-byte CJK chars", () => {
    const source = "let 变量 = 1";
    // UTF-8: let(3) + space(1) + 变(3) + 量(3) + " = "(3) + 1(1) = 14 bytes
    // UTF-16: let(3) + space(1) + 变(1) + 量(1) + " = "(3) + 1(1) = 10 code units

    expect(utf8OffsetToUtf16(source, 0)).toBe(0);   // start of "let"
    expect(utf8OffsetToUtf16(source, 4)).toBe(4);   // start of "变"
    expect(utf8OffsetToUtf16(source, 10)).toBe(6);  // end of "量"
  });

  it("handles multiple emoji", () => {
    const source = "a🎉🎊b";
    // UTF-8: a(1) + 🎉(4) + 🎊(4) + b(1) = 10 bytes
    // UTF-16: a(1) + 🎉(2) + 🎊(2) + b(1) = 6 code units

    expect(utf8OffsetToUtf16(source, 0)).toBe(0);   // start of "a"
    expect(utf8OffsetToUtf16(source, 1)).toBe(1);   // end of "a" / start of first emoji
    expect(utf8OffsetToUtf16(source, 5)).toBe(3);   // end of first emoji / start of second
    expect(utf8OffsetToUtf16(source, 9)).toBe(5);   // end of second emoji / start of "b"
    expect(utf8OffsetToUtf16(source, 10)).toBe(6);  // end of string
  });
});

describe("spansToHtml", () => {
  it("handles ASCII text correctly", () => {
    const source = "let x = 42;";
    const spans: Span[] = [
      { ...getUtf8Offsets(source, "let"), capture: "keyword" },
      { ...getUtf8Offsets(source, "42"), capture: "number" },
    ];

    const html = spansToHtml(source, spans);
    expect(html).toBe("<a-k>let</a-k> x = <a-n>42</a-n>;");
  });

  it("handles emoji correctly", () => {
    const source = "hello🌍world";
    const spans: Span[] = [
      { ...getUtf8Offsets(source, "hello"), capture: "string" },
      { ...getUtf8Offsets(source, "world"), capture: "keyword" },
    ];

    const html = spansToHtml(source, spans);
    expect(html).toBe("<a-s>hello</a-s>🌍<a-k>world</a-k>");
  });

  it("handles Chinese characters correctly", () => {
    const source = "let 变量 = 1";
    const spans: Span[] = [
      { ...getUtf8Offsets(source, "let"), capture: "keyword" },
      { ...getUtf8Offsets(source, "变量"), capture: "variable" },
      { ...getUtf8Offsets(source, "1"), capture: "number" },
    ];

    const html = spansToHtml(source, spans);
    expect(html).toBe("<a-k>let</a-k> <a-v>变量</a-v> = <a-n>1</a-n>");
  });

  it("handles multiple emoji in sequence", () => {
    const source = "a🎉🎊b";
    const spans: Span[] = [
      { ...getUtf8Offsets(source, "a"), capture: "variable" },
      { ...getUtf8Offsets(source, "b"), capture: "variable" },
    ];

    const html = spansToHtml(source, spans);
    expect(html).toBe("<a-v>a</a-v>🎉🎊<a-v>b</a-v>");
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
    const html = spansToHtml(source, []);
    expect(html).toBe("hello world");
  });

  it("escapes HTML special characters", () => {
    const source = "<div>&</div>";
    const spans: Span[] = [
      { ...getUtf8Offsets(source, "<div>"), capture: "tag" },
    ];

    const html = spansToHtml(source, spans);
    expect(html).toBe("<a-tg>&lt;div&gt;</a-tg>&amp;&lt;/div&gt;");
  });

  it("handles 2-byte UTF-8 characters (Latin extended)", () => {
    const source = "café";
    const spans: Span[] = [
      { ...getUtf8Offsets(source, "café"), capture: "string" },
    ];

    const html = spansToHtml(source, spans);
    expect(html).toBe("<a-s>café</a-s>");
  });

  it("handles mixed content with µ and á (the cpp sample case)", () => {
    // This is the actual case that was failing - cpp sample has these chars
    const source = 'fmt::format("{}", std::chrono::microseconds(42)), "42µs"';
    const spans: Span[] = [
      { ...getUtf8Offsets(source, '"42µs"'), capture: "string" },
    ];

    const html = spansToHtml(source, spans);
    expect(html).toContain("<a-s>&quot;42µs&quot;</a-s>");
  });
});
