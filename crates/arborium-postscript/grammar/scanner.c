/**************************************************************************
 *
 * Copyright (c) 2024, 2025 Stefan Möding
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *
 **************************************************************************/


#include <stdlib.h>
#include <string.h>
#include <ctype.h>

#include "tree_sitter/parser.h"
#include "tree_sitter/alloc.h"
#include "tree_sitter/array.h"


/**
 * The tokens that this scanner will detect. The order must be the same as
 * defined in the 'externals' field in the grammar.
 */

enum TokenType {
  DOCUMENT_STRUCTURE_COMMENT,
  LITERAL_STRING,
  HEXADECIMAL_STRING,
  BASE85_STRING,
  NUMERIC,
};


/**
 * The state of the scanner to keep track of the level of parentheses when
 * a literal string is scanned.
 */

typedef struct ScannerState {
  int paren_level;
} ScannerState;


/**
 * Skip over all following whitespace characters.
 */

static void skip_whitespace(TSLexer *lexer) {
  while ((!lexer->eof(lexer)) && (isspace(lexer->lookahead))) {
    lexer->advance(lexer, true);
  }
}


/**
 * Test for a delimiting character.
 */

static bool isdelimiter(char ch) {
  switch (ch) {
  case U'%':
  case U'/':
  case U'(':
  case U')':
  case U'[':
  case U']':
  case U'{':
  case U'}':
  case U'<':
  case U'>':
    return true;
  default:
    return isspace(ch);
  }
}

/**
 * Scan a structure comment.
 */

static bool document_structure_comment(TSLexer *lexer) {
  bool found_dsc = false;

  skip_whitespace(lexer);

  // We are done if the end of file is reached
  if (lexer->eof(lexer)) return false;

  // Structure comments can only start at the beginning of a line.
  if ((lexer->lookahead == U'%') && (lexer->get_column(lexer) == 0)) {
    lexer->advance(lexer, false);

    if (lexer->lookahead == U'!') {
      lexer->advance(lexer, false);
      found_dsc = true;
    }
    else if (lexer->lookahead == U'%') {
      lexer->advance(lexer, false);
      found_dsc = (lexer->lookahead == U'+') ||
                  (lexer->lookahead == U'?') ||
                  (isalpha(lexer->lookahead) && isupper(lexer->lookahead));
    }

    // Skip until the end of the line is reached.
    while ((!lexer->eof(lexer)) &&
           (lexer->lookahead != U'\r') && (lexer->lookahead != U'\n')) {
      lexer->advance(lexer, false);
    }
  }

  return found_dsc;
}


/**
 * Scan for a literal string. Balanced parenthesis are not special in
 * a string so we need to keep track of them. A backslash starts an escape
 * sequence which is handled by the normal grammar.
 */

static bool literal_string(TSLexer *lexer, ScannerState *state) {
  for(bool has_content=false;; has_content=true) {
    // We are done if the end of file is reached
    if (lexer->eof(lexer)) return false;

    switch (lexer->lookahead) {
    case U'\\':
      return has_content;
      break;

    case U'(':
      state->paren_level++;  // balanced parenthesis are allowed in a string
      break;

    case U')':
      if (state->paren_level == 0) return has_content;
      state->paren_level--;
      break;

    default:
      break;
    }

    lexer->advance(lexer, false);
  }
}


/**
 * Scan for a hexadecimal string.
 */

static bool hexadecimal_string(TSLexer *lexer) {
  bool has_content = false;

  skip_whitespace(lexer);

  while ((!lexer->eof(lexer)) && (isxdigit(lexer->lookahead))) {
    lexer->advance(lexer, false);
    has_content = true;
  }

  return has_content;
}


/**
 * Scan for a Base85/ASCII85 string. The valid code points go from 33 (!) to
 * 117 (u) and also 122 (z) indicating four zeros.
 */

static bool base85_string(TSLexer *lexer) {

  skip_whitespace(lexer);

  for(bool has_content=false;; has_content=true) {
    // We are done if the end of file is reached
    if (lexer->eof(lexer)) return false;

    if ((lexer->lookahead != 122) &&
        ((lexer->lookahead < 33) || (lexer->lookahead > 117)))
      return has_content;

    lexer->advance(lexer, false);
  }
}


/**
 * Scan for a radix number.
 */

static bool numeric(TSLexer *lexer) {
  bool has_sign = false;
  bool valid_number = true;
  bool valid_radix = false;
  bool valid_exponent = false;
  unsigned int base = 0;

  typedef enum NumericState {
    SIGN,
    INTEGER,
    RADIX,
    REAL,
    MAGNITUDE,
    EXPONENT,
  } NumericState;

  skip_whitespace(lexer);

  // We are done if the end of file is reached
  if (lexer->eof(lexer)) return false;

  for(NumericState state=SIGN;;) {
    switch (state) {
    case SIGN:
      if ((lexer->lookahead == U'+') || (lexer->lookahead == U'-')) {
        lexer->advance(lexer, false);
        has_sign = true;
      }
      else if (isdigit(lexer->lookahead)) {
        state = INTEGER;
      }
      else if (lexer->lookahead == U'.') {
        lexer->advance(lexer, false);
        state = REAL;
      }
      else {
        return false;
      }
      break;

    case INTEGER:
      if ((lexer->eof(lexer)) || (isdelimiter(lexer->lookahead))) {
        return valid_number;
      }
      else if (isdigit(lexer->lookahead)) {
        if (base <= 36) {
          base *= 10;
          base += (lexer->lookahead - U'0');
        }

        lexer->advance(lexer, false);
        valid_number = true;
      }
      else if (lexer->lookahead == U'#') {
        lexer->advance(lexer, false);
        state = RADIX;
      }
      else if (lexer->lookahead == U'.') {
        lexer->advance(lexer, false);
        state = REAL;
      }
      else if (tolower(lexer->lookahead) == U'e') {
        lexer->advance(lexer, false);
        state = MAGNITUDE;
      }
      else {
        return false;
      }
      break;

    case RADIX:
      if ((base < 2) || (base > 36) || has_sign) return false;

      if ((lexer->eof(lexer)) || (isdelimiter(lexer->lookahead))) {
        return valid_radix;
      }
      else if (isdigit(lexer->lookahead)) {
        // Check if the character value is valid wrt the base
        if ((lexer->lookahead - U'0') > base) return false;

        lexer->advance(lexer, false);
        valid_radix = true;
      }
      else if (isalpha(lexer->lookahead)) {
        // Check if the character value is valid wrt the base
        if ((tolower(lexer->lookahead) - U'a' + 10) > base) return false;

        lexer->advance(lexer, false);
        valid_radix = true;
      }
      else {
        return false;
      }
      break;

    case REAL:
      if ((lexer->eof(lexer)) || (isdelimiter(lexer->lookahead))) {
        return valid_number;
      }
      else if (isdigit(lexer->lookahead)) {
        lexer->advance(lexer, false);
        valid_number = true;
      }
      else if (tolower(lexer->lookahead) == U'e') {
        lexer->advance(lexer, false);
        state = MAGNITUDE;
      }
      else {
        return false;
      }
      break;

    case MAGNITUDE:
      if ((lexer->lookahead == U'+') || (lexer->lookahead == U'-')) {
        lexer->advance(lexer, false);
      }
      state = EXPONENT;
      // fall thru

    case EXPONENT:
      if ((lexer->eof(lexer)) || (isdelimiter(lexer->lookahead))) {
        return valid_number && valid_exponent;
      }
      else if (isdigit(lexer->lookahead)) {
        lexer->advance(lexer, false);
        valid_exponent = true;
      }
      else {
        return false;
      }
      break;
    }
  }
}


/**
 * The public interface used by the tree-sitter parser
 */

void *tree_sitter_postscript_external_scanner_create() {
  ScannerState *state = ts_malloc(sizeof(ScannerState));

  if (state) {
    state->paren_level = 0;
  }

  return state;
}

void tree_sitter_postscript_external_scanner_destroy(void *payload) {
  ScannerState *state = (ScannerState*)payload;

  if (state) {
    ts_free(state);
  }
}

unsigned tree_sitter_postscript_external_scanner_serialize(void *payload, char *buffer) {
  ScannerState *state = (ScannerState*)payload;
  unsigned length = 0;  // total size of the serialized data in bytes
  unsigned objsiz;      // size of the current object to serialize

  // Serialize the scanner state. To prevent serialization of each struct
  // item we also include the bogus pointer to the array contents. The
  // pointer will be reset when the object is deserialized (see below).
  objsiz = sizeof(*state);
  memcpy(buffer, payload, objsiz);
  buffer += objsiz;
  length += objsiz;

  return length;
}

void tree_sitter_postscript_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
  ScannerState *state = (ScannerState*)payload;
  unsigned objsiz;      // size of the current object to deserialize

  // Initialize the structure since the deserialization function will
  // sometimes also be called with length set to zero.
  state->paren_level = 0;

  // Deserialize the scanner state.
  if (length >= sizeof(ScannerState)) {
    objsiz = sizeof(ScannerState);
    memcpy(payload, buffer, objsiz);
    buffer += objsiz;
    length -= objsiz;
  }
}

bool tree_sitter_postscript_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
  ScannerState *state = (ScannerState*)payload;

  if (valid_symbols[DOCUMENT_STRUCTURE_COMMENT]) {
    if (document_structure_comment(lexer)) {
      lexer->result_symbol = DOCUMENT_STRUCTURE_COMMENT;
      return true;
    }
  }

  if (valid_symbols[LITERAL_STRING]) {
    if (literal_string(lexer, state)) {
      lexer->result_symbol = LITERAL_STRING;
      return true;
    }
  }

  if (valid_symbols[HEXADECIMAL_STRING]) {
    if (hexadecimal_string(lexer)) {
      lexer->result_symbol = HEXADECIMAL_STRING;
      return true;
    }
  }

  if (valid_symbols[BASE85_STRING]) {
    if (base85_string(lexer)) {
      lexer->result_symbol = BASE85_STRING;
      return true;
    }
  }

  if (valid_symbols[NUMERIC]) {
    if (numeric(lexer)) {
      lexer->result_symbol = NUMERIC;
      return true;
    }
  }

  return false;
}
