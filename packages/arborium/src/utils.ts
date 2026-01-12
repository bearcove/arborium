import type { Span } from "./types.js";

// Shared TextEncoder instance
const encoder = new TextEncoder();

/**
 * Get the UTF-8 byte length of a string.
 */
export function utf8ByteLength(str: string): number {
  return encoder.encode(str).length;
}

/**
 * Convert a UTF-8 byte offset to a UTF-16 code unit index.
 *
 * This is needed because tree-sitter returns UTF-8 byte offsets,
 * but JavaScript's String methods use UTF-16 code unit indices.
 */
export function utf8OffsetToUtf16(source: string, utf8Offset: number): number {
  const utf8Bytes = encoder.encode(source);

  if (utf8Offset <= 0) return 0;
  if (utf8Offset >= utf8Bytes.length) return source.length;

  // Decode just the bytes up to the offset to get the UTF-16 length
  const decoder = new TextDecoder();
  const prefix = decoder.decode(utf8Bytes.slice(0, utf8Offset));
  return prefix.length;
}

/**
 * Build a mapping from UTF-8 byte offsets to UTF-16 code unit indices.
 *
 * More efficient than calling utf8OffsetToUtf16 repeatedly when you
 * have many offsets to convert for the same string.
 */
function buildUtf8ToUtf16Map(source: string): Uint32Array {
  const utf8Bytes = encoder.encode(source);
  const map = new Uint32Array(utf8Bytes.length + 1);

  let utf8Offset = 0;
  let utf16Index = 0;

  for (const char of source) {
    // Record mapping at current UTF-8 position
    map[utf8Offset] = utf16Index;

    // Get UTF-8 byte length of this character
    const charUtf8Len = encoder.encode(char).length;

    // Fill intermediate bytes (they map to the same UTF-16 index)
    for (let i = 1; i < charUtf8Len; i++) {
      map[utf8Offset + i] = utf16Index;
    }

    utf8Offset += charUtf8Len;
    // Characters >= U+10000 are surrogate pairs (2 UTF-16 code units)
    utf16Index += char.codePointAt(0)! >= 0x10000 ? 2 : 1;
  }

  // Map the end-of-string position
  map[utf8Offset] = utf16Index;

  return map;
}

/** Convert spans to HTML */
export function spansToHtml(source: string, spans: Span[]): string {
  // Build UTF-8 to UTF-16 offset mapping
  const utf8ToUtf16 = buildUtf8ToUtf16Map(source);

  // Convert span offsets from UTF-8 bytes to UTF-16 code units
  const convertedSpans = spans.map(span => ({
    ...span,
    start: utf8ToUtf16[span.start] ?? span.start,
    end: utf8ToUtf16[span.end] ?? span.end,
  }));

  // Sort spans by start position
  const sorted = [...convertedSpans].sort((a, b) => a.start - b.start);

  let html = "";
  let pos = 0;

  for (const span of sorted) {
    // Skip overlapping spans
    if (span.start < pos) continue;

    // Add text before span
    if (span.start > pos) {
      html += escapeHtml(source.slice(pos, span.start));
    }

    // Get tag for capture
    const tag = getTagForCapture(span.capture);
    const text = escapeHtml(source.slice(span.start, span.end));

    if (tag) {
      html += `<a-${tag}>${text}</a-${tag}>`;
    } else {
      html += text;
    }

    pos = span.end;
  }

  // Add remaining text
  if (pos < source.length) {
    html += escapeHtml(source.slice(pos));
  }

  return html;
}

/** Get the short tag for a capture name */
function getTagForCapture(capture: string): string | null {
  if (capture.startsWith("keyword") || capture === "include" || capture === "conditional") {
    return "k";
  }
  if (capture.startsWith("function") || capture.startsWith("method")) {
    return "f";
  }
  if (capture.startsWith("string") || capture === "character") {
    return "s";
  }
  if (capture.startsWith("comment")) {
    return "c";
  }
  if (capture.startsWith("type")) {
    return "t";
  }
  if (capture.startsWith("variable")) {
    return "v";
  }
  if (capture.startsWith("number") || capture === "float") {
    return "n";
  }
  if (capture.startsWith("operator")) {
    return "o";
  }
  if (capture.startsWith("punctuation")) {
    return "p";
  }
  if (capture.startsWith("tag")) {
    return "tg";
  }
  if (capture.startsWith("attribute")) {
    return "at";
  }
  return null;
}

/** Escape HTML special characters */
export function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
