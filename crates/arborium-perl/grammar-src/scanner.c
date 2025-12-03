#include "tree_sitter/array.h"
#include "tree_sitter/parser.h"
#include "tsp_unicode.h"

// grumble grumble no stdlib
static char *tsp_strchr(register const char *s, int c) {
  do {
    if (*s == c) {
      return (char *)s;
    }
  } while (*s++);
  return (0);
}
/* Set this to #define instead to enable debug printing */
#undef DEBUGGING

/* for debug */
#ifdef DEBUGGING
#include <stdio.h>
#define DEBUG(fmt, ...) fprintf(stderr, "scanner.c:%d DEBUG: " fmt, __LINE__, __VA_ARGS__)
#else
#define DEBUG(fmt, ...)
#endif

#define streq(a, b) (strcmp(a, b) == 0)

#include <wctype.h>

enum TokenType {
  /* non-ident tokens */
  TOKEN_APOSTROPHE,
  TOKEN_DOUBLE_QUOTE,
  TOKEN_BACKTICK,
  TOKEN_SEARCH_SLASH,
  NO_TOKEN_SEARCH_SLASH_PLZ,
  TOKEN_OPEN_READLINE_BRACKET,
  TOKEN_OPEN_FILEGLOB_BRACKET,
  PERLY_SEMICOLON,
  PERLY_HEREDOC,
  TOKEN_CTRL_Z,
  /* immediates */
  TOKEN_QUOTELIKE_BEGIN,
  TOKEN_QUOTELIKE_MIDDLE_CLOSE,
  TOKEN_QUOTELIKE_MIDDLE_SKIP,
  TOKEN_QUOTELIKE_END_ZW,
  TOKEN_QUOTELIKE_END,
  TOKEN_Q_STRING_CONTENT,
  TOKEN_QQ_STRING_CONTENT,
  TOKEN_ESCAPE_SEQUENCE,
  TOKEN_ESCAPED_DELIMITER,
  TOKEN_DOLLAR_IN_REGEXP,
  TOKEN_POD,
  TOKEN_GOBBLED_CONTENT,
  TOKEN_ATTRIBUTE_VALUE_BEGIN,
  TOKEN_ATTRIBUTE_VALUE,
  TOKEN_PROTOTYPE,
  TOKEN_SIGNATURE_START,
  TOKEN_HEREDOC_DELIM,
  TOKEN_COMMAND_HEREDOC_DELIM,
  TOKEN_HEREDOC_START,
  TOKEN_HEREDOC_MIDDLE,
  TOKEN_HEREDOC_END,
  TOKEN_FAT_COMMA_AUTOQUOTED,
  TOKEN_FILETEST,
  TOKEN_BRACE_AUTOQUOTED,
  /* zero-width lookahead tokens */
  TOKEN_BRACE_END_ZW,
  TOKEN_DOLLAR_IDENT_ZW,
  TOKEN_NO_INTERP_WHITESPACE_ZW,
  /* zero-width high priority token */
  TOKEN_NONASSOC,
  /* error condition is always last */
  TOKEN_ERROR
};

#define MAX_TSPSTRING_LEN 8
/* this is a arbitrary string where we only care about the first
 * MAX_TSPSTRING_LEN chars */
typedef struct {
  int length;
  int32_t contents[MAX_TSPSTRING_LEN];
} TSPString;

/* we record the length, b/c that's still relevant for our cheapo comparison */
static void tspstring_push(TSPString *s, int32_t c) {
  if (s->length++ < MAX_TSPSTRING_LEN) s->contents[s->length - 1] = c;
}

static bool tspstring_eq(TSPString *s1, TSPString *s2) {
  if (s1->length != s2->length) return false;
  int max_len = s1->length < MAX_TSPSTRING_LEN ? s1->length : MAX_TSPSTRING_LEN;
  for (int i = 0; i < max_len; i++) {
    if (s1->contents[i] != s2->contents[i]) return false;
  }
  return true;
}

static void tspstring_reset(TSPString *s) { s->length = 0; }

static int32_t close_for_open(int32_t c) {
  switch (c) {
    case '(':
      return ')';
    case '[':
      return ']';
    case '{':
      return '}';
    case '<':
      return '>';
    /* TODO: Add aaaaalll the Unicode ones */
    default:
      return 0;
  }
}

typedef struct {
  int32_t open, close, count;
} TSPQuote;

static TSPQuote tspquote_new() { return (TSPQuote){0, 0, 0}; }

enum HeredocState { HEREDOC_NONE, HEREDOC_START, HEREDOC_UNKNOWN, HEREDOC_CONTINUE, HEREDOC_END };
typedef struct {
  Array(TSPQuote) quotes;
  /* heredoc - we need to track if we should start the heredoc, if it's
   * interpolating, how many chars the delimiter is and what the delimiter is */
  bool heredoc_interpolates, heredoc_indents;
  enum HeredocState heredoc_state;
  TSPString heredoc_delim;
} LexerState;

static void lexerstate_push_quote(LexerState *state, int32_t opener) {
  TSPQuote q = tspquote_new();
  // if it's not a paired delim, we get a 0 here
  int32_t closer = close_for_open(opener);
  q.close = closer ? closer : opener;
  q.open = closer ? opener : 0;
  q.count = 0;
  array_push(&state->quotes, q);
}

// consider `qq( hi ${\q(there)} $sner )`; we MUST start our check from the END of the
// quote stack
static int32_t lexerstate_is_quote_opener(LexerState *state, int32_t check) {
  // this will loop over all of the TSPQuotes in the state's quote and check if
  // it has an opener (like '{'). we return i + 1 b/c 0 is a valid index + we need that
  // for falsy
  for (int i = state->quotes.size - 1; i >= 0; i--) {
    TSPQuote *q = array_get(&state->quotes, i);
    if (q->open && check == q->open) return i + 1;
  }
  return 0;
}

static void lexerstate_saw_opener(LexerState *state, int32_t idx) {
  // we must loop over all the quotes and increment the count for the first one
  // that matches the codepoint passed in c
  TSPQuote *q = array_get(&state->quotes, idx - 1);
  q->count++;
  DEBUG("Got a opener for %c, we are at %d \n", q->open, q->count);
}

static int32_t lexerstate_is_quote_closer(LexerState *state, int32_t c) {
  // same as above, just for the closer
  for (int i = state->quotes.size - 1; i >= 0; i--) {
    TSPQuote *q = array_get(&state->quotes, i);
    if (q->close && c == q->close) return i + 1;
  }
  return 0;
}

static void lexerstate_saw_closer(LexerState *state, int32_t idx) {
  // same as above, just for the closer
  TSPQuote *q = array_get(&state->quotes, idx - 1);
  if (q->count) {
    q->count--;
    DEBUG("Got a closer, we are at %d \n", q->count);
  }
}

static bool lexerstate_is_quote_closed(LexerState *state, int32_t idx) {
  TSPQuote *q = array_get(&state->quotes, idx - 1);
  return !q->count;
}

static void lexerstate_pop_quote(LexerState *state, int32_t idx) {
  array_erase(&state->quotes, idx - 1);
}

static bool lexerstate_is_paired_delimiter(LexerState *state) {
  TSPQuote *q = array_back(&state->quotes);
  return !!q->open;
}

//   in order to emulate a sublex, we basically need to have a new escape type character
//   that can pop up anywhere so long as there's an active string, to escape the string
//   char
//   with nested strings of the same type, it escapes them top down, observe:
//       s/things ${\q\/hello\/} / ${\q[hi again]} /;
//   the outer string gets a single escape, which is then invisible to the inner string
//
//   here's another relevant example:
//        qq( ${\\('hi')} );
//   the backslash needs to be escaped so it doesn't escape the quote_char from being counted for
//   pairs
//
//   ooh, a better solution to the backslash issue is that we should just add it to the
//   lexer, which will let us check if there's some quote_{opener,closer} in front of it
//   (which only happens inside of a string)

// we can match perl's behavior if we are intentionally destructive here and find our match

static void lexerstate_add_heredoc(LexerState *state, TSPString *delim, bool interp, bool indent) {
  state->heredoc_delim = *delim;
  state->heredoc_interpolates = interp;
  state->heredoc_indents = indent;
  state->heredoc_state = HEREDOC_START;
}

static void lexerstate_finish_heredoc(LexerState *state) {
  state->heredoc_delim.length = 0;
  state->heredoc_state = HEREDOC_NONE;
}

#define ADVANCE_C                                                             \
  do {                                                                        \
    if (lexer->lookahead == '\r')                                             \
      DEBUG("> advance U+%04X = \\r\n", lexer->lookahead);                    \
    else if (lexer->lookahead == '\n')                                        \
      DEBUG("> advance U+%04X = \\n\n", lexer->lookahead);                    \
    else                                                                      \
      DEBUG("> advance U+%04X = '%c'\n", lexer->lookahead, lexer->lookahead); \
    lexer->advance(lexer, false);                                             \
    c = lexer->lookahead;                                                     \
  } while (0)

#define TOKEN(type)              \
  do {                           \
    DEBUG("token(%s)\n", #type); \
    lexer->result_symbol = type; \
    return true;                 \
  } while (0)

#define MARK_END                        \
  do {                                  \
    lexer->mark_end(lexer);             \
    DEBUG("marking end of token\n", 0); \
  } while (0)

static void skip_whitespace(TSLexer *lexer) {
  while (1) {
    int32_t c = lexer->lookahead;
    if (!c) return;
    if (is_tsp_whitespace(c)) lexer->advance(lexer, true);
    /* continue */
    else
      return;
  }
}

static void skip_ws_to_eol(TSLexer *lexer) {
  while (1) {
    int32_t c = lexer->lookahead;
    if (!c) return;
    if (is_tsp_whitespace(c)) {
      lexer->advance(lexer, true);
      // return after eating the newline
      if (c == '\n') return;
    } else
      return;
  }
}

static void _skip_chars(TSLexer *lexer, int maxlen, const char *allow) {
  int32_t c = lexer->lookahead;

  while (maxlen)
    if (!c)
      return;
    else if (tsp_strchr(allow, c)) {
      ADVANCE_C;
      if (maxlen > 0) maxlen--;
    } else
      break;
}
#define skip_hexdigits(lexer, maxlen) _skip_chars(lexer, maxlen, "0123456789ABCDEFabcdef")
#define skip_digits(lexer, maxlen) _skip_chars(lexer, maxlen, "0123456789")
#define skip_octdigits(lexer, maxlen) _skip_chars(lexer, maxlen, "01234567")

static void skip_braced(TSLexer *lexer) {
  int32_t c = lexer->lookahead;

  if (c != '{') return;

  ADVANCE_C;
  while (c && c != '}') ADVANCE_C;

  ADVANCE_C;
}

static bool isidfirst(int32_t c) { return c == '_' || is_tsp_id_start(c); }

static bool isidcont(int32_t c) { return c == '_' || is_tsp_id_continue(c); }

// in any interpolatable case, we wanna stop parsing on these chars
// there's a matching rule in the grammar to catch when it doesn't match a rule
static bool is_interpolation_escape(int32_t c) { return c < 256 && tsp_strchr("$@-[{\\", c); }

unsigned int tree_sitter_perl_external_scanner_serialize(void *payload, char *buffer) {
  LexerState *state = payload;
  size_t size = 0;

  // Serialize the quotes array
  size_t quote_count = state->quotes.size;
  if (quote_count > UINT8_MAX) {
    quote_count = UINT8_MAX;
  }
  buffer[size++] = (char)quote_count;

  if (quote_count > 0) {
    memcpy(&buffer[size], state->quotes.contents, quote_count * sizeof(TSPQuote));
  }
  size += quote_count * sizeof(TSPQuote);

  // Serialize the heredoc state and delimiter
  buffer[size++] = (char)state->heredoc_interpolates;
  buffer[size++] = (char)state->heredoc_indents;
  buffer[size++] = (char)state->heredoc_state;
  memcpy(&buffer[size], &state->heredoc_delim, sizeof(TSPString));
  size += sizeof(TSPString);

  return size;
}

void tree_sitter_perl_external_scanner_deserialize(void *payload, const char *buffer,
                                                   unsigned int length) {
  LexerState *state = payload;
  size_t size = 0;
  array_delete(&state->quotes);
  if (length > 0) {
    // Deserialize the quotes array
    size_t quote_count = (uint8_t)buffer[size++];
    if (quote_count > 0) {
      array_reserve(&state->quotes, quote_count);
      state->quotes.size = quote_count;
      memcpy(state->quotes.contents, &buffer[size], quote_count * sizeof(TSPQuote));
      size += quote_count * sizeof(TSPQuote);
    }

    // Deserialize the heredoc state and delimiter
    state->heredoc_interpolates = (bool)buffer[size++];
    state->heredoc_indents = (bool)buffer[size++];
    state->heredoc_state = (enum HeredocState)buffer[size++];

    memcpy(&state->heredoc_delim, &buffer[size], sizeof(TSPString));
    size += sizeof(TSPString);
  }
}

bool tree_sitter_perl_external_scanner_scan(void *payload, TSLexer *lexer,
                                            const bool *valid_symbols) {
  LexerState *state = payload;

  bool is_ERROR = valid_symbols[TOKEN_ERROR];
  bool skipped_whitespace = false;

  int32_t c = lexer->lookahead;

  if (!is_ERROR && valid_symbols[TOKEN_GOBBLED_CONTENT]) {
    while (!lexer->eof(lexer)) ADVANCE_C;

    TOKEN(TOKEN_GOBBLED_CONTENT);
  }

  /* we use this to force tree-sitter to stay on the error branch of a nonassoc
   * operator */
  if (!is_ERROR && valid_symbols[TOKEN_NONASSOC]) TOKEN(TOKEN_NONASSOC);

  // this is whitespace sensitive, so it must go before any whitespace is
  // skipped
  if (valid_symbols[TOKEN_HEREDOC_MIDDLE] && !is_ERROR) {
    DEBUG("Beginning heredoc contents\n", 0);
    if (state->heredoc_state != HEREDOC_CONTINUE) {
      TSPString line;
      // read as many lines as we can
      while (!lexer->eof(lexer)) {
        tspstring_reset(&line);
        // interpolating heredocs may need to stop in the middle of the line;
        // indented heredocs may START in the beggining of a known line
        bool is_valid_start_pos =
            state->heredoc_state == HEREDOC_END || lexer->get_column(lexer) == 0;
        bool saw_escape = false;
        DEBUG("Starting loop at col %d\n", lexer->get_column(lexer));
        if (is_valid_start_pos && state->heredoc_indents) {
          DEBUG("Skipping initial whitespace in heredoc\n", 0);
          skip_whitespace(lexer);
          c = lexer->lookahead;
        }
        // we may be doing lookahead now
        MARK_END;
        // read the whole line, b/c we want it
        while (c != '\n' && !lexer->eof(lexer)) {
          // we need special handling for windows line ending, b/c we can't
          // count it in our lookahead
          if (c == '\r') {
            ADVANCE_C;
            if (c == '\n') break;
            tspstring_push(&line, '\r');
          }
          tspstring_push(&line, c);
          if (c == '$' || c == '@' || c == '\\') saw_escape = true;
          ADVANCE_C;
        }
        DEBUG("got length %d, want length %d\n", line.length, state->heredoc_delim.length);
        if (is_valid_start_pos && tspstring_eq(&line, &state->heredoc_delim)) {
          // if we've read already, we return everything up until now
          if (state->heredoc_state != HEREDOC_END) {
            state->heredoc_state = HEREDOC_END;
            TOKEN(TOKEN_HEREDOC_MIDDLE);
          }
          MARK_END;
          lexerstate_finish_heredoc(state);
          TOKEN(TOKEN_HEREDOC_END);
        }
        if (saw_escape && state->heredoc_interpolates) {
          // we'll repeat this line in continue mode where we'll pause midline
          state->heredoc_state = HEREDOC_CONTINUE;
          TOKEN(TOKEN_HEREDOC_MIDDLE);
        }
        // eat the \n and loop again; can't skip whitespace b/c the next line
        // may care
        ADVANCE_C;
      }
    } else {
      DEBUG("Entering heredoc continue mode\n", 0);
      // handle the continue case; read ahead until we get a \n or an escape
      bool saw_chars = false;
      while (1) {
        if (is_interpolation_escape(c)) {
          MARK_END;
          break;
        }
        if (c == '\n') {
          MARK_END;
          state->heredoc_state = HEREDOC_UNKNOWN;
          TOKEN(TOKEN_HEREDOC_MIDDLE);
        }
        saw_chars = true;
        ADVANCE_C;
      }
      if (saw_chars) TOKEN(TOKEN_HEREDOC_MIDDLE);
    }
  }

  if (iswspace(c) && valid_symbols[TOKEN_NO_INTERP_WHITESPACE_ZW]) {
    TOKEN(TOKEN_NO_INTERP_WHITESPACE_ZW);
  }
  skip_ws_to_eol(lexer);
  /* heredocs override everything, so they must be here before */
  if (valid_symbols[TOKEN_HEREDOC_START]) {
    if (state->heredoc_state == HEREDOC_START && lexer->get_column(lexer) == 0) {
      state->heredoc_state = HEREDOC_UNKNOWN;
      TOKEN(TOKEN_HEREDOC_START);
    }
  }

  if (!is_ERROR && valid_symbols[TOKEN_ATTRIBUTE_VALUE_BEGIN] && c == '(') {
    /* This has to be an external scanner symbol so it takes precedence over
     * signature_or_prototype */
    TOKEN(TOKEN_ATTRIBUTE_VALUE_BEGIN);
  }

  if (!is_ERROR && valid_symbols[TOKEN_ATTRIBUTE_VALUE]) {
    DEBUG("Attribute value started...\n", 0);

    int delimcount = 0;
    while (!lexer->eof(lexer)) {
      if (c == '\\') {
        ADVANCE_C;
        /* ignore the next char */
      } else if (c == '(')
        delimcount++;
      else if (c == ')') {
        if (delimcount)
          delimcount--;
        else {
          /* Do not consume the ')' */
          break;
        }
      }

      ADVANCE_C;
    }

    TOKEN(TOKEN_ATTRIBUTE_VALUE);
  }

  if (is_tsp_whitespace(c)) {
    // NOTE - the first whitespace skipping is skip_ws_to_eol over in heredoc
    // handling
    skipped_whitespace = true;
    skip_whitespace(lexer);
    c = lexer->lookahead;
  }

  // CTRL-Z must be here, b/c it cares about whitespace
  if (c == 26 && valid_symbols[TOKEN_CTRL_Z]) TOKEN(TOKEN_CTRL_Z);

  if (valid_symbols[PERLY_SEMICOLON]) {
    if (c == '}' || lexer->eof(lexer)) {
      // do a PERLY_SEMICOLON unless we're in brace autoquoting
      if (is_ERROR || !valid_symbols[TOKEN_BRACE_END_ZW]) {
        DEBUG("Fake PERLY_SEMICOLON at end-of-scope\n", 0);
        // no advance
        TOKEN(PERLY_SEMICOLON);
      }
    }
  }
  if (lexer->eof(lexer)) return false;

  if (valid_symbols[TOKEN_OPEN_FILEGLOB_BRACKET] || valid_symbols[TOKEN_OPEN_READLINE_BRACKET] || valid_symbols[PERLY_HEREDOC]) {
      if (c == '<') {
          ADVANCE_C;
          MARK_END;
          // ah, we have a heredoc; let's just go down to that section then
          if (c == '<') goto heredoc_token_handling;
          if (c == '$') ADVANCE_C;
          // we now zoooom as many ident chars as we can
          while (isidcont(c)) ADVANCE_C;
          // if ident chars took us until the closing `>` then we're readline FILEHANDLE
          if (c == '>') TOKEN(TOKEN_OPEN_READLINE_BRACKET);
          // otherwise we're a fileglob operator, and we set up our string parse + be excellent
          lexerstate_push_quote(state, '<');
          TOKEN(TOKEN_OPEN_FILEGLOB_BRACKET);
      }

  }

  if (valid_symbols[TOKEN_DOLLAR_IDENT_ZW]) {
    // false on word chars, another dollar or {
    if (!isidcont(c) && !tsp_strchr("${", c)) {
      if (c == ':') {
        // NOTE - it's a syntax error to do $$:, so that's why we return
        // dollar_ident_zw in that case
        MARK_END;
        ADVANCE_C;
        if (c == ':') {
          // we can safely bail out here b/c we know that $:: is handled in the
          // grammar, and that's the only place we can ever get to this rule
          // here
          return false;
        }
      }
      TOKEN(TOKEN_DOLLAR_IDENT_ZW);
    }
  }

  if ((valid_symbols[TOKEN_SEARCH_SLASH] && c == '/') &&
      !valid_symbols[NO_TOKEN_SEARCH_SLASH_PLZ]) {
    ADVANCE_C;
    MARK_END;

    if (c != '/') {
      lexerstate_push_quote(state, '/');
      TOKEN(TOKEN_SEARCH_SLASH);
    }
    /* if we didn't get a search-slash, we fall back to the main parser */
    return false;
  }
  if (valid_symbols[TOKEN_APOSTROPHE] && c == '\'') {
    ADVANCE_C;
    lexerstate_push_quote(state, '\'');
    TOKEN(TOKEN_APOSTROPHE);
  }
  if (valid_symbols[TOKEN_DOUBLE_QUOTE] && c == '"') {
    ADVANCE_C;
    lexerstate_push_quote(state, '"');
    TOKEN(TOKEN_DOUBLE_QUOTE);
  }
  if (valid_symbols[TOKEN_BACKTICK] && c == '`') {
    ADVANCE_C;
    lexerstate_push_quote(state, '`');
    TOKEN(TOKEN_BACKTICK);
  }

  if (valid_symbols[TOKEN_DOLLAR_IN_REGEXP] && c == '$') {
    DEBUG("Dollar in regexp\n", 0);
    ADVANCE_C;

    /* Accept this literal dollar if it's followed by closing delimiter */
    if (lexerstate_is_quote_closer(state, c)) TOKEN(TOKEN_DOLLAR_IN_REGEXP);

    /* Several other situations are interpreted literally */
    switch (c) {
      case '(':
      case ')':
      case '|':
        TOKEN(TOKEN_DOLLAR_IN_REGEXP);
    }

    return false;
  }

  if (valid_symbols[TOKEN_POD]) {
    int column = lexer->get_column(lexer);
    if (column == 0 && c == '=') {
      DEBUG("POD started...\n", 0);

      /* Keep going until the linefeed after a line beginning `=cut` */
      static const char *cut_marker = "=cut";
      int stage = -1;

      while (!lexer->eof(lexer)) {
        if (c == '\r')
          ; /* ignore */
        else if (stage < 1 && c == '\n')
          stage = 0;
        else if (stage >= 0 && stage < 4 && c == cut_marker[stage])
          stage++;
        else if (stage == 4 && (c == ' ' || c == '\t'))
          stage = 5;
        else if (stage == 4 && c == '\n')
          stage = 6;
        else
          stage = -1;

        if (stage > 4) break;

        ADVANCE_C;
      }
      if (stage < 6)
        while (!lexer->eof(lexer)) {
          if (c == '\n') break;

          ADVANCE_C;
        }
      /* If we got this far then either we reached stage 6, or we're at EOF */
      TOKEN(TOKEN_POD);
    }
  }

  /* By now if we haven't recognised the token we shouldn't attempt to look
   * for the remaining ones when in an error condition */
  if (is_ERROR) return false;

  if (valid_symbols[TOKEN_HEREDOC_DELIM] || valid_symbols[TOKEN_COMMAND_HEREDOC_DELIM]) {
    // by default, indentation is false and interpolation is true
    bool should_indent = false;
    bool should_interpolate = true;

    TSPString delim;
    tspstring_reset(&delim);
    if (!skipped_whitespace) {
      if (c == '~') {
        ADVANCE_C;
        should_indent = true;
      }
      if (c == '\\') {
        ADVANCE_C;
        should_interpolate = false;
      }
      if (isidfirst(c)) {
        while (isidcont(c)) {
          tspstring_push(&delim, c);
          ADVANCE_C;
        }
        lexerstate_add_heredoc(state, &delim, should_interpolate, should_indent);
        TOKEN(TOKEN_HEREDOC_DELIM);
      }
    }
    // if we picked up a ~ before, we may have to skip to hit the quote
    if (should_indent) {
      skip_whitespace(lexer);
      c = lexer->lookahead;
    }
    // if we picked up a \ before, we cannot allow even an immediate quote
    if (should_interpolate && (c == '\'' || c == '"' || c == '`')) {
      int delim_open = c;
      should_interpolate = c != '\'';
      ADVANCE_C;
      while (c != delim_open && !lexer->eof(lexer)) {
        // backslashes escape the quote char and nothing else, not even a
        // backslash
        if (c == '\\') {
          int to_add = c;
          ADVANCE_C;
          if (c == delim_open) {
            to_add = delim_open;
            ADVANCE_C;
          }
          tspstring_push(&delim, to_add);
        } else {
          tspstring_push(&delim, c);
          ADVANCE_C;
        }
      }
      if (delim.length > 0) {
        // gotta eat that delimiter
        ADVANCE_C;
        // gotta null terminate up in here
        lexerstate_add_heredoc(state, &delim, should_interpolate, should_indent);
        if (delim_open == '`') TOKEN(TOKEN_COMMAND_HEREDOC_DELIM);
        TOKEN(TOKEN_HEREDOC_DELIM);
      }
    }
  }

  // the idea here is in a 3 part quotelike, we return a skip instead of a begin
  if (valid_symbols[TOKEN_QUOTELIKE_MIDDLE_SKIP]) {
    if (!lexerstate_is_paired_delimiter(state)) TOKEN(TOKEN_QUOTELIKE_MIDDLE_SKIP);
  }

  if (valid_symbols[TOKEN_QUOTELIKE_BEGIN]) {
    int delim = c;
    if (skipped_whitespace && c == '#') return false;
    // we must do a two char lookahead to avoid turning the "=" in => into a
    // quote char
    MARK_END;
    ADVANCE_C;

    // gotta safely handle $hash{q}
    if (valid_symbols[TOKEN_BRACE_END_ZW] && delim == '}') {
      TOKEN(TOKEN_BRACE_END_ZW);
    }
    MARK_END;

    lexerstate_push_quote(state, delim);
    // TODO - fill in this debug print here?
    // DEBUG("Generic QSTRING open='%c' close='%c'\n", state->delim_open,
    // state->delim_close);
    TOKEN(TOKEN_QUOTELIKE_BEGIN);
  }

  if (c == '\\' &&
      // If we're inside a quotelike that is using the `\` as a delimiter then
      // this doesn't count
      !(valid_symbols[TOKEN_QUOTELIKE_END] && lexerstate_is_quote_closer(state, '\\'))) {
    // eat the reverse-solidus
    ADVANCE_C;
    // let's see what that reverse-solidus was hiding!
    // note that we may have fallen through here from a HEREDOC_MIDDLE, so we
    // need to accept the token explicitly after we've read our heart's content
    int esc_c = c;
    // if we escaped a whitespace, the space comes through, it just hides the
    // \ char
    if (!is_tsp_whitespace(c)) ADVANCE_C;

    if (valid_symbols[TOKEN_ESCAPED_DELIMITER]) {
      if (lexerstate_is_quote_opener(state, esc_c) || lexerstate_is_quote_closer(state, esc_c)) {
        MARK_END;
        TOKEN(TOKEN_ESCAPED_DELIMITER);
      }
    }

    if (valid_symbols[TOKEN_ESCAPE_SEQUENCE]) {
      // Inside any kind of string, \\ is always an escape sequence
      MARK_END;
      if (esc_c == '\\') TOKEN(TOKEN_ESCAPE_SEQUENCE);

      if (valid_symbols[TOKEN_Q_STRING_CONTENT]) {
        // Inside a q() string, only \\ is a valid escape sequence; all else is
        // literal
        TOKEN(TOKEN_Q_STRING_CONTENT);
      }

      switch (esc_c) {
        case 'x':
          if (c == '{')
            skip_braced(lexer);
          else
            skip_hexdigits(lexer, 2);
          break;

        case 'N':
          skip_braced(lexer);
          break;

        case 'o':
          /* TODO: contents should just be octal */
          skip_braced(lexer);
          break;

        case '0':
          skip_octdigits(lexer, 3);
          break;

        default:
          break;
      }

      TOKEN(TOKEN_ESCAPE_SEQUENCE);
    }
  }

  if (valid_symbols[TOKEN_Q_STRING_CONTENT] || valid_symbols[TOKEN_QQ_STRING_CONTENT]) {
    bool is_qq = valid_symbols[TOKEN_QQ_STRING_CONTENT];
    bool valid = false;

    while (c) {
      if (c == '\\') break;
      int32_t quote_index = lexerstate_is_quote_opener(state, c);
      if (quote_index)
        lexerstate_saw_opener(state, quote_index);
      else {
        quote_index = lexerstate_is_quote_closer(state, c);
        if (quote_index) {
          if (lexerstate_is_quote_closed(state, quote_index)) {
            break;
          }
          lexerstate_saw_closer(state, quote_index);
        } else if (is_qq && is_interpolation_escape(c))
          break;
      }

      valid = true;
      ADVANCE_C;
    }

    if (valid) {
      if (is_qq)
        TOKEN(TOKEN_QQ_STRING_CONTENT);
      else
        TOKEN(TOKEN_Q_STRING_CONTENT);
    }
  }

  if (valid_symbols[TOKEN_QUOTELIKE_MIDDLE_CLOSE]) {
    int32_t quote_index = lexerstate_is_quote_closer(state, c);
    if (quote_index && lexerstate_is_quote_closed(state, quote_index)) {
      ADVANCE_C;
      TOKEN(TOKEN_QUOTELIKE_MIDDLE_CLOSE);
    }
  }

  if (valid_symbols[TOKEN_QUOTELIKE_END]) {
    int32_t quote_index = lexerstate_is_quote_closer(state, c);
    if (quote_index) {
      if (valid_symbols[TOKEN_QUOTELIKE_END_ZW]) TOKEN(TOKEN_QUOTELIKE_END_ZW);

      ADVANCE_C;
      lexerstate_pop_quote(state, quote_index);
      TOKEN(TOKEN_QUOTELIKE_END);
    }
  }

  // we only check for the prototype token b/c it 100% overlaps w/ signatures
  if (c == '(' && (valid_symbols[TOKEN_PROTOTYPE] || valid_symbols[TOKEN_SIGNATURE_START])) {
    // you can't reallllllly know if you're getting a prototype or a signature, but we can
    // be optimistic as follows - all alnums are invalid in a prototype, so if there's ANY
    // valid identifier char, then we assume we got a sig; otherwise we make it to the end
    // of the nested parens + assume it was all a prototype
    ADVANCE_C;
    // Now, we begin lookahead
    lexer->mark_end(lexer);

    int count = 0;

    while (!lexer->eof(lexer)) {
      if (c == ')' && !count) {
        ADVANCE_C;
        break;
      } else if (c == ')')
        count--;
      else if (c == '(')
        count++;
      else if (is_tsp_id_continue(c))
        TOKEN(TOKEN_SIGNATURE_START);

      ADVANCE_C;
    }

    // we gotta accept all the stuff that was in the prototype now
    lexer->mark_end(lexer);
    TOKEN(TOKEN_PROTOTYPE);
  }

  // we hold on to the current char in case we need to do some fancy stuff w/ it in 2 char
  // lookaheads below
  int32_t c1 = c;
  if (c == '-' && valid_symbols[TOKEN_FILETEST]) {
    ADVANCE_C;
    if (tsp_strchr("rwxoRWXOezsfdlpSbctugkTBMAC", c)) {
      ADVANCE_C;
      if (!isidcont(c)) TOKEN(TOKEN_FILETEST);
    }
    return false;
  }
  if (isidfirst(c) &&
      (valid_symbols[TOKEN_FAT_COMMA_AUTOQUOTED] || valid_symbols[TOKEN_BRACE_AUTOQUOTED])) {
    // we zip until the end of the identifier; then we do a lookeahed to see if it's autoquoted
    do {
      ADVANCE_C;
    } while (c && isidcont(c));
    MARK_END;
    // TODO - carefully check if we got a 2 char quote such that we need to guard against
    // the comment char here - the quoting is space-sensitive so we'll have to track that
    // also

    // NOTE - TS is annoying about skipping chars after you've hit done
    // mark_end, so we have to do the regular advance so our token actually shows up
    while (is_tsp_whitespace(c) || c == '#') {
      while (is_tsp_whitespace(c)) ADVANCE_C;
      // now we need to skip comments - we get in a funny way if we have a quotelike
      // operator followed by a comment as the quote char
      if (c == '#') {
        ADVANCE_C;
        while (lexer->get_column(lexer)) ADVANCE_C;
      }
      if (lexer->eof(lexer)) return false;
      // TODO - in theory there could be POD here that we needa skip over (EYES ROLL)
    }
    c1 = lexer->lookahead;
    ADVANCE_C;
    if (valid_symbols[TOKEN_FAT_COMMA_AUTOQUOTED]) {
      if (c1 == '=' && c == '>') TOKEN(TOKEN_FAT_COMMA_AUTOQUOTED);
    }
    if (valid_symbols[TOKEN_BRACE_AUTOQUOTED]) {
      if (c1 == '}') TOKEN(TOKEN_BRACE_AUTOQUOTED);
    }
  } else {
    /* it's ZW time! */
    MARK_END;
    /* let's get the next lookahead */
    ADVANCE_C;
    int32_t c2 = c;
    if (lexer->eof(lexer)) return false;
#define EQ2(s) (c1 == s[0] && c2 == s[1])

    /* NOTE - we need this to NOT be valid_symbol guarded, b/c we need this to
     * crash errant GLR branches, see gh#92 */
    if (EQ2("<<")) {
        // this branch is mainly for the GLR crashing; we enter here from the FILEGLOB vs
        // READLINE handling further up. that's why there's this ugly label here
heredoc_token_handling:
      DEBUG("checking if << is indeed a heredoc\n", 0);
      ADVANCE_C;
      MARK_END;
      if (c == '\\' || c == '~' || isidfirst(c)) {
        TOKEN(PERLY_HEREDOC);
      }
      skip_whitespace(lexer);
      c = lexer->lookahead;
      if (c == '\'' || c == '"' || c == '`') {
        TOKEN(PERLY_HEREDOC);
      }
      return false;
    }

    if (valid_symbols[TOKEN_BRACE_END_ZW]) {
      DEBUG("ZW-lookahead for brace-end in autoquote\n", 0);
      if (c1 == '}') TOKEN(TOKEN_BRACE_END_ZW);
    }
  }

  return false;
}

void *tree_sitter_perl_external_scanner_create() {
  LexerState *state = calloc(1, sizeof(LexerState));
  array_init(&state->quotes);
  tree_sitter_perl_external_scanner_deserialize(state, NULL, 0);
  return state;
}

void tree_sitter_perl_external_scanner_destroy(void *payload) {
  LexerState *state = payload;
  array_delete(&state->quotes);
  free(state);
}
