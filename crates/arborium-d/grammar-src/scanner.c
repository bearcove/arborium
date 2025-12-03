// =============================================================================
// ARBORIUM FORK: This file is a modified version of the original scanner.cc
// from https://github.com/CyberShadow/tree-sitter-d
//
// Changes made for WASM compatibility:
// - Converted from C++ to C (original used std::vector)
// - Replaced std::vector<int32_t> with a fixed-size stack buffer
// - The delimiter buffer is limited to 256 characters, which should be
//   sufficient for any reasonable D delimited string identifier
//
// Original file: scanner.cc (C++)
// Original license: BSL-1.0 (Boost Software License)
// =============================================================================

#include <tree_sitter/parser.h>
#include <stdlib.h>
#include <stdbool.h>
#include <stdint.h>

enum TokenType {
  NESTING_BLOCK_COMMENT,
  DELIMITED_STRING,
};

// This is only an approximation of the exact definition.
static bool is_identifier_char(int32_t c) {
  return
    (c >= 'a' && c <= 'z') ||
    (c >= 'A' && c <= 'Z') ||
    (c >= '0' && c <= '9') ||
    c == '_';
}

void *tree_sitter_d_external_scanner_create(void) {
  return NULL;
}

bool tree_sitter_d_external_scanner_scan(void *payload, TSLexer *lexer,
                                         const bool *valid_symbols) {
  if (lexer->lookahead == '/' && valid_symbols[NESTING_BLOCK_COMMENT]) {
    lexer->advance(lexer, false);
    if (lexer->lookahead != '+') {
      return false;
    }
    lexer->advance(lexer, false);

    size_t depth = 1;
    int32_t last = 0;
    while (depth > 0) {
      last = lexer->lookahead;
      lexer->advance(lexer, false);
      if (last == '/' && lexer->lookahead == '+') {
        depth++;
        last = 0;
        lexer->advance(lexer, false);
      } else if (last == '+' && lexer->lookahead == '/') {
        depth--;
        last = 0;
        lexer->advance(lexer, false);
      } else if (lexer->lookahead == 0) {
        return false; // EOF
      }
    }
    lexer->result_symbol = NESTING_BLOCK_COMMENT;
    return true;
  }

  if (lexer->lookahead == 'q' && valid_symbols[DELIMITED_STRING]) {
    lexer->advance(lexer, false);
    if (lexer->lookahead != '"') {
      return false;
    }
    lexer->advance(lexer, false);
    lexer->result_symbol = DELIMITED_STRING;

    int32_t opener = lexer->lookahead, closer;
    switch (opener) {
      case '(': closer = ')'; break;
      case '[': closer = ']'; break;
      case '{': closer = '}'; break;
      case '<': closer = '>'; break;
      default:
      {
        // Handle the identifier case using a fixed-size buffer
        // Identifiers in D delimited strings are typically short
        #define MAX_DELIMITER_LEN 256
        int32_t delimiter[MAX_DELIMITER_LEN];
        size_t delimiter_len = 0;

        delimiter[delimiter_len++] = '\n';
        while (lexer->lookahead != '\n') {
          if (!is_identifier_char(lexer->lookahead))
            return false; // bad syntax or EOF
          if (delimiter_len >= MAX_DELIMITER_LEN - 1)
            return false; // delimiter too long
          delimiter[delimiter_len++] = lexer->lookahead;
          lexer->advance(lexer, false);
        }
        delimiter[delimiter_len++] = '"';

        size_t delimiter_pos = 0;
        while (true) {
          if (lexer->lookahead == 0)
            return false; // EOF
          if (delimiter_pos == delimiter_len)
            return true;
          if (lexer->lookahead == delimiter[delimiter_pos])
            delimiter_pos++;
          else
            delimiter_pos = lexer->lookahead == delimiter[0] ? 1 : 0;
          lexer->advance(lexer, false);
        }
      }
    }

    // Handle the punctuation case
    size_t depth = 1;
    while (depth > 0) {
      lexer->advance(lexer, false);
      if (lexer->lookahead == opener) {
        depth++;
      } else if (lexer->lookahead == closer) {
        depth--;
      } else if (lexer->lookahead == 0) {
        return false; // EOF
      }
    }
    lexer->advance(lexer, false); // last closer
    if (lexer->lookahead != '"')
      return false;
    lexer->advance(lexer, false); // "
    return true;
  }

  return false;
}

unsigned tree_sitter_d_external_scanner_serialize(void *payload, char *buffer) {
  return 0;
}

void tree_sitter_d_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
}

void tree_sitter_d_external_scanner_destroy(void *payload) {
}
