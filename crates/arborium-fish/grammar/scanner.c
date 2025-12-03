#include <tree_sitter/parser.h>
#include <wctype.h>

enum TokenType {
    CONCAT,
    BRACKET_CONCAT,
    CONCAT_LIST
};

void *tree_sitter_fish_external_scanner_create() { return NULL; }
void tree_sitter_fish_external_scanner_destroy(void *p) {}
void tree_sitter_fish_external_scanner_reset(void *p) {}
unsigned tree_sitter_fish_external_scanner_serialize(void *p, char *buffer) { return 0; }
void tree_sitter_fish_external_scanner_deserialize(void *p, const char *b, unsigned n) {}

bool tree_sitter_fish_external_scanner_scan(
    void *payload, TSLexer *lexer, const bool *valid_symbols
) {
    if (valid_symbols[CONCAT_LIST]) {
        if (!(
            lexer->lookahead == 0 ||
            lexer->lookahead != '['
        )) {
            lexer->result_symbol = CONCAT_LIST;
            return true;
        }
    }
    
    if (valid_symbols[CONCAT]) {
        if (!(
            lexer->lookahead == 0 ||
            lexer->lookahead == '>' ||
            lexer->lookahead == '<' ||
            lexer->lookahead == ')' ||
            lexer->lookahead == ';' ||
            lexer->lookahead == '&' ||
            lexer->lookahead == '|' ||
            iswspace(lexer->lookahead)
        )) {
            lexer->result_symbol = CONCAT;
            return true;
        }
    }

    if (valid_symbols[BRACKET_CONCAT]) {
        if (!(
            lexer->lookahead == 0 ||
            lexer->lookahead == ')' ||
            lexer->lookahead == '(' ||
            lexer->lookahead == '}' ||
            lexer->lookahead == ',' ||
            iswspace(lexer->lookahead)
        )) {
            lexer->result_symbol = BRACKET_CONCAT;
            return true;
        }
    }

    return false;
}
