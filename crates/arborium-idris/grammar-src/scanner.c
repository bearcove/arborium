/**
 * Print input and result information.
 */
// #define DEBUG 1

/**
 * Print the upcoming token after parsing finished.
 * Note: May change parser behaviour.
 */
// #define DEBUG_NEXT_TOKEN 1

#include "tree_sitter/parser.h"
#include "tree_sitter/array.h"
#include <assert.h>
#ifdef DEBUG
#include <stdio.h>
#endif
#include <string.h>
#include <stdbool.h>
#include <wctype.h>
#include "unicode.h"

// short circuit
#define SHORT_SCANNER if (res.finished) return res;
#define PEEK state->lexer->lookahead
// Move the parser position one character to the right.
#define S_ADVANCE state->lexer->advance(state->lexer, false)
// Move the parser position one character to the right, treating the consumed character as whitespace.
#define S_SKIP state->lexer->advance(state->lexer, true)
#define SYM(s) (state->symbols[s])

#ifdef DEBUG
#define DEBUG_PRINTF(...) do{ fprintf( stderr, __VA_ARGS__ ); } while( false )
#else
#define DEBUG_PRINTF(...) do{ } while ( false )
#endif


// --------------------------------------------------------------------------------------------------------
// Symbols
// --------------------------------------------------------------------------------------------------------

/**
 * This enum is mapped to the `externals` list in the grammar according to how they are ordered (the names are
 * abitrary).
 *
 * When the `scan` function is called, the parameter `syms` contains a bool for each enum attribute indicating whether
 * the parse tree at the current position can accept the corresponding symbol.
 *
 * The attribute `fail` is not part of the parse tree, it is used to indicate that no matching symbol was found.
 *
 * The meanings are:
 *   - semicolon: An implicit end of a decl or statement, a newline in place of a semicolon
 *   - start: Start an implicit new layout after `where`, `do`, `of` or `in`, in place of an opening brace
 *   - end: End an implicit layout, in place of a closing brace
 *   - dot: For qualified modules `Data.List.null`, which have to be disambiguated from the `(.)` operator based on
 *     surrounding whitespace.
 *   - where: Parse an inline `where` token. This is necessary because `where` tokens can end layouts and it's necesary
 *     to know whether it is valid at that position, which can mean that it belongs to the last statement of the layout
 *   - varid: A variable id
 *   - varsym: A symbolic operator
 *   - comment: A line or block comment, because they interfere with operators
 *   - cpp: A preprocessor directive. Needs to push and pop indent stacks
 *   - comma: Needed to terminate inline layouts like `of`, `do`
 *   - bar: The vertical bar `|`, used for guards and list comprehension
 *   - in: Closes the layout of a `let` and consumes the token `in`
 *   - indent: Used as a dummy symbol for initialization; uses newline in the grammar to ensure the scanner is called
 *     for each token
 *   - empty: The empty file
 *   - fail: special indicator of failure
 */
typedef enum {
  SEMICOLON,
  LAYOUT_START,
  LAYOUT_END,
  LAYOUT_RESTART,
  DOT,
  WHERE,
  VARID,
  VARSYM,
  COMMENT,
  IN_STRING,
  RAW_STRING_START,
  RAW_STRING_END,
  CPP,
  COMMA,
  IN,
  INDENT,
  EMPTY,
  FAIL,
  ERROR_SENTINEL,
} Sym;

#ifdef DEBUG
static char *sym_names[] = {
  "semicolon",
  "start",
  "end",
  "layout_restart",
  "dot",
  "where",
  "varid",
  "varsym",
  "comment",
  "in_string",
  "raw_string_start",
  "raw_string_end",
  "cpp",
  "comma",
  "in",
  "indent",
  "empty",
  "fail",
};
#endif

static char *keywords[] = {
  "private",
  "public",
  "export",
  "total",
  "covering",
  "partial",
  "module",
  "namespace",
  "parameters",
  "import",
  "using",
  "mutual",
  "data",
  "interface",
  "implementation",
  "record",
  "constructor",
  "infix",
  "infixl",
  "infixr",
  "prefix",
  "let",
  "rewrite",
  "if",
  "then",
  "else",
  "case",
  "of",
  "do",
  "impossible",
  "with",
  "proof",
  "forall",
  "auto",
  "default",
};

static char *invalid_varops[] = {
  "|",
  ":",
  "=",
  "@",
  "\\",
  "->",
  "=>",
  "<-",
  ":=",
  "$=",
  "**",
};

#ifdef DEBUG
/**
 * Produce a comma-separated string of valid symbols.
 */
static void debug_valid(const bool *syms) {
  if (syms[ERROR_SENTINEL]) {
    DEBUG_PRINTF("all");
    return;
  }
  bool fst = true;
  DEBUG_PRINTF("\"");
  for (Sym i = SEMICOLON; i <= ERROR_SENTINEL; i++) {
    if (syms[i]) {
      if (!fst) DEBUG_PRINTF(",");
      DEBUG_PRINTF("%s", sym_names[i]);
      fst = false;
    }
  }
  DEBUG_PRINTF("\"");
}
#endif

typedef Array(char) String;

typedef struct {
  Array(uint32_t) indents;
  Array(uint32_t) raw_string_sharp_counts;
} Payload;

// --------------------------------------------------------------------------------------------------------
// State
// --------------------------------------------------------------------------------------------------------

/**
 * This structure contains the external and internal state.
 *
 * The parser provides the lexer interface and the list of valid symbols.
 *
 * The internal state consists of a stack of indentation widths that is manipulated whenever a layout is started or
 * terminated.
 */
typedef struct {
  TSLexer *lexer;
  const bool *symbols;
  Payload *payload;
  String token;
#ifdef DEBUG
  int marked;
  char *marked_by;
  bool needs_free;
#endif
} State;

static State state_new(TSLexer *l, const bool * restrict vs, Payload *is) {
  return (State) {
    .lexer = l,
    .symbols = vs,
    .payload = is,
#ifdef DEBUG
    .marked = -1,
    .marked_by = "",
    .needs_free = false,
#endif
  };
}

#ifdef DEBUG
static void debug_indents(Payload *payload) {
  DEBUG_PRINTF("[");
  for (size_t i = 0; i < payload->indents.size; i++) {
    if (0 < i) DEBUG_PRINTF(",");
    DEBUG_PRINTF("%d", *array_get(&payload->indents, i));
  }
  DEBUG_PRINTF("]");
}

static void debug_sharps(Payload *payload) {
  DEBUG_PRINTF("[");
  for (size_t i = 0; i < payload->raw_string_sharp_counts.size; i++) {
    if (0 < i) DEBUG_PRINTF(",");
    DEBUG_PRINTF("%d", *array_get(&payload->raw_string_sharp_counts, i));
  }
  DEBUG_PRINTF("]");
}

static void debug_payload(Payload *payload) {
  DEBUG_PRINTF("{ indents = ");
  debug_indents(payload);
  DEBUG_PRINTF(", sharps = ");
  debug_sharps(payload);
  DEBUG_PRINTF(" }");

}

void debug_state(State *state) {
  DEBUG_PRINTF("State { syms = ");
  debug_valid(state->symbols);
  DEBUG_PRINTF(", payload = ");
  debug_payload(state->payload);
  DEBUG_PRINTF(" }\n");
}
#endif

/**
 * These functions provide the basic interface to the lexer.
 * They are not defined as members for easier composition.
 */
static bool is_eof(State *state) { return state->lexer->eof(state->lexer); }

/**
 * The parser's position in the current line.
 */
static uint32_t column(State *state) {
  return is_eof(state) ? 0 : state->lexer->get_column(state->lexer);
}

/**
 * Instruct the lexer that the current position is the end of the potentially detected symbol, causing the next run to
 * be started after this character in the success case.
 *
 * This is useful if the validity of the detected symbol depends on what follows, e.g. in the case of a layout end
 * before a `where` token.
 */

// Only use string literals we actually need
#ifdef DEBUG
static void MARK(char *marked_by, bool needs_free, State *state) {
  state->marked = column(state);
  if (state->needs_free) free(state->marked_by);
  state->marked_by = marked_by;
  state->needs_free = needs_free;
  state->lexer->mark_end(state->lexer);
}
#else
#define MARK(s, nf, state) state->lexer->mark_end(state->lexer);
#endif


// --------------------------------------------------------------------------------------------------------
// Condition
// --------------------------------------------------------------------------------------------------------

/**
 * The set of conditions used in the parser implementation.
 */
static bool varid_start_char(const uint32_t c) { return c == '_' || iswlower(c); }

static bool varid_char(const uint32_t c) {
  switch (c) {
    case '_':
    case '\'':
      return true;
    default:
      // TODO(414owen) is haskell C_LOCALE sensitive?
      return iswalnum(c);
  }
}

static bool quoter_char(const uint32_t c) { return varid_char(c) || c == '.'; };

static String read_varid(State* state) {
  String str = array_new();
  if (varid_start_char(PEEK)) {
    array_push(&str, PEEK);
    S_ADVANCE;
  }
  while (varid_char(PEEK)) {
    array_push(&str, PEEK);
    S_ADVANCE;
  }
  array_push(&str, 0);
  return str;
}

static void read_token(State* state) {
  state->token = read_varid(state);
}

static bool is_token(const char* str, State* state) {
  return 0 == strcmp(str, state->token.contents);
}

static bool seq(const char * restrict s, State *state) {
  size_t len = strlen(s);
  for (size_t i = 0; i < len; i++) {
    int32_t c = s[i];
    int32_t c2 = PEEK;
    if (c != c2) return false;
    S_ADVANCE;
  }
  return true;
}

static void consume_until(char *target, State *state) {
  int32_t first = target[0];
  assert(first != 0);
  while (PEEK != 0 && !seq(target, state)) {
    while (PEEK != 0 && PEEK != first) S_ADVANCE;
    // TODO(414owen): This mimics the combinator's behaviour, but it seems a bit silly.
    // Why mark where the first char matched? Let's just not do this check.
    if (first == PEEK) {
#ifdef DEBUG
      char *prefix = "consume_until ";
      char *mark_target = calloc(strlen(prefix) + strlen(target) + 1, 1);
      sprintf(mark_target, "%s%s", prefix, target);
      MARK(mark_target, true, state);
#else
      state->lexer->mark_end(state->lexer);
#endif
    }
  }
}


static bool streq(const char* s, const char* t) {
  return 0 == strcmp(s, t);
}

static bool strall(const char c, const char* s) {
  while (c == *s) { ++s; }
  return 0 == *s;
}

static String read_string(bool (*cond)(uint32_t), State *state) {
  String res = array_new();
  while (cond(PEEK)) {
    array_push(&res, PEEK);
    S_ADVANCE;
  }
  array_push(&res, 0);
  return res;
}

#define WS_CASES \
  case ' ': \
  case '\f': \
  case '\n': \
  case '\r': \
  case '\t': \
  case '\v'

/**
 * Require that the next character is whitespace (space or newline) without advancing the parser.
 */
static bool isws(uint32_t c) {
  switch (c) {
    WS_CASES: return true;
    default: return false;
  }
}

/**
 * A token like a varsym can be terminated by whitespace or brackets.
 */
static bool token_end(uint32_t c) {
  switch (c) {
    WS_CASES:
    case 0:
    case '(':
    case ')':
    case '[':
    case ']':
      return true;
    default:
      return false;
  }
}

/**
 * Require that the argument string follows the current position and is followed by whitespace.
 * See `seq`
 */
static bool token(const char *restrict s, State *state) {
  return seq(s, state) && token_end(PEEK);
}

/**
 * Require that the stack of layout indentations is not empty.
 * This is mostly used for safety.
 */
static bool indent_exists(State *state) { return state->payload->indents.size != 0; };

/**
 * Require that the current line's indent is greater or equal than the containing layout's, so the current layout is
 * continued.
 */
static bool keep_layout(uint32_t indent, State *state) {
  return indent_exists(state) && indent >= *array_back(&state->payload->indents);
}

/**
 * Require that the current line's indent is equal to the containing layout's, so the line may start a new `decl`.
 */
static bool same_indent(uint32_t indent, State *state) { 
  return indent_exists(state) && indent == *array_back(&state->payload->indents);
}

/**
 * Require that the current line's indent is smaller than the containing layout's, so the layout may be ended.
 */
static bool less_indent(uint32_t indent, State *state) {
  return indent_exists(state) && indent < *array_back(&state->payload->indents);
}

/**
 * Dedented but still indented in outer scope. Needed to detect the following layout.
 * 
 * parent
 *     foo
 *   bar
 */
static bool less_and_more_indent(uint32_t indent, State *state) {
  return 2 <= state->payload->indents.size && 
    indent < *array_back(&state->payload->indents) &&
    indent > *array_get(&state->payload->indents, state->payload->indents.size - 2);
}

static bool indent_lesseq(uint32_t indent, State *state) {
   return indent_exists(state) && indent <= *array_back(&state->payload->indents);
}

static bool more_indent(uint32_t indent, State *state) {
  return indent_exists(state) && indent > *array_back(&state->payload->indents);
}


/**
 * Composite condition examining whether the current layout can be terminated if the line after the position where the
 * scan started begins with a `where`.
 *
 * This is needed because `where` can appear on the same indent as, for example, a `do` statement in a `decl`, while
 * being part of the latter and therefore having to end the `do`'s layout before parsing the `where`.
 *
 * This does only check whether the line begins with a `w`, the entire `where` is consumed by the calling parser below.
 */
static bool is_newline_where(uint32_t indent, State *state) {
  return keep_layout(indent, state)
    && (SYM(SEMICOLON) || SYM(LAYOUT_END))
    && !SYM(WHERE);
}

#define NEWLINE_CASES \
  case '\n': \
  case '\r': \
  case '\f'

static bool is_newline(uint32_t c) {
  switch (c) {
    NEWLINE_CASES:
      return true;
    default:
      return false;
  }
}

/**
 * Require that the state has not been initialized after parsing has started.
 *
 * This is necessary to handle a nonexistent `module` declaration.
 */
static bool uninitialized(State *state) { return !indent_exists(state); }

/**
 * When recovering from an error, it sets all token valid, including unexposed token.
 * Use ERROR_SENTINEL for this purpose.
 */
static bool after_error(State *state) { return SYM(ERROR_SENTINEL); }

#define SYMBOLIC_CASES \
    case '!': \
    case '$': \
    case '%': \
    case '&': \
    case '*': \
    case '+': \
    case '.': \
    case '/': \
    case '<': \
    case '>': \
    case '?': \
    case '^': \
    case ':': \
    case '=': \
    case '-': \
    case '~': \
    case '@': \
    case '\\': \
    case '|'

static bool symbolic(uint32_t c) {
  switch (c) {
    SYMBOLIC_CASES:
      return true;
    case '#':
      return true;
    default:
      return unicode_symbol(c);
  }
}


typedef enum {
  S_OP,
  S_IMPLICIT,
  S_MODIFIER,
  S_COMMENT,
  S_INVALID,
} Symbolic;


/**
 * Check all conditions for symbolic expression operators and return a variant of the enum `Symbolic`.
 *
 *  - does not match a reserved operator
 *  - is not a comment
 *
 */
static Symbolic s_symop(String s, State *state) {
  if (s.size == 0 || s.contents[0] == 0) return S_INVALID;

  if (2 <= s.size && s.contents[0] == '-' && s.contents[1] == '-') return S_COMMENT;
  if (3 <= s.size && s.contents[0] == '|' && s.contents[1] == '|' && s.contents[2] == '|') return S_COMMENT;

  for (int i = 0; i < sizeof(invalid_varops)/sizeof(invalid_varops[0]); ++i) {
    if (streq(invalid_varops[i], s.contents)) return S_INVALID;
  }

  // raw string open
  if (streq("#", s.contents) && '"' == PEEK) return S_INVALID;

  // pragma
  if (streq("%", s.contents) && iswalnum(PEEK)) return S_INVALID;

  return S_OP;
}

// --------------------------------------------------------------------------------------------------------
// Result
// --------------------------------------------------------------------------------------------------------

/**
 * Returned by a parser, indicating whether to continue with the next parser (`finished`) which symbol to select when
 * successful (`sym`).
 *
 * Whether parsing was successful is indicated by which symbol is selected – `FAIL` signals failure.
 */
typedef struct {
  Sym sym;
  bool finished;
} Result;

#ifdef DEBUG
void debug_result(Result res) {
  DEBUG_PRINTF("Result { finished = %d", res.finished);
  if (res.finished)
    DEBUG_PRINTF(", result = %s }\n", sym_names[res.sym]);
}
#endif

/**
 * Constructors for the continue, failure and success results.
 */
static Result res_cont = {.sym = FAIL, .finished = false};
static Result res_finish(Sym t) { return (Result) {.sym = t, .finished = true}; }
static Result res_fail = {.sym = FAIL, .finished = true};


// --------------------------------------------------------------------------------------------------------
// Parser
// --------------------------------------------------------------------------------------------------------

/**
 * Parser that terminates the execution with the successful detection of the given symbol.
 */
static Result finish(const Sym s, char *restrict desc) {
  DEBUG_PRINTF("finish: %s\n", desc);
  return res_finish(s);
}

/**
 * Parser that terminates the execution with the successful detection of the given symbol, but only if it is expected.
 */
static Result finish_if_valid(const Sym s, char *restrict desc, State *state) {
  return SYM(s) ? finish(s, desc) : res_cont;
}

/**
 * Add one level of indentation to the stack, caused by starting a layout.
 */
static void push_indent(uint32_t ind, State *state) {
  DEBUG_PRINTF("push: %d\n", ind);
  array_push(&state->payload->indents, ind);
}

/**
 * Remove one level of indentation from the stack, caused by the end of a layout.
 */
static void pop_indent(State *state) {
  if (indent_exists(state)) {
    DEBUG_PRINTF("pop\n");
    array_pop(&state->payload->indents);
  }
}

/**
 * Advance the lexer until the following character is neither space nor tab.
 */
static void skipspace(State *state) {
  for (;;) {
    switch (PEEK) {
      case ' ':
      case '\t':
        S_SKIP;
        break;
      default:
        return;
    }
  }
}

/**
 * If the indent meets the condition, it updates the indents.
 */
static Result layout_restart(uint32_t indent, State *state) {
  if (SYM(LAYOUT_RESTART) && less_and_more_indent(indent, state)) {
    pop_indent(state);
    push_indent(indent, state);
    return finish(LAYOUT_RESTART, "layout_restart");
  }
  return res_cont;
}

/**
 * If a layout end is valid at this position, remove one indentation layer and succeed with layout end.
 */
static Result layout_end(char *desc, State *state) {
  if (SYM(LAYOUT_END)) {
    pop_indent(state);
    return finish(LAYOUT_END, desc);
  }
  return res_cont;
}

/**
 * Convenience parser, since those two are often used together.
 */
static Result end_or_semicolon(char *desc, State *state) {
  Result res = layout_end(desc, state);
  SHORT_SCANNER;
  return finish_if_valid(SEMICOLON, desc, state);
}

// --------------------------------------------------------------------------------------------------------
// Logic
// --------------------------------------------------------------------------------------------------------

/**
 * These parsers constitute the higher-level logic, loosely.
 */


/**
 * Advance the parser until a non-whitespace character is encountered, while counting whitespace according to the rules
 * in the syntax reference, resetting the counter on each newline.
 *
 * This advances to the first nonwhite character in the next nonempty line and determines its indentation.
 */
static uint32_t count_indent(State *state) {
  uint32_t indent = 0;
  for (;;) {
    switch (PEEK) {
      NEWLINE_CASES:
        S_SKIP;
        indent = 0;
        break;
      case ' ':
        S_SKIP;
        indent++;
        break;
      case '\t':
        S_SKIP;
        indent += 8;
        break;
      default:
        return indent;
    }
  }
}

/**
 * End-of-file check.
 *
 * If EOF has been reached, two scenarios are valid:
 *  - The file is empty, in which case the parser is still at the root rule, where `S_EMPTY` is valid.
 *  - The current layout can be ended. This may happen multiple times, since the parser will restart until the last
 *    layout end rule has been parsed.
 * If those cases do not apply, parsing fails.
 */
static Result eof(State *state) {
  if (is_eof(state)) {
    if (SYM(EMPTY)) {
      return finish(EMPTY, "eof");
    }
    Result res = end_or_semicolon("eof", state);
    SHORT_SCANNER;
    return res_fail;
  }
  return res_cont;
}

/**
 * Set the initial indentation at the beginning of the file or module decl to the column of first nonwhite character,
 * then succeed with the dummy symbol `INDENT`.
 *
 * If there is a `module` declaration, this will be handled by the grammar.
 */
static Result initialize(uint32_t column, State *state) {
  if (uninitialized(state)) {
    MARK("initialize", false, state);
    bool match = token("module", state);
    if (match) return res_fail;
    push_indent(column, state);
    return finish(INDENT, "init");
  }
  return res_cont;
}

static Result initialize_init(State *state) {
  if (uninitialized(state)) {
    uint32_t col = column(state);
    if (col == 0) return initialize(col, state);
  };
  return res_cont;
}

/**
 * Consume the body of a cpp directive.
 *
 * Since they can contain escaped newlines, they have to be consumed, after which the parser recurses.
 */
static void cpp_consume(State *state) {
  for (;;) {
    while (PEEK != 0 && !is_newline(PEEK) && PEEK != '\\') S_ADVANCE;
    if (PEEK == '\\') {
      S_ADVANCE;
      S_ADVANCE;
      continue;
    }
    return;
  }
}


/**
 * Parse a cpp directive.
 *
 * This is a workaround for the problem described in `cpp`. It will simply consume all code between `#else` or `#elif`
 * and `#endif`.
 */
static Result cpp_workaround(State *state) {
  if (PEEK == '#') {
    S_ADVANCE;
    if (seq("el", state)) {
      consume_until("#endif", state);
      if (PEEK == 0) {
        Result res = eof(state);
        SHORT_SCANNER;
        return res_fail;
      }
      return finish(CPP, "cpp-else");
    }
    cpp_consume(state);
    MARK("cpp_workaround", false, state);
    return finish(CPP, "cpp");
  }
  return res_cont;
}

/**
 * If the current column is 0, a cpp directive may begin.
 */
static Result cpp(State *state) {
  if (column(state) == 0) {
    return cpp_workaround(state);
  }
  return res_cont;
}

/**
 * End a layout by removing an indentation from the stack, but only if the current column (which should be in the next
 * line after skipping whitespace) is smaller than the layout indent.
 */
static Result dedent(uint32_t indent, State *state) {
  if (less_indent(indent, state)) {
    return layout_end("dedent", state);
  }
  return res_cont;
}

/**
 * Succeed if a `where` on a newline can end a statement or layout (see `is_newline_where`).
 *
 * This is the case after `do` or `of`, where the `where` can be on the same indent.
 */
static Result newline_where(uint32_t indent, State *state) {
  MARK("newline_where", false, state);
  if (SYM (WHERE)) return finish(WHERE, "newline_where");
  return res_fail;
}

/**
 * Succeed for `SEMICOLON` if the indent of the next line is equal to the current layout's.
 */
static Result newline_semicolon(uint32_t indent, State *state) {
  if (SYM(SEMICOLON) && same_indent(indent, state)) {
    return finish(SEMICOLON, "newline_semicolon");
  }
  return res_cont;
}

/**
 * A layout may be closed by an infix operator on the same column as a `do` layout:
 *
 * a :: IO Int
 * a = do a <- pure 5
 *        pure a
 *        >>= pure
 *
 * In this situation, the entire `do` block is the left operand of the `>>=`.
 * The same applies for `infix` functions.
 */
static bool end_on_infix(uint32_t indent, Symbolic type, State *state) {
  return indent_lesseq(indent, state) && (S_OP == type || PEEK == '`');
}

/**
 * End a layout if the next token is an infix operator and the indent is equal to or less than the current layout.
 */
static Result newline_infix(uint32_t indent, Symbolic type, State *state) {
  if (end_on_infix(indent, type, state)) {
    return layout_end("newline_infix", state);
  }
  return res_cont;
}

/**
 * Parse an inline `where` token.
 *
 * Necessary because `is_newline_where` needs to know that no `where` may follow.
 */
static Result where(State *state) {
  if (SYM(WHERE)) {
    MARK("where", false, state);
    return finish(WHERE, "where");
  }
  return layout_end("where", state);
}

/**
 * An `in` token ends the layout openend by a `let` and its nested layouts.
 */
static Result in(State *state) {
  if (SYM(IN)) {
    MARK("in", false, state);
    pop_indent(state);
    return finish(IN, "in");
  }
  return res_fail;
}

/**
 * An `else` token may end a layout opened in the body of a `then`.
 */
static Result else_(State *state) {
  return end_or_semicolon("else", state);
}


/**
 * Consume all characters up to the end of line and succeed with `syms::commment`.
 */
static Result inline_comment(State *state) {
  if (SYM(IN_STRING)) return res_cont;
  for (;;) {
    switch (PEEK) {
      NEWLINE_CASES:
      case 0:
        goto inline_comment_after_skip;
      default:
        S_ADVANCE;
        break;
    }
  }

inline_comment_after_skip:
  MARK("inline_comment", false, state);
  return finish(COMMENT, "inline_comment");
}

/**
 * Parse a sequence of symbolic characters and convert it into the enum `Symbolic`.
 * This decides whether the sequence is an operator or a special case.
 */
static Symbolic read_symop(State *state) {
  String s = read_string(symbolic, state);
  return s_symop(s, state);
}


static Result symop_marked(Symbolic type, State *state) {
  switch (type) {
    case S_INVALID:
      return res_fail;
    case S_IMPLICIT:
      return res_fail;
    case S_COMMENT:
      return inline_comment(state);
    default:
      return res_cont;
  }
}

/**
 * Map a `Symbolic` variant to the appropriate symbol, focusing on operators and their edge cases.
 *
 *  - Star, tilde and minus are only valid as type operators
 *  - Implicit `?` with immediate varid is always invalid, to be parsed by the grammar
 *  - `%` can be a modifier TODO currently only checked for types
 *
 * Otherwise succeed with `VARSYM` if they are valid.
 */

static Result symop(Symbolic type, State *state) {
  MARK("symop", false, state);
  Result res = symop_marked(type, state);
  SHORT_SCANNER;
  res = finish_if_valid(VARSYM, "symop", state);
  SHORT_SCANNER;
  return res_fail;
}

/**
 * Parse an inline comment if the next chars are two or more minuses and the char after the last minus is not
 * symbolic.
 *
 * To be called when it is certain that two minuses cannot succeed as a symbolic operator.
 * Those cases are:
 *   - `LAYOUT_START` is valid
 *   - Operator matching was done already
 */
static Result minus(State *state) {
  if (!seq("--", state)) return res_cont;
  while (PEEK == '-')  S_ADVANCE;
  if (symbolic(PEEK)) return res_fail;
  return inline_comment(state);
}

/**
 * Same to minus but bar.
 */
static Result bar(State *state) {
  if (!seq("|||", state)) return res_cont;
  while (PEEK == '|')  S_ADVANCE;
  if (symbolic(PEEK)) return res_fail;
  return inline_comment(state);
}

static Result raw_string_start(String s, State *state) {
  if (
    SYM(RAW_STRING_START) && 
    strall('#', s.contents) &&
    '"' == PEEK) {
    S_ADVANCE;
    array_push(&state->payload->raw_string_sharp_counts, strlen(s.contents));
    MARK("raw_string_start", false, state);
    return finish(RAW_STRING_START, "raw_string_start");
  }
  return res_cont;
}

static Result raw_string_end(State *state) {
  if (SYM(RAW_STRING_END)) {
    if (!seq("\"", state)) return res_cont;
    uint32_t i = *array_back(&state->payload->raw_string_sharp_counts);
    for (; 0 < i; --i) {
      if (PEEK != '#') return res_fail;
      S_ADVANCE;
    }
    if (PEEK == '#') return res_fail;
    array_pop(&state->payload->raw_string_sharp_counts);
    MARK("raw_string_end", false, state);
    return finish(RAW_STRING_END, "raw_string_end");
  }
  return res_cont;
}

/**
 * Succeed for a comment.
 */
static Result multiline_comment_success(State *state) {
  MARK("multiline_comment", false, state);
  return finish(COMMENT, "multiline_comment");
}

/**
 * See `nested_comment`.
 *
 * Since {- -} comments can be nested arbitrarily, this has to keep track of how many have been openend, so that the
 * outermost comment isn't closed prematurely.
 */
static Result multiline_comment(State *state) {
  uint32_t level = 0;
  for (;;) {
    switch (PEEK) {
      case '{':
        S_ADVANCE;
        if (PEEK == '-') {
          S_ADVANCE;
          level++;
        }
        break;
      case '-':
        S_ADVANCE;
        if (PEEK == '}') {
          S_ADVANCE;
          if (level == 0) return multiline_comment_success(state);
          level--;
        }
        break;
      case 0: {
        Result res = eof(state);
        SHORT_SCANNER;
        return res_fail;
      }
      default:
        S_ADVANCE;
        break;
    }
  }
}

/**
 * When a brace is encountered, it can be an explicitly started layout, a pragma, or a comment. In the latter case, the
 * comment is parsed, otherwise parsing fails to delegate to the corresponding grammar rule.
 */
static Result brace(State *state) {
  if (PEEK != '{') return res_fail;
  S_ADVANCE;
  if (PEEK != '-') return res_fail;
  S_ADVANCE;
  if (PEEK == '#') return res_fail;
  return multiline_comment(state);
}

/**
 * Parse either inline or block comments.
 */
static Result comment(State *state) {
  if (SYM(IN_STRING)) return res_cont;
  switch (PEEK) {
    case '-': {
      Result res = minus(state);
      SHORT_SCANNER;
      return res_fail;
    }
    case '|': {
      Result res = bar(state);
      SHORT_SCANNER;
      return res_fail;
    }
    case '{': {
      Result res = brace(state);
      SHORT_SCANNER;
      return res_fail;
    }
  }
  return res_cont;
}

/**
 * `case` can open a layout in a list:
 *
 * [case a of a -> a, case a of a -> a]
 * [case a of a -> a | a <- a]
 *
 * Commas, vertical bars and closing brackets are able to close those.
 *
 * Because commas can also occur in class layouts at the top level, e.g. in fixity decls, the comma rule has to be
 * parsed here as well.
 */
static Result close_layout_in_list(State *state) {
  if (0 < state->token.size) return res_cont;
  switch (PEEK) {
    case ']': {
      if (SYM(LAYOUT_END)) {
        pop_indent(state);
        return finish(LAYOUT_END, "bracket");
      }
      break;
    }
    case ',': {
      S_ADVANCE;
      if (SYM(COMMA)) {
        MARK("comma", false, state);
        return finish(COMMA, "comma");
      }
      Result res = layout_end("comma", state);
      SHORT_SCANNER;
      return res_fail;
    }
  }
  return res_cont;
}

/**
 * Parse special tokens before the first newline that can't be reliably detected by tree-sitter:
 *
 *   - `where` here is just for the actual valid token
 *   - `in` closes a layout when inline
 *   - `)` can end the layout of an `of`
 *   - symbolic operators are complicated to implement with regex
 *   - '[' can be a list or a quasiquote
 */
static Result inline_tokens(State *state) {
  uint32_t c = PEEK;
  bool is_symbolic = false;

  if (varid_start_char(c)) {
    read_token(state);
    if (is_token("_", state)) return res_cont;
    if (is_token("where", state)) return where(state);
    if (is_token("in", state)) return in(state);
    if (is_token("else", state)) return else_(state);

    for (int i = 0; i < sizeof(keywords)/sizeof(keywords[0]); ++i) {
      if (is_token(keywords[i], state)) return res_cont;
    }
        
    if (SYM(VARID)) {
      MARK("varid", false, state);
      return finish(VARID, "varid");
    }
  }

  switch (c) {
    case ')': {
      Result res = layout_end(")", state);
      SHORT_SCANNER;
      return res_fail;
    }
    case '[': {
      return res_cont;
    }
    case '#': {
      String s = read_string(symbolic, state);
      Result res = raw_string_start(s, state);
      SHORT_SCANNER;
      return symop(s_symop(s,state), state);
    }
    case '"': {
      Result res = raw_string_end(state);
      SHORT_SCANNER;
      return res_fail;
    }
    case '{': {
      Result res = comment(state);
      SHORT_SCANNER;
    }
    SYMBOLIC_CASES: {
      is_symbolic = true;
    }
  }
  if (is_symbolic || unicode_symbol(c)) {
    Symbolic s = read_symop(state);
    return symop(s, state);
  }
  return close_layout_in_list(state);
}

/**
 * If the symbol `LAYOUT_START` is valid, starting a new layout is almost always indicated.
 *
 * If the next character is a left brace, it is either a comment, pragma or an explicit layout. In the comment case, the
 * it must be parsed here.
 * If the next character is a minus, it might be a comment.
 *
 * In all of those cases, the layout can't be started now. In the comment and pragma case, it will be started in the
 * next run.
 *
 * This pushes the indentation of the first non-whitespace character onto the stack.
 */
static Result layout_start(uint32_t column, State *state) {
  if (SYM(LAYOUT_START)) {
    switch (PEEK) {
      case '-': {
        Result res = minus(state);
        SHORT_SCANNER;
        break;
      }
      default:
        break;
    }
    push_indent(column, state);
    return finish(LAYOUT_START, "layout_start");
  }
  return res_cont;
}

/**
 * After a layout has ended, the originator might need to be terminated by semicolon as well, but since the layout end
 * advances until the next line, it cannot be done in the newline checks.
 *
 * This can happen, for example, with nested `do` layouts:
 *
 * f = do
 *   a <- b
 *   do c <- d
 *      e
 *   f
 *
 * Here, when the inner `do`'s  layout is ended, the next step is started at `f`, but the outer `do`'s layout expects a
 * semicolon. Since `f` is on the same indent as the outer `do`'s layout, this parser matches.
 *
 * `|` needs special care because an alternative pattern match does not finish the statement.
 * _ = do
 *   Just a <- b
 *   | Nothing => ...
 */
static Result post_end_semicolon(uint32_t column, State *state) {
  if ('|' == PEEK) return res_cont;
  return SYM(SEMICOLON) && indent_lesseq(column, state)
    ? finish(SEMICOLON, "post_end_semicolon")
    : res_cont;
}

/**
 * Like `post_end_semicolon`, but for layout end.
 */
static Result repeat_end(uint32_t column, State *state) {
  if (SYM(LAYOUT_END) && less_indent(column, state)) {
    return layout_end("repeat_end", state);
  }
  return res_cont;
}

/**
 * Rules that decide based on the indent of the next line.
 */
static Result newline_indent(uint32_t indent, State *state) {
  Result res = dedent(indent, state);
  SHORT_SCANNER;
  res = close_layout_in_list(state);
  SHORT_SCANNER;
  return newline_semicolon(indent, state);
}

/**
 * Rules that decide based on the first token on the next line.
 */
static Result newline_token(uint32_t indent, State *state) {
  uint32_t c = PEEK;
  bool is_symbolic = false;
  switch (c) {
    SYMBOLIC_CASES:
    case '`': {
      is_symbolic = true;
    }
  }
  if (is_symbolic || unicode_symbol(c)) {
    Symbolic s = read_symop(state);
    Result res = newline_infix(indent, s, state);
    SHORT_SCANNER;
    return res_fail;
  }
  read_token(state);
  if (is_token("where", state)) return newline_where(indent, state);
  if (is_token("in", state)) return in(state);
  return res_cont;
}

/**
 * To be called after parsing a newline, with the indent of the next line as argument.
 */
static Result newline(uint32_t indent, State *state) {
  Result res = eof(state);
  SHORT_SCANNER;
  res = initialize(indent, state);
  SHORT_SCANNER;
  res = layout_restart(indent, state);
  SHORT_SCANNER;
  res = cpp(state);
  SHORT_SCANNER;
  res = comment(state);
  SHORT_SCANNER;
  res = newline_token(indent, state);
  SHORT_SCANNER;
  return newline_indent(indent, state);
}

/**
 * Parsers that have to run when the next non-space character is not a newline:
 *
 *   - Layout start
 *   - ending nested layouts at the same position
 *   - symbolic operators
 *   - Tokens `where`, `in`, `$`, `)`, `]`, `,`
 *   - comments
 */
static Result immediate(uint32_t column, State *state) {
  Result res = layout_start(column, state);
  SHORT_SCANNER;
  res = post_end_semicolon(column, state);
  SHORT_SCANNER;
  res = repeat_end(column, state);
  SHORT_SCANNER;
  return inline_tokens(state);
}


/**
 * If a dot is neither preceded nor succeeded by whitespace, it may be parsed as a qualified module dot or a field
 * projection.
 *
 * The preceding space is ensured by sequencing this parser before `skipspace` in `init`.
 * Since this parser cannot look back to see whether the preceding name is a conid, this has to be ensured by the
 * grammar, represented here by the requirement of a valid symbol `DOT`.
 *
 * Since the dot is consumed here, the alternative interpretation, a `VARSYM`, has to be emitted here.
 */
static Result dot(State *state) {
  if (SYM(DOT) && '.' == PEEK) {
    S_ADVANCE;
    MARK("dot", false, state);
    if (iswalpha(PEEK) || '(' == PEEK) return finish(DOT, "dot");
    if (SYM(VARSYM)) {
      Symbolic s = read_symop(state);
      MARK("symop", false, state);
      return finish(VARSYM, "symop");
    }
  }
  return res_cont;
}

/**
 * Parsers that have to run _before_ parsing whitespace:
 *
 *   - Error check
 *   - Indent stack initialization
 *   - Qualified module dot (leading whitespace would mean it would be `(.)`)
 *   - cpp
 */
static Result init(State *state) {
  Result res = eof(state);
  SHORT_SCANNER;
  res = after_error(state) ? res_fail : res_cont;
  SHORT_SCANNER;
  res = initialize_init(state);
  SHORT_SCANNER;
  res = dot(state);
  SHORT_SCANNER;
  res = cpp(state);
  SHORT_SCANNER;
  return res_cont;
}

/**
 * The main parser checks whether the first non-space character is a newline and delegates accordingly.
 */
static Result scan_main(State *state) {
  skipspace(state);
  Result res = eof(state);
  SHORT_SCANNER;
  MARK("main", false, state);

  if (is_newline(PEEK)) {
    S_SKIP;
    uint32_t indent = count_indent(state);
    return newline(indent, state);
  }

  uint32_t col = column(state);
  return immediate(col, state);
}

/**
 * The entry point to the parser.
 */
static Result scan_all(State *state) {
  Result res = init(state);
  SHORT_SCANNER;
  return scan_main(state);
}

// --------------------------------------------------------------------------------------------------------
// Evaluation
// --------------------------------------------------------------------------------------------------------

/**
  * Helper that consume_if all characters up to the next whitespace, for debugging after a run.
  *
  * Note: This may break the parser, since not all paths use `mark`.
  */
#ifdef DEBUG
static void debug_lookahead(State *state) {
  bool first = true;
  for (;;) {
    if (isws(PEEK) || PEEK == 0) break;
    else {
      if (first) DEBUG_PRINTF("next: ");
      DEBUG_PRINTF("%c\n", PEEK);
      S_ADVANCE;
      first = false;
    }
  }
}
#endif


/**
  * The main function of the parsing machinery, executing the parser by passing in the initial state and analyzing the
  * result.
  *
  * If the parser concluded with success, the `result_symbol` attribute of the lexer is set, by which the parsed symbol
  * is communicated to tree-sitter, and `true` is returned, indicating to tree-sitter to use the result.
  *
  * If the parser concluded with failure, no `result_symbol` is set and `false` is returned.
  *
  * If the parser did _not_ conclude, i.e. all steps finished with `cont`, a failure is reported as well.
  *
  * If the `DEBUG_NEXT_TOKEN` flag is set, the next token will be printed.
  */
static bool eval(Result (*chk)(State *state), State *state) {
  Result result = chk(state);
#ifdef DEBUG_NEXT_TOKEN
  debug_lookahead(state);
#endif
  if (result.finished && result.sym != FAIL) {
#ifdef DEBUG
    // TODO(414owen) can names[] fail?
    DEBUG_PRINTF("result: %s, ", sym_names[result.sym]);
    if (state->marked == -1) {
      DEBUG_PRINTF("%d\n", column(state));
    } else {
      DEBUG_PRINTF("%s@%d\n", state->marked_by, state->marked);
    }
#endif
    state->lexer->result_symbol = result.sym;
    return true;
  } else return false;
}

// --------------------------------------------------------------------------------------------------------
// API
// --------------------------------------------------------------------------------------------------------

/**
 * This function allocates the persistent state of the parser that is passed into the other API functions.
 */
void *tree_sitter_idris_external_scanner_create() {
  void *res = calloc(sizeof(Payload), 1);
  return res;
}

/**
 * Main logic entry point.
 */
bool tree_sitter_idris_external_scanner_scan(Payload *payload, TSLexer *lexer, const bool *syms) {
  State state = {
    .lexer = lexer,
    .symbols = syms,
    .payload = payload
  };
#ifdef DEBUG
  debug_state(&state);
  if (state.needs_free) free(state.marked_by);
#endif
  return eval(scan_all, &state);
}

/**
 * Copy the current state to another location for later reuse.
 * This is normally more complex, but since this parser's state constists solely of a vector of integers, it can just be
 * copied.
 */
unsigned tree_sitter_idris_external_scanner_serialize(Payload *payload, char *buffer) {
  unsigned size = 0;
  {
    unsigned len = payload->indents.size;
    if (size + 1 + len > TREE_SITTER_SERIALIZATION_BUFFER_SIZE) return 0;
    buffer[size++] = len;
    memcpy(&buffer[size], payload->indents.contents, len);
    size += len;
  }
  {
    unsigned len = payload->raw_string_sharp_counts.size;
    if (size + 1 + len > TREE_SITTER_SERIALIZATION_BUFFER_SIZE) return 0;
    buffer[size++] = len;
    memcpy(&buffer[size], payload->raw_string_sharp_counts.contents, len);
    size += len;
  }
  return size;
}

/**
 * Load another parser state into the currently active state.
 * `payload` is the state of the previous parser execution, while `buffer` is the saved state of a different position
 * (e.g. when doing incremental parsing).
 */
void tree_sitter_idris_external_scanner_deserialize(Payload *payload, char *buffer, unsigned length) {
  if(length == 0) return;

  unsigned size = 0;
  {
    unsigned len = buffer[size++];
    if (len > 0) {
      array_reserve(&payload->indents, len);
      memcpy(payload->indents.contents, &buffer[size], len);
      payload->indents.size = len;
    }
    size += len;
  }
  {
    unsigned len = buffer[size++];
    if (len > 0) {
      array_reserve(&payload->raw_string_sharp_counts, len);
      memcpy(payload->raw_string_sharp_counts.contents, &buffer[size], len);
     payload->raw_string_sharp_counts.size = len;
    }
    size += len;
  }
  assert(size == length);
}

/**
 * Destroy the state.
 */
void tree_sitter_idris_external_scanner_destroy(Payload *payload) {
  free(payload);
}
