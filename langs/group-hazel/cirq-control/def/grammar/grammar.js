/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

/**
 * Tree-sitter grammar for the ngspice .control scripting language.
 *
 * The control language is line-oriented: one statement per line, with
 * block statements (if/while/repeat/foreach) terminating on a bare
 * `end` line. Comments start with `*` at the first non-whitespace
 * position of a line; an inline `$ ` (dollar-space) also starts a
 * comment that runs to end-of-line.
 *
 * Expressions inside `let`, `if`, `while`, `print`, etc. share a single
 * vecexpr grammar — vector arithmetic, comparison, function calls,
 * `v(node)` / `i(elem)` / `@dev[param]` references, and SPICE-style
 * numeric literals.
 *
 * This grammar is the CST surface used for editor highlighting and
 * static analysis. The thevenin-control interpreter does its own
 * (string-based) parse for execution — keeping the two in sync is the
 * responsibility of corpus tests, not the type system.
 */

module.exports = grammar({
  name: "control",

  extras: ($) => [/[ \t\r]/, $.line_comment, $.inline_comment, $.line_continuation],

  word: ($) => $.identifier,


  rules: {
    source_file: ($) => repeat(choice($._statement, $._nl)),

    // Statements are line-terminated. Block statements consume their
    // bodies (which are themselves newline-separated statements) and
    // close with a bare `end` line.
    _statement: ($) =>
      choice(
        $.let_stmt,
        $.echo_stmt,
        $.if_stmt,
        $.foreach_stmt,
        $.while_stmt,
        $.repeat_stmt,
        $.save_stmt,
        $.quit_stmt,
        $.set_stmt,
        $.setplot_stmt,
        $.define_stmt,
        $.compose_stmt,
        $.alter_stmt,
        $.strcmp_stmt,
        $.print_stmt,
        $.write_stmt,
        $.run_analysis,
        $.eprint_stmt,
        $.stop_when_stmt,
        $.resume_stmt,
        $.source_stmt,
        $.measure_stmt,
        $.unknown_stmt
      ),

    // ── Block-terminating statements ────────────────────────────

    let_stmt: ($) =>
      seq(
        "let",
        field("name", choice($.indexed_target, $.identifier)),
        "=",
        field("value", $._expression),
        $._nl
      ),

    indexed_target: ($) =>
      seq(field("name", $.identifier), "[", field("index", $._expression), "]"),

    echo_stmt: ($) =>
      seq("echo", repeat($._echo_fragment), $._nl),

    _echo_fragment: ($) =>
      choice(
        $.vec_scalar_ref,
        $.var_ref,
        $.string_literal,
        $.echo_word
      ),

    var_ref: ($) => seq("$", field("name", $._var_name)),
    vec_scalar_ref: ($) => seq("$&", field("name", $._var_name)),
    _var_name: ($) => alias($.identifier, $.var_name),

    // Anything else on the echo line that isn't whitespace, a var ref,
    // a string, or a newline is treated as a literal word. The negative
    // precedence keeps this from competing with proper tokens.
    echo_word: (_$) => token(prec(-2, /[^\s$"\n][^\s\n]*/)),

    if_stmt: ($) =>
      seq(
        "if",
        field("condition", $._expression),
        $._nl,
        field("body", $.block),
        optional(
          seq(
            "else",
            $._nl,
            field("else_body", $.block)
          )
        ),
        "end",
        $._nl
      ),

    foreach_stmt: ($) =>
      seq(
        "foreach",
        field("var", $.identifier),
        field("values", repeat1($._foreach_value)),
        $._nl,
        field("body", $.block),
        "end",
        $._nl
      ),

    _foreach_value: ($) => choice($.number_literal, $.identifier, $.string_literal),

    while_stmt: ($) =>
      seq(
        "while",
        field("condition", $._expression),
        $._nl,
        field("body", $.block),
        "end",
        $._nl
      ),

    repeat_stmt: ($) =>
      seq(
        "repeat",
        field("count", $._expression),
        $._nl,
        field("body", $.block),
        "end",
        $._nl
      ),

    // A block is just a run of statements (possibly empty). Newlines
    // between statements are folded by the `_nl` rule.
    block: ($) => repeat1(choice($._statement, $._nl)),

    // ── Single-line statements ──────────────────────────────────

    save_stmt: ($) =>
      seq("save", repeat($._save_spec), $._nl),

    _save_spec: ($) => choice($.vector_ref, $.identifier, $.device_param),

    quit_stmt: ($) =>
      seq("quit", optional(field("code", $.number_literal)), $._nl),

    set_stmt: ($) =>
      seq("set", repeat1($._set_pair), $._nl),

    _set_pair: ($) =>
      choice(
        seq(field("name", $.identifier), "=", field("value", $._set_value)),
        field("name", $.identifier)
      ),

    _set_value: ($) =>
      choice($.string_literal, $.number_literal, $.identifier),

    setplot_stmt: ($) =>
      seq("setplot", field("plot", $.identifier), $._nl),

    define_stmt: ($) =>
      seq(
        "define",
        field("name", $.identifier),
        "(",
        optional(field("args", $.arg_list)),
        ")",
        field("body", $._expression),
        $._nl
      ),

    arg_list: ($) =>
      seq($.identifier, repeat(seq(",", $.identifier)), optional(",")),

    compose_stmt: ($) =>
      seq(
        "compose",
        field("name", $.identifier),
        optional("values"),
        repeat1($._expression),
        $._nl
      ),

    alter_stmt: ($) =>
      seq(
        "alter",
        field("target", choice($.device_param, $.identifier)),
        "=",
        field("value", choice($.alter_vector, $._expression)),
        $._nl
      ),

    alter_vector: ($) =>
      seq("[", repeat($.number_literal), "]"),

    strcmp_stmt: ($) =>
      seq(
        "strcmp",
        field("result", $.identifier),
        field("a", $._expression),
        field("b", $._expression),
        $._nl
      ),

    print_stmt: ($) =>
      seq(
        "print",
        repeat1($._print_item),
        optional(seq(">", field("file", $._expression))),
        $._nl
      ),

    _print_item: ($) =>
      choice(
        alias("col", $.print_mode),
        alias("line", $.print_mode),
        $._expression
      ),

    write_stmt: ($) =>
      seq("write", repeat($._expression), $._nl),

    run_analysis: ($) =>
      seq(
        field(
          "kind",
          choice("op", "dc", "ac", "tran", "sens", "noise", "pz", "tf", "run")
        ),
        repeat($._analysis_arg),
        $._nl
      ),

    _analysis_arg: ($) =>
      choice($.number_literal, $.identifier, $.string_literal),

    eprint_stmt: ($) =>
      seq(choice("eprint", "eprvcd"), repeat($._expression), $._nl),

    stop_when_stmt: ($) =>
      seq(
        "stop",
        "when",
        field("var", $.identifier),
        choice("=", "<", ">", "<=", ">="),
        field("value", $._expression),
        $._nl
      ),

    resume_stmt: ($) => seq("resume", $._nl),

    source_stmt: ($) =>
      seq("source", field("path", $._expression), $._nl),

    measure_stmt: ($) =>
      seq(
        choice("measure", "meas"),
        field("kind", $.identifier),
        field("name", $.identifier),
        repeat($._expression),
        $._nl
      ),

    // Catch-all so unknown commands don't break the whole script. The
    // executor treats unknown statements as no-ops, so mirroring that
    // here keeps the CST useful for partial files.
    unknown_stmt: ($) =>
      seq(
        field("command", $.identifier),
        repeat($._unknown_arg),
        $._nl
      ),

    _unknown_arg: ($) =>
      choice($.identifier, $.number_literal, $.string_literal, $.vector_ref, $.device_param),

    // ── Expressions ─────────────────────────────────────────────

    _expression: ($) =>
      choice(
        $.binary_expression,
        $.unary_expression,
        $.call_expression,
        $.vector_ref,
        $.device_param,
        $.indexed_expression,
        $.range_expression,
        $.paren_expression,
        $.number_literal,
        $.string_literal,
        $.var_ref,
        $.vec_scalar_ref,
        $.identifier
      ),

    binary_expression: ($) =>
      choice(
        prec.right(
          7,
          seq(
            field("left", $._expression),
            field("operator", choice("^", "**")),
            field("right", $._expression)
          )
        ),
        ...[
          [["or"], 1],
          [["and"], 2],
          [["=", "<>"], 3],
          [["<", ">", "<=", ">="], 4],
          [["+", "-", ".+", ".-"], 5],
          [["*", "/", ".*", "./"], 6],
        ].flatMap(([ops, p]) =>
          /** @type {readonly string[]} */ (ops).map((op) =>
            prec.left(
              /** @type {number} */ (p),
              seq(
                field("left", $._expression),
                field("operator", op),
                field("right", $._expression)
              )
            )
          )
        )
      ),

    unary_expression: ($) =>
      prec(
        8,
        seq(
          field("operator", choice("-", "!", "not")),
          field("operand", $._expression)
        )
      ),

    call_expression: ($) =>
      prec(
        9,
        seq(
          field("function", $.identifier),
          "(",
          optional(
            seq($._expression, repeat(seq(",", $._expression)), optional(","))
          ),
          ")"
        )
      ),

    // `v(node)`, `v(n1, n2)`, `i(element)` — special-cased so highlights
    // can colour them as references rather than function calls.
    vector_ref: ($) =>
      prec(
        10,
        seq(
          field("kind", choice("v", "i", "vm", "vp", "vr", "vi", "vdb", "im", "ip", "ir", "ii", "idb")),
          "(",
          field("node", $.identifier),
          optional(seq(",", field("node2", $.identifier))),
          ")"
        )
      ),

    // `@device[param]` — device-parameter query. The grammar treats it
    // atomically since the lexer-level interaction between brackets in
    // the spec and brackets used for postfix indexing is delicate; the
    // executor splits the two apart at evaluation time.
    device_param: (_$) =>
      token(seq("@", /[A-Za-z_][A-Za-z0-9_]*/, "[", /[A-Za-z_][A-Za-z0-9_]*/, "]")),

    indexed_expression: ($) =>
      prec.left(
        10,
        seq(field("value", $._expression), "[", field("index", $._expression), "]")
      ),

    range_expression: ($) =>
      prec.left(
        10,
        seq(
          field("value", $._expression),
          "[",
          field("start", $._expression),
          ":",
          field("end", $._expression),
          "]"
        )
      ),

    paren_expression: ($) =>
      seq(
        "(",
        $._expression,
        repeat(seq(",", $._expression)),
        optional(","),
        ")"
      ),

    // ── Literals ────────────────────────────────────────────────

    number_literal: (_$) =>
      token(
        seq(
          choice(
            seq(/[0-9][0-9_]*/, optional(seq(".", /[0-9_]*/))),
            seq(".", /[0-9][0-9_]*/)
          ),
          optional(seq(choice("e", "E"), optional(choice("+", "-")), /[0-9][0-9_]*/)),
          // Optional SPICE SI suffix + optional unit
          optional(choice("T", "G", "Meg", "K", "k", "M", "m", "u", "U", "n", "N", "p", "P", "f", "F", "a", "A")),
          optional(choice("V", "v", "A", "a", "W", "w", "S", "s", "Hz", "hz", "Ohm", "ohm", "F", "f", "H", "h"))
        )
      ),

    string_literal: (_$) =>
      token(seq('"', repeat(choice(/[^"\\\n]/, seq("\\", /./))), '"')),

    // ── Identifiers, comments, newlines ─────────────────────────

    identifier: (_$) => /[A-Za-z_][A-Za-z0-9_.#]*/,

    // Statement separator — one or more newlines, possibly with
    // a leading `;` (ngspice treats `;` like a newline in many contexts).
    _nl: (_$) => token(prec(-1, /[;\n]+/)),

    // Backslash-newline continues the logical line (carried over from
    // some ngspice host shells).
    line_continuation: (_$) => token(seq("\\", "\n")),

    // Full-line comment: `*` as the first non-whitespace char of the
    // line consumes the rest of the line. Modelled with a `^` anchor
    // approximation: `*` must be followed by either non-newline content
    // or be alone. Tree-sitter does not honour `^`, so we accept `*`
    // anywhere and rely on the surrounding grammar (no `*` token is
    // valid mid-statement at the start of an expression) to keep this
    // unambiguous.
    line_comment: (_$) =>
      token(prec(-1, seq("*", /[^\n]*/))),

    // Inline `$` followed by whitespace or end-of-line starts a comment.
    // Models the ngspice convention without breaking `$var` references.
    inline_comment: (_$) =>
      token(prec(-1, seq("$", /[ \t][^\n]*/))),
  },
});
