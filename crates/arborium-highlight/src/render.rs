//! HTML rendering from highlight spans.
//!
//! This module converts raw spans from grammar parsers into HTML with proper
//! handling of overlapping spans (deduplication) and span coalescing.
//!
//! # Span Coalescing
//!
//! Adjacent spans that map to the same theme slot are merged into a single HTML element.
//! For example, if we have:
//! - `keyword.function` at bytes 0-4
//! - `keyword` at bytes 5-8
//!
//! Both map to the "keyword" slot (`k` tag), so they become a single `<a-k>` element.

use crate::{HtmlFormat, Span};
use arborium_theme::{
    Theme, capture_to_slot, slot_to_highlight_index, tag_for_capture, tag_to_name,
};
use std::collections::HashMap;
use std::io::{self, Write};

/// A span with a theme style index for rendering.
///
/// This is the output of processing raw `Span` objects through the theme system.
/// The `theme_index` can be used with `Theme::style()` to get colors and modifiers.
#[derive(Debug, Clone)]
pub struct ThemedSpan {
    /// Byte offset where the span starts (inclusive).
    pub start: u32,
    /// Byte offset where the span ends (exclusive).
    pub end: u32,
    /// Index into the theme's style array.
    pub theme_index: usize,
}

/// Convert raw spans to themed spans by resolving capture names to theme indices.
///
/// This performs deduplication and returns spans with theme style indices that can
/// be used with `Theme::style()` to get colors and modifiers.
pub fn spans_to_themed(spans: Vec<Span>) -> Vec<ThemedSpan> {
    if spans.is_empty() {
        return Vec::new();
    }

    // Sort spans by (start, -end) so longer spans come first at same start
    let mut spans = spans;
    spans.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| b.end.cmp(&a.end)));

    // Deduplicate ranges - prefer spans with higher pattern_index (later in highlights.scm wins)
    // This matches tree-sitter convention: later patterns override earlier ones
    let mut deduped: HashMap<(u32, u32), Span> = HashMap::new();
    for span in spans {
        let key = (span.start, span.end);
        let new_has_slot = slot_to_highlight_index(capture_to_slot(&span.capture)).is_some();

        if let Some(existing) = deduped.get(&key) {
            let existing_has_slot =
                slot_to_highlight_index(capture_to_slot(&existing.capture)).is_some();
            // Prefer spans with styling over unstyled spans
            // Among equally-styled spans, prefer higher pattern_index (later in query)
            let should_replace = match (new_has_slot, existing_has_slot) {
                (true, false) => true,  // New has styling, existing doesn't
                (false, true) => false, // Existing has styling, new doesn't
                _ => span.pattern_index >= existing.pattern_index, // Both same styling status: higher pattern_index wins
            };
            if should_replace {
                deduped.insert(key, span);
            }
        } else {
            deduped.insert(key, span);
        }
    }

    // Convert to themed spans
    let mut themed: Vec<ThemedSpan> = deduped
        .into_values()
        .filter_map(|span| {
            let slot = capture_to_slot(&span.capture);
            let theme_index = slot_to_highlight_index(slot)?;
            Some(ThemedSpan {
                start: span.start,
                end: span.end,
                theme_index,
            })
        })
        .collect();

    // Sort by start position
    themed.sort_by_key(|s| s.start);

    themed
}

#[cfg(feature = "unicode-width")]
use unicode_width::UnicodeWidthChar;

/// Generate opening and closing HTML tags based on the configured format.
///
/// Returns (opening_tag, closing_tag) for the given short tag and format.
fn make_html_tags(short_tag: &str, format: &HtmlFormat) -> (String, String) {
    match format {
        HtmlFormat::CustomElements => {
            let open = format!("<a-{short_tag}>");
            let close = format!("</a-{short_tag}>");
            (open, close)
        }
        HtmlFormat::CustomElementsWithPrefix(prefix) => {
            let open = format!("<{prefix}-{short_tag}>");
            let close = format!("</{prefix}-{short_tag}>");
            (open, close)
        }
        HtmlFormat::ClassNames => {
            if let Some(name) = tag_to_name(short_tag) {
                let open = format!("<span class=\"{name}\">");
                let close = "</span>".to_string();
                (open, close)
            } else {
                // Fallback for unknown tags
                ("<span>".to_string(), "</span>".to_string())
            }
        }
        HtmlFormat::ClassNamesWithPrefix(prefix) => {
            if let Some(name) = tag_to_name(short_tag) {
                let open = format!("<span class=\"{prefix}-{name}\">");
                let close = "</span>".to_string();
                (open, close)
            } else {
                // Fallback for unknown tags
                ("<span>".to_string(), "</span>".to_string())
            }
        }
    }
}

/// A normalized span with theme slot tag.
#[derive(Debug, Clone)]
struct NormalizedSpan {
    start: u32,
    end: u32,
    tag: &'static str,
}

/// Normalize spans: map captures to theme slots and merge adjacent spans with same tag.
fn normalize_and_coalesce(spans: Vec<Span>) -> Vec<NormalizedSpan> {
    if spans.is_empty() {
        return vec![];
    }

    // First, normalize all spans to their theme slot tags
    let mut normalized: Vec<NormalizedSpan> = spans
        .into_iter()
        .filter_map(|span| {
            tag_for_capture(&span.capture).map(|tag| NormalizedSpan {
                start: span.start,
                end: span.end,
                tag,
            })
        })
        .collect();

    if normalized.is_empty() {
        return vec![];
    }

    // Sort by start position
    normalized.sort_by_key(|s| (s.start, s.end));

    // Coalesce adjacent spans with the same tag
    let mut coalesced: Vec<NormalizedSpan> = Vec::with_capacity(normalized.len());

    for span in normalized {
        if let Some(last) = coalesced.last_mut() {
            // If this span is adjacent (or overlapping) and has the same tag, merge
            if span.tag == last.tag && span.start <= last.end {
                // Extend the last span to cover this one
                last.end = last.end.max(span.end);
                continue;
            }
        }
        coalesced.push(span);
    }

    coalesced
}

/// Deduplicate spans and convert to HTML.
///
/// This handles:
/// 1. Mapping captures to theme slots (many -> few)
/// 2. Coalescing adjacent spans with the same tag
/// 3. Handling overlapping spans
///
/// The `format` parameter controls the HTML output style.
pub fn spans_to_html(source: &str, spans: Vec<Span>, format: &HtmlFormat) -> String {
    if spans.is_empty() {
        return html_escape(source);
    }

    // Sort spans by (start, -end) so longer spans come first at same start
    let mut spans = spans;
    spans.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| b.end.cmp(&a.end)));

    // Deduplicate: for spans with the exact same (start, end), prefer spans with higher pattern_index
    // This matches tree-sitter convention: later patterns in highlights.scm override earlier ones.
    // We also prefer styled spans over unstyled (e.g., @comment over @spell).
    let mut deduped: HashMap<(u32, u32), Span> = HashMap::new();
    for span in spans {
        let key = (span.start, span.end);
        let new_has_styling = tag_for_capture(&span.capture).is_some();

        if let Some(existing) = deduped.get(&key) {
            let existing_has_styling = tag_for_capture(&existing.capture).is_some();
            // Prefer spans with styling over unstyled spans
            // Among equally-styled spans, prefer higher pattern_index (later in query)
            let should_replace = match (new_has_styling, existing_has_styling) {
                (true, false) => true,  // New has styling, existing doesn't
                (false, true) => false, // Existing has styling, new doesn't
                _ => span.pattern_index >= existing.pattern_index, // Both same styling status: higher pattern_index wins
            };
            if should_replace {
                deduped.insert(key, span);
            }
        } else {
            deduped.insert(key, span);
        }
    }

    // Convert back to vec
    let spans: Vec<Span> = deduped.into_values().collect();

    // Normalize to theme slots and coalesce adjacent same-tag spans
    let spans = normalize_and_coalesce(spans);

    if spans.is_empty() {
        return html_escape(source);
    }

    // Re-sort after coalescing
    let mut spans = spans;
    spans.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| b.end.cmp(&a.end)));

    // Build events from spans
    let mut events: Vec<(u32, bool, usize)> = Vec::new(); // (pos, is_start, span_index)
    for (i, span) in spans.iter().enumerate() {
        events.push((span.start, true, i));
        events.push((span.end, false, i));
    }

    // Sort events: by position, then ends before starts at same position
    events.sort_by(|a, b| {
        a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)) // false (end) < true (start)
    });

    // Process events with a stack
    let mut html = String::with_capacity(source.len() * 2);
    let mut last_pos: usize = 0;
    let mut stack: Vec<usize> = Vec::new(); // indices into spans

    for (pos, is_start, span_idx) in events {
        let pos = pos as usize;

        // Emit any source text before this position
        if pos > last_pos && pos <= source.len() {
            let text = &source[last_pos..pos];
            if let Some(&top_idx) = stack.last() {
                let tag = spans[top_idx].tag;
                let (open_tag, close_tag) = make_html_tags(tag, format);
                html.push_str(&open_tag);
                html.push_str(&html_escape(text));
                html.push_str(&close_tag);
            } else {
                html.push_str(&html_escape(text));
            }
            last_pos = pos;
        }

        // Update the stack
        if is_start {
            stack.push(span_idx);
        } else {
            // Remove this span from stack
            if let Some(idx) = stack.iter().rposition(|&x| x == span_idx) {
                stack.remove(idx);
            }
        }
    }

    // Emit remaining text
    if last_pos < source.len() {
        let text = &source[last_pos..];
        if let Some(&top_idx) = stack.last() {
            let tag = spans[top_idx].tag;
            let (open_tag, close_tag) = make_html_tags(tag, format);
            html.push_str(&open_tag);
            html.push_str(&html_escape(text));
            html.push_str(&close_tag);
        } else {
            html.push_str(&html_escape(text));
        }
    }

    html
}

/// Write spans as HTML to a writer.
///
/// This is more efficient than `spans_to_html` for streaming output.
pub fn write_spans_as_html<W: Write>(
    w: &mut W,
    source: &str,
    spans: Vec<Span>,
    format: &HtmlFormat,
) -> io::Result<()> {
    let html = spans_to_html(source, spans, format);
    w.write_all(html.as_bytes())
}

/// Escape HTML special characters.
pub fn html_escape(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(c),
        }
    }
    result
}

/// Options controlling ANSI rendering behavior.
#[derive(Debug, Clone)]
pub struct AnsiOptions {
    /// If true, apply the theme's foreground/background as a base style
    /// for all text (including un-highlighted regions).
    pub use_theme_base_style: bool,
    /// Optional hard wrap width (in columns). When None, no wrapping is
    /// performed and the original line structure is preserved.
    pub width: Option<usize>,
    /// If true and `width` is set, pad each visual line with spaces up
    /// to exactly `width` columns.
    pub pad_to_width: bool,
    /// Tab width (in columns) used when computing display width.
    pub tab_width: usize,
    /// Horizontal margin (in columns) outside the border/background.
    /// This is empty space with no styling.
    pub margin_x: usize,
    /// Vertical margin (in rows) outside the border/background.
    /// This is empty space with no styling.
    pub margin_y: usize,
    /// Horizontal padding (in columns) on left and right sides.
    /// Inside the background, between border and content.
    pub padding_x: usize,
    /// Vertical padding (in rows) on top and bottom.
    /// Inside the background, between border and content.
    pub padding_y: usize,
    /// If true, draw a border around the code block using half-block characters.
    pub border: bool,
}

/// Unicode block drawing characters used to create visual borders around ANSI output.
///
/// These characters create a "half-block" border style that works well in terminals:
/// - `TOP`/`BOTTOM`: half blocks that create smooth edges
/// - `LEFT`/`RIGHT`: full blocks for solid vertical borders
pub struct BoxChars;

impl BoxChars {
    /// Unicode lower half block (`▄`) used for the top border.
    pub const TOP: char = '▄';
    /// Unicode upper half block (`▀`) used for the bottom border.
    pub const BOTTOM: char = '▀';
    /// Unicode full block (`█`) used for the left border.
    pub const LEFT: char = '█';
    /// Unicode full block (`█`) used for the right border.
    pub const RIGHT: char = '█';
}

fn detect_terminal_width() -> Option<usize> {
    #[cfg(all(feature = "terminal-size", not(target_arch = "wasm32")))]
    {
        use terminal_size::{Width, terminal_size};
        if let Some((Width(w), _)) = terminal_size() {
            Some(w as usize)
        } else {
            None
        }
    }
    #[cfg(any(not(feature = "terminal-size"), target_arch = "wasm32"))]
    {
        None
    }
}

impl Default for AnsiOptions {
    fn default() -> Self {
        let width = detect_terminal_width();
        Self {
            use_theme_base_style: false,
            width,
            pad_to_width: width.is_some(),
            tab_width: 4,
            margin_x: 0,
            margin_y: 0,
            padding_x: 0,
            padding_y: 0,
            border: false,
        }
    }
}

#[cfg(feature = "unicode-width")]
fn char_display_width(c: char, col: usize, tab_width: usize) -> usize {
    if c == '\t' {
        let next_tab = ((col / tab_width) + 1) * tab_width;
        next_tab - col
    } else {
        UnicodeWidthChar::width(c).unwrap_or(0)
    }
}

#[cfg(not(feature = "unicode-width"))]
fn char_display_width(c: char, col: usize, tab_width: usize) -> usize {
    if c == '\t' {
        let next_tab = ((col / tab_width) + 1) * tab_width;
        next_tab - col
    } else {
        1
    }
}

fn write_wrapped_text(
    out: &mut String,
    text: &str,
    options: &AnsiOptions,
    current_col: &mut usize,
    base_ansi: &str,
    active_style: Option<usize>,
    theme: &Theme,
    use_base_bg: bool,
    border_style: &str,
) {
    // No wrapping requested: just track column and append text.
    let Some(inner_width) = options.width else {
        for ch in text.chars() {
            match ch {
                '\n' | '\r' => {
                    *current_col = 0;
                    out.push(ch);
                }
                other => {
                    let w = char_display_width(other, *current_col, options.tab_width);
                    if other == '\t' {
                        for _ in 0..w {
                            out.push(' ');
                        }
                    } else {
                        out.push(other);
                    }
                    *current_col += w;
                }
            }
        }
        return;
    };

    let padding_x = options.padding_x;
    let margin_x = options.margin_x;
    let border = options.border;
    // Inner width excludes border characters, with a minimum to handle narrow terminals
    const MIN_CONTENT_WIDTH: usize = 10;
    let width = if border {
        inner_width.saturating_sub(2).max(MIN_CONTENT_WIDTH)
    } else {
        inner_width.max(MIN_CONTENT_WIDTH)
    };
    let content_end = width.saturating_sub(padding_x); // where content should stop (before right padding)
    let pad_to_width = options.pad_to_width;

    for ch in text.chars() {
        // At the start of a visual line, emit margin + left border + left padding
        if *current_col == 0 {
            // Left margin
            for _ in 0..margin_x {
                out.push(' ');
            }
            // Left border (full block)
            if border && !border_style.is_empty() {
                out.push_str(border_style);
                out.push(BoxChars::LEFT);
                out.push_str(Theme::ANSI_RESET);
                if !base_ansi.is_empty() {
                    out.push_str(base_ansi);
                }
            }
            // Left padding
            if padding_x > 0 {
                for _ in 0..padding_x {
                    out.push(' ');
                }
                *current_col += padding_x;
            }
        }

        if ch == '\n' || ch == '\r' {
            // Pad to full width (including right padding)
            if pad_to_width && *current_col < width {
                let pad = width - *current_col;
                for _ in 0..pad {
                    out.push(' ');
                }
            }
            // Right border (full block)
            if border && !border_style.is_empty() {
                out.push_str(Theme::ANSI_RESET);
                out.push_str(border_style);
                out.push(BoxChars::RIGHT);
            }
            // Reset before newline so background doesn't extend to terminal edge
            out.push_str(Theme::ANSI_RESET);
            out.push('\n');
            *current_col = 0;

            if !base_ansi.is_empty() {
                out.push_str(base_ansi);
            }
            if let Some(idx) = active_style {
                let style = if use_base_bg {
                    theme.ansi_style_with_base_bg(idx)
                } else {
                    theme.ansi_style(idx)
                };
                out.push_str(&style);
            }
            continue;
        }

        let w = char_display_width(ch, *current_col, options.tab_width);
        // Wrap when we would exceed the content area (before right padding)
        if w > 0 && *current_col + w > content_end {
            // Pad to full width (including right padding)
            if pad_to_width && *current_col < width {
                let pad = width - *current_col;
                for _ in 0..pad {
                    out.push(' ');
                }
            }
            // Right border (full block)
            if border && !border_style.is_empty() {
                out.push_str(Theme::ANSI_RESET);
                out.push_str(border_style);
                out.push(BoxChars::RIGHT);
            }
            // Reset before newline so background doesn't extend to terminal edge
            out.push_str(Theme::ANSI_RESET);
            out.push('\n');
            *current_col = 0;

            if !base_ansi.is_empty() {
                out.push_str(base_ansi);
            }
            // New visual line after wrap: emit left margin + border + padding
            // Left margin
            for _ in 0..margin_x {
                out.push(' ');
            }
            // Left border (full block)
            if border && !border_style.is_empty() {
                out.push_str(border_style);
                out.push(BoxChars::LEFT);
                out.push_str(Theme::ANSI_RESET);
                if !base_ansi.is_empty() {
                    out.push_str(base_ansi);
                }
            }
            // Re-apply active style after border
            if let Some(idx) = active_style {
                let style = if use_base_bg {
                    theme.ansi_style_with_base_bg(idx)
                } else {
                    theme.ansi_style(idx)
                };
                out.push_str(&style);
            }
            // Left padding
            if padding_x > 0 {
                for _ in 0..padding_x {
                    out.push(' ');
                }
                *current_col += padding_x;
            }
        }

        if ch == '\t' {
            let w = char_display_width('\t', *current_col, options.tab_width);
            for _ in 0..w {
                out.push(' ');
            }
            *current_col += w;
        } else {
            out.push(ch);
            *current_col += w;
        }
    }
}

/// Deduplicate spans and convert to ANSI-colored text using a theme.
///
/// This mirrors the HTML rendering logic but emits ANSI escape sequences
/// instead of `<a-*>` tags, using `Theme::ansi_style` for each slot.
pub fn spans_to_ansi(source: &str, spans: Vec<Span>, theme: &Theme) -> String {
    spans_to_ansi_with_options(source, spans, theme, &AnsiOptions::default())
}

/// ANSI rendering with additional configuration options.
pub fn spans_to_ansi_with_options(
    source: &str,
    spans: Vec<Span>,
    theme: &Theme,
    options: &AnsiOptions,
) -> String {
    // Trim trailing newlines from source
    let source = source.trim_end_matches('\n');

    if spans.is_empty() {
        return source.to_string();
    }

    // Sort spans by (start, -end) so longer spans come first at same start
    let mut spans = spans;
    spans.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| b.end.cmp(&a.end)));

    // Deduplicate ranges - prefer spans with higher pattern_index (later in highlights.scm wins)
    // This matches tree-sitter convention: later patterns override earlier ones
    let mut deduped: HashMap<(u32, u32), Span> = HashMap::new();
    for span in spans {
        let key = (span.start, span.end);
        let new_has_slot = slot_to_highlight_index(capture_to_slot(&span.capture)).is_some();

        if let Some(existing) = deduped.get(&key) {
            let existing_has_slot =
                slot_to_highlight_index(capture_to_slot(&existing.capture)).is_some();
            // Prefer spans with styling over unstyled spans
            // Among equally-styled spans, prefer higher pattern_index (later in query)
            let should_replace = match (new_has_slot, existing_has_slot) {
                (true, false) => true,  // New has styling, existing doesn't
                (false, true) => false, // Existing has styling, new doesn't
                _ => span.pattern_index >= existing.pattern_index, // Both same styling status: higher pattern_index wins
            };
            if should_replace {
                deduped.insert(key, span);
            }
        } else {
            deduped.insert(key, span);
        }
    }

    let spans: Vec<Span> = deduped.into_values().collect();

    // Normalize to highlight indices and coalesce adjacent spans with same style
    #[derive(Debug, Clone)]
    struct StyledSpan {
        start: u32,
        end: u32,
        index: usize,
    }

    let mut normalized: Vec<StyledSpan> = spans
        .into_iter()
        .filter_map(|span| {
            let slot = capture_to_slot(&span.capture);
            let index = slot_to_highlight_index(slot)?;
            // Filter out empty styles when using base style - they'll just use the base
            if options.use_theme_base_style {
                if let Some(style) = theme.style(index) {
                    if style.is_empty() {
                        return None;
                    }
                }
            }
            Some(StyledSpan {
                start: span.start,
                end: span.end,
                index,
            })
        })
        .collect();

    if normalized.is_empty() {
        return source.to_string();
    }

    // Sort by start
    normalized.sort_by_key(|s| (s.start, s.end));

    // Coalesce adjacent/overlapping spans with the same style index
    let mut coalesced: Vec<StyledSpan> = Vec::with_capacity(normalized.len());
    for span in normalized {
        if let Some(last) = coalesced.last_mut() {
            if span.index == last.index && span.start <= last.end {
                last.end = last.end.max(span.end);
                continue;
            }
        }
        coalesced.push(span);
    }

    if coalesced.is_empty() {
        return source.to_string();
    }

    // Build events from spans
    let mut events: Vec<(u32, bool, usize)> = Vec::new();
    for (i, span) in coalesced.iter().enumerate() {
        events.push((span.start, true, i));
        events.push((span.end, false, i));
    }

    events.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut out = String::with_capacity(source.len() * 2);
    let mut last_pos: usize = 0;
    let mut stack: Vec<usize> = Vec::new();
    let mut active_style: Option<usize> = None;
    let mut current_col: usize = 0;

    let base_ansi = if options.use_theme_base_style {
        theme.ansi_base_style()
    } else {
        String::new()
    };
    let use_base_bg = options.use_theme_base_style;

    // Track if we've output anything yet to avoid duplicate base style at start
    let mut output_started = false;

    let padding_y = options.padding_y;
    let margin_x = options.margin_x;
    let margin_y = options.margin_y;
    let border = options.border;
    let border_style = if border {
        theme.ansi_border_style()
    } else {
        String::new()
    };

    // Minimum width to ensure usable output on narrow terminals
    const MIN_WIDTH: usize = 10;

    if let Some(width) = options.width.map(|w| w.max(MIN_WIDTH)) {
        // Top margin (empty lines)
        for _ in 0..margin_y {
            out.push('\n');
        }

        // Top border row
        if border {
            // Left margin spaces
            for _ in 0..margin_x {
                out.push(' ');
            }
            out.push_str(&border_style);
            for _ in 0..width {
                out.push(BoxChars::TOP);
            }
            out.push_str(Theme::ANSI_RESET);
            out.push('\n');
        }

        // Top padding rows (inside the background)
        if padding_y > 0 {
            for _ in 0..padding_y {
                // Left margin
                for _ in 0..margin_x {
                    out.push(' ');
                }
                // Left border (full block)
                if border {
                    out.push_str(&border_style);
                    out.push(BoxChars::LEFT);
                }
                // Apply base style for the padding content
                if !base_ansi.is_empty() {
                    out.push_str(&base_ansi);
                    output_started = true;
                }
                // Inner width (minus border chars if present)
                let inner = if border {
                    width.saturating_sub(2)
                } else {
                    width
                };
                for _ in 0..inner {
                    out.push(' ');
                }
                // Right border (full block)
                if border {
                    out.push_str(Theme::ANSI_RESET);
                    out.push_str(&border_style);
                    out.push(BoxChars::RIGHT);
                }
                out.push_str(Theme::ANSI_RESET);
                out.push('\n');
                // Reapply base style for next line
                if !base_ansi.is_empty() {
                    out.push_str(&base_ansi);
                }
            }
        } else if !base_ansi.is_empty() {
            // No top padding but we need base style for content
            out.push_str(&base_ansi);
            output_started = true;
        }
    } else {
        // No width specified, just apply base style if needed
        if !base_ansi.is_empty() {
            out.push_str(&base_ansi);
            output_started = true;
        }
    }

    for (pos, is_start, span_idx) in events {
        let pos = pos as usize;
        if pos > last_pos && pos <= source.len() {
            let text = &source[last_pos..pos];
            let desired = stack.last().copied().map(|idx| coalesced[idx].index);

            match (active_style, desired) {
                (Some(a), Some(d)) if a == d => {
                    // Style hasn't changed, just write text
                    write_wrapped_text(
                        &mut out,
                        text,
                        options,
                        &mut current_col,
                        &base_ansi,
                        Some(a),
                        theme,
                        use_base_bg,
                        &border_style,
                    );
                }
                (Some(_), Some(d)) => {
                    // Style change: reset and apply new style
                    out.push_str(Theme::ANSI_RESET);
                    let style = if use_base_bg {
                        theme.ansi_style_with_base_bg(d)
                    } else {
                        theme.ansi_style(d)
                    };
                    // If using base_bg, the style already includes base colors, so don't emit base_ansi separately
                    // If the style is identical to base, just emit base once
                    if use_base_bg {
                        out.push_str(&style);
                    } else {
                        if !base_ansi.is_empty() {
                            out.push_str(&base_ansi);
                        }
                        out.push_str(&style);
                    }
                    write_wrapped_text(
                        &mut out,
                        text,
                        options,
                        &mut current_col,
                        &base_ansi,
                        Some(d),
                        theme,
                        use_base_bg,
                        &border_style,
                    );
                    active_style = Some(d);
                }
                (None, Some(d)) => {
                    // First styled span or transitioning from unstyled to styled
                    let style = if use_base_bg {
                        theme.ansi_style_with_base_bg(d)
                    } else {
                        theme.ansi_style(d)
                    };

                    // When using base_bg, if the style is identical to base_ansi, don't emit it
                    if !style.is_empty() && style != base_ansi {
                        // Emit the style code
                        out.push_str(&style);
                        output_started = true;
                    } else if !output_started && !base_ansi.is_empty() {
                        // No distinct style, just ensure base is active
                        out.push_str(&base_ansi);
                        output_started = true;
                    }

                    write_wrapped_text(
                        &mut out,
                        text,
                        options,
                        &mut current_col,
                        &base_ansi,
                        Some(d),
                        theme,
                        use_base_bg,
                        &border_style,
                    );
                    active_style = Some(d);
                }
                (Some(_), None) => {
                    // Transitioning from styled to unstyled
                    out.push_str(Theme::ANSI_RESET);
                    if !base_ansi.is_empty() {
                        out.push_str(&base_ansi);
                    }
                    write_wrapped_text(
                        &mut out,
                        text,
                        options,
                        &mut current_col,
                        &base_ansi,
                        None,
                        theme,
                        use_base_bg,
                        &border_style,
                    );
                    active_style = None;
                }
                (None, None) => {
                    // No styling, just plain text
                    if !output_started && !base_ansi.is_empty() {
                        out.push_str(&base_ansi);
                        output_started = true;
                    }
                    write_wrapped_text(
                        &mut out,
                        text,
                        options,
                        &mut current_col,
                        &base_ansi,
                        None,
                        theme,
                        use_base_bg,
                        &border_style,
                    );
                }
            }

            last_pos = pos;
        }

        if is_start {
            stack.push(span_idx);
        } else if let Some(idx) = stack.iter().rposition(|&x| x == span_idx) {
            stack.remove(idx);
        }
    }

    if last_pos < source.len() {
        let text = &source[last_pos..];
        let desired = stack.last().copied().map(|idx| coalesced[idx].index);
        match (active_style, desired) {
            (Some(a), Some(d)) if a == d => {
                write_wrapped_text(
                    &mut out,
                    text,
                    options,
                    &mut current_col,
                    &base_ansi,
                    Some(a),
                    theme,
                    use_base_bg,
                    &border_style,
                );
            }
            (Some(_), Some(d)) => {
                out.push_str(Theme::ANSI_RESET);
                let style = if use_base_bg {
                    theme.ansi_style_with_base_bg(d)
                } else {
                    theme.ansi_style(d)
                };
                // If using base_bg, the style already includes base colors
                if use_base_bg {
                    out.push_str(&style);
                } else {
                    if !base_ansi.is_empty() {
                        out.push_str(&base_ansi);
                    }
                    out.push_str(&style);
                }
                write_wrapped_text(
                    &mut out,
                    text,
                    options,
                    &mut current_col,
                    &base_ansi,
                    Some(d),
                    theme,
                    use_base_bg,
                    &border_style,
                );
                active_style = Some(d);
            }
            (None, Some(d)) => {
                let style = if use_base_bg {
                    theme.ansi_style_with_base_bg(d)
                } else {
                    theme.ansi_style(d)
                };

                // When using base_bg, if the style is identical to base_ansi, don't emit it
                if !style.is_empty() && style != base_ansi {
                    out.push_str(&style);
                } else if !output_started && !base_ansi.is_empty() {
                    out.push_str(&base_ansi);
                }

                write_wrapped_text(
                    &mut out,
                    text,
                    options,
                    &mut current_col,
                    &base_ansi,
                    Some(d),
                    theme,
                    use_base_bg,
                    &border_style,
                );
                active_style = Some(d);
            }
            (Some(_), None) => {
                out.push_str(Theme::ANSI_RESET);
                if !base_ansi.is_empty() {
                    out.push_str(&base_ansi);
                }
                write_wrapped_text(
                    &mut out,
                    text,
                    options,
                    &mut current_col,
                    &base_ansi,
                    None,
                    theme,
                    use_base_bg,
                    &border_style,
                );
                active_style = None;
            }
            (None, None) => {
                if !output_started && !base_ansi.is_empty() {
                    out.push_str(&base_ansi);
                }
                write_wrapped_text(
                    &mut out,
                    text,
                    options,
                    &mut current_col,
                    &base_ansi,
                    None,
                    theme,
                    use_base_bg,
                    &border_style,
                );
            }
        }
    }

    if let Some(width) = options.width {
        let padding_y = options.padding_y;
        let pad_to_width = options.pad_to_width;
        // Inner width excludes border characters
        let inner_width = if border {
            width.saturating_sub(2)
        } else {
            width
        };

        // Pad the final content line out to the full width.
        if pad_to_width && current_col < inner_width {
            let pad = inner_width - current_col;
            for _ in 0..pad {
                out.push(' ');
            }
        }

        // Right border on final content line
        if border && !border_style.is_empty() {
            out.push_str(Theme::ANSI_RESET);
            out.push_str(&border_style);
            out.push(BoxChars::RIGHT);
        }

        // Reset before newline so background doesn't extend to terminal edge
        out.push_str(Theme::ANSI_RESET);

        // Bottom padding rows.
        if padding_y > 0 {
            for _ in 0..padding_y {
                out.push('\n');
                // Left margin
                for _ in 0..margin_x {
                    out.push(' ');
                }
                // Left border
                if border {
                    out.push_str(&border_style);
                    out.push(BoxChars::LEFT);
                }
                // Background fill
                if !base_ansi.is_empty() {
                    out.push_str(&base_ansi);
                }
                let inner = if border {
                    width.saturating_sub(2)
                } else {
                    width
                };
                for _ in 0..inner {
                    out.push(' ');
                }
                // Right border
                if border {
                    out.push_str(Theme::ANSI_RESET);
                    out.push_str(&border_style);
                    out.push(BoxChars::RIGHT);
                }
                out.push_str(Theme::ANSI_RESET);
            }
        }

        // Bottom border row
        if border {
            out.push('\n');
            // Left margin spaces
            for _ in 0..margin_x {
                out.push(' ');
            }
            out.push_str(&border_style);
            for _ in 0..width {
                out.push(BoxChars::BOTTOM);
            }
            out.push_str(Theme::ANSI_RESET);
        }

        // Bottom margin (empty lines)
        for _ in 0..margin_y {
            out.push('\n');
        }
    } else if active_style.is_some() || !base_ansi.is_empty() {
        out.push_str(Theme::ANSI_RESET);
    }

    out
}

/// Write spans as ANSI-colored text to a writer.
pub fn write_spans_as_ansi<W: Write>(
    w: &mut W,
    source: &str,
    spans: Vec<Span>,
    theme: &Theme,
) -> io::Result<()> {
    let ansi = spans_to_ansi(source, spans, theme);
    w.write_all(ansi.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_highlight() {
        let source = "fn main";
        let spans = vec![
            Span {
                start: 0,
                end: 2,
                capture: "keyword".into(),
                pattern_index: 0,
            },
            Span {
                start: 3,
                end: 7,
                capture: "function".into(),
                pattern_index: 0,
            },
        ];
        let html = spans_to_html(source, spans, &HtmlFormat::CustomElements);
        assert_eq!(html, "<a-k>fn</a-k> <a-f>main</a-f>");
    }

    #[test]
    fn test_keyword_variants_coalesce() {
        // Different keyword captures should all map to "k" and coalesce
        let source = "with use import";
        let spans = vec![
            Span {
                start: 0,
                end: 4,
                capture: "include".into(), // nvim-treesitter name
                pattern_index: 0,
            },
            Span {
                start: 5,
                end: 8,
                capture: "keyword".into(),
                pattern_index: 0,
            },
            Span {
                start: 9,
                end: 15,
                capture: "keyword.import".into(),
                pattern_index: 0,
            },
        ];
        let html = spans_to_html(source, spans, &HtmlFormat::CustomElements);
        // All should use "k" tag - but they're not adjacent so still separate
        assert!(html.contains("<a-k>with</a-k>"));
        assert!(html.contains("<a-k>use</a-k>"));
        assert!(html.contains("<a-k>import</a-k>"));
    }

    #[test]
    fn test_adjacent_same_tag_coalesce() {
        // Adjacent spans with same tag should merge
        let source = "keyword";
        let spans = vec![
            Span {
                start: 0,
                end: 3,
                capture: "keyword".into(),
                pattern_index: 0,
            },
            Span {
                start: 3,
                end: 7,
                capture: "keyword.function".into(), // Maps to same slot
                pattern_index: 0,
            },
        ];
        let html = spans_to_html(source, spans, &HtmlFormat::CustomElements);
        // Should be one tag, not two
        assert_eq!(html, "<a-k>keyword</a-k>");
    }

    #[test]
    fn test_overlapping_spans_dedupe() {
        let source = "apiVersion";
        // Two spans for the same range - should keep only one
        let spans = vec![
            Span {
                start: 0,
                end: 10,
                capture: "property".into(),
                pattern_index: 0,
            },
            Span {
                start: 0,
                end: 10,
                capture: "variable".into(),
                pattern_index: 0,
            },
        ];
        let html = spans_to_html(source, spans, &HtmlFormat::CustomElements);
        // Should only have one tag, not two
        assert!(!html.contains("apiVersionapiVersion"));
        assert!(html.contains("apiVersion"));
    }

    #[test]
    fn test_html_escape() {
        let source = "<script>";
        let spans = vec![];
        let html = spans_to_html(source, spans, &HtmlFormat::CustomElements);
        assert_eq!(html, "&lt;script&gt;");
    }

    #[test]
    fn test_nospell_filtered() {
        // Captures like "spell" and "nospell" should produce no output
        let source = "hello world";
        let spans = vec![
            Span {
                start: 0,
                end: 5,
                capture: "spell".into(),
                pattern_index: 0,
            },
            Span {
                start: 6,
                end: 11,
                capture: "nospell".into(),
                pattern_index: 0,
            },
        ];
        let html = spans_to_html(source, spans, &HtmlFormat::CustomElements);
        // No tags should be emitted
        assert_eq!(html, "hello world");
    }

    #[test]
    fn test_simple_ansi_highlight() {
        let theme = arborium_theme::theme::builtin::catppuccin_mocha();
        let source = "fn main";
        let spans = vec![
            Span {
                start: 0,
                end: 2,
                capture: "keyword".into(),
                pattern_index: 0,
            },
            Span {
                start: 3,
                end: 7,
                capture: "function".into(),
                pattern_index: 0,
            },
        ];

        let kw_idx = slot_to_highlight_index(capture_to_slot("keyword")).unwrap();
        let fn_idx = slot_to_highlight_index(capture_to_slot("function")).unwrap();

        let ansi = spans_to_ansi(source, spans, &theme);

        let expected = format!(
            "{}fn{} {}main{}",
            theme.ansi_style(kw_idx),
            Theme::ANSI_RESET,
            theme.ansi_style(fn_idx),
            Theme::ANSI_RESET
        );
        assert_eq!(ansi, expected);
    }

    #[test]
    fn test_ansi_with_base_background() {
        let theme = arborium_theme::theme::builtin::tokyo_night();
        let source = "fn";
        let spans = vec![Span {
            start: 0,
            end: 2,
            capture: "keyword".into(),
            pattern_index: 0,
        }];

        let mut options = AnsiOptions::default();
        options.use_theme_base_style = true;

        let ansi = spans_to_ansi_with_options(source, spans, &theme, &options);
        let base = theme.ansi_base_style();

        assert!(ansi.starts_with(&base));
        assert!(ansi.ends_with(Theme::ANSI_RESET));
    }

    #[test]
    fn test_ansi_wrapping_inserts_newline() {
        let theme = arborium_theme::theme::builtin::dracula();
        // Source must be longer than MIN_CONTENT_WIDTH (10) to trigger wrapping
        let source = "abcdefghijklmnop";
        let spans = vec![Span {
            start: 0,
            end: source.len() as u32,
            capture: "string".into(),
            pattern_index: 0,
        }];

        let mut options = AnsiOptions::default();
        options.use_theme_base_style = true;
        options.width = Some(12); // Must be > MIN_CONTENT_WIDTH (10) for wrapping to occur
        options.pad_to_width = false;

        let ansi = spans_to_ansi_with_options(source, spans, &theme, &options);

        assert!(
            ansi.contains('\n'),
            "Expected newline for wrapping, got: {:?}",
            ansi
        );
        assert!(ansi.ends_with(Theme::ANSI_RESET));
    }

    #[test]
    fn test_ansi_coalesces_same_style() {
        let theme = arborium_theme::theme::builtin::catppuccin_mocha();
        let source = "keyword";
        let spans = vec![
            Span {
                start: 0,
                end: 3,
                capture: "keyword".into(),
                pattern_index: 0,
            },
            Span {
                start: 3,
                end: 7,
                capture: "keyword.function".into(),
                pattern_index: 0,
            },
        ];

        let kw_idx = slot_to_highlight_index(capture_to_slot("keyword")).unwrap();
        let ansi = spans_to_ansi(source, spans, &theme);

        let expected = format!("{}keyword{}", theme.ansi_style(kw_idx), Theme::ANSI_RESET);
        assert_eq!(ansi, expected);
    }

    #[test]
    fn test_comment_spell_dedupe() {
        // When a node has @comment @spell, both produce spans with the same range.
        // The @spell should NOT overwrite @comment - we should keep @comment.
        let source = "# a comment";
        let spans = vec![
            Span {
                start: 0,
                end: 11,
                capture: "comment".into(),
                pattern_index: 0,
            },
            Span {
                start: 0,
                end: 11,
                capture: "spell".into(),
                pattern_index: 0,
            },
        ];
        let html = spans_to_html(source, spans, &HtmlFormat::CustomElements);
        // Should have comment styling, not be unstyled
        assert_eq!(html, "<a-c># a comment</a-c>");
    }

    #[test]
    fn test_html_format_custom_elements() {
        let source = "fn main";
        let spans = vec![
            Span {
                start: 0,
                end: 2,
                capture: "keyword".into(),
                pattern_index: 0,
            },
            Span {
                start: 3,
                end: 7,
                capture: "function".into(),
                pattern_index: 0,
            },
        ];
        let html = spans_to_html(source, spans, &HtmlFormat::CustomElements);
        assert_eq!(html, "<a-k>fn</a-k> <a-f>main</a-f>");
    }

    #[test]
    fn test_html_format_custom_elements_with_prefix() {
        let source = "fn main";
        let spans = vec![
            Span {
                start: 0,
                end: 2,
                capture: "keyword".into(),
                pattern_index: 0,
            },
            Span {
                start: 3,
                end: 7,
                capture: "function".into(),
                pattern_index: 0,
            },
        ];
        let html = spans_to_html(
            source,
            spans,
            &HtmlFormat::CustomElementsWithPrefix("code".to_string()),
        );
        assert_eq!(html, "<code-k>fn</code-k> <code-f>main</code-f>");
    }

    #[test]
    fn test_html_format_class_names() {
        let source = "fn main";
        let spans = vec![
            Span {
                start: 0,
                end: 2,
                capture: "keyword".into(),
                pattern_index: 0,
            },
            Span {
                start: 3,
                end: 7,
                capture: "function".into(),
                pattern_index: 0,
            },
        ];
        let html = spans_to_html(source, spans, &HtmlFormat::ClassNames);
        assert_eq!(
            html,
            "<span class=\"keyword\">fn</span> <span class=\"function\">main</span>"
        );
    }

    #[test]
    fn test_html_format_class_names_with_prefix() {
        let source = "fn main";
        let spans = vec![
            Span {
                start: 0,
                end: 2,
                capture: "keyword".into(),
                pattern_index: 0,
            },
            Span {
                start: 3,
                end: 7,
                capture: "function".into(),
                pattern_index: 0,
            },
        ];
        let html = spans_to_html(
            source,
            spans,
            &HtmlFormat::ClassNamesWithPrefix("arb".to_string()),
        );
        assert_eq!(
            html,
            "<span class=\"arb-keyword\">fn</span> <span class=\"arb-function\">main</span>"
        );
    }

    #[test]
    fn test_html_format_all_tags() {
        // Test a variety of different tags to ensure mapping works
        let source = "kfsctvcopprattgmlnscrttstemdadder";
        let mut offset = 0;
        let mut spans = vec![];
        let tags = [
            ("k", "keyword", "keyword"),
            ("f", "function", "function"),
            ("s", "string", "string"),
            ("c", "comment", "comment"),
            ("t", "type", "type"),
            ("v", "variable", "variable"),
            ("co", "constant", "constant"),
            ("p", "punctuation", "punctuation"),
            ("pr", "property", "property"),
            ("at", "attribute", "attribute"),
            ("tg", "tag", "tag"),
            ("m", "macro", "macro"),
            ("l", "label", "label"),
            ("ns", "namespace", "namespace"),
            ("cr", "constructor", "constructor"),
            ("tt", "text.title", "title"),
            ("st", "text.strong", "strong"),
            ("em", "text.emphasis", "emphasis"),
            ("da", "diff.addition", "diff-add"),
            ("dd", "diff.deletion", "diff-delete"),
            ("er", "error", "error"),
        ];

        for (tag, capture_name, _class_name) in &tags {
            let len = tag.len() as u32;
            spans.push(Span {
                start: offset,
                end: offset + len,
                capture: capture_name.to_string(),
                pattern_index: 0,
            });
            offset += len;
        }

        // Test ClassNames format
        let html = spans_to_html(source, spans.clone(), &HtmlFormat::ClassNames);
        for (_tag, _capture, class_name) in &tags {
            assert!(
                html.contains(&format!("class=\"{}\"", class_name)),
                "Missing class=\"{}\" in output: {}",
                class_name,
                html
            );
        }
    }
}

#[cfg(test)]
mod html_tests {
    use super::*;
    use crate::Span;

    #[test]
    fn test_spans_to_html_cpp_sample() {
        let sample = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../demo/samples/cpp.cc"
        ))
        .expect("Failed to read cpp sample");

        // Create some fake spans that cover the whole file
        let spans = vec![
            Span {
                start: 0,
                end: 10,
                capture: "comment".into(),
                pattern_index: 0,
            },
            Span {
                start: 100,
                end: 110,
                capture: "keyword".into(),
                pattern_index: 0,
            },
        ];

        // This should not panic
        let html = spans_to_html(&sample, spans, &HtmlFormat::default());
        assert!(!html.is_empty());
    }

    #[test]
    fn test_spans_to_html_real_cpp_grammar() {
        use crate::{CompiledGrammar, GrammarConfig, ParseContext};

        let sample = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../demo/samples/cpp.cc"
        ))
        .expect("Failed to read cpp sample");

        // Load the actual cpp grammar
        let config = GrammarConfig {
            language: arborium_cpp::language().into(),
            highlights_query: &arborium_cpp::HIGHLIGHTS_QUERY,
            injections_query: arborium_cpp::INJECTIONS_QUERY,
            locals_query: "",
        };

        let grammar = CompiledGrammar::new(config).expect("Failed to compile grammar");
        let mut ctx = ParseContext::for_grammar(&grammar).expect("Failed to create context");

        // Parse the sample
        let result = grammar.parse(&mut ctx, &sample);

        println!("Got {} spans from parsing", result.spans.len());

        // Check some spans for validity
        for (i, span) in result.spans.iter().enumerate().take(20) {
            println!(
                "Span {}: {}..{} {:?}",
                i, span.start, span.end, span.capture
            );
            let start = span.start as usize;
            let end = span.end as usize;
            assert!(
                start <= sample.len(),
                "Span {} start {} > len {}",
                i,
                start,
                sample.len()
            );
            assert!(
                end <= sample.len(),
                "Span {} end {} > len {}",
                i,
                end,
                sample.len()
            );
            assert!(
                sample.is_char_boundary(start),
                "Span {} start {} not char boundary",
                i,
                start
            );
            assert!(
                sample.is_char_boundary(end),
                "Span {} end {} not char boundary",
                i,
                end
            );
        }

        // Now try to render - this should not panic
        let html = spans_to_html(&sample, result.spans, &HtmlFormat::default());
        assert!(!html.is_empty());
        println!("Generated {} bytes of HTML", html.len());
    }

    /// Test that pattern_index deduplication works correctly.
    ///
    /// This simulates what the plugin runtime returns: two spans covering the same
    /// text ("name") with different captures (@string and @property) and different
    /// pattern indices. The higher pattern_index should win.
    #[test]
    fn test_pattern_index_deduplication() {
        let source = "name value";

        // Simulate plugin runtime output: both @string and @property cover "name"
        // @property has higher pattern_index (11) than @string (7)
        let spans = vec![
            Span {
                start: 0,
                end: 4,
                capture: "string".into(),
                pattern_index: 7,
            },
            Span {
                start: 0,
                end: 4,
                capture: "property".into(),
                pattern_index: 11,
            },
            Span {
                start: 5,
                end: 10,
                capture: "string".into(),
                pattern_index: 7,
            },
        ];

        let html = spans_to_html(source, spans, &HtmlFormat::CustomElements);

        eprintln!("Generated HTML: {}", html);

        // "name" should be rendered as property (a-pr), not string (a-s)
        // because property has higher pattern_index
        assert!(
            html.contains("<a-pr>name</a-pr>"),
            "Expected 'name' to be rendered as <a-pr> (property), got: {}",
            html
        );

        // "value" should be rendered as string (a-s)
        assert!(
            html.contains("<a-s>value</a-s>"),
            "Expected 'value' to be rendered as <a-s> (string), got: {}",
            html
        );
    }

    /// Test that pattern_index deduplication works when @string has higher index.
    /// This is the opposite case - verifying the logic works both ways.
    #[test]
    fn test_pattern_index_deduplication_string_wins() {
        let source = "name";

        // @string has higher pattern_index (11) than @property (7)
        let spans = vec![
            Span {
                start: 0,
                end: 4,
                capture: "property".into(),
                pattern_index: 7,
            },
            Span {
                start: 0,
                end: 4,
                capture: "string".into(),
                pattern_index: 11,
            },
        ];

        let html = spans_to_html(source, spans, &HtmlFormat::CustomElements);

        eprintln!("Generated HTML: {}", html);

        // "name" should be rendered as string (a-s) because it has higher pattern_index
        assert!(
            html.contains("<a-s>name</a-s>"),
            "Expected 'name' to be rendered as <a-s> (string), got: {}",
            html
        );
    }
}
