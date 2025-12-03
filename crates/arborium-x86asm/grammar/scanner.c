#include "tree_sitter/parser.h"

enum TokenType {
  NEWLINE,
  LINE_COMMENT,
};

void *tree_sitter_x86asm_external_scanner_create(void) {
  return NULL;
}

void tree_sitter_x86asm_external_scanner_destroy(void *payload) {
}

unsigned tree_sitter_x86asm_external_scanner_serialize(void *payload, char *buffer) {
  return 0;
}

void tree_sitter_x86asm_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
}

static void advance(TSLexer *lexer) {
  lexer->advance(lexer, false);
}

static void skip(TSLexer *lexer) {
  lexer->advance(lexer, true);
}

bool tree_sitter_x86asm_external_scanner_scan(
  void *payload,
  TSLexer *lexer,
  const bool *valid_symbols
) {
  // Skip whitespace
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
    skip(lexer);
  }

  // Check for comment
  if (valid_symbols[LINE_COMMENT]) {
    if (lexer->lookahead == ';' || lexer->lookahead == '#') {
      advance(lexer);
      while (lexer->lookahead != '\n' && lexer->lookahead != '\r' && lexer->lookahead != 0) {
        advance(lexer);
      }
      lexer->result_symbol = LINE_COMMENT;
      return true;
    }
    if (lexer->lookahead == '/') {
      advance(lexer);
      if (lexer->lookahead == '/') {
        advance(lexer);
        while (lexer->lookahead != '\n' && lexer->lookahead != '\r' && lexer->lookahead != 0) {
          advance(lexer);
        }
        lexer->result_symbol = LINE_COMMENT;
        return true;
      }
      // Not a comment, don't consume
      return false;
    }
  }

  // Check for newline
  if (valid_symbols[NEWLINE]) {
    if (lexer->lookahead == '\n') {
      advance(lexer);
      lexer->result_symbol = NEWLINE;
      return true;
    }
    if (lexer->lookahead == '\r') {
      advance(lexer);
      if (lexer->lookahead == '\n') {
        advance(lexer);
      }
      lexer->result_symbol = NEWLINE;
      return true;
    }
  }

  return false;
}
