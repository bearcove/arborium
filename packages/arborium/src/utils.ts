import type { Span } from "./types.js";

/** Convert spans to HTML */
export function spansToHtml(source: string, spans: Span[]): string {
  // Sort spans by start position
  const sorted = [...spans].sort((a, b) => a.start - b.start);

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
