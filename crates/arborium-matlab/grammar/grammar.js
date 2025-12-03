/**
 * @file Matlab grammar for tree-sitter
 * @author Álan Crístoffer <acristoffers@startmail.com>
 * @author Amaan Qureshi <amaanq12@gmail.com>
 * @license MIT
 */

/* eslint-disable arrow-parens */
/* eslint-disable camelcase */
/* eslint-disable-next-line spaced-comment */
/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
  parenthesis: -1,
  or: 10,
  and: 11,
  not: 12,
  compare: 13,
  bitwise_or: 14,
  bitwise_and: 15,
  xor: 16,
  shift: 17,
  plus: 18,
  times: 19,
  unary: 20,
  postfix: 21,
  power: 22,
  call: 23,
  member: 23,
};

module.exports = grammar({
  name: 'matlab',

  conflicts: ($) => [
    [$._expression, $._range_element],
    [$._expression, $._index_expression],
    [$.range],
    [$.block],
    [$._index_range],
    [$._index_arguments],
    [$._index_matrix, $.matrix],
    [$._index_row, $.row],
    [$.function_call, $._function_call_with_keywords],
    [$.block, $._functionless_block],
    [$.function_definition, $._function_definition_with_end],
    [$._function_definition_with_end],
  ],

  externals: ($) => [
    $.comment,
    $.line_continuation,
    $.command_name,
    $.command_argument,
    $._single_quote_string_start,
    $._single_quote_string_end,
    $._double_quote_string_start,
    $._double_quote_string_end,
    $.formatting_sequence,
    $.escape_sequence,
    $.string_content,
    $._entry_delimiter,
    $._multioutput_variable_start,
    $._external_identifier,
    $._catch_identifier,
    $._transpose,
    $._ctranspose,
    $.error_sentinel,
  ],

  extras: ($) => [/\s/, $.comment, $.line_continuation],

  rules: {
    source_file: ($) =>
      seq(
        repeat($._end_of_line),
        choice(
          optional(seq($._block, repeat($.function_definition))),
          repeat1($.function_definition),
        ),
      ),

    _block: ($) =>
      repeat1(
        seq(
          choice(
            $._statement,
            $._expression,
            alias($._function_definition_with_end, $.function_definition),
          ),
          repeat1($._end_of_line),
        ),
      ),
    _functionless_block: ($) =>
      prec(1, repeat1(
        seq(
          choice($._statement, $._expression),
          repeat1($._end_of_line),
        ),
      )),
    block: ($) => $._block,

    identifier: ($) => choice($._external_identifier, $._keywords),

    _statement: ($) =>
      choice(
        $.assignment,
        $.break_statement,
        $.class_definition,
        $.command,
        $.continue_statement,
        $.for_statement,
        $.global_operator,
        $.if_statement,
        $.persistent_operator,
        $.return_statement,
        $.spmd_statement,
        $.switch_statement,
        $.try_statement,
        $.while_statement,
      ),

    _expression: ($) =>
      choice(
        $.binary_operator,
        $.boolean_operator,
        $.cell,
        $.comparison_operator,
        $.function_call,
        $.handle_operator,
        $.identifier,
        $.lambda,
        $.matrix,
        $.metaclass_operator,
        $.not_operator,
        $.number,
        $.parenthesis,
        $.postfix_operator,
        $.range,
        $.string,
        $.unary_operator,
        $.field_expression,
      ),

    parenthesis: ($) => prec(PREC.parenthesis, seq('(', $._expression, ')')),

    _binary_expression: ($) =>
      prec(
        1,
        choice(
          $.binary_operator,
          $.boolean_operator,
          $.cell,
          $.comparison_operator,
          $.function_call,
          $.identifier,
          $.matrix,
          $.not_operator,
          $.number,
          $.parenthesis,
          $.postfix_operator,
          $.string,
          $.field_expression,
          $.unary_operator,
        ),
      ),

    binary_operator: ($) => {
      const table = [
        [prec.left, '+', PREC.plus],
        [prec.left, '.+', PREC.plus],
        [prec.left, '-', PREC.plus],
        [prec.left, '.-', PREC.plus],
        [prec.left, '*', PREC.times],
        [prec.left, '.*', PREC.times],
        [prec.left, '/', PREC.times],
        [prec.left, './', PREC.times],
        [prec.left, '\\', PREC.times],
        [prec.left, '.\\', PREC.times],
        [prec.right, '^', PREC.power],
        [prec.right, '.^', PREC.power],
        [prec.left, '|', PREC.bitwise_or],
        [prec.left, '&', PREC.bitwise_and],
      ];

      return choice(
        // @ts-ignore
        ...table.map(([fn, operator, precedence]) =>
          fn(
            precedence,
            seq(
              field('left', $._binary_expression),
              // @ts-ignore
              operator,
              field('right', $._binary_expression),
            ),
          ),
        ),
      );
    },

    unary_operator: ($) =>
      prec(
        PREC.unary,
        seq(
          choice('+', '-'),
          field(
            'operand',
            choice(
              $.cell,
              $.field_expression,
              $.function_call,
              $.identifier,
              $.matrix,
              $.not_operator,
              $.number,
              $.parenthesis,
              $.postfix_operator,
              $.string,
              $.unary_operator,
            ),
          ),
        ),
      ),

    indirect_access: ($) =>
      prec(
        -2,
        seq(
          '(',
          choice(
            $.number,
            $.binary_operator,
            $.field_expression,
            $.function_call,
            $.identifier,
            $.matrix,
            $.parenthesis,
            $.postfix_operator,
            $.string,
            $.unary_operator,
          ),
          ')',
        ),
      ),
    field_expression: ($) =>
      prec.left(
        PREC.member,
        seq(
          field('object', choice($.identifier, $.function_call)),
          repeat1(
            seq(
              '.',
              field(
                'field',
                choice(
                  $.identifier,
                  alias($._extended_keywords, $.identifier),
                  alias($._function_call_with_keywords, $.function_call),
                  $.indirect_access
                ),
              ),
            ),
          ),
        ),
      ),

    not_operator: ($) => prec(PREC.not, seq('~', $._expression)),

    metaclass_operator: ($) => prec.left(seq('?', seq($.identifier, repeat(seq('.', $.identifier))))),

    handle_operator: ($) =>
      seq('@', seq($.identifier, repeat(seq('.', $.identifier)))),

    comparison_operator: ($) =>
      prec.left(
        PREC.compare,
        seq(
          $._expression,
          choice('<', '<=', '==', '~=', '>=', '>'),
          $._expression,
        ),
      ),

    boolean_operator: ($) =>
      choice(
        prec.left(
          PREC.and,
          seq(field('left', $._expression), '&&', field('right', $._expression)),
        ),
        prec.left(
          PREC.or,
          seq(field('left', $._expression), '||', field('right', $._expression)),
        ),
      ),

    postfix_operator: ($) =>
      prec(
        PREC.postfix,
        seq(
          field(
            'operand',
            choice(
              $.binary_operator,
              $.cell,
              $.function_call,
              $.identifier,
              $.matrix,
              $.number,
              $.parenthesis,
              $.postfix_operator,
              $.string,
              $.field_expression,
              $.unary_operator,
            ),
          ),
          choice(alias($._transpose, "'"), alias($._ctranspose, ".'")),
        ),
      ),

    string: ($) =>
      choice(
        seq(
          alias($._double_quote_string_start, '"'),
          repeat(
            choice($.string_content, $.escape_sequence, $.formatting_sequence),
          ),
          alias($._double_quote_string_end, '"'),
        ),
        seq(
          alias($._single_quote_string_start, '\''),
          repeat(
            choice($.string_content, $.escape_sequence, $.formatting_sequence),
          ),
          alias($._single_quote_string_end, '\''),
        ),
      ),

    row: ($) =>
      seq(
        optional(','),
        choice($._expression, $.ignored_argument),
        repeat(
          seq(alias($._entry_delimiter, ','), choice($._expression, $.ignored_argument)),
        ),
        optional(alias($._entry_delimiter, ',')),
      ),
    matrix: ($) =>
      seq(
        '[',
        repeat(choice(/[,]*;[ ,;]*/, /[\r\n]/)),
        optional(seq($.row, repeat(seq(choice(/[,]*;[ ,;]*/, /[\r\n]/), optional($.row))))),
        ']',
      ),
    cell: ($) =>
      seq(
        '{',
        repeat(choice(/[,]*;[ ,;]*/, /[\r\n]/)),
        optional(seq($.row, repeat(seq(choice(/[,]*;[ ,;]*/, /[\r\n]/), optional($.row))))),
        '}',
      ),

    ignored_argument: (_) => prec(PREC.not + 1, '~'),

    // A = B
    // A(1) = B
    // A{1} = B
    // A.b = B
    // [A, B, C] = D
    assignment: ($) =>
      seq(
        field(
          'left',
          choice(
            $.identifier,
            $.field_expression,
            $.ignored_argument,
            $.function_call,
            $.multioutput_variable,
          ),
        ),
        '=',
        field('right', $._expression),
      ),

    multioutput_variable: ($) =>
      seq(
        alias($._multioutput_variable_start, '['),
        repeat("\n"),
        optionalCommaSep(
          choice(
            $.identifier,
            $.field_expression,
            $.ignored_argument,
            $.function_call,
            $.ignored_argument,
          ),
        ),
        ']',
      ),

    spread_operator: (_) => ':',

    _index_boolean_operator: ($) =>
      choice(
        prec.left(
          PREC.and + 1,
          seq(field('left', alias($._index_expression, $._expression)), '&&', field('right', alias($._index_expression, $._expression))),
        ),
        prec.left(
          PREC.or + 1,
          seq(field('left', alias($._index_expression, $._expression)), '||', field('right', alias($._index_expression, $._expression))),
        ),
      ),
    _index_comparison_operator: ($) =>
      prec.left(
        PREC.compare + 1,
        seq(
          $._index_expression,
          choice('<', '<=', '==', '~=', '>=', '>'),
          $._index_expression,
        ),
      ),
    _index_not_operator: ($) => prec(PREC.not + 1, seq('~', alias($._index_expression, $._expression))),
    _index_unary_operator: ($) =>
      prec(
        PREC.unary + 1,
        seq(
          choice('+', '-'),
          field(
            'operand',
            choice(
              $.cell,
              $.field_expression,
              $.function_call,
              $.identifier,
              alias($._index_matrix, $.matrix),
              alias($._index_not_operator, $.not_operator),
              $.number,
              alias($._index_parenthesis, $.parenthesis),
              alias($._index_postfix_operator, $.postfix_operator),
              $.string,
              alias($._index_unary_operator, $.unary_operator),
            ),
          ),
        ),
      ),
    _index_postfix_operator: ($) =>
      prec(
        PREC.postfix + 1,
        seq(
          field(
            'operand',
            choice(
              alias($._index_binary_operator, $.binary_operator),
              $.cell,
              $.function_call,
              $.identifier,
              alias($._index_matrix, $.matrix),
              $.number,
              alias($._index_parenthesis, $.parenthesis),
              alias($._index_postfix_operator, $.postfix_operator),
              $.string,
              $.field_expression,
              alias($._index_unary_operator, $.unary_operator),
            ),
          ),
          choice(alias($._transpose, "'"), alias($._ctranspose, ".'")),
        ),
      ),
    _index_row: ($) =>
      seq(
        optional(','),
        choice($._index_expression, $.ignored_argument),
        repeat(
          seq(alias($._entry_delimiter, ','), choice($._index_expression, $.ignored_argument)),
        ),
        optional(alias($._entry_delimiter, ',')),
      ),
    _index_matrix: ($) =>
      seq(
        '[',
        repeat("\n"),
        optional(seq(alias($._index_row, $.row), repeat(seq(choice(';', /[\r\n]/), optional(alias($._index_row, $.row)))))),
        ']',
      ),
    _index_range_element: ($) =>
      prec(1, choice(
        $.field_expression,
        $.function_call,
        $.identifier,
        alias($._index_matrix, $.matrix),
        alias($._index_not_operator, $.not_operator),
        $.number,
        alias($._index_parenthesis, $.parenthesis),
        alias($._index_postfix_operator, $.postfix_operator),
        $.string,
        prec.dynamic(-1, alias($._index_unary_operator, $.unary_operator)),
        prec.dynamic(1, alias($._index_binary_operator, $.binary_operator)),
        $.end_keyword,
      )),
    _index_range: ($) =>
      prec.right(
        PREC.postfix + 1,
        seq(
          $._index_range_element,
          ':',
          $._index_range_element,
          optional(seq(':', $._index_range_element)),
        ),
      ),
    _index_parenthesis: ($) => seq('(', $._index_expression, ')'),
    _index_expression: ($) =>
      choice(
        alias($._index_boolean_operator, $.boolean_operator),
        $.field_expression,
        $.function_call,
        $.identifier,
        $.cell,
        alias($._index_matrix, $.matrix),
        alias($._index_not_operator, $.not_operator),
        $.number,
        alias($._index_comparison_operator, $.comparison_operator),
        alias($._index_parenthesis, $.parenthesis),
        alias($._index_range, $.range),
        alias($._index_binary_operator, $.binary_operator),
        alias($._index_postfix_operator, $.postfix_operator),
        $.string,
        alias($._index_unary_operator, $.unary_operator),
        $.end_keyword,
      ),
    _index_binary_expression: ($) =>
      prec(
        2,
        choice(
          alias($._index_binary_operator, $.binary_operator),
          alias($._index_boolean_operator, $.boolean_operator),
          $.cell,
          alias($._index_comparison_operator, $.comparison_operator),
          $.function_call,
          $.identifier,
          alias($._index_matrix, $.matrix),
          alias($._index_not_operator, $.not_operator),
          $.number,
          alias($._index_parenthesis, $.parenthesis),
          alias($._index_postfix_operator, $.postfix_operator),
          $.string,
          $.field_expression,
          alias($._index_unary_operator, $.unary_operator),
          $.end_keyword,
        ),
      ),
    _index_binary_operator: ($) => {
      const table = [
        [prec.left, '+', PREC.plus],
        [prec.left, '.+', PREC.plus],
        [prec.left, '-', PREC.plus],
        [prec.left, '.-', PREC.plus],
        [prec.left, '*', PREC.times],
        [prec.left, '.*', PREC.times],
        [prec.left, '/', PREC.times],
        [prec.left, './', PREC.times],
        [prec.left, '\\', PREC.times],
        [prec.left, '.\\', PREC.times],
        [prec.right, '^', PREC.power],
        [prec.right, '.^', PREC.power],
        [prec.left, '|', PREC.bitwise_or],
        [prec.left, '&', PREC.bitwise_and],
      ];
      return choice(
        ...table.map(([fn, op, p]) =>
          fn(p, seq(field('left', $._index_binary_expression), op, field('right', $._index_binary_expression))),
        ),
      );
    },
    _index_argument: ($) => choice($.spread_operator, choice(prec.dynamic(1, $._index_expression), prec.dynamic(-1, $._expression))),
    _index_arguments: ($) => commaSep1(field('argument', $._index_argument)),

    arguments: ($) =>
      choice(
        seq(
          $._index_arguments,
          optional(seq(',', commaSep1(seq($.identifier, '=', $._expression)))),
        ),
        commaSep1(seq($.identifier, '=', $._expression)),
      ),
    _args: ($) =>
      choice(
        seq('(', optional($.arguments), ')'),
        seq('{', optional($.arguments), '}'),
      ),
    function_call: ($) =>
      choice(
        prec.right(
          PREC.call,
          seq(
            field(
              'name',
              choice(
                alias($.end_keyword, $.identifier),
                $.identifier,
                $.function_call,
                $.indirect_access,
              ),
            ),
            optional(seq('@', alias($.property_name, $.superclass))),
            $._args,
          ),
        ),
        prec.right(
          PREC.call,
          seq(
            field(
              'name',
              choice(
                $.identifier,
                $.function_call,
                $.indirect_access,
              ),
            ),
            seq('@', alias($.property_name, $.superclass)),
            optional($._args),
          ),
        ),),
    _function_call_with_keywords: ($) =>
      choice(
        prec.right(
          PREC.call,
          seq(
            field(
              'name',
              choice(
                alias($.end_keyword, $.identifier),
                alias($._extended_keywords, $.identifier),
                $.identifier,
                $.function_call,
                $.indirect_access,
              ),
            ),
            optional(seq('@', alias($.property_name, $.superclass))),
            $._args,
          ),
        ),
        prec.right(
          PREC.call,
          seq(
            field(
              'name',
              choice(
                $.identifier,
                $.function_call,
                $.indirect_access,
              ),
            ),
            seq('@', alias($.property_name, $.superclass)),
            optional($._args),
          ),
        ),),

    command: ($) => prec.right(seq($.command_name, repeat($.command_argument))),

    // Unary operators cannot bind stronger in this case_clause, lest the world falls apart.
    _range_element: ($) =>
      choice(
        $.field_expression,
        $.function_call,
        $.identifier,
        $.matrix,
        $.not_operator,
        $.number,
        $.parenthesis,
        $.postfix_operator,
        $.string,
        prec.dynamic(-1, $.unary_operator),
        prec.dynamic(1, $.binary_operator),
      ),
    range: ($) =>
      prec.right(
        PREC.postfix,
        seq(
          $._range_element,
          ':',
          $._range_element,
          optional(seq(':', $._range_element)),
        ),
      ),

    return_statement: (_) => 'return',
    continue_statement: (_) => 'continue',
    break_statement: (_) => 'break',

    elseif_clause: ($) => prec.left(
      seq(
        'elseif',
        field('condition', $._expression),
        repeat($._end_of_line),
        optional($.block),
      )),
    else_clause: ($) => prec.left(seq('else', repeat($._end_of_line), optional($.block))),
    if_statement: ($) =>
      seq(
        'if',
        field('condition', $._expression),
        repeat($._end_of_line),
        optional($.block),
        repeat($.elseif_clause),
        optional($.else_clause),
        'end',
      ),

    iterator: ($) => seq($.identifier, '=', $._expression),
    parfor_options: ($) => choice($.number, $.identifier, $.field_expression, $.function_call, $.string),
    for_statement: ($) =>
      choice(
        seq(
          choice('for', 'parfor'),
          choice($.iterator, seq("(", $.iterator, ")")),
          repeat($._end_of_line),
          optional($.block),
          'end',
        ),
        seq(
          'parfor',
          '(',
          $.iterator,
          ',',
          $.parfor_options,
          ')',
          repeat($._end_of_line),
          optional($.block),
          'end',
        ),
      ),

    while_statement: ($) =>
      seq(
        'while',
        field('condition', $._expression),
        repeat($._end_of_line),
        optional($.block),
        'end',
      ),

    case_clause: ($) => prec.left(
      seq(
        'case',
        // MATLAB says it should be a `switch_expr`, but then accepts any expression
        field('condition', $._expression),
        repeat1($._end_of_line),
        optional($.block),
      )),
    otherwise_clause: ($) => prec.left(seq('otherwise', repeat($._end_of_line), optional($.block))),
    switch_statement: ($) =>
      seq(
        'switch',
        field('condition', $._expression),
        repeat($._end_of_line),
        repeat($.case_clause),
        optional($.otherwise_clause),
        'end',
      ),

    _lambda_arguments: ($) =>
      commaSep1(choice($.ignored_argument, $.identifier)),
    lambda: ($) =>
      prec.right(
        seq(
          '@',
          '(',
          alias(optional($._lambda_arguments), $.arguments),
          ')',
          field('expression', $._expression),
        ),
      ),

    global_operator: ($) => seq('global', repeat($.identifier)),

    persistent_operator: ($) => seq('persistent', repeat($.identifier)),

    _argument_attributes: ($) =>
      seq(
        '(',
        field('argument', $.identifier),
        repeat(seq(',', field('argument', $.identifier))),
        ')',
      ),
    class_property: ($) => seq($.identifier, '.?', $.identifier, repeat(seq('.', $.identifier))),
    arguments_statement: ($) =>
      prec.right(1, seq(
        'arguments',
        optional(alias($._argument_attributes, $.attributes)),
        repeat1($._end_of_line),
        repeat(seq(choice($.property, $.class_property), repeat1($._end_of_line))),
        'end',
        optional($._end_of_line),
      )),

    function_output: ($) =>
      seq(choice($.identifier, $.multioutput_variable), '='),
    function_arguments: ($) =>
      seq('(', field('arguments', optional($._lambda_arguments)), ')'),
    function_definition: ($) =>
      prec.dynamic(0, seq(
        'function',
        optional($.function_output),
        optional(choice('get.', 'set.')),
        field('name', choice($.identifier, $.property_name, alias($.end_keyword, $.identifier))),
        optional($.function_arguments),
        $._end_of_line,
        repeat(seq(repeat(choice($.comment, $._end_of_line)), $.arguments_statement)),
        repeat(choice($.comment, $._end_of_line)),
        optional(alias($._functionless_block, $.block)),
        optional(seq(choice('end', 'endfunction'), optional(';'))),
      )),
    _function_definition_with_end: ($) =>
      prec.dynamic(1, seq(
        'function',
        optional($.function_output),
        optional(choice('get.', 'set.')),
        field('name', choice($.identifier, $.property_name, alias($.end_keyword, $.identifier))),
        optional($.function_arguments),
        optional(
          seq(
            $._end_of_line,
            repeat(seq(repeat(choice($.comment, $._end_of_line)), $.arguments_statement)),
            repeat(choice($.comment, $._end_of_line)),
            optional($.block),
          )
        ),
        choice('end', 'endfunction'),
        optional(';'),
      )),

    _negated_attribute: ($) => seq("~", $.identifier),
    attribute: ($) => choice(alias($._negated_attribute, $.not_operator), seq($.identifier, optional(seq('=', $._expression)))),
    attributes: ($) =>
      seq('(', $.attribute, repeat(seq(',', $.attribute)), ')'),
    superclasses: ($) =>
      seq('<', $.property_name, repeat(seq('&', $.property_name))),
    dimensions: ($) =>
      seq('(', commaSep1(choice($.number, $.spread_operator)), ')'),
    validation_functions: ($) =>
      seq('{', choice($.function_call, $.identifier, $.field_expression), repeat(seq(optional(","), choice($.function_call, $.identifier, $.field_expression))), '}'),
    default_value: ($) => seq('=', $._expression),
    property_name: ($) =>
      prec.right(
        -1,
        seq(
          $.identifier,
          repeat(seq('.', $.identifier)),
          optional(seq('.', '*')),
        ),
      ),
    property: ($) =>
      prec.right(
        choice(
          seq(
            field(
              'name',
              choice($.identifier, $.property_name, $.ignored_argument),
            ),
            optional($.dimensions),
            optional(choice($.identifier, $.property_name)),
            optional($.validation_functions),
            optional($.default_value),
          ),
          seq(
            field(
              'name',
              choice($.identifier, $.property_name, $.ignored_argument),
            ),
            '@',
            $.identifier,
            alias(optional(choice('vector', 'matrix', 'scalar')), $.identifier),
            optional($.default_value),
          )
        ),
      ),
    properties: ($) =>
      seq(
        'properties',
        optional($.attributes),
        $._end_of_line,
        repeat(choice(seq($.property, $._end_of_line), $._end_of_line, $.comment)),
        'end',
      ),
    function_signature: ($) =>
      seq(
        optional($.function_output),
        optional(choice('get.', 'set.')),
        field('name', alias(choice($._external_identifier, 'get', 'set', 'arguments'), $.identifier)),
        optional($.function_arguments),
      ),
    methods: ($) =>
      seq(
        'methods',
        optional($.attributes),
        repeat1($._end_of_line),
        repeat(
          seq(
            choice(
              $.comment,
              alias(seq($.function_output, field('name', alias('end', $.identifier)), $.function_arguments), $.function_signature),
              $.function_signature,
              alias($._function_definition_with_end, $.function_definition)),
            repeat($._end_of_line)),
        ),
        'end',
      ),
    events: ($) =>
      seq(
        'events',
        optional($.attributes),
        $._end_of_line,
        repeat(choice(seq($.identifier, $._end_of_line), $._end_of_line, $.comment)),
        'end',
      ),
    enum: ($) =>
      seq($.identifier, optional(seq('(', commaSep1($._expression), ')'))),
    enumeration: ($) =>
      seq(
        'enumeration',
        optional($.attributes),
        $._end_of_line,
        repeat(choice(seq($.enum, $._end_of_line), $._end_of_line, $.comment)),
        'end',
      ),
    class_definition: ($) =>
      seq(
        'classdef',
        optional($.attributes),
        field('name', $.identifier),
        optional($.superclasses),
        $._end_of_line,
        repeat(choice($.properties, $.methods, $.events, $.enumeration, $._end_of_line, $.comment)),
        'end',
      ),

    catch_clause: ($) => prec.left(
      seq(
        'catch',
        optional(seq(alias($._catch_identifier, $.identifier), $._end_of_line)),
        repeat($._end_of_line),
        optional($.block),
      )),
    try_statement: ($) =>
      seq(
        'try',
        repeat($._end_of_line),
        optional($.block),
        optional($.catch_clause),
        'end',
      ),

    spmd_statement: ($) =>
      seq(
        'spmd',
        optional(seq('(', optional(choice($._expression, seq($._expression, ',', $._expression))), ')')),
        repeat($._end_of_line),
        optional($.block),
        'end',
      ),

    number: ($) => choice(
      /(\d+|\d+\.\d*|\.\d+)([eEdD][+-]?\d+)?[ij]?/,
      /0[xX][\dA-Fa-f]+([suSU](8|16|32|64))?/,
      /0[bB][01]+([suSU](8|16|32|64))?/
    ),

    end_keyword: ($) => 'end',

    _keywords: ($) => choice("get", "set", "properties", "arguments", "enumeration", "events", "methods"),

    _extended_keywords: ($) => prec(100, choice(
      "break", "case", "catch", "classdef", "continue", "else", "elseif", "end", "for", "function",
      "global", "if", "otherwise", "parfor", "persistent", "return", "spmd", "switch", "try",
      "while",
    )),

    _end_of_line: ($) => choice(';', '\n', '\r', ','),
  },
});

/**
 * Creates a rule to match one or more of the rules separated by a comma
 *
 * @param {Rule} rule
 *
 * @return {SeqRule}
 *
 */
function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}

/**
 * Creates a rule to match zero or more of the rules optionally separated by a comma
 *
 * @param {Rule} rule
 *
 * @return {SeqRule}
 *
 */
function optionalCommaSep(rule) {
  return optional(seq(rule, repeat(seq(optional(','), rule)), optional(',')));
}
