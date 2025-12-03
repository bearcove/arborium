#include <tree_sitter/parser.h>

enum TokenType {
  NEWLINE,
};

static void advance(TSLexer *lexer) {
  lexer->advance(lexer, false);
}

static void skip(TSLexer *lexer) {
  lexer->advance(lexer, true);
}

static bool scan(TSLexer *lexer, const bool *valid_symbols) {
  if (valid_symbols[NEWLINE]) {
    if (lexer->lookahead == '\n' || lexer->lookahead == 0) {
      skip(lexer);
      lexer->result_symbol = NEWLINE;
      return true;
    }
  }
  return false;
}

void *tree_sitter_lean_external_scanner_create() {
  return NULL;
}

bool tree_sitter_lean_external_scanner_scan(void *payload, TSLexer *lexer,
                                            const bool *valid_symbols) {
  return scan(lexer, valid_symbols);
}

unsigned tree_sitter_lean_external_scanner_serialize(void *payload,
                                                     char *buffer) {
  return 0;
}

void tree_sitter_lean_external_scanner_deserialize(void *payload,
                                                   const char *buffer,
                                                   unsigned length) {}

void tree_sitter_lean_external_scanner_destroy(void *payload) {}
