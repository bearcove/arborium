/**
 * @file Yuri grammar for tree-sitter
 * @author addie
 * @license Apache 2.0
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
  call: 14,
  field: 13,
  unary: 12,
  exponential: 11,
  multiplicative: 10,
  additive: 9,
  shift: 8,
  bitand: 7,
  bitxor: 6,
  bitor: 5,
  comparative: 4,
  and: 3,
  xor: 2,
  or: 1,
};

const primitives = [
  // TODO: "f" or "f1"? placeholder for now
  "f",
  "f2",
  "f3",
  "f4",
  "u",
  "u2",
  "u3",
  "u4",
  // TODO: "i" or "s"? placeholder for now
  "i",
  "i2",
  "i3",
  "i4",
  "mat2",
  "mat3",
  "mat4",
  "bool",
];

module.exports = grammar({
  name: "yuri",

  extras: ($) => [/\s/, $.line_comment],

  rules: {
    source_file: ($) => repeat($._outer_declaration),

    line_comment: ($) => token(seq("#", /[^\n]*/)),

    _type: ($) =>
      choice(
        $.primitive_type,
        $.identifier,
        $.compound_type_item,
        $.array_type_item,
      ),

    symbol_path: ($) => seq($.symbol, repeat1(seq(".", $.symbol))),

    _callable: ($) => choice($.primitive_type, $.identifier),

    identifier: ($) => choice($.symbol, $.symbol_path),

    // @ts-expect-error
    symbol: (_) => /[_\p{XID_Start}][_\p{XID_Continue}]*/,

    primitive_type: (_) => choice(...primitives),

    // declarations
    _outer_declaration: ($) =>
      choice(
        $.function_item,
        $.type_alias_item,
        $.module_item,
        $.global_item,
        $.import_item,
      ),

    // TODO: this is wrong; needs the export keyword,
    type_alias_item: ($) =>
      seq(
        repeat($.attribute_item),
        "type",
        field("name", $.symbol),
        "=",
        field("value", $._type),
      ),

    variable_item: ($) =>
      seq(
        repeat($.attribute_item),
        "let",
        field("name", $.symbol),
        optional(seq(":", $._type)),
        "=",
        field("value", $._expression),
      ),

    global_item: ($) =>
      seq(
        repeat($.attribute_item),
        optional("export"),
        "let",
        field("name", $.symbol),
        optional(seq(":", $._type)),
        "=",
        field("value", $._expression),
      ),

    compound_type_item: ($) =>
      seq(
        "{{",
        optional(comma_separated(field("field", $.compound_type_field))),
        "}}",
      ),

    compound_type_field: ($) => alias($.parameter, ""),

    array_type_item: ($) =>
      seq(
        "[",
        field("element", $._type),
        ";",
        field("length", $._expression),
        "]",
      ),

    module_item: ($) =>
      seq(
        repeat($.attribute_item),
        "module",
        field("name", $.symbol),
        "{",
        repeat($._outer_declaration),
        "}",
      ),

    function_item: ($) =>
      seq(
        repeat($.attribute_item),
        optional("export"),
        "fn",
        field("name", $.symbol),
        field("parameters", $.function_parameters),
        ":",
        field("return_type", $._type),
        field("body", $.block),
      ),

    import_item: ($) => seq("import", field("module", $.identifier)),

    function_parameters: ($) =>
      seq("(", optional(comma_separated(field("parameter", $.parameter))), ")"),

    parameter: ($) =>
      seq(
        repeat($.attribute_item),
        field("name", $.symbol),
        ":",
        field("type", $._type),
      ),

    compound_value_field: ($) =>
      seq(
        repeat($.attribute_item),
        field("name", $.symbol),
        optional(seq("=", field("value", $._expression))),
      ),

    // statements (control flow that is not an expression or a declaration)
    _statement: ($) =>
      choice($.return_statement, $.break_statement, $.continue_statement),
    return_statement: ($) => seq("return", field("value", $._expression)),
    break_statement: ($) => seq("break", field("value", $._expression)),
    continue_statement: ($) => seq("continue", field("value", $._expression)),

    // expressions
    _expression: ($) =>
      choice(
        $._literal,
        $.compound_value_expression,
        $.paren_expression,
        $.array_expression,
        $.identifier,
        $.block,
        $.unary_expression,
        $.binary_expression,
        $.if_expression,
        $.call_expression,
      ),
    attribute_item: ($) => seq("@", $.identifier),
    _literal: ($) =>
      choice($.boolean_literal, $.integer_literal, $.float_literal),
    boolean_literal: (_) => choice("true", "false"),
    integer_literal: (_) =>
      token(choice(/[0-9][0-9_]*/, /0x[0-9a-fA-F_]+/, /0b[01_]+/)),
    float_literal: (_) => /[0-9][0-9_]*\.(?:[0-9][0-9_]*)?/,

    paren_expression: ($) => seq("(", $._expression, ")"),

    // TODO: figure out a way to make unary negation and positivity easier to look at. maybe move into numeric literals, limit to 1 per ungrouped expression?
    unary_expression: ($) =>
      prec(PREC.unary, seq(choice("-", "+", "!", "~"), $._expression)),

    array_expression: ($) =>
      seq(
        "[",
        choice($.array_fill, comma_separated(field("element", $._expression))),
        "]",
      ),

    array_fill: ($) =>
      seq(field("with", $._expression), ";", field("count", $._expression)),

    binary_expression: ($) => {
      const table = [
        [PREC.and, "and"],
        [PREC.or, "or"],
        [PREC.xor, "xor"],
        [PREC.bitand, "&"],
        [PREC.bitor, "|"],
        [PREC.bitxor, "^"],
        [PREC.comparative, choice("==", "!=", "<", "<=", ">", ">=")],
        [PREC.shift, choice("<<", ">>")],
        [PREC.additive, choice("+", "-")],
        [PREC.multiplicative, choice("*", "/", "%")],
        [PREC.exponential, "**"],
      ];

      return choice(
        ...table.map(([precedence, operator]) =>
          prec.left(
            // @ts-expect-error
            precedence,
            seq(
              field("left", $._expression),
              // @ts-expect-error
              field("operator", operator),
              field("right", $._expression),
            ),
          ),
        ),
      );
    },

    compound_value_expression: ($) =>
      seq(
        "{{",
        optional(comma_separated(field("field", $.compound_value_field))),
        "}}",
      ),

    call_expression: ($) =>
      seq(
        field("function", $._callable),
        "(",
        optional(comma_separated(field("argument", $._expression))),
        ")",
      ),

    if_expression: ($) =>
      seq(
        "if",
        field("condition", $._expression),
        field("consequence", $.block),
        optional(field("else", $.else_clause)),
      ),
    else_clause: ($) => seq("else", choice($.block, $.if_expression)),

    block: ($) =>
      seq(
        "{",
        repeat(
          seq(
            choice(
              $.function_item,
              $.type_alias_item,
              $.variable_item,
              $.import_item,
              $._statement,
              $._expression,
            ),
            /\n/,
          ),
        ),
        "}",
      ),
  },
});

/**
 * @param {RuleOrLiteral} element
 */
function comma_separated(element) {
  return seq(element, repeat(seq(",", element)), optional(","));
}
