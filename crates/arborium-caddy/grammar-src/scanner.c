#include "tree_sitter/parser.h"
#include "wctype.h"
#include <ctype.h>

enum { EOF };

void *tree_sitter_caddy_external_scanner_create() { return NULL; }

void tree_sitter_caddy_external_scanner_destroy(void *payload) {}

unsigned tree_sitter_caddy_external_scanner_serialize(void *payload,
                                                    char *buffer) {
  return 0;
}

void tree_sitter_caddy_external_scanner_deserialize(void *payload,
                                                  const char *buffer,
                                                  unsigned length) {}

bool tree_sitter_caddy_external_scanner_scan(void *payload, TSLexer *lexer,
                                           const bool *valid_symbols) {
  // check for eof
  if (lexer->lookahead == 0) {
    lexer->result_symbol = EOF;
    lexer->advance(lexer, true);
    return true;
  }

  return false;
}

