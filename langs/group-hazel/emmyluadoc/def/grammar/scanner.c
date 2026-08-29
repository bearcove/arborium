#include "tree_sitter/parser.h"
#include <wctype.h>
#include <stdbool.h>

enum TokenType {
  TEXT_LINE,
  DESCRIPTION,
};

void *tree_sitter_emmyluadoc_external_scanner_create() { return NULL; }

void tree_sitter_emmyluadoc_external_scanner_destroy(void *payload) {}

unsigned tree_sitter_emmyluadoc_external_scanner_serialize(void *payload, char *buffer) { return 0; }

void tree_sitter_emmyluadoc_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {}

static bool scan_text_line(TSLexer *lexer) {
  // First, skip any leading newlines to get to the start of a line
  while (lexer->lookahead == '\n' || lexer->lookahead == '\r') {
    lexer->advance(lexer, true);
  }

  // After skipping newlines, we should be at column 0 (start of a line).
  // If not at column 0, we're in the middle of a line -> not a text_line
  if (lexer->get_column(lexer) != 0) {
    return false;
  }

  // Now skip horizontal whitespace
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
    lexer->advance(lexer, true);
  }

  // If EOF -> not a text_line
  if (lexer->lookahead == 0) return false;

  // If another newline (empty line) -> not a text_line
  if (lexer->lookahead == '\n' || lexer->lookahead == '\r') return false;

  // Case 1: Starts with @ directly (e.g., "@class MyClass")
  // According to new rules, @ without --- prefix should be treated as text_line
  if (lexer->lookahead == '@') {
    // Consume the entire line as text_line
    while (lexer->lookahead != '\n' && lexer->lookahead != '\r' && lexer->lookahead != 0) {
      lexer->advance(lexer, false);
    }
    lexer->result_symbol = TEXT_LINE;
    lexer->mark_end(lexer);
    return true;
  }
  
  // Case 2: Starts with dash sequence (comment-like) e.g. ---something
  if (lexer->lookahead == '-') {
    int dash_count = 0;
    // Count the dashes
    while (lexer->lookahead == '-') {
      dash_count++;
      lexer->advance(lexer, true);
    }

    // Skip horizontal whitespace after dashes
    while (lexer->lookahead == ' ' || lexer->lookahead == '\t') lexer->advance(lexer, true);

    // If exactly 3 dashes followed by @ or @[ -> this is an annotation, not text_line
    if (dash_count == 3 && lexer->lookahead == '@') return false;
    
    // If exactly 3 dashes followed by | -> this is a type continuation, not text_line
    if (dash_count == 3 && lexer->lookahead == '|') return false;

    // Otherwise consume to end of line and return TEXT_LINE
    while (lexer->lookahead != '\n' && lexer->lookahead != '\r' && lexer->lookahead != 0) {
      lexer->advance(lexer, true);
    }
    lexer->mark_end(lexer);
    lexer->result_symbol = TEXT_LINE;
    return true;
  }
  
  // Case 3: Regular text line (doesn't start with - or @)
  // Consume the entire line
  while (lexer->lookahead != '\n' && lexer->lookahead != '\r' && lexer->lookahead != 0) {
    lexer->advance(lexer, false);
  }
  
  lexer->result_symbol = TEXT_LINE;
  lexer->mark_end(lexer);
  return true;
}

static bool scan_description(TSLexer *lexer) {
  // Description starts with at least one space/tab
  if (lexer->lookahead != ' ' && lexer->lookahead != '\t') {
    return false;
  }
  
  // Skip leading whitespace
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
    lexer->advance(lexer, false);
  }
  
  // If we hit EOF or newline immediately, no description
  if (lexer->lookahead == 0 || lexer->lookahead == '\n' || lexer->lookahead == '\r') {
    return false;
  }
  
  // Description cannot start with | or ,
  if (lexer->lookahead == '|' || lexer->lookahead == ',') {
    return false;
  }
  
  // Consume until comma, newline, or EOF
  // This allows description to work with comma-separated lists
  while (lexer->lookahead != ',' && 
         lexer->lookahead != '\n' && 
         lexer->lookahead != '\r' && 
         lexer->lookahead != 0) {
    lexer->advance(lexer, false);
  }
  
  lexer->mark_end(lexer);
  lexer->result_symbol = DESCRIPTION;
  return true;
}

bool tree_sitter_emmyluadoc_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
  if (valid_symbols[DESCRIPTION]) {
    return scan_description(lexer);
  }
  
  if (valid_symbols[TEXT_LINE]) {
    return scan_text_line(lexer);
  }
  
  return false;
}
