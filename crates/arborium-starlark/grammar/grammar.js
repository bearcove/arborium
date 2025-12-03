/**
 * @file Starlark grammar for tree-sitter
 * @author Amos Wenger <amos@bearcove.net>
 * @license MIT
 * @see {@link https://github.com/bazelbuild/starlark/blob/master/spec.md|Starlark spec}
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
  lambda: -2,
  conditional: -1,
  or: 1,
  and: 2,
  not: 3,
  compare: 4,
  bitwise_or: 5,
  bitwise_xor: 6,
  bitwise_and: 7,
  shift: 8,
  plus: 9,
  times: 10,
  unary: 11,
  call: 12,
  attribute: 13,
};

module.exports = grammar({
  name: 'starlark',

  extras: $ => [
    /\s/,
    $.line_continuation,
  ],

  externals: $ => [
    $._newline,
    $._indent,
    $._dedent,
    $.string_start,
    $._string_content,
    $._escape_interpolation,
    $.string_end,
    $.comment,
    ')',
    ']',
    '}',
    'except',
  ],

  word: $ => $.identifier,

  conflicts: $ => [
    [$.primary_expression, $._left_hand_side],
    [$.tuple, $.tuple_pattern],
    [$.list, $.list_pattern],
  ],

  rules: {
    // Top level
    module: $ => repeat($._statement),

    _statement: $ => choice(
      $._simple_statement,
      $._compound_statement,
    ),

    // Simple statements (on a single line, semicolon or newline terminated)
    _simple_statement: $ => seq(
      choice(
        $.expression_statement,
        $.return_statement,
        $.pass_statement,
        $.break_statement,
        $.continue_statement,
        $.load_statement,
      ),
      choice($._newline, ';'),
    ),

    expression_statement: $ => choice(
      $.expression,
      $.assignment,
      $.augmented_assignment,
    ),

    return_statement: $ => seq(
      'return',
      optional($.expression),
    ),

    pass_statement: _ => 'pass',
    break_statement: _ => 'break',
    continue_statement: _ => 'continue',

    // Load statement: load("@repo//path:file.bzl", "symbol", alias = "symbol")
    load_statement: $ => seq(
      'load',
      '(',
      $.string,
      repeat(seq(',', choice(
        $.string,
        $.aliased_load,
      ))),
      optional(','),
      ')',
    ),

    aliased_load: $ => seq(
      field('alias', $.identifier),
      '=',
      field('name', $.string),
    ),

    // Compound statements (multi-line, with blocks)
    _compound_statement: $ => choice(
      $.if_statement,
      $.for_statement,
      $.function_definition,
    ),

    if_statement: $ => seq(
      'if',
      field('condition', $.expression),
      ':',
      field('consequence', $.block),
      repeat($.elif_clause),
      optional($.else_clause),
    ),

    elif_clause: $ => seq(
      'elif',
      field('condition', $.expression),
      ':',
      field('consequence', $.block),
    ),

    else_clause: $ => seq(
      'else',
      ':',
      field('body', $.block),
    ),

    for_statement: $ => seq(
      'for',
      field('left', $._left_hand_side),
      'in',
      field('right', $.expression),
      ':',
      field('body', $.block),
    ),

    function_definition: $ => seq(
      'def',
      field('name', $.identifier),
      field('parameters', $.parameters),
      ':',
      field('body', $.block),
    ),

    parameters: $ => seq(
      '(',
      optional($._parameters),
      ')',
    ),

    _parameters: $ => seq(
      commaSep1($.parameter),
      optional(','),
    ),

    parameter: $ => choice(
      $.identifier,
      $.default_parameter,
      $.list_splat_pattern,
      $.dictionary_splat_pattern,
    ),

    default_parameter: $ => seq(
      field('name', $.identifier),
      '=',
      field('value', $.expression),
    ),

    list_splat_pattern: $ => seq('*', $.identifier),
    dictionary_splat_pattern: $ => seq('**', $.identifier),

    block: $ => seq(
      $._newline,
      $._indent,
      repeat1($._statement),
      $._dedent,
    ),

    // Assignments
    assignment: $ => seq(
      field('left', $._left_hand_side),
      '=',
      field('right', $.expression),
    ),

    augmented_assignment: $ => seq(
      field('left', $._left_hand_side),
      field('operator', choice('+=', '-=', '*=', '/=', '//=', '%=', '&=', '|=', '^=', '<<=', '>>=')),
      field('right', $.expression),
    ),

    _left_hand_side: $ => choice(
      $.identifier,
      $.subscript,
      $.attribute,
      $.tuple_pattern,
      $.list_pattern,
    ),

    tuple_pattern: $ => seq(
      '(',
      commaSep1($._left_hand_side),
      optional(','),
      ')',
    ),

    list_pattern: $ => seq(
      '[',
      commaSep1($._left_hand_side),
      optional(','),
      ']',
    ),

    // Expressions
    expression: $ => choice(
      $.conditional_expression,
      $.lambda,
      $.primary_expression,
      $.not_operator,
      $.boolean_operator,
      $.comparison_operator,
      $.binary_operator,
      $.unary_operator,
    ),

    primary_expression: $ => choice(
      $.identifier,
      $.string,
      $.integer,
      $.float,
      $.true,
      $.false,
      $.none,
      $.list,
      $.dictionary,
      $.tuple,
      $.parenthesized_expression,
      $.call,
      $.subscript,
      $.attribute,
      $.list_comprehension,
      $.dictionary_comprehension,
    ),

    conditional_expression: $ => prec.right(PREC.conditional, seq(
      field('body', $.expression),
      'if',
      field('condition', $.expression),
      'else',
      field('alternative', $.expression),
    )),

    lambda: $ => prec(PREC.lambda, seq(
      'lambda',
      optional($.lambda_parameters),
      ':',
      $.expression,
    )),

    lambda_parameters: $ => $._parameters,

    not_operator: $ => prec(PREC.not, seq('not', $.expression)),

    boolean_operator: $ => choice(
      prec.left(PREC.and, seq($.expression, 'and', $.expression)),
      prec.left(PREC.or, seq($.expression, 'or', $.expression)),
    ),

    comparison_operator: $ => prec.left(PREC.compare, seq(
      $.expression,
      repeat1(seq(
        choice('<', '<=', '==', '!=', '>=', '>', 'in', seq('not', 'in')),
        $.expression,
      )),
    )),

    binary_operator: $ => choice(
      prec.left(PREC.plus, seq($.expression, choice('+', '-'), $.expression)),
      prec.left(PREC.times, seq($.expression, choice('*', '/', '//', '%'), $.expression)),
      prec.left(PREC.bitwise_or, seq($.expression, '|', $.expression)),
      prec.left(PREC.bitwise_xor, seq($.expression, '^', $.expression)),
      prec.left(PREC.bitwise_and, seq($.expression, '&', $.expression)),
      prec.left(PREC.shift, seq($.expression, choice('<<', '>>'), $.expression)),
    ),

    unary_operator: $ => prec(PREC.unary, seq(
      choice('+', '-', '~'),
      $.expression,
    )),

    // Primary expressions
    parenthesized_expression: $ => seq('(', $.expression, ')'),

    call: $ => prec(PREC.call, seq(
      field('function', $.primary_expression),
      field('arguments', $.argument_list),
    )),

    argument_list: $ => seq(
      '(',
      optional(seq(
        commaSep1(choice(
          $.expression,
          $.keyword_argument,
          $.list_splat,
          $.dictionary_splat,
        )),
        optional(','),
      )),
      ')',
    ),

    keyword_argument: $ => seq(
      field('name', $.identifier),
      '=',
      field('value', $.expression),
    ),

    list_splat: $ => seq('*', $.expression),
    dictionary_splat: $ => seq('**', $.expression),

    subscript: $ => prec(PREC.call, seq(
      field('value', $.primary_expression),
      '[',
      field('subscript', choice($.expression, $.slice)),
      ']',
    )),

    slice: $ => seq(
      optional($.expression),
      ':',
      optional($.expression),
      optional(seq(':', optional($.expression))),
    ),

    attribute: $ => prec(PREC.attribute, seq(
      field('object', $.primary_expression),
      '.',
      field('attribute', $.identifier),
    )),

    // Collections
    list: $ => seq(
      '[',
      optional(seq(
        commaSep1(choice($.expression, $.list_splat)),
        optional(','),
      )),
      ']',
    ),

    dictionary: $ => seq(
      '{',
      optional(seq(
        commaSep1(choice($.pair, $.dictionary_splat)),
        optional(','),
      )),
      '}',
    ),

    pair: $ => seq(
      field('key', $.expression),
      ':',
      field('value', $.expression),
    ),

    tuple: $ => seq(
      '(',
      choice(
        seq($.expression, ','),
        seq($.expression, repeat1(seq(',', $.expression)), optional(',')),
      ),
      ')',
    ),

    // Comprehensions
    list_comprehension: $ => seq(
      '[',
      $.expression,
      $._comprehension_clauses,
      ']',
    ),

    dictionary_comprehension: $ => seq(
      '{',
      $.pair,
      $._comprehension_clauses,
      '}',
    ),

    _comprehension_clauses: $ => seq(
      $.for_in_clause,
      repeat(choice($.for_in_clause, $.if_clause)),
    ),

    for_in_clause: $ => seq(
      'for',
      field('left', $._left_hand_side),
      'in',
      field('right', $.expression),
    ),

    if_clause: $ => seq('if', $.expression),

    // Literals
    identifier: _ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    integer: _ => token(choice(
      /0[xX][0-9a-fA-F]+/,
      /0[oO][0-7]+/,
      /0[bB][01]+/,
      /[0-9]+/,
    )),

    float: _ => token(choice(
      /[0-9]+\.[0-9]*([eE][+-]?[0-9]+)?/,
      /\.[0-9]+([eE][+-]?[0-9]+)?/,
      /[0-9]+[eE][+-]?[0-9]+/,
    )),

    true: _ => 'True',
    false: _ => 'False',
    none: _ => 'None',

    // String - handled by external scanner for proper multi-line support
    string: $ => seq(
      $.string_start,
      repeat($._string_content),
      $.string_end,
    ),

    line_continuation: _ => token(seq('\\', /\r?\n/)),
  },
});

function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}

function commaSep(rule) {
  return optional(commaSep1(rule));
}
