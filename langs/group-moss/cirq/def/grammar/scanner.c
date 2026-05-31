#include "tree_sitter/parser.h"

// External scanner for the cirq grammar.
//
// Scans a single token type: `code_body`, the raw verbatim text between
// the braces of `code "lang" { ... }`.
//
// The body may contain arbitrary embedded-language code, including nested
// `{` / `}` and string-/comment-/template-like constructs whose contents
// must not affect the outer brace counter. This scanner walks the body
// character-by-character so the body extends exactly to the matching `}`
// of the `code` block.
//
// ─────────────────────────────────────────────────────────────────────
// LANGUAGES SUPPORTED
// ─────────────────────────────────────────────────────────────────────
//
// First-class support is for the three languages the cirq surface is
// designed to host:
//
//   1. javascript — `//` line comments, `/* */` block comments, `"..."`
//      and `'...'` strings with backslash escapes, and template literals
//      `` `...${ expr }...` `` including nested templates inside the
//      interpolation.
//
//   2. bash — `#` line comments (only at the first non-whitespace
//      position of a line, so JS private fields `#name` aren't mis-
//      treated), `"..."` and `'...'` strings.
//
//   3. control (ngspice .control) — `*` line comments at the first
//      non-whitespace position of a line, and `"..."` strings.
//
// The scanner is universal across the three: every special-case rule is
// either disjoint between the languages or guarded so a feature of one
// language doesn't trip in another. The cases not handled are listed
// below; they fall back to "consume as ordinary character" and so a `}`
// inside one of them will still prematurely terminate the block.
//
// ─────────────────────────────────────────────────────────────────────
// KNOWN LIMITATIONS
// ─────────────────────────────────────────────────────────────────────
//
//   1. Bash heredocs (`<<EOF ... EOF`, `<<-EOF ...`, `<<"EOF" ...`).
//      Bodies frequently embed JSON/YAML and so contain `}`. Users who
//      need a heredoc with literal `}` inside should write the script
//      to an external file and `source` it from the code block instead.
//
//   2. Python triple-quoted strings, Rust/C++ raw strings, Lua long
//      brackets — none of the three primary languages need these.
//
//   3. JS regex literals (`/[}]/` in a character class). Distinguishing
//      regex from division is context-sensitive and would require a
//      mini-parser; skipped on the grounds that regexes containing `}`
//      in character classes are rare in practice.
//
//   4. JS private fields used at the first non-whitespace position of a
//      line with no leading dot — e.g. `#priv = 1` at the very start of
//      a class body line. The line-comment guard already excludes
//      `#[a-zA-Z_]` to avoid this, but `#0` or `#$` at SOL would still
//      be misread; both are syntax errors in JS so the resulting parse
//      error is no worse than the source's.
//
// ─────────────────────────────────────────────────────────────────────

#include <stdbool.h>
#include <stdint.h>

enum TokenType {
    CODE_BODY,
};

void *tree_sitter_cirq_external_scanner_create(void) {
    return NULL;
}

void tree_sitter_cirq_external_scanner_destroy(void *payload) {
    (void)payload;
}

unsigned tree_sitter_cirq_external_scanner_serialize(void *payload, char *buffer) {
    (void)payload;
    (void)buffer;
    return 0;
}

void tree_sitter_cirq_external_scanner_deserialize(
    void *payload, const char *buffer, unsigned length
) {
    (void)payload;
    (void)buffer;
    (void)length;
}

static inline bool is_id_start(int32_t c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
}

// Consume characters of a `"..."` or `'...'` string up to and including
// the closing quote, honoring backslash escapes. Assumes the opening
// quote is the current lookahead.
//
// Bash single-quoted strings don't actually honour `\`, but the only
// place that matters is `'\''`, which contains no braces. Treating it
// as an escape across the board is safe for the brace counter.
static void consume_string(TSLexer *lexer) {
    int32_t quote = lexer->lookahead;
    lexer->advance(lexer, false);

    while (!lexer->eof(lexer)) {
        int32_t c = lexer->lookahead;

        if (c == '\\') {
            lexer->advance(lexer, false);
            if (!lexer->eof(lexer)) {
                lexer->advance(lexer, false);
            }
            continue;
        }

        if (c == quote) {
            lexer->advance(lexer, false);
            return;
        }

        lexer->advance(lexer, false);
    }
}

// Forward declaration — template literals can nest through `${...}`.
static void consume_template_literal(TSLexer *lexer);

// Skip the interpolated `${...}` expression of a JS template literal,
// starting from the position immediately after the `${`. Balances `{`
// and `}` inside, and recurses through nested strings and template
// literals so `${ \`inner ${x}\` }` works.
static void consume_template_interpolation(TSLexer *lexer) {
    int depth = 1;
    while (!lexer->eof(lexer) && depth > 0) {
        int32_t c = lexer->lookahead;
        if (c == '"' || c == '\'') {
            consume_string(lexer);
            continue;
        }
        if (c == '`') {
            consume_template_literal(lexer);
            continue;
        }
        if (c == '{') {
            depth++;
            lexer->advance(lexer, false);
            continue;
        }
        if (c == '}') {
            depth--;
            lexer->advance(lexer, false);
            continue;
        }
        lexer->advance(lexer, false);
    }
}

// Consume a JS template literal: `...${expr}...` Assumes the opening
// backtick is the current lookahead.
static void consume_template_literal(TSLexer *lexer) {
    lexer->advance(lexer, false);  // opening `

    while (!lexer->eof(lexer)) {
        int32_t c = lexer->lookahead;

        if (c == '\\') {
            lexer->advance(lexer, false);
            if (!lexer->eof(lexer)) {
                lexer->advance(lexer, false);
            }
            continue;
        }

        if (c == '`') {
            lexer->advance(lexer, false);
            return;
        }

        if (c == '$') {
            lexer->advance(lexer, false);
            if (lexer->lookahead == '{') {
                lexer->advance(lexer, false);  // past `{`
                consume_template_interpolation(lexer);
            }
            continue;
        }

        lexer->advance(lexer, false);
    }
}

// Advance past the rest of the current line. Stops at (but does not
// consume) the terminating `\n`, so the main loop can observe the
// newline and update the start-of-line flag.
static void skip_to_eol(TSLexer *lexer) {
    while (!lexer->eof(lexer) && lexer->lookahead != '\n') {
        lexer->advance(lexer, false);
    }
}

// Skip from inside a `/*` block comment to past the closing `*/`. The
// caller has already consumed the opening `/*`.
static void skip_block_comment(TSLexer *lexer) {
    while (!lexer->eof(lexer)) {
        int32_t c = lexer->lookahead;
        lexer->advance(lexer, false);
        if (c == '*' && lexer->lookahead == '/') {
            lexer->advance(lexer, false);
            return;
        }
    }
}

bool tree_sitter_cirq_external_scanner_scan(
    void *payload, TSLexer *lexer, const bool *valid_symbols
) {
    (void)payload;

    if (!valid_symbols[CODE_BODY]) {
        return false;
    }

    // Skip leading whitespace without including it in the token. We don't
    // want the body to start with a run of spaces/newlines, because the
    // grammar's `extras` would normally handle them at this position;
    // refusing to consume them here lets the parser apply extras and
    // call us back at the first non-whitespace character.
    while (!lexer->eof(lexer)) {
        int32_t c = lexer->lookahead;
        if (c == ' ' || c == '\t' || c == '\n' || c == '\r') {
            lexer->advance(lexer, true);
        } else {
            break;
        }
    }

    // If the next character is the closing brace of the `code` block at
    // depth 0, the body is empty. Refuse the token so that the optional()
    // in the grammar lets the outer `}` close the block directly.
    if (lexer->eof(lexer) || lexer->lookahead == '}') {
        return false;
    }

    int depth = 0;
    bool consumed_any = false;
    bool at_sol = true;  // first non-whitespace position of the line

    while (!lexer->eof(lexer)) {
        int32_t c = lexer->lookahead;

        if (c == '}' && depth == 0) {
            // Reached the closing brace of the `code` block. Stop without
            // consuming it — the outer grammar will match the `}` token.
            break;
        }

        consumed_any = true;

        // Newline / whitespace handling has to come before the at_sol
        // reset so that runs of `   #comment` and `\t* comment` still
        // count as starting-of-line. Tabs/spaces leave at_sol alone.
        if (c == '\n') {
            at_sol = true;
            lexer->advance(lexer, false);
            continue;
        }
        if (c == ' ' || c == '\t' || c == '\r') {
            lexer->advance(lexer, false);
            continue;
        }

        bool was_at_sol = at_sol;
        at_sol = false;

        switch (c) {
            case '"':
            case '\'':
                consume_string(lexer);
                break;

            case '`':
                consume_template_literal(lexer);
                break;

            case '/': {
                lexer->advance(lexer, false);
                int32_t next = lexer->lookahead;
                if (next == '/') {
                    skip_to_eol(lexer);
                } else if (next == '*') {
                    lexer->advance(lexer, false);
                    skip_block_comment(lexer);
                }
                // else: a plain `/`. Already consumed.
                break;
            }

            case '#': {
                if (was_at_sol) {
                    lexer->advance(lexer, false);
                    // Guard JS private-field syntax: `#priv` at SOL is
                    // not a comment, it's an identifier. Anything else
                    // (`#!`, `# `, `##`, `#0`) treats the rest of the
                    // line as a comment.
                    if (!is_id_start(lexer->lookahead)) {
                        skip_to_eol(lexer);
                    }
                } else {
                    lexer->advance(lexer, false);
                }
                break;
            }

            case '*': {
                if (was_at_sol) {
                    // ngspice .control line comment. Consume from `*`
                    // through the rest of the line.
                    skip_to_eol(lexer);
                } else {
                    lexer->advance(lexer, false);
                }
                break;
            }

            case '{':
                depth++;
                lexer->advance(lexer, false);
                break;

            case '}':
                // depth > 0 here; this `}` closes a nested block inside
                // the embedded language.
                depth--;
                lexer->advance(lexer, false);
                break;

            default:
                lexer->advance(lexer, false);
                break;
        }
    }

    if (!consumed_any) {
        return false;
    }

    lexer->result_symbol = CODE_BODY;
    return true;
}
