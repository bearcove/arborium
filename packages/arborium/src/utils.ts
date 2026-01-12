import type { Span } from "./types.js";

/**
 * Convert UTF-8 byte offsets to UTF-16 code unit indices.
 *
 * The grammar plugins return UTF-8 byte offsets (native to tree-sitter),
 * but JavaScript's String.slice() uses UTF-16 code unit indices.
 * This function builds a mapping for efficient conversion.
 */
function buildUtf8ToUtf16Map(source: string): Uint32Array {
  // Encode to UTF-8 to get byte length
  const encoder = new TextEncoder();
  const utf8Bytes = encoder.encode(source);

  // Map from UTF-8 byte offset to UTF-16 code unit index
  // Size is utf8Bytes.length + 1 to handle end-of-string offset
  const map = new Uint32Array(utf8Bytes.length + 1);

  let utf8Offset = 0;
  let utf16Index = 0;

  for (const codePoint of source) {
    // Record mapping at current position
    map[utf8Offset] = utf16Index;

    // Advance by UTF-8 byte length of this code point
    const utf8Len = encoder.encode(codePoint).length;

    // Fill intermediate bytes (for multi-byte chars, only first byte is valid boundary)
    for (let i = 1; i < utf8Len; i++) {
      map[utf8Offset + i] = utf16Index; // Map to same UTF-16 index
    }

    utf8Offset += utf8Len;
    // Surrogate pairs (code points >= 0x10000) take 2 UTF-16 code units
    utf16Index += codePoint.codePointAt(0)! >= 0x10000 ? 2 : 1;
  }

  // Handle end-of-string offset
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
