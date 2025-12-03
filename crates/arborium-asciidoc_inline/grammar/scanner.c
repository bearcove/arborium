#include "include/base_types.h"
#include "tree_sitter/parser.h"

typedef enum TokenType {
    TOKEN_TYPE_EOF,
} TokenType;

void *tree_sitter_asciidoc_inline_external_scanner_create() {
    return NULL;
}

void tree_sitter_asciidoc_inline_external_scanner_destroy(void *payload) {
    // ...
}

unsigned tree_sitter_asciidoc_inline_external_scanner_serialize(void *payload, char *buffer) {
    return 0;
}

void tree_sitter_asciidoc_inline_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
    // ...
}

bool tree_sitter_asciidoc_inline_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
    if(lexer->eof(lexer)) {
        if(valid_symbols[TOKEN_TYPE_EOF]) {
            lexer->result_symbol = TOKEN_TYPE_EOF;
            return true;
        }
    }

    return false;
}
