/**
 * @file FSharp grammar for tree-sitter
 * @author Nikolaj Sidorenco
 * @license MIT
 * @see {@link https://fsharp.org/specs/language-spec/4.1/FSharpSpec-4.1-latest.pdf f# grammar}
 */

/* eslint-disable arrow-parens */
/* eslint-disable camelcase */
/* eslint-disable-next-line spaced-comment */
/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
  SEQ_EXPR: 1,
  APP_EXPR: 16,
  THEN_EXPR: 2,
  RARROW: 3,
  INFIX_OP: 4,
  LET_EXPR: 60,
  LET_DECL: 7,
  DO_EXPR: 8,
  FUN_EXPR: 8,
  MATCH_EXPR: 8,
  MATCH_DECL: 9,
  DO_DECL: 10,
  ELSE_EXPR: 11,
  INTERFACE: 12,
  COMMA: 13,
  PREFIX_EXPR: 15,
  SPECIAL_INFIX: 16,
  LARROW: 16,
  TUPLE_EXPR: 16,
  CE_EXPR: 15,
  SPECIAL_PREFIX: 17,
  // DO_EXPR: 17,
  IF_EXPR: 18,
  DOT: 19,
  INDEX_EXPR: 20,
  PAREN_APP: 21,
  TYPED_EXPR: 22,
  PAREN_EXPR: 21,
  DOTDOT: 22,
  DOTDOT_SLICE: 23,
  NEW_OBJ: 24,
};

module.exports = grammar({
  name: "fsharp",

  extras: ($) => [
    /[ \s\f\uFEFF\u2060\u200B]|\\\r?n/,
    $.block_comment,
    $.line_comment,
    $.preproc_line,
    $.compiler_directive_decl,
    $.fsi_directive_decl,
    ";",
  ],

  // The external scanner (scanner.c) allows us to inject "dummy" tokens into the grammar.
  // These tokens are used to track the indentation-based scoping used in F#
  externals: ($) => [
    $._newline, // we distinguish new scoped based on newlines.
    $._indent, // starts a new indentation-based scope.
    $._dedent, // signals that the current indentation scope has ended.
    "then",
    "else",
    "elif",
    "#if",
    "#else",
    "#endif",
    "class",
    $._struct_begin,
    $._interface_begin,
    "end",
    "and",
    "with",
    $._triple_quoted_content,
    $.block_comment_content,
    $._inside_string_marker,
    $._newline_not_aligned,
    $._tuple_marker,

    $._error_sentinel, // unused token to detect parser errors in external parser.
  ],

  conflicts: ($) => [
    [$.long_identifier, $._identifier_or_op],
    [$.simple_type, $.type_argument],
    [$.preproc_if, $.preproc_if_in_expression],
    [$.rules],
  ],

  word: ($) => $.identifier,

  inline: ($) => [
    $._module_elem,
    $._expression_or_range,
    $._object_expression_inner,
    $._record_type_defn_inner,
    $._union_type_defn_inner,
    $._then_expression,
  ],

  supertypes: ($) => [
    $._module_elem,
    $._pattern,
    $._expression,
    $._type,
    $._type_defn_body,
    $._static_parameter,
  ],

  rules: {
    //
    // Top-level rules (BEGIN)
    //
    file: ($) =>
      choice($.named_module, repeat1($.namespace), repeat($._module_elem)),

    namespace: ($) =>
      seq(
        "namespace",
        choice(
          "global",
          field("name", seq(optional("rec"), $.long_identifier)),
        ),
        repeat($._module_elem),
      ),

    named_module: ($) =>
      seq(
        optional($.attributes),
        "module",
        optional($.access_modifier),
        field("name", $.long_identifier),
        repeat($._module_elem),
      ),

    _module_elem: ($) =>
      choice(
        alias($.value_declaration, $.declaration_expression),
        $.module_defn,
        $.module_abbrev,
        $.import_decl,
        $.fsi_directive_decl,
        $.type_definition,
        $.exception_definition,
        $._expression,
        $.preproc_if,
        // $.exception_defn
      ),

    module_abbrev: ($) =>
      seq(
        optional($.attributes),
        "module",
        $.identifier,
        "=",
        scoped($.long_identifier, $._indent, $._dedent),
      ),

    module_defn: ($) =>
      prec.left(
        seq(
          optional($.attributes),
          "module",
          optional($.access_modifier),
          $.identifier,
          "=",
          scoped($._module_body, $._indent, $._dedent),
        ),
      ),

    _module_body: ($) =>
      seq(
        $._module_elem,
        repeat(
          prec(
            // Make sure to parse a module node before a sequential expression
            // NOTE: This removes all sequential expressions from module bodies
            PREC.SEQ_EXPR + 1,
            seq(alias($._newline, ";"), $._module_elem),
          ),
        ),
      ),

    import_decl: ($) => seq("open", $.long_identifier),

    //
    // Attributes (BEGIN)
    //
    attributes: ($) => prec.left(repeat1($._attribute_set)),
    _attribute_set: ($) =>
      seq(
        "[<",
        $.attribute,
        prec(PREC.SEQ_EXPR + 1, repeat(seq($._newline, $.attribute))),
        ">]",
      ),
    attribute: ($) =>
      seq(
        optional(seq(field("target", $.identifier), ":")),
        $._object_construction,
      ),

    _object_construction: ($) =>
      prec.left(PREC.SEQ_EXPR + 1, seq($._type, optional($._expression))),

    //
    // Attributes (END)
    //

    value_declaration: ($) =>
      seq(
        optional($.attributes),
        choice(
          prec(PREC.LET_DECL, $.function_or_value_defn),
          prec(PREC.DO_DECL, $.do),
        ),
      ),

    do: ($) => prec(PREC.DO_EXPR + 1, seq("do", $._expression_block)),

    _function_or_value_defns: ($) =>
      prec.right(
        seq(
          $._function_or_value_defn_body,
          repeat(seq("and", $._function_or_value_defn_body)),
        ),
      ),

    function_or_value_defn: ($) =>
      seq(
        choice("let", "let!"),
        choice(
          $._function_or_value_defn_body,
          seq("rec", $._function_or_value_defns),
        ),
      ),

    _function_or_value_defn_body: ($) =>
      seq(
        choice($.function_declaration_left, $.value_declaration_left),
        optional(seq(":", $._type)),
        "=",
        field("body", $._expression_block),
      ),

    function_declaration_left: ($) =>
      prec.left(
        3,
        seq(
          optional("inline"),
          optional($.access_modifier),
          prec(100, $._identifier_or_op),
          optional($.type_arguments),
          $.argument_patterns,
        ),
      ),

    value_declaration_left: ($) =>
      prec.left(
        2,
        seq(
          optional("mutable"),
          optional($.access_modifier),
          $._pattern,
          optional($.type_arguments),
        ),
      ),

    access_modifier: (_) =>
      prec(100, token(prec(1000, choice("private", "internal", "public")))),
    //
    // Top-level rules (END)
    //

    class_as_reference: ($) => seq("as", $.identifier),

    primary_constr_args: ($) =>
      seq(
        optional($.attributes),
        optional($.access_modifier),
        "(",
        optional($._pattern),
        ")",
        optional($.class_as_reference),
      ),

    //
    // Pattern rules (BEGIN)

    repeat_pattern: ($) =>
      prec.right(seq($._pattern, repeat1(prec(1, seq(",", $._pattern))))),

    _pattern: ($) =>
      choice(
        "null",
        alias("_", $.wildcard_pattern),
        $.const,
        $.as_pattern,
        $.disjunct_pattern,
        $.conjunct_pattern,
        $.cons_pattern,
        $.repeat_pattern,
        $.paren_pattern,
        $.list_pattern,
        $.array_pattern,
        $.record_pattern,
        $.typed_pattern,
        $.attribute_pattern,
        $.type_check_pattern,
        $.optional_pattern,
        $.identifier_pattern,
        $.named_field_pattern,
      ),

    optional_pattern: ($) => prec.left(seq("?", $._pattern)),

    type_check_pattern: ($) =>
      prec.right(seq(":?", $.atomic_type, optional(seq("as", $.identifier)))),

    attribute_pattern: ($) => prec.left(seq($.attributes, $._pattern)),

    paren_pattern: ($) => prec(1, seq("(", $._pattern, ")")),

    as_pattern: ($) => prec.left(0, seq($._pattern, "as", $.identifier)),
    cons_pattern: ($) => prec.left(0, seq($._pattern, "::", $._pattern)),
    disjunct_pattern: ($) => prec.left(0, seq($._pattern, "|", $._pattern)),
    conjunct_pattern: ($) => prec.left(0, seq($._pattern, "&", $._pattern)),
    typed_pattern: ($) =>
      prec.left(
        -1,
        seq(
          $._pattern,
          ":",
          $._type,
          field("constraints", optional($.type_argument_constraints)),
        ),
      ),

    argument_patterns: ($) =>
      // argument patterns are generally no different from normal patterns.
      // however, any time an argument pattern is a valid node, (i.e. inside a beginning fun decl)
      // it is always the correct node to construct.
      prec.left(1000, repeat1($._atomic_pattern)),

    field_pattern: ($) => prec(1, seq($.long_identifier, "=", $._pattern)),

    _atomic_pattern: ($) =>
      choice(
        "null",
        "_",
        $.const,
        $.long_identifier,
        $.list_pattern,
        $.record_pattern,
        $.array_pattern,
        seq("(", $._pattern, ")"),
        // :? atomic_type
      ),

    _list_pattern_content: ($) =>
      scoped(
        seq(
          optional($._newline),
          $._pattern,
          repeat(seq($._newline, $._pattern)),
        ),
        $._indent,
        $._dedent,
      ),

    list_pattern: ($) => seq("[", optional($._list_pattern_content), "]"),
    array_pattern: ($) => seq("[|", optional($._list_pattern_content), "|]"),
    record_pattern: ($) =>
      prec.left(
        seq(
          "{",
          $.field_pattern,
          repeat(seq($._newline, $.field_pattern)),
          "}",
        ),
      ),

    named_field: ($) => seq(optional(seq($.identifier, "=")), $._pattern),

    named_field_pattern: ($) =>
      prec.left(
        seq("(", $.named_field, repeat(seq($._newline, $.named_field)), ")"),
      ),

    identifier_pattern: ($) =>
      prec.left(
        1,
        seq(
          $.long_identifier_or_op,
          optional($._pattern),
          optional($._pattern),
        ),
      ),

    //
    // Pattern rules (END)
    //

    //
    // Expressions (BEGIN)
    //

    _expression_block: ($) => seq($._indent, $._expression, $._dedent),

    _expression: ($) =>
      choice(
        "null",
        $.const,
        $.paren_expression,
        $.begin_end_expression,
        $.long_identifier_or_op,
        $.typed_expression,
        $.infix_expression,
        $.index_expression,
        $.mutate_expression,
        $.list_expression,
        $.array_expression,
        $.ce_expression,
        $.prefixed_expression,
        $.brace_expression,
        $.anon_record_expression,
        $.typecast_expression,
        $.declaration_expression,
        $.do_expression,
        $.fun_expression,
        $.function_expression,
        $.sequential_expression,
        $.if_expression,
        $.while_expression,
        $.for_expression,
        $.match_expression,
        $.try_expression,
        $.literal_expression,
        $.tuple_expression,
        $.application_expression,
        $.dot_expression,
        alias($.preproc_if_in_expression, $.preproc_if),
        // (static-typars : (member-sig) expr)
      ),

    literal_expression: ($) =>
      prec(
        PREC.PAREN_EXPR,
        choice(
          seq("<@", $._expression, "@>"),
          seq("<@@", $._expression, "@@>"),
        ),
      ),

    long_identifier_or_op: ($) =>
      prec.right(
        choice(
          $.long_identifier,
          seq($.long_identifier, ".", $._identifier_or_op),
          $._identifier_or_op,
        ),
      ),

    tuple_expression: ($) =>
      prec.right(
        PREC.TUPLE_EXPR,
        seq($._expression, ",", optional($._tuple_marker), $._expression),
      ),

    brace_expression: ($) =>
      prec(
        PREC.CE_EXPR + 1,
        seq(
          "{",
          scoped(
            choice(
              $.field_initializers,
              $.object_expression,
              $.with_field_expression,
            ),
            $._indent,
            $._dedent,
          ),
          "}",
        ),
      ),

    anon_record_expression: ($) =>
      prec(
        PREC.PAREN_EXPR,
        seq(
          "{|",
          scoped(
            choice($.field_initializers, $.with_field_expression),
            $._indent,
            $._dedent,
          ),
          "|}",
        ),
      ),

    _object_expression_inner: ($) =>
      seq($._object_members, repeat($.interface_implementation)),

    object_expression: ($) =>
      prec(
        PREC.NEW_OBJ + 1,
        seq(
          "new",
          $._expression,
          optional(seq("as", $.identifier)),
          $._object_expression_inner,
        ),
      ),

    with_field_expression: ($) =>
      seq(
        $._expression,
        "with",
        scoped($.field_initializers, $._indent, $._dedent),
      ),

    prefixed_expression: ($) =>
      seq(
        choice(
          "return",
          "return!",
          "yield",
          "yield!",
          "lazy",
          "assert",
          "upcast",
          "downcast",
          "new",
          $.prefix_op,
        ),
        prec.right(PREC.PREFIX_EXPR, $._expression),
      ),

    typecast_expression: ($) =>
      prec.right(
        PREC.SPECIAL_INFIX,
        seq($._expression, choice(":", ":>", ":?", ":?>"), $._type),
      ),

    for_expression: ($) =>
      prec(
        PREC.DO_EXPR + 1,
        seq(
          "for",
          choice(
            seq($._pattern, "in", $._expression_or_range),
            seq(
              $.identifier,
              "=",
              $._expression,
              choice("to", "downto"),
              $._expression,
            ),
          ),
          "do",
          $._expression_block,
          optional("done"),
        ),
      ),

    while_expression: ($) =>
      prec(
        PREC.DO_EXPR + 1,
        seq(
          choice("while", "while!"),
          $._expression,
          "do",
          $._expression_block,
          optional("done"),
        ),
      ),

    _else_expression: ($) => seq("else", field("else", $._expression_block)),

    _then_expression: ($) => seq("then", field("then", $._expression_block)),

    elif_expression: ($) =>
      seq("elif", field("guard", $._expression_block), $._then_expression),

    _if_branch: ($) => seq("if", field("guard", $._expression_block)),

    if_expression: ($) =>
      seq(
        $._if_branch,
        $._then_expression,
        repeat($.elif_expression),
        optional($._else_expression),
      ),

    fun_expression: ($) =>
      prec.right(
        PREC.FUN_EXPR,
        seq("fun", $.argument_patterns, "->", $._expression_block),
      ),

    try_expression: ($) =>
      prec(
        PREC.MATCH_EXPR,
        seq(
          "try",
          $._expression_block,
          optional($._newline),
          choice(seq("with", $.rules), seq("finally", $._expression_block)),
        ),
      ),

    match_expression: ($) =>
      seq(
        choice("match", "match!"),
        $._expression,
        optional($._newline),
        "with",
        choice(seq($._newline, $.rules), scoped($.rules, $._indent, $._dedent)),
      ),

    function_expression: ($) =>
      prec(
        PREC.MATCH_EXPR,
        seq("function", scoped($.rules, $._indent, $._dedent)),
      ),

    mutate_expression: ($) =>
      prec.right(
        PREC.LARROW,
        seq(
          field("assignee", $._expression),
          "<-",
          field("value", $._expression),
        ),
      ),

    index_expression: ($) =>
      prec(
        PREC.INDEX_EXPR,
        seq(
          $._expression,
          ".[",
          choice(field("index", $._expression), $.slice_ranges),
          "]",
        ),
      ),

    typed_expression: ($) =>
      prec(
        PREC.PAREN_EXPR,
        seq(
          $._expression,
          token.immediate(prec(PREC.PAREN_EXPR, "<")),
          optional($.types),
          prec(PREC.PAREN_EXPR, ">"),
        ),
      ),

    declaration_expression: ($) =>
      seq(
        choice(
          seq(choice("use", "use!"), $.identifier, "=", $._expression_block),
          $.function_or_value_defn,
        ),
        field("in", $._expression),
      ),

    do_expression: ($) =>
      prec(PREC.DO_EXPR, seq(choice("do", "do!"), $._expression_block)),

    _list_elements: ($) =>
      prec.right(
        PREC.COMMA + 100,
        seq(
          optional($._newline),
          $._expression,
          repeat(
            prec.right(
              PREC.COMMA + 100,
              seq(alias($._newline, ";"), $._expression),
            ),
          ),
        ),
      ),

    _list_element: ($) =>
      seq(
        $._indent,
        choice($._list_elements, $._comp_or_range_expression, $.slice_ranges),
        $._dedent,
      ),

    list_expression: ($) => seq("[", optional($._list_element), "]"),

    array_expression: ($) => seq("[|", optional($._list_element), "|]"),

    range_expression: ($) =>
      prec(
        PREC.DOTDOT,
        seq(
          $._expression,
          "..",
          $._expression,
          optional(seq("..", $._expression)),
        ),
      ),

    _expression_or_range: ($) => choice($._expression, $.range_expression),

    rule: ($) =>
      prec.right(
        seq(
          field("pattern", $._pattern),
          optional(seq("when", field("guard", $._expression))),
          "->",
          field("block", $._expression_block),
        ),
      ),

    rules: ($) =>
      seq(
        optional("|"),
        $.rule,
        repeat(seq(optional($._newline), "|", $.rule)),
      ),

    begin_end_expression: ($) =>
      prec(
        PREC.PAREN_EXPR,
        seq("begin", scoped($._expression, $._indent, $._dedent), "end"),
      ),

    paren_expression: ($) =>
      prec(PREC.PAREN_EXPR, seq("(", $._expression_block, ")")),

    _high_prec_app: ($) =>
      prec.left(
        PREC.DOT + 1,
        seq(
          $._expression,
          choice(
            $.unit,
            seq(token.immediate(prec(10000, "(")), $._expression_block, ")"),
          ),
        ),
      ),

    _low_prec_app: ($) =>
      prec.left(PREC.APP_EXPR, seq($._expression, $._expression)),

    application_expression: ($) => choice($._high_prec_app, $._low_prec_app),

    dot_expression: ($) =>
      prec.right(
        PREC.DOT,
        seq(
          field("base", $._expression),
          ".",
          field("field", $.long_identifier_or_op),
        ),
      ),

    infix_expression: ($) =>
      prec.left(
        PREC.SPECIAL_INFIX,
        seq($._expression, $.infix_op, $._expression),
      ),

    ce_expression: ($) =>
      prec.left(
        PREC.CE_EXPR,
        seq(
          prec(-1, $._expression),
          "{",
          scoped($._comp_or_range_expression, $._indent, $._dedent),
          "}",
        ),
      ),

    sequential_expression: ($) =>
      prec.right(
        PREC.SEQ_EXPR,
        seq(
          $._expression,
          repeat1(
            prec.right(
              PREC.SEQ_EXPR,
              seq(alias($._newline, ";"), $._expression),
            ),
          ),
        ),
      ),

    //
    // Expressions (END)
    //

    //
    // Computation expression (BEGIN)
    //

    _comp_or_range_expression: ($) =>
      choice(
        $._expression,
        $.short_comp_expression,
        // $.range_expression, TODO
      ),

    // _comp_expressions: $ =>
    //   choice(
    //     $.let_ce_expressions,
    //     $.do_ce_expressions,
    //     $.use_ce_expressions,
    //     $.yield_ce_expressions,
    //     $.return_ce_expressions,
    //     $.if_ce_expressions,
    //     $.match_ce_expressions,
    //     $.try_ce_expressions,
    //     $.while_expressions,
    //     $.for_ce_expressions,
    //     $.sequential_ce_expressions,
    //     $._expression,
    //   ),

    // for_ce_expressions: $ =>
    //   prec.left(
    //   seq(
    //     "for",
    //     choice(
    //         seq($._pattern, "in", $._expression_or_range),
    //         seq($.identifier, "=", $._expression, "to", $._expression),
    //     ),
    //     "do",
    //       $._virtual_open_section,
    //       $._comp_expressions,
    //       $._virtual_end_section,
    //     optional("done"),
    //   )),
    //
    // try_ce_expressions: $ =>
    //   prec(PREC.MATCH_EXPR,
    //   seq(
    //     "try",
    //     $._virtual_open_section,
    //     $._comp_expressions,
    //     $._virtual_end_section,
    //     choice(
    //       seq("with", $.comp_rules),
    //       seq("finally", $.comp_rules)
    //     ),
    //   )),
    //
    // match_ce_expressions: $ =>
    //   prec(PREC.MATCH_EXPR,
    //   seq(
    //     "match",
    //     $._expression,
    //     "with",
    //     $.comp_rules,
    //   )),
    //
    // sequential_ce_expressions: $ =>
    //   prec.left(PREC.SEQ_EXPR,
    //   seq(
    //     $._comp_expressions,
    //     repeat1(prec.right(PREC.SEQ_EXPR, seq(choice(";", $._newline), $._comp_expressions))),
    //   )),
    //
    // _else_ce_expressions: $ =>
    //   prec(PREC.ELSE_EXPR,
    //   seq(
    //     "else",
    //     $._virtual_open_section,
    //     field("else_branch", $._comp_expressions),
    //     $._virtual_end_section,
    //   )),
    //
    // elif_ce_expressions: $ =>
    //   prec(PREC.ELSE_EXPR,
    //   seq(
    //     "elif",
    //     $._virtual_open_section,
    //     field("guard", $._expression),
    //     $._virtual_end_section,
    //     "then",
    //     $._virtual_open_section,
    //     field("then", $._comp_expressions),
    //     $._virtual_end_section,
    //   )),
    //
    // if_ce_expressions: $ =>
    //   prec.left(PREC.IF_EXPR,
    //   seq(
    //     "if",
    //     $._virtual_open_section,
    //     field("guard", $._expression),
    //     $._virtual_end_section,
    //     "then",
    //     field("then", $._comp_expressions),
    //     repeat($.elif_ce_expressions),
    //     optional($._else_ce_expressions),
    //   )),
    //
    // return_ce_expressions: $ =>
    //   prec.left(PREC.PREFIX_EXPR,
    //   seq(
    //     choice("return!", "return"),
    //     $._expression,
    //   )),
    //
    // yield_ce_expressions: $ =>
    //   prec.left(PREC.PREFIX_EXPR,
    //   seq(
    //     choice("yield!", "yield"),
    //     $._expression,
    //   )),
    //
    // do_ce_expressions: $ =>
    //   seq(
    //     choice("do!", "do"),
    //     $._expression,
    //     $._comp_expressions,
    //   ),
    //
    // use_ce_expressions: $ =>
    //   seq(
    //     choice("use!", "use"),
    //     $._pattern,
    //     "=",
    //     $._virtual_open_section,
    //     $._expression,
    //     $._virtual_end_section,
    //     $._comp_expressions,
    //   ),
    //
    // let_ce_expressions: $ =>
    //   seq(
    //     choice("let!", "let"),
    //     $._pattern,
    //     "=",
    //     $._virtual_open_section,
    //     $._expression,
    //     $._virtual_end_section,
    //     $._comp_expressions,
    //   ),

    short_comp_expression: ($) =>
      seq("for", $._pattern, "in", $._expression_or_range, "->", $._expression),

    // comp_rule: $ =>
    //   seq(
    //     $._pattern,
    //     "->",
    //     $._comp_expressions,
    //   ),
    //
    // comp_rules: $ =>
    //   prec.left(2,
    //   seq(
    //     optional("|"),
    //     $.comp_rule,
    //     repeat(seq("|", $.comp_rule)),
    //   )),

    slice_ranges: ($) => seq($.slice_range, repeat(seq(",", $.slice_range))),

    _slice_range_special: ($) =>
      prec.left(
        PREC.DOTDOT_SLICE,
        choice(
          seq(field("from", $._expression), token(prec(PREC.DOTDOT, ".."))),
          seq(
            token(prec(PREC.DOTDOT + 100000, "..")),
            field("to", $._expression),
          ),
          seq(
            field("from", $._expression),
            token(prec(PREC.DOTDOT, "..")),
            field("to", $._expression),
          ),
        ),
      ),

    slice_range: ($) => choice($._slice_range_special, $._expression, "*"),

    //
    // Computation expression (END)
    //

    //
    // Type rules (BEGIN)
    //
    _type: ($) =>
      prec(
        4,
        choice(
          $.simple_type,
          $.generic_type,
          $.paren_type,
          $.function_type,
          $.compound_type,
          $.postfix_type,
          $.list_type,
          $.static_type,
          $.type_argument,
          $.constrained_type,
          $.flexible_type,
          $.anon_record_type,
        ),
      ),

    simple_type: ($) => choice($.long_identifier, $._static_type_identifier),
    generic_type: ($) =>
      prec.right(
        5,
        seq($.long_identifier, "<", optional($.type_attributes), ">"),
      ),
    paren_type: ($) => seq("(", $._type, ")"),
    function_type: ($) => prec.right(seq($._type, "->", $._type)),
    compound_type: ($) =>
      prec.right(seq($._type, repeat1(prec.right(seq("*", $._type))))),
    postfix_type: ($) => prec.left(4, seq($._type, $.long_identifier)),
    list_type: ($) => seq($._type, "[]"),
    static_type: ($) => prec(10, seq($._type, $.type_arguments)),
    constrained_type: ($) => prec.right(seq($.type_argument, ":>", $._type)),
    flexible_type: ($) => prec.right(seq("#", $._type)),
    anon_record_type: ($) =>
      seq("{|", scoped($.record_fields, $._indent, $._dedent), "|}"),
    types: ($) =>
      seq($._type, repeat(prec.left(PREC.COMMA - 1, seq(",", $._type)))),

    _static_type_identifier: ($) =>
      prec(10, seq(choice("^", token(prec(100, "'"))), $.identifier)),

    _static_parameter: ($) =>
      // $.named_static_parameter,
      choice($.static_parameter_value, $.named_static_parameter),

    named_static_parameter: ($) =>
      prec(3, seq($.identifier, "=", $.static_parameter_value)),

    type_attribute: ($) =>
      choice(
        $._type,
        $._static_parameter,
        // measure
      ),

    type_attributes: ($) =>
      seq(
        $.type_attribute,
        repeat(prec.right(PREC.COMMA, seq(",", $.type_attribute))),
      ),

    atomic_type: ($) =>
      prec.right(
        choice(
          seq("#", $._type),
          $.type_argument,
          seq("(", $._type, ")"),
          $.long_identifier,
          seq($.long_identifier, "<", $.type_attributes, ">"),
        ),
      ),

    constraint: ($) =>
      prec(
        1000000,
        choice(
          seq($.type_argument, ":>", $._type),
          seq($.type_argument, ":", "null"),
          seq(
            $.type_argument,
            ":",
            "(",
            choice(
              $.trait_member_constraint,
              seq("new", ":", "unit", "->", $._type),
            ),
            ")",
          ),
          seq($.type_argument, ":", "struct"),
          seq($.type_argument, ":", "not", "struct"),
          seq($.type_argument, ":", "enum", "<", $._type, ">"),
          seq($.type_argument, ":", "unmanaged"),
          seq($.type_argument, ":", "equality"),
          seq($.type_argument, ":", "comparison"),
          seq(
            $.type_argument,
            ":",
            "delegate",
            "<",
            $._type,
            ",",
            $._type,
            ">",
          ),
          seq("default", $.type_argument, ":", $._type),
        ),
      ),

    type_argument_constraints: ($) =>
      seq("when", $.constraint, repeat(seq("and", $.constraint))),

    type_argument: ($) =>
      prec(
        10,
        choice(
          "_",
          seq(
            $._static_type_identifier,
            repeat(seq("or", $._static_type_identifier)),
          ),
        ),
      ),

    type_argument_defn: ($) => seq(optional($.attributes), $.type_argument),

    type_arguments: ($) =>
      seq(
        "<",
        $.type_argument_defn,
        repeat(prec.left(PREC.COMMA, seq(",", $.type_argument_defn))),
        optional($.type_argument_constraints),
        ">",
      ),

    trait_member_constraint: ($) =>
      seq(optional("static"), "member", $._identifier_or_op, ":", $._type),

    member_signature: ($) =>
      prec.left(
        seq(
          $.identifier,
          optional($.type_arguments),
          ":",
          $.curried_spec,
          optional(
            choice(
              seq("with", "get"),
              seq("with", "set"),
              seq("with", "get", ",", "set"),
              seq("with", "set", ",", "get"),
            ),
          ),
        ),
      ),

    curried_spec: ($) => seq(repeat(seq($.arguments_spec, "->")), $._type),

    argument_spec: ($) =>
      prec.left(
        seq(optional($.attributes), optional($.argument_name_spec), $._type),
      ),

    arguments_spec: ($) =>
      seq($.argument_spec, repeat(seq("*", $.argument_spec))),

    argument_name_spec: ($) =>
      seq(optional("?"), field("name", $.identifier), ":"),

    interface_spec: ($) => seq("interface", $._type),

    static_parameter: ($) =>
      choice(
        $.static_parameter_value,
        seq("id", "=", $.static_parameter_value),
      ),

    static_parameter_value: ($) => choice($.const, seq($.const, $._expression)),

    exception_definition: ($) =>
      seq(
        optional($.attributes),
        "exception",
        optional($.access_modifier),
        field("exception_name", $.long_identifier),
        optional(
          seq(
            "of",
            $._type,
          )
        ),
      ),

    type_definition: ($) =>
      prec.left(
        seq(
          optional($.attributes),
          "type",
          $._type_defn_body,
          repeat(seq(optional($.attributes), "and", $._type_defn_body)),
        ),
      ),

    _type_defn_body: ($) =>
      choice(
        $.delegate_type_defn,
        $.record_type_defn,
        $.union_type_defn,
        $.interface_type_defn,
        $.anon_type_defn,
        $.enum_type_defn,
        $.type_abbrev_defn,
        $.type_extension,
      ),

    type_name: ($) =>
      prec(
        2,
        seq(
          optional($.attributes),
          optional($.access_modifier),
          choice(
            seq(
              field("type_name", $.long_identifier),
              optional($.type_arguments),
            ),
            seq(optional($.type_argument), field("type_name", $.identifier)), // Covers `type 'a option = Option<'a>`
          ),
        ),
      ),

    type_extension: ($) => seq($.type_name, $.type_extension_elements),

    delegate_type_defn: ($) =>
      seq($.type_name, "=", scoped($.delegate_signature, $._indent, $._dedent)),

    delegate_signature: ($) => seq("delegate", "of", $._type),

    type_abbrev_defn: ($) =>
      seq($.type_name, "=", scoped($._type, $._indent, $._dedent)),

    _class_type_body_inner: ($) =>
      choice($.class_inherits_decl, $.type_extension_elements),

    _class_type_body: ($) =>
      seq(
        $._class_type_body_inner,
        repeat(seq($._newline, $._class_type_body_inner)),
      ),

    _record_type_defn_inner: ($) =>
      seq(
        optional($.access_modifier),
        "{",
        scoped($.record_fields, $._indent, $._dedent),
        "}",
        optional($.type_extension_elements),
      ),

    record_type_defn: ($) =>
      prec.left(
        seq(
          $.type_name,
          "=",
          scoped($._record_type_defn_inner, $._indent, $._dedent),
        ),
      ),

    record_fields: ($) =>
      seq(
        $.record_field,
        repeat(seq($._newline, $.record_field)),
        optional($._newline),
      ),

    record_field: ($) =>
      seq(
        optional($.attributes),
        optional("mutable"),
        optional($.access_modifier),
        $.identifier,
        ":",
        $._type,
      ),

    enum_type_defn: ($) =>
      seq(
        $.type_name,
        "=",
        choice(
          scoped($.enum_type_cases, $._indent, $._dedent),
          $.enum_type_cases,
        ),
      ),

    enum_type_cases: ($) =>
      seq(optional("|"), $.enum_type_case, repeat(seq("|", $.enum_type_case))),

    enum_type_case: ($) => seq($.identifier, "=", $.const),

    _union_type_defn_inner: ($) =>
      seq(
        optional($.access_modifier),
        $.union_type_cases,
        optional($.type_extension_elements),
      ),

    union_type_defn: ($) =>
      prec.left(
        seq(
          $.type_name,
          "=",
          choice(
            scoped($._union_type_defn_inner, $._indent, $._dedent),
            $._union_type_defn_inner,
          ),
        ),
      ),

    union_type_cases: ($) =>
      seq(
        optional("|"),
        $.union_type_case,
        repeat(seq("|", $.union_type_case)),
      ),

    union_type_case: ($) =>
      prec(
        8,
        seq(
          optional($.attributes),
          $.identifier,
          optional(choice(seq("of", $.union_type_fields), seq(":", $._type))),
        ),
      ),

    union_type_fields: ($) =>
      seq($.union_type_field, repeat(seq("*", $.union_type_field))),

    union_type_field: ($) =>
      prec.left(choice($._type, seq($.identifier, ":", $._type))),

    interface_type_defn: ($) =>
      prec.left(
        1,
        seq(
          $.type_name,
          "=",
          seq(
            alias($._interface_begin, "interface"),
            scoped(repeat($._type_defn_elements), $._indent, $._dedent),
            "end",
          ),
        ),
      ),

    anon_type_defn: ($) =>
      prec.left(
        seq(
          $.type_name,
          optional($.primary_constr_args),
          "=",
          choice(
            scoped($._class_type_body, $._indent, $._dedent),
            seq(
              choice("begin", "class"),
              scoped(optional($._class_type_body), $._indent, $._dedent),
              "end",
            ),
            seq(
              alias($._struct_begin, "struct"),
              scoped(repeat($._type_defn_elements), $._indent, $._dedent),
              "end",
            ),
          ),
        ),
      ),

    _class_function_or_value_defn: ($) =>
      seq(
        optional($.attributes),
        optional("static"),
        choice($.function_or_value_defn, seq("do", $._expression_block)),
      ),

    _type_extension_inner: ($) =>
      repeat1(choice($._class_function_or_value_defn, $._type_defn_elements)),

    type_extension_elements: ($) =>
      prec.left(
        seq(
          choice(
            seq("with", scoped($._type_extension_inner, $._indent, $._dedent)),
            $._type_extension_inner,
          ),
        ),
      ),

    _type_defn_elements: ($) =>
      choice(
        $.member_defn,
        $.interface_implementation,
        alias($.preproc_if_in_class_definition, $.preproc_if),
        // $._interface_signature
      ),

    interface_implementation: ($) =>
      prec.left(seq("interface", $._type, optional($._object_members))),

    _member_defns: ($) =>
      prec.left(
        seq($.member_defn, repeat(seq(optional($._newline), $.member_defn))),
      ),

    _object_members: ($) =>
      seq("with", scoped($._member_defns, $._indent, $._dedent)),

    member_defn: ($) =>
      prec(
        PREC.APP_EXPR + 100000,
        seq(
          optional($.attributes),
          choice(
            seq(
              optional("static"),
              "member",
              optional("inline"),
              optional($.access_modifier),
              $.method_or_prop_defn,
            ),
            seq(
              "abstract",
              optional("member"),
              optional($.access_modifier),
              $.member_signature,
            ),
            seq("member", "val", $.property_or_ident, $._val_property_defn),
            seq("override", optional($.access_modifier), $.method_or_prop_defn),
            seq("default", optional($.access_modifier), $.method_or_prop_defn),
            seq(
              optional("static"),
              "val",
              optional("mutable"),
              optional($.access_modifier),
              $.identifier,
              ":",
              $._type,
            ),
            seq("static", $.value_declaration),
            $.additional_constr_defn,
          ),
        ),
      ),

    property_or_ident: ($) =>
      choice(
        seq(
          field("instance", $.identifier),
          ".",
          field("method", $.identifier),
        ),
        $._identifier_or_op,
      ),

    _method_defn: ($) =>
      choice(
        seq(
          optional($.type_arguments),
          field("args", repeat1($._pattern)),
          "=",
          $._expression_block,
        ),
      ),

    _property_accessor_body: ($) =>
      seq(
        $.argument_patterns,
        optional(seq(":", $._type)),
        "=",
        $._expression_block,
      ),

    property_accessor: ($) =>
      seq(choice("get", "set"), $._property_accessor_body),

    _property_defn: ($) =>
      prec.left(
        PREC.APP_EXPR + 100001,
        seq(
          optional(seq(":", $._type)),
          choice(
            seq("=", $._expression_block),
            seq(
              "with",
              scoped(
                seq(
                  $.property_accessor,
                  repeat(seq("and", $.property_accessor)),
                ),
                $._indent,
                $._dedent,
              ),
            ),
          ),
        ),
      ),

    _val_property_defn: ($) =>
      prec.left(
        PREC.APP_EXPR + 100000,
        seq(
          optional(seq(":", $._type)),
          "=",
          $._expression,
          optional(
            seq(
              "with",
              choice(
                "get",
                "set",
                seq("get", ",", "set"),
                seq("set", ",", "get"),
              ),
            ),
          ),
        ),
      ),

    method_or_prop_defn: ($) =>
      prec(
        3,
        seq(
          field("name", $.property_or_ident),
          choice(
            $._method_defn,
            $._property_defn,
            seq(
              "with",
              scoped($._function_or_value_defns, $._indent, $._dedent),
            ),
          ),
        ),
      ),

    additional_constr_defn: ($) =>
      seq(
        optional($.access_modifier),
        "new",
        $._pattern,
        "=",
        $._expression_block,
      ),

    // additional_constr_expr: $ =>
    //   prec.left(
    //     choice(
    //       // seq($.additional_constr_expr, ';', $.additional_constr_expr),
    //       // seq($.additional_constr_expr, 'then', $._expression),
    //       // seq('if', $._expression, 'then', $.additional_constr_expr, 'else', $.additional_constr_expr),
    //       // seq('let', $._function_or_value_defn_body, 'in', $.additional_constr_expr), // TODO: "in" is optional?
    //       $.additional_constr_init_expr,
    //     )),
    //
    // additional_constr_init_expr: $ =>
    //   choice(
    //     // seq('{', $.class_inherits_decl, $.field_initializers, '}'),
    //     $._expression,
    //   ),

    class_inherits_decl: ($) =>
      prec.left(
        seq(
          "inherit",
          scoped(seq($._type, optional($._expression)), $._indent, $._dedent),
        ),
      ),

    field_initializer: ($) =>
      prec(
        PREC.SPECIAL_INFIX + 1,
        seq(
          field("field", $.long_identifier),
          token(prec(10000000, "=")),
          field("value", $._expression),
        ),
      ),

    field_initializers: ($) =>
      prec(
        10000000,
        seq($.field_initializer, repeat(seq($._newline, $.field_initializer))),
      ),

    //
    // Type rules (END)
    //

    //
    // Constants (BEGIN)
    //
    _escape_char: (_) => token.immediate(prec(100, /\\["\'ntbrafv]/)),
    _non_escape_char: (_) => token.immediate(prec(100, /\\[^"\'ntbrafv]/)),
    // using \u0008 to model \b
    _simple_char_char: (_) => token.immediate(/[^\n\t\r\u0008\a\f\v'\\]/),
    _unicodegraph_short: (_) => /\\u[0-9a-fA-F]{4}/,
    _unicodegraph_long: (_) => /\\u[0-9a-fA-F]{8}/,
    _trigraph: (_) => /\\[0-9]{3}/,

    _char_char: ($) =>
      choice(
        $._simple_char_char,
        $._escape_char,
        $._trigraph,
        $._unicodegraph_short,
      ),

    // note: \n is allowed in strings
    _simple_string_char: ($) =>
      choice(
        $._inside_string_marker,
        token.immediate(prec(1, /[^\t\r\u0008\a\f\v\\"]/)),
      ),

    _string_char: ($) =>
      choice(
        $._simple_string_char,
        $._escape_char,
        $._trigraph,
        $._unicodegraph_short,
        $._non_escape_char,
        $._unicodegraph_long,
      ),

    char: (_) =>
      prec(
        -1,
        /'([^\n\t\r\u0008\a\f\v\\]|\\["\'ntbrafv]|\\[0-9]{3}|\\u[0-9a-fA-F]{4}|(\\\\))?'B?/,
      ),

    format_string_eval: ($) =>
      seq(token.immediate(prec(1000, "{")), $._expression, "}"),

    format_string: ($) =>
      seq(
        token(prec(100, '$"')),
        repeat(choice($.format_string_eval, $._string_char)),
        '"',
      ),

    _string_literal: ($) => seq('"', repeat($._string_char), '"'),

    string: ($) => choice($._string_literal, $.format_string),

    _verbatim_string_char: ($) =>
      choice($._simple_string_char, $._non_escape_char, "\\", /\"\"/),
    verbatim_string: ($) =>
      seq('@"', repeat($._verbatim_string_char), token.immediate('"')),
    bytearray: ($) => seq('"', repeat($._string_char), token.immediate('"B')),
    verbatim_bytearray: ($) =>
      seq('@"', repeat($._verbatim_string_char), token.immediate('"B')),

    format_triple_quoted_string: ($) =>
      seq(
        token(prec(100, '$"""')),
        // repeat(choice($.format_string_eval, $._string_char)),
        $._triple_quoted_content,
        '"""',
      ),

    triple_quoted_string: ($) =>
      choice(
        seq('"""', $._triple_quoted_content, '"""'),
        $.format_triple_quoted_string,
      ),

    bool: (_) => token(choice("true", "false")),

    unit: (_) => token(prec(100000, "()")),

    const: ($) =>
      choice(
        $.sbyte,
        $.int16,
        $.int32,
        $.int64,
        $.byte,
        $.uint16,
        $.uint32,
        $.int,
        $.xint,
        $.nativeint,
        $.unativeint,
        $.decimal,
        $.float,
        $.uint64,
        $.ieee32,
        $.ieee64,
        $.bignum,
        $.char,
        $.string,
        $.verbatim_string,
        $.triple_quoted_string,
        $.bytearray,
        $.verbatim_bytearray,
        $.bool,
        $.unit,
      ),

    // Identifiers:
    identifier: (_) =>
      token(
        choice(/[_\p{XID_Start}][_'\p{XID_Continue}]*/, /``([^`\n\r\t])+``/),
      ),

    long_identifier: ($) =>
      prec.right(seq($.identifier, repeat(seq(".", $.identifier)))),

    active_pattern: ($) =>
      prec(
        1000,
        seq(
          "(|",
          alias($.identifier, $.active_pattern_op_name),
          repeat(seq("|", alias($.identifier, $.active_pattern_op_name))),
          optional(seq("|", alias("_", $.wildcard_active_pattern_op))),
          "|)",
        ),
      ),

    op_identifier: (_) =>
      token(
        prec(
          1000,
          seq(
            "(",
            /\s*/,
            choice("?", /[!%&*+-./<=>@^|~$?][!%&*+-./<=>@^|~?]*/, ".. .."),
            /\s*/,
            ")",
          ),
        ),
      ),

    _identifier_or_op: ($) =>
      choice($.identifier, $.op_identifier, $.active_pattern),

    _infix_or_prefix_op: (_) => choice("+", "-", "+.", "-.", "%", "&", "&&"),

    prefix_op: ($) =>
      prec.left(
        choice($._infix_or_prefix_op, repeat1("~"), /[!?][!%&*+-./<=>@^|~?]*/),
      ),

    infix_op: ($) =>
      prec(
        PREC.INFIX_OP,
        choice(
          $._infix_or_prefix_op,
          token.immediate(prec(1, /[+-]/)),
          /[-+<>|&^*/'%@?][!%&*+./<=>@^|~?-]*/,
          "||",
          "=",
          "!=",
          ":=",
          "::",
          "$",
          "or",
          "?",
          "?",
          "?<-",
          "?->",
        ),
      ),

    // Numbers
    int: (_) => token(/[+-]?([0-9]_?)+/),
    xint: (_) =>
      token(
        choice(/0[xX]([0-9a-fA-F]_?)+/, /0[oO]([0-7]_?)+/, /0[bB]([0-1]_?)+/),
      ),

    sbyte: ($) => seq(choice($.int, $.xint), token.immediate("y")),
    byte: ($) => seq(choice($.int, $.xint), token.immediate("uy")),
    int16: ($) => seq(choice($.int, $.xint), token.immediate("s")),
    uint16: ($) => seq(choice($.int, $.xint), token.immediate("us")),
    int32: ($) => seq(choice($.int, $.xint), token.immediate("l")),
    uint32: ($) =>
      seq(choice($.int, $.xint), token.immediate(choice("ul", "u"))),
    nativeint: ($) => seq(choice($.int, $.xint), token.immediate("n")),
    unativeint: ($) => seq(choice($.int, $.xint), token.immediate("un")),
    int64: ($) => seq(choice($.int, $.xint), token.immediate("L")),
    uint64: ($) =>
      seq(choice($.int, $.xint), token.immediate(choice("UL", "uL"))),

    ieee32: ($) =>
      choice(
        seq($.float, token.immediate("f")),
        seq($.xint, token.immediate("lf")),
      ),
    ieee64: ($) => seq($.xint, token.immediate("LF")),

    bignum: ($) => seq($.int, token.immediate(/[QRZING]/)),
    decimal: ($) => seq(choice($.float, $.int), token.immediate(/[Mm]/)),

    float: ($) =>
      prec.right(
        alias(
          choice(
            seq($.int, token.immediate("."), optional($.int)),
            seq(
              $.int,
              optional(seq(token.immediate("."), $.int)),
              token.immediate(/[eE][+-]?/),
              $.int,
            ),
          ),
          "float",
        ),
      ),

    //
    // Constants (END)
    //
    //
    block_comment: ($) =>
      seq("(*", $.block_comment_content, token.immediate("*)")),
    line_comment: (_) => token(/\/\/+[^\n\r]*/),

    // preprocessors
    compiler_directive_decl: ($) =>
      prec(
        100000,
        choice(
          seq(
            "#nowarn",
            alias($._string_literal, $.string),
            $._newline_not_aligned,
          ),
          seq("#light", $._newline_not_aligned),
        ),
      ),

    fsi_directive_decl: ($) =>
      seq(
        choice("#r", "#load"),
        optional(choice(alias($._string_literal, $.string), $.verbatim_string)),
        /\n/,
      ),

    preproc_line: ($) =>
      seq(
        alias(/#(line)?/, "#line"),
        $.int,
        optional(choice(alias($._string_literal, $.string), $.verbatim_string)),
        $._newline_not_aligned,
      ),

    ...preprocIf("", ($) => $._module_elem),
    ...preprocIf(
      "_in_expression",
      ($) => repeat(seq(optional($._newline), $._expression)),
      -2,
    ),
    ...preprocIf(
      "_in_class_definition",
      ($) => repeat(seq(optional($._newline), $._class_type_body_inner)),
      -2,
    ),
    ...preprocIf("_in_member_definition", ($) => repeat($.member_defn), -2),
  },
});

/**
 *
 * @param {Rule} rule
 *
 * @param {Rule} indent
 *
 * @param {Rule} dedent
 *
 * @return {Rule}
 */
function scoped(rule, indent, dedent) {
  return field("block", seq(indent, rule, dedent));
}

/**
 *
 * @param {string} suffix
 *
 * @param {RuleBuilder<string>} content
 *
 * @param {number} precedence
 *
 * @return {RuleBuilders<string, string>}
 */
function preprocIf(suffix, content, precedence = 0) {
  /**
   *
   * @param {GrammarSymbols<string>} $
   *
   * @return {Rule}
   *
   */
  function alternativeBlock($) {
    return suffix
      ? alias($["preproc_else" + suffix], $.preproc_else)
      : $.preproc_else;
  }

  return {
    ["preproc_if" + suffix]: ($) =>
      prec(
        precedence,
        seq(
          "#if",
          field("condition", $.identifier),
          $._newline_not_aligned,
          content($),
          field("alternative", optional(alternativeBlock($))),
          "#endif",
        ),
      ),

    ["preproc_else" + suffix]: ($) =>
      prec(precedence, seq("#else", content($))),
  };
}
