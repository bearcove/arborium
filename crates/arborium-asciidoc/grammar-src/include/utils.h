#pragma once

#include "../tree_sitter/parser.h"
#include "base_types.h"

typedef enum TokenType {
    TOKEN_TYPE_EOF,
    TOKEN_TITLE_H0_MARKER,
    TOKEN_TITLE_H1_MARKER,
    TOKEN_TITLE_H2_MARKER,
    TOKEN_TITLE_H3_MARKER,
    TOKEN_TITLE_H4_MARKER,
    TOKEN_TITLE_H5_MARKER,
    TOKEN_LIST_MARKER_STAR,
    TOKEN_LIST_MARKER_HYPHEN,
    TOKEN_LIST_MARKER_DOT,
    TOKEN_LIST_MARKER_DIGIT,
    TOKEN_LIST_MARKER_GEEK,
    TOKEN_LIST_MARKER_ALPHA,
    TOKEN_DOCUMENT_ATTR_MARKER,
    TOKEN_ELEMENT_ATTR_MARKER,
    TOKEN_BLOCK_TITLE_MARKER,
    TOKEN_BREAKS_MARKS,
    TOKEN_TABLE_BLOCK_MARKER,
    TOKEN_NTABLE_BLOCK_MARKER,
    TOKEN_CELL_ATTR,
    TOKEN_DELIMITED_BLOCK_START_MARKER,
    TOKEN_DELIMITED_BLOCK_END_MARKER,
    TOKEN_LISTING_BLOCK_START_MARKER,
    TOKEN_LISTING_BLOCK_END_MARKER,
    TOKEN_LITERAL_BLOCK_MARKER,
    TOKEN_QUOTED_BLOCK_MARKER,
    TOKEN_QUOTED_BLOCK_MD_MARKER,
    TOKEN_QUOTED_PARAGRAPH_MARKER,
    TOKEN_OPEN_BLOCK_MARKER,
    TOKEN_PASSTHROUGH_BLOCK_MARKER,
    TOKEN_BLOCK_MACRO_NAME,
    TOKEN_CALLOUT_MARKER,
    TOKEN_CALLOUT_LIST_MARKER,
    TOKEN_LINE_COMMENT_MARKER,
    TOKEN_BLOCK_COMMENT_START_MARKER,
    TOKEN_BLOCK_COMMENT_END_MARKER,

    TOKEN_ADMONITION_NOTE,
    TOKEN_ADMONITION_TIP,
    TOKEN_ADMONITION_IMPORTANT,
    TOKEN_ADMONITION_CAUTION,
    TOKEN_ADMONITION_WARNING,
    TOKEN_IDENT_MARKER,
    TOKEN_LIST_CONTINUATION
} TokenType;

static inline bool parse_table_attr(TSLexer *lexer);
static inline bool parse_number(TSLexer *lexer);
static inline bool parse_sequence_impl(TSLexer *lexer, char const *sequence, usize len);
#define parse_sequence(lexer, sequence) parse_sequence_impl(lexer, sequence, sizeof(sequence) - 1)
static inline bool parse_ordered_marker(TSLexer *lexer);
static inline bool parse_breaks(char start, TSLexer *lexer);
static inline bool consume(i32 ch, TSLexer *lexer, bool skip_space, usize *counter, usize max);
static inline bool skip_white_space(TSLexer *lexer);
static inline bool is_white_space(i32 ch);
static inline bool is_new_line(i32 ch);
static inline bool is_ascii_digit(i32 ch);
static inline bool is_ascii_alpha_lower(i32 ch);
static inline bool is_geek_lower(i32 ch);
static inline bool is_newline(i32 ch);
static inline bool is_eof(TSLexer *lexer);
