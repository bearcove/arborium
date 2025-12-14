#include "tree_sitter/parser.h"

enum { _EOF, MULTI_LINE_COMMENT, RAW_STRING };

void *tree_sitter_kdl_external_scanner_create() { return NULL; }

void tree_sitter_kdl_external_scanner_destroy(void *payload) {}

unsigned tree_sitter_kdl_external_scanner_serialize(void *payload, char *buffer) { return 0; }

void tree_sitter_kdl_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {}

static void advance(TSLexer *lexer) { lexer->advance(lexer, false); }

static bool is_newline(int32_t c) {
    switch (c) {
        case '\r':
        case '\n':
        case 0x0085: // NEL
        case 0x000C: // FF
        case 0x2028: // LS
        case 0x2029: // PS
            return true;
        default:
            return false;
    }
}

static void consume_newline(TSLexer *lexer) {
    if (lexer->lookahead == '\r') {
        advance(lexer);
        if (lexer->lookahead == '\n') {
            advance(lexer);
        }
        return;
    }

    // LF, NEL, FF, LS, PS
    advance(lexer);
}

static bool scan_multiline_comment(TSLexer *lexer) {
    if (lexer->lookahead != '/') {
        return false;
    }
    advance(lexer);
    if (lexer->lookahead != '*') {
        return false;
    }
    advance(lexer);

    bool after_star = false;
    unsigned nesting_depth = 1;
    for (;;) {
        switch (lexer->lookahead) {
            case 0:
                return false;
            case '*':
                advance(lexer);
                after_star = true;
                break;
            case '/':
                if (after_star) {
                    advance(lexer);
                    after_star = false;
                    nesting_depth--;
                    if (nesting_depth == 0) {
                        lexer->result_symbol = MULTI_LINE_COMMENT;
                        return true;
                    }
                } else {
                    advance(lexer);
                    after_star = false;
                    if (lexer->lookahead == '*') {
                        nesting_depth++;
                        advance(lexer);
                    }
                }
                break;
            default:
                advance(lexer);
                after_star = false;
                break;
        }
    }
}

static bool try_consume_hashes(TSLexer *lexer, unsigned hashes) {
    for (unsigned i = 0; i < hashes; i++) {
        if (lexer->lookahead != '#') {
            return false;
        }
        advance(lexer);
    }
    return true;
}

static bool scan_raw_string_single_line(TSLexer *lexer, unsigned hashes) {
    // We are positioned right after the opening quote.
    for (;;) {
        if (lexer->eof(lexer) || lexer->lookahead == 0) {
            return false;
        }

        // Single-line raw strings cannot contain literal newlines.
        if (is_newline(lexer->lookahead)) {
            return false;
        }

        if (lexer->lookahead != '"') {
            advance(lexer);
            continue;
        }

        // Potential closing delimiter: `"` + hashes
        advance(lexer); // consume '"'
        if (try_consume_hashes(lexer, hashes)) {
            lexer->result_symbol = RAW_STRING;
            return true;
        }

        // Not a close; any consumed hashes are part of the body.
    }
}

static bool scan_raw_string_multi_line(TSLexer *lexer, unsigned hashes) {
    // We are positioned immediately after the opening newline following `#"""`.
    for (;;) {
        if (lexer->eof(lexer) || lexer->lookahead == 0) {
            return false;
        }

        if (lexer->lookahead != '"') {
            if (is_newline(lexer->lookahead)) {
                consume_newline(lexer);
            } else {
                advance(lexer);
            }
            continue;
        }

        // Potential closing delimiter: `"""` + hashes
        advance(lexer); // first '"'
        if (lexer->lookahead != '"') {
            continue;
        }
        advance(lexer); // second '"'
        if (lexer->lookahead != '"') {
            continue;
        }
        advance(lexer); // third '"'

        if (try_consume_hashes(lexer, hashes)) {
            lexer->result_symbol = RAW_STRING;
            return true;
        }

        // Not a close; keep scanning. Any hashes we consumed are part of the body.
    }
}

static bool scan_raw_string(TSLexer *lexer) {
    if (lexer->lookahead != '#') {
        return false;
    }

    unsigned hashes = 0;
    while (lexer->lookahead == '#') {
        hashes++;
        advance(lexer);
    }

    if (lexer->lookahead != '"') {
        return false;
    }

    // Consume opening quote.
    advance(lexer);

    // If the next character is a quote, this is either:
    // - an empty single-line raw string: `#""#` (closing quote followed by hashes), OR
    // - the start of a multi-line raw string: `#"""` newline
    if (lexer->lookahead == '"') {
        advance(lexer); // second quote

        if (lexer->lookahead != '"') {
            // Empty string must close with the same number of hashes.
            if (!try_consume_hashes(lexer, hashes)) {
                return false;
            }
            lexer->result_symbol = RAW_STRING;
            return true;
        }

        // Third quote => must be multi-line raw string start, followed by newline.
        advance(lexer);
        if (!is_newline(lexer->lookahead)) {
            return false;
        }
        consume_newline(lexer);
        return scan_raw_string_multi_line(lexer, hashes);
    }

    // Otherwise it's a non-empty single-line raw string.
    return scan_raw_string_single_line(lexer, hashes);
}

bool tree_sitter_kdl_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
    // EOF
    if (valid_symbols[_EOF] && lexer->lookahead == 0) {
        lexer->result_symbol = _EOF;
        advance(lexer);
        return true;
    }

    if (valid_symbols[RAW_STRING] && lexer->lookahead == '#') {
        return scan_raw_string(lexer);
    }

    if (valid_symbols[MULTI_LINE_COMMENT] && lexer->lookahead == '/') {
        return scan_multiline_comment(lexer);
    }

    return false;
}
