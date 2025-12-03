#include "include/scanner.h"
#include "include/base_types.h"
#include "include/quick_buffer.h"
#include "include/utils.h"
#include "tree_sitter/alloc.h"
#include "tree_sitter/parser.h"

#ifndef LIST_INIT_SIZE
#define LIST_INIT_SIZE 10
#endif  // ifndef LIST_INIT_SIZE

#ifndef LIST_GROW_SIZE
#define LIST_GROW_SIZE 10
#endif  // ifndef LIST_GROW_SIZE

void *tree_sitter_asciidoc_external_scanner_create() {
    Scanner *scanner = (Scanner *)malloc(sizeof(Scanner));
    scanner_init(scanner);
    return scanner;
}

void tree_sitter_asciidoc_external_scanner_destroy(void *payload) {
    Scanner *scanner = (Scanner *)payload;
    scanner_free(scanner);
    ts_free(scanner);
}

unsigned tree_sitter_asciidoc_external_scanner_serialize(void *payload, char *buffer) {
    Scanner *scanner = (Scanner *)payload;

    QuickBuffer qb = quick_buffer_new(buffer, TREE_SITTER_SERIALIZATION_BUFFER_SIZE);

    scanner_serialize(scanner, &qb);

    return qb.pos;
}

void tree_sitter_asciidoc_external_scanner_deserialize(void *payload, const char *buffer, unsigned length) {
    if(!buffer) {
        return;
    }

    Scanner *s = (Scanner *)payload;
    QuickBuffer qb = quick_buffer_new((void *)buffer, length);
    scanner_deserialize(s, &qb);
}

bool tree_sitter_asciidoc_external_scanner_scan(void *payload, TSLexer *lexer, const bool *valid_symbols) {
    Scanner *s = (Scanner *)payload;

    if(lexer->eof(lexer)) {
        if(valid_symbols[TOKEN_TYPE_EOF]) {
            lexer->result_symbol = TOKEN_TYPE_EOF;
            return true;
        }
    }

    usize start_pos = lexer->get_column(lexer);

    if(valid_symbols[TOKEN_BLOCK_COMMENT_END_MARKER]) {
        if(start_pos != 0) {
            return false;
        }
        if(parse_sequence(lexer, "////")) {
            if(is_newline(lexer->lookahead)) {
                lexer->result_symbol = TOKEN_BLOCK_COMMENT_END_MARKER;
                return true;
            }
        }

        return false;
    }

    if(start_pos == 0) {
        if(valid_symbols[TOKEN_ADMONITION_NOTE]) {
            do {
                if(parse_sequence(lexer, "NOTE")) {
                    lexer->result_symbol = TOKEN_ADMONITION_NOTE;
                } else if(parse_sequence(lexer, "TIP")) {
                    lexer->result_symbol = TOKEN_ADMONITION_TIP;
                } else if(parse_sequence(lexer, "IMPORTANT")) {
                    lexer->result_symbol = TOKEN_ADMONITION_IMPORTANT;
                } else if(parse_sequence(lexer, "CAUTION")) {
                    lexer->result_symbol = TOKEN_ADMONITION_CAUTION;
                } else if(parse_sequence(lexer, "WARNING")) {
                    lexer->result_symbol = TOKEN_ADMONITION_WARNING;
                } else {
                    break;
                }

                lexer->mark_end(lexer);
                if(lexer->lookahead == ':') {
                    lexer->advance(lexer, true);
                    if(lexer->lookahead == ' ') {
                        return true;
                    }
                }
            } while(0);
        }

        bool is_alpha_lower = is_ascii_alpha_lower(lexer->lookahead);
        if(valid_symbols[TOKEN_LIST_MARKER_ALPHA]) {
            if(parse_ordered_marker(lexer)) {
                return true;
            }
        }

        if(is_alpha_lower && scanner_is_matching_raw_block(s)) {
            if(valid_symbols[TOKEN_BLOCK_MACRO_NAME]) {
                if(parse_sequence(lexer, "include")) {
                    lexer->mark_end(lexer);
                    if(parse_sequence(lexer, "::")) {
                        lexer->result_symbol = TOKEN_BLOCK_MACRO_NAME;
                        return true;
                    }
                }
            }
        }

        if(is_alpha_lower && !scanner_is_matching_raw_block(s)) {
            if(valid_symbols[TOKEN_BLOCK_MACRO_NAME]) {
                while(is_ascii_alpha_lower(lexer->lookahead)) {
                    lexer->advance(lexer, false);
                }
                lexer->mark_end(lexer);
                if(parse_sequence(lexer, "::")) {
                    lexer->result_symbol = TOKEN_BLOCK_MACRO_NAME;
                    return true;
                }
            }
        }

        if(lexer->get_column(lexer) == 0) {
            switch(lexer->lookahead) {
                case '=': {
                    consume('=', lexer, false, NULL, USIZE_MAX);
                    lexer->mark_end(lexer);
                    if(!scanner_is_matching_raw_block(s)) {
                        usize counter = lexer->get_column(lexer);
                        if(counter >= 4 && is_new_line(lexer->lookahead)) {
                            if(scanner_is_matching(s, BLOCK_KIND_DELIMITED, counter)) {
                                lexer->result_symbol = TOKEN_DELIMITED_BLOCK_END_MARKER;
                                scanner_pop(s);
                            } else {
                                scanner_push(s, BLOCK_KIND_DELIMITED, counter);
                                lexer->result_symbol = TOKEN_DELIMITED_BLOCK_START_MARKER;
                            }
                            return true;
                        }
                    }

                    if(!scanner_is_matching_raw_block(s)) {
                        usize level = TOKEN_TITLE_H0_MARKER - 1 + lexer->get_column(lexer);
                        if(level <= TOKEN_TITLE_H5_MARKER && is_white_space(lexer->lookahead)) {
                            lexer->result_symbol = level;
                            return true;
                        }
                    }
                    break;
                }
                case '*': {
                    usize counter = 0;
                    consume('*', lexer, false, &counter, USIZE_MAX);
                    lexer->mark_end(lexer);
                    bool is_unordered_marker = is_white_space(lexer->lookahead);

                    if(valid_symbols[TOKEN_BREAKS_MARKS]) {
                        skip_white_space(lexer);
                        while(lexer->lookahead == '*') {
                            lexer->advance(lexer, false);
                            skip_white_space(lexer);
                            ++counter;
                            if(counter > 3) {
                                break;
                            }
                        }
                        if(counter == 3 && is_newline(lexer->lookahead)) {
                            lexer->result_symbol = TOKEN_BREAKS_MARKS;
                            lexer->mark_end(lexer);
                            return true;
                        }
                    }

                    if(valid_symbols[TOKEN_LIST_MARKER_STAR]) {
                        lexer->result_symbol = TOKEN_LIST_MARKER_STAR;
                        return is_unordered_marker;
                    }

                    break;
                }
                case '-': {
                    usize counter = 0;
                    consume('-', lexer, false, &counter, USIZE_MAX);
                    lexer->mark_end(lexer);
                    bool is_unordered_marker = is_white_space(lexer->lookahead);
                    if(valid_symbols[TOKEN_OPEN_BLOCK_MARKER]) {
                        if(lexer->get_column(lexer) == 2 && is_newline(lexer->lookahead)) {
                            lexer->result_symbol = TOKEN_OPEN_BLOCK_MARKER;
                            return true;
                        }
                    }

                    if(valid_symbols[TOKEN_QUOTED_PARAGRAPH_MARKER]) {
                        if(lexer->get_column(lexer) == 2 && is_white_space(lexer->lookahead)) {
                            lexer->result_symbol = TOKEN_QUOTED_PARAGRAPH_MARKER;
                            return true;
                        }
                    }

                    if(valid_symbols[TOKEN_LISTING_BLOCK_START_MARKER] || valid_symbols[TOKEN_LISTING_BLOCK_END_MARKER]) {
                        usize counter = lexer->get_column(lexer);
                        if(counter >= 4 && is_newline(lexer->lookahead)) {
                            if(scanner_is_matching(s, BLOCK_KIND_LISTING, counter)) {
                                scanner_pop(s);
                                lexer->result_symbol = TOKEN_LISTING_BLOCK_END_MARKER;
                                return true;
                            } else {
                                if(!scanner_is_matching_raw_block(s)) {
                                    scanner_push(s, BLOCK_KIND_LISTING, counter);
                                    lexer->result_symbol = TOKEN_LISTING_BLOCK_START_MARKER;
                                    return true;
                                }
                            }
                        }
                    }

                    if(valid_symbols[TOKEN_BREAKS_MARKS]) {
                        skip_white_space(lexer);
                        while(lexer->lookahead == '-') {
                            lexer->advance(lexer, false);
                            skip_white_space(lexer);
                            ++counter;
                            if(counter > 3) {
                                break;
                            }
                        }
                        if(counter == 3 && is_newline(lexer->lookahead)) {
                            lexer->result_symbol = TOKEN_BREAKS_MARKS;
                            lexer->mark_end(lexer);
                            return true;
                        }
                    }

                    if(valid_symbols[TOKEN_LIST_MARKER_HYPHEN] && is_unordered_marker) {
                        lexer->result_symbol = TOKEN_LIST_MARKER_HYPHEN;
                        return true;
                    }
                    break;
                }
                case '.': {
                    lexer->advance(lexer, false);
                    lexer->mark_end(lexer);
                    if(valid_symbols[TOKEN_LITERAL_BLOCK_MARKER]) {
                        if(parse_sequence(lexer, "...")) {
                            if(is_newline(lexer->lookahead)) {
                                lexer->mark_end(lexer);
                                lexer->result_symbol = TOKEN_LITERAL_BLOCK_MARKER;
                                if(scanner_is_matching(s, BLOCK_KIND_LITERAL, 0)) {
                                    scanner_pop(s);
                                } else {
                                    scanner_push(s, BLOCK_KIND_LITERAL, 4);
                                }
                                return true;
                            }
                        }
                    }

                    if(valid_symbols[TOKEN_LIST_MARKER_DOT]) {
                        consume('.', lexer, false, NULL, USIZE_MAX);
                        if(is_white_space(lexer->lookahead)) {
                            lexer->mark_end(lexer);
                            lexer->result_symbol = TOKEN_LIST_MARKER_DOT;
                            return true;
                        }
                    }

                    if(valid_symbols[TOKEN_BLOCK_TITLE_MARKER]) {
                        if(lexer->get_column(lexer) == 1) {
                            lexer->mark_end(lexer);
                            lexer->result_symbol = TOKEN_BLOCK_TITLE_MARKER;
                            return true;
                        }
                    }

                    break;
                }
                case ':': {
                    if(valid_symbols[TOKEN_DOCUMENT_ATTR_MARKER]) {
                        lexer->advance(lexer, false);
                        lexer->mark_end(lexer);
                        lexer->result_symbol = TOKEN_DOCUMENT_ATTR_MARKER;
                        return true;
                    }
                    break;
                }
                case '[': {
                    if(valid_symbols[TOKEN_ELEMENT_ATTR_MARKER]) {
                        lexer->advance(lexer, false);
                        if(lexer->lookahead == '[' || lexer->lookahead == '#') {
                            return false;
                        }
                        lexer->mark_end(lexer);
                        lexer->result_symbol = TOKEN_ELEMENT_ATTR_MARKER;
                        return true;
                    }
                    break;
                }
                case '\'': {
                    if(valid_symbols[TOKEN_BREAKS_MARKS]) {
                        if(parse_breaks('\'', lexer)) {
                            lexer->result_symbol = TOKEN_BREAKS_MARKS;
                            return true;
                        }
                    }
                    break;
                }
                case '<': {
                    if(valid_symbols[TOKEN_BREAKS_MARKS]) {
                        if(parse_breaks('<', lexer)) {
                            lexer->result_symbol = TOKEN_BREAKS_MARKS;
                            return true;
                        }
                    }
                    if(valid_symbols[TOKEN_CALLOUT_LIST_MARKER]) {
                        if(lexer->get_column(lexer) == 1) {
                            if(parse_sequence(lexer, ".>")) {
                                lexer->mark_end(lexer);
                                lexer->result_symbol = TOKEN_CALLOUT_LIST_MARKER;
                                if(is_white_space(lexer->lookahead)) {
                                    return true;
                                }
                            }
                            if(parse_number(lexer)) {
                                if(lexer->lookahead == '>') {
                                    lexer->advance(lexer, false);
                                    lexer->mark_end(lexer);
                                    if(is_white_space(lexer->lookahead)) {
                                        lexer->result_symbol = TOKEN_CALLOUT_LIST_MARKER;
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                    break;
                }
                case '|': {  // Table
                    if(valid_symbols[TOKEN_TABLE_BLOCK_MARKER]) {
                        lexer->advance(lexer, false);
                        usize counter = 0;
                        consume('=', lexer, false, &counter, USIZE_MAX);
                        if(counter >= 3 && is_newline(lexer->lookahead)) {
                            if(scanner_is_matching(s, BLOCK_KIND_TABLE, 0)) {
                                if(scanner_is_matching(s, BLOCK_KIND_TABLE, counter)) {
                                    lexer->mark_end(lexer);
                                    lexer->result_symbol = TOKEN_TABLE_BLOCK_MARKER;
                                    scanner_pop(s);
                                    return true;
                                }
                            } else {
                                lexer->mark_end(lexer);
                                lexer->result_symbol = TOKEN_TABLE_BLOCK_MARKER;
                                scanner_push(s, BLOCK_KIND_TABLE, counter);
                                return true;
                            }
                        }
                    }
                    break;
                }
                case '!': {  // NTable
                    if(valid_symbols[TOKEN_NTABLE_BLOCK_MARKER]) {
                        lexer->advance(lexer, false);
                        consume('=', lexer, false, NULL, USIZE_MAX);
                        if(is_newline(lexer->lookahead)) {
                            lexer->mark_end(lexer);
                            lexer->result_symbol = TOKEN_NTABLE_BLOCK_MARKER;
                            return true;
                        }
                    }
                    break;
                }
                case '/': {
                    if(valid_symbols[TOKEN_LINE_COMMENT_MARKER] ||
                       valid_symbols[TOKEN_BLOCK_COMMENT_START_MARKER]) {
                        if(parse_sequence(lexer, "//")) {
                            lexer->mark_end(lexer);
                            if(valid_symbols[TOKEN_BLOCK_COMMENT_START_MARKER]) {
                                if(parse_sequence(lexer, "//")) {
                                    if(is_newline(lexer->lookahead)) {
                                        lexer->result_symbol = TOKEN_BLOCK_COMMENT_START_MARKER;
                                        lexer->mark_end(lexer);
                                        return true;
                                    }
                                }
                            }
                            lexer->result_symbol = TOKEN_LINE_COMMENT_MARKER;
                            return true;
                        }
                    }
                    break;
                }
                case '_': {
                    if(valid_symbols[TOKEN_QUOTED_BLOCK_MARKER]) {
                        if(parse_sequence(lexer, "____")) {
                            lexer->mark_end(lexer);
                            lexer->result_symbol = TOKEN_QUOTED_BLOCK_MARKER;
                            if(is_newline(lexer->lookahead) || is_eof(lexer)) {
                                return true;
                            }
                        }
                    }
                    break;
                }
                case '>': {
                    if(valid_symbols[TOKEN_QUOTED_BLOCK_MD_MARKER]) {
                        lexer->advance(lexer, false);
                        lexer->mark_end(lexer);
                        lexer->result_symbol = TOKEN_QUOTED_BLOCK_MD_MARKER;
                        if(is_white_space(lexer->lookahead) || is_eof(lexer) || is_newline(lexer->lookahead)) {
                            return true;
                        }
                    }
                    break;
                }
                case '+': {
                    lexer->advance(lexer, false);
                    if(valid_symbols[TOKEN_LIST_CONTINUATION]) {
                        if(is_newline(lexer->lookahead)) {
                            lexer->result_symbol = TOKEN_LIST_CONTINUATION;
                            lexer->mark_end(lexer);
                            return true;
                        }
                    }
                    if(valid_symbols[TOKEN_PASSTHROUGH_BLOCK_MARKER]) {
                        if(parse_sequence(lexer, "+++")) {
                            lexer->mark_end(lexer);
                            lexer->result_symbol = TOKEN_PASSTHROUGH_BLOCK_MARKER;
                            if(is_new_line(lexer->lookahead) || is_eof(lexer)) {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        if(
            lexer->get_column(lexer) == 0 && valid_symbols[TOKEN_IDENT_MARKER] && is_white_space(lexer->lookahead)
        ) {
            skip_white_space(lexer);
            if(!is_new_line(lexer->lookahead) && !is_eof(lexer)) {
                lexer->mark_end(lexer);
                lexer->result_symbol = TOKEN_IDENT_MARKER;
                return true;
            }
        }
    }

    if(valid_symbols[TOKEN_CALLOUT_MARKER]) {
        parse_sequence(lexer, "#");
        parse_sequence(lexer, "//");
        parse_sequence(lexer, ";;");
        skip_white_space(lexer);

        if(parse_sequence(lexer, "<.>")) {
            lexer->mark_end(lexer);
            if(is_newline(lexer->lookahead)) {
                lexer->result_symbol = TOKEN_CALLOUT_MARKER;
                return true;
            }
        }

        if(parse_number(lexer)) {
            if(lexer->lookahead == '>') {
                lexer->advance(lexer, false);
                lexer->mark_end(lexer);
                if(is_newline(lexer->lookahead)) {
                    lexer->result_symbol = TOKEN_CALLOUT_MARKER;
                    return true;
                }
            }
        }

        if(parse_sequence(lexer, "!--")) {
            if(lexer->lookahead == '.' || parse_number(lexer)) {
                if(parse_sequence(lexer, "-->")) {
                    lexer->mark_end(lexer);
                    if(is_new_line(lexer->lookahead)) {
                        lexer->result_symbol = TOKEN_CALLOUT_MARKER;
                        return true;
                    }
                }
            }
        }
    }

    if(
        start_pos == lexer->get_column(lexer) &&
        valid_symbols[TOKEN_CELL_ATTR]
    ) {
        bool has_token = false;
        while(parse_table_attr(lexer)) {
            has_token = true;
        }

        if(has_token) {
            if(lexer->lookahead == '|' || lexer->lookahead == '!') {
                lexer->result_symbol = TOKEN_CELL_ATTR;
                lexer->mark_end(lexer);
                return true;
            }
        }
    }

    return false;
}

bool parse_ordered_marker(TSLexer *lexer) {
    if(lexer->get_column(lexer) != 0) {
        return false;
    }

    if(is_ascii_digit(lexer->lookahead)) {
        while(is_ascii_digit(lexer->lookahead)) {
            lexer->advance(lexer, false);
        }
        lexer->result_symbol = TOKEN_LIST_MARKER_DIGIT;
    } else if(is_ascii_alpha_lower(lexer->lookahead)) {
        lexer->result_symbol = TOKEN_LIST_MARKER_ALPHA;
        lexer->advance(lexer, false);
    } else if(is_geek_lower(lexer->lookahead)) {
        lexer->result_symbol = TOKEN_LIST_MARKER_GEEK;
        lexer->advance(lexer, false);
    } else {
        return false;
    }

    if(lexer->lookahead != '.') {
        return false;
    }
    lexer->advance(lexer, false);
    if(!is_white_space(lexer->lookahead)) {
        return false;
    }
    lexer->mark_end(lexer);
    return true;
}

static inline bool is_white_space(i32 ch) {
    return ch == ' ' || ch == '\t';
}

static inline bool is_ascii_digit(i32 ch) {
    return ch >= '0' && ch <= '9';
}

static inline bool is_ascii_alpha_lower(i32 ch) {
    return ch >= 'a' && ch <= 'z';
}

static inline bool is_geek_lower(i32 ch) {
    return ch >= 945 && ch <= 969;
}

static inline bool parse_breaks(char start, TSLexer *lexer) {
    i32 counter = 0;

    while(lexer->lookahead == start) {
        lexer->advance(lexer, false);
        skip_white_space(lexer);
        ++counter;
        if(counter > 3) {
            return false;
        }
    }

    lexer->mark_end(lexer);

    return counter == 3 && is_newline(lexer->lookahead);
}

static inline bool skip_white_space(TSLexer *lexer) {
    bool has_skiped = false;
    while(is_white_space(lexer->lookahead)) {
        lexer->advance(lexer, false);
        has_skiped = true;
    }

    return has_skiped;
}

static inline bool is_newline(i32 ch) {
    return ch == '\r' || ch == '\n';
}

static inline bool is_eof(TSLexer *lexer) {
    return lexer->eof(lexer);
}

static inline bool is_new_line(i32 ch) {
    return ch == '\r' || ch == '\n';
}

static inline bool consume(i32 ch, TSLexer *lexer, bool skip_space, usize *counter, usize max) {
    bool has_space = false;
    if(skip_space) {
        has_space |= skip_white_space(lexer);
    }

    while(lexer->lookahead == ch) {
        lexer->advance(lexer, false);
        if(skip_space) {
            has_space |= skip_white_space(lexer);
        }
        --max;
        if(counter != NULL) {
            *counter += 1;
        }
        if(max == 0) {
            break;
        }
    }

    lexer->mark_end(lexer);
    return has_space;
}

static inline bool parse_sequence_impl(TSLexer *lexer, char const *sequence, usize len) {
    usize pos = 0;

    while(pos < len) {
        if(lexer->lookahead != sequence[pos]) {
            return false;
        }

        lexer->advance(lexer, false);
        ++pos;
    }

    return true;
}

static inline bool parse_number(TSLexer *lexer) {
    bool has_number = false;
    while(is_ascii_digit(lexer->lookahead)) {
        lexer->advance(lexer, false);
        has_number = true;
    }
    return has_number;
}

char table_cell_attr_align_left = '<';
char table_cell_attr_align_right = '>';
char table_cell_attr_align_middle = '^';
char table_cell_attr_style_none = 'd';
char table_cell_attr_style_strong = 's';
char table_cell_attr_style_emphasis = 'e';
char table_cell_attr_style_monospaced = 'm';
char table_cell_attr_style_header = 'h';
char table_cell_attr_style_literal = 'l';
char table_cell_attr_style_asciidoc = 'a';

static inline bool parse_table_attr(TSLexer *lexer) {
    if(lexer->lookahead == table_cell_attr_align_left ||
       lexer->lookahead == table_cell_attr_align_right ||
       lexer->lookahead == table_cell_attr_align_middle ||
       lexer->lookahead == table_cell_attr_style_none ||
       lexer->lookahead == table_cell_attr_style_strong ||
       lexer->lookahead == table_cell_attr_style_emphasis ||
       lexer->lookahead == table_cell_attr_style_monospaced ||
       lexer->lookahead == table_cell_attr_style_header ||
       lexer->lookahead == table_cell_attr_style_literal ||
       lexer->lookahead == table_cell_attr_style_asciidoc) {
        lexer->advance(lexer, false);
        return true;
    }
    // table_cell_span_vertical = /\.\d+\+/;
    if(lexer->lookahead == '.') {
        lexer->advance(lexer, false);
        return parse_number(lexer);
    }
    // table_cell_span_width = /\d+\+/;
    // table_cell_span_vw = /\d+\.\d+\+/;
    if(parse_number(lexer)) {
        if(lexer->lookahead == '.') {
            lexer->advance(lexer, false);
            if(!parse_number(lexer)) {
                return false;
            }
        }
        if(lexer->lookahead == '+') {
            lexer->advance(lexer, false);
            return true;
        }
    }

    return false;
}

static inline bool scanner_is_matching(Scanner const *self, BlockKind kind, usize counter) {
    Node *top = scanner_top(self);
    if(!top) {
        return false;
    }

    if(counter == 0) {
        return top->kind == kind;
    }
    return top->kind == kind && top->counter == counter;
}
static inline Result scanner_serialize(Scanner const *self, QuickBuffer *qb) {
    Result ret = RESULT_OK;

    ret &= quick_buffer_write_usize(qb, self->len);
    ret &= quick_buffer_extend_bytes(qb, self->buffer, sizeof(Node) * self->len);

    return ret;
}

static inline Result scanner_deserialize(Scanner *self, QuickBuffer *qb) {
    Result ret = RESULT_OK;

    usize len = 0;
    ret &= quick_buffer_read_usize(qb, &len);
    if(self->capacity < len) {
        self->buffer = ts_realloc(self->buffer, len * sizeof(Node));
        self->capacity = len;
    }
    self->len = len;
    ret &= quick_buffer_read_bytes(qb, self->buffer, len * sizeof(Node));

    return ret;
}

static inline bool scanner_pop_kind(Scanner *self, BlockKind kind, usize counter) {
    if(scanner_is_matching(self, kind, counter)) {
        scanner_pop(self);
        return true;
    }

    return false;
}

static inline void scanner_pop(Scanner *self) {
    if(self->len == 0) {
        return;
    }
    self->len -= 1;
}

static inline void scanner_push(Scanner *self, BlockKind kind, usize counter) {
    const usize BUFFER_SIZE = TREE_SITTER_SERIALIZATION_BUFFER_SIZE - sizeof(usize);

    if(self->len == self->capacity) {
        usize grow_counter = BUFFER_SIZE / sizeof(Node) - self->capacity;
        if(grow_counter == 0) {
            return;
        }
        if(LIST_GROW_SIZE < grow_counter) {
            grow_counter = LIST_GROW_SIZE;
        }

        self->capacity = self->capacity + grow_counter;
        self->buffer = ts_realloc(self->buffer, sizeof(Node) * self->capacity);
    }

    Node *top = &(self->buffer[self->len++]);
    top->kind = kind;
    top->counter = counter;
}

static inline void scanner_init(Scanner *self) {
    bzero(self, sizeof(Scanner));
    self->buffer = ts_malloc(LIST_INIT_SIZE * sizeof(Node));
    self->capacity = LIST_INIT_SIZE;
}

static inline void scanner_free(Scanner *self) {
    ts_free(self->buffer);
    bzero(self, sizeof(Scanner));
}

static inline bool scanner_is_matching_raw_block(Scanner *self) {
    return scanner_is_matching(self, BLOCK_KIND_LISTING, 0) ||
           scanner_is_matching(self, BLOCK_KIND_LITERAL, 0);
}

static inline Node *scanner_top(Scanner const *self) {
    if(self->len == 0) {
        return NULL;
    }

    return &(self->buffer[self->len - 1]);
}
