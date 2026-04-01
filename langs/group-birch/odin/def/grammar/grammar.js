/**
 * @file Odin grammar for tree-sitter
 * @author Amaan Qureshi <amaanq12@gmail.com>
 * @license MIT
 * @see {@link https://odin-lang.org|Official website}
 * @see {@link https://odin-lang.org/docs/overview|Official documentation}
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
  PARENTHESES: -1,
  ASSIGNMENT: 1,
  TERNARY: 2,
  LOGICAL_OR: 3,
  LOGICAL_AND: 4,
  COMPARE: 5,
  EQUALITY: 6,
  BITWISE_OR: 7,
  BITWISE_XOR: 8,
  BITWISE_AND: 9,
  BITWISE_AND_NOT: 10,
  SHIFT: 11,
  ADD: 12,
  MULTIPLY: 13,
  CAST: 14,
  IN: 15,
  UNARY: 16,
  CALL: 17,
  MEMBER: 18,
  MATRIX: 19,
  VARIADIC: 20,
};

module.exports = grammar({
  name: 'odin',

  conflicts: $ => [
    // because of optional($.tag)
    [$.array_type],
    // lol: size_of(Map_Cell(T){}.data) / size_of(T) when size_of(T) > 0 else 1
    [$._expression_no_tag, $.struct],
  ],

  externals: $ => [
    $._newline,
    $._backslash,
    $._nl_comma,
    $.float,
    $.block_comment,
    '{',
    '"',
  ],

  extras: $ => [
    $.comment,
    $.block_comment,
    /\s/,
    $._backslash,
  ],

  supertypes: $ => [
    $.declaration,
    $.expression,
    $.literal,
    $.statement,
  ],

  word: $ => $.identifier,

  rules: {
    source_file: $ => seq(repeat(seq($.declaration, $._separator)), optional($.declaration)),

    block: $ => prec(2, seq(
      '{',
      sep(seq(optional($.tag), $.statement), $._separator),
      '}',
    )),

    tagged_block: $ => seq($.tag, $.block),

    declaration: $ => choice(
      $.build_tag,
      $.package_declaration,
      $.import_declaration,
      $.procedure_declaration,
      $.overloaded_procedure_declaration,
      $.struct_declaration,
      $.enum_declaration,
      $.union_declaration,
      $.bit_field_declaration,
      $.variable_declaration,
      $.var_declaration,
      $.const_declaration,
      $.const_type_declaration,
      $.foreign_block,
      $.when_statement,
      $._expression_no_tag,
    ),

    build_tag: $ => seq('#+', /.+/),

    package_declaration: $ => seq('package', $.identifier),

    import_declaration: $ => seq(
      optional($.attributes),
      optional('foreign'),
      'import',
      optional(field('alias', $.identifier)),
      choice(
        $.string,
        seq(
          '{',
          commaSep1(choice($.string, $.identifier)),
          optional(','),
          '}',
        ),
      ),
    ),

    procedure_declaration: $ => seq(
      optional($.attributes),
      $.expression,
      '::',
      optional($.tag),
      $.procedure,
    ),
    procedure: $ => prec.right(seq(
      'proc',
      optional($.calling_convention),
      $.parameters,
      optional(seq(
        '->',
        optional($.tag),
        choice($.type, $.named_type),
        optional($.tag),
      )),
      optional($.where_clause),
      optional($.tag),
      optional(choice($.block, $.uninitialized)),
    )),

    where_clause: $ => prec.right(seq('where', commaSep1(prec.right($.expression)))),

    calling_convention: _ => choice(
      '"odin"',
      '"contextless"',
      '"stdcall"',
      '"std"',
      '"cdecl"',
      '"c"',
      '"fastcall"',
      '"fast"',
      '"none"',
      '"system"',
    ),

    overloaded_procedure_declaration: $ => seq(
      optional($.attributes),
      $.expression,
      '::',
      'proc',
      '{',
      optional(seq(
        commaSep1($.expression),
        optional(','),
      )),
      '}',
    ),

    struct_declaration: $ => seq(
      optional($.attributes),
      $.expression,
      '::',
      'struct',
      optional($.polymorphic_parameters),
      repeat(seq($.tag, optional(choice($.identifier, $.number)))), // #align 16
      optional($.where_clause),
      '{',
      optional(seq(
        commaSep1($.field),
        optional(','),
      )),
      '}',
    ),

    enum_declaration: $ => seq(
      optional($.attributes),
      optional('using'),
      $.expression,
      '::',
      'enum',
      optional($.type),
      '{',
      optional(seq(
        commaSep1(seq($.identifier, optional(seq('=', $.expression)))),
        optional(','),
      )),
      '}',
    ),

    union_declaration: $ => seq(
      optional($.attributes),
      $.expression,
      '::',
      'union',
      optional($.polymorphic_parameters),
      optional($.tag),
      '{',
      optional(seq(
        commaSep1($.type),
        optional(','),
      )),
      '}',
    ),

    bit_field_declaration: $ => seq(
      optional($.attributes),
      $.expression,
      '::',
      'bit_field',
      $.type,
      '{',
      optional(seq(
        commaSep1(seq($.identifier, ':', $.type, '|', $.expression)),
        optional(','),
      )),
      '}',
    ),

    variable_declaration: $ => seq(
      optional($.attributes),
      commaSep1($.expression),
      ':=',
      commaSep1(choice($.expression, $.procedure)),
      optional(','),
    ),

    const_declaration: $ => seq(
      optional($.attributes),
      commaSep1($.expression),
      '::',
      optional($.tag),
      commaSep1(
        choice(
          $.expression,
          seq(alias('#type', $.tag), $.type),
          $.array_type,
          $.bit_set_type,
          $.pointer_type,
        ),
      ),
    ),

    const_type_declaration: $ => prec(1, seq(
      optional($.attributes),
      $.expression,
      ':',
      $.type,
      ':',
      $.expression,
    )),

    foreign_block: $ => seq(
      optional($.attributes),
      'foreign',
      optional($.identifier),
      $.block,
    ),

    attributes: $ => repeat1($.attribute),

    attribute: $ => seq(
      '@',
      choice(
        $.identifier,
        seq(
          '(',
          commaSep1(seq($.identifier, optional(seq('=', $.expression)))),
          ')',
        ),
      ),
    ),

    parameters: $ => seq(
      '(',
      optional(seq(
        commaSep1(choice($.parameter, $.default_parameter)),
        optional(','),
      )),
      ')',
    ),
    parameter: $ => prec.right(seq(
      commaSep1($._param_header),
      optional($._param_type),
    )),
    _param_header: $ => seq(
      optional($.tag),
      optional('using'),
      optional('$'),
      choice(
        $.identifier,
        $.variadic_type,
        $.array_type,
        $.pointer_type,
        $.field_type,
        $._procedure_type,
      ),
    ),
    _param_type: $ => seq(
      ':',
      optional($.tag),
      $.type,
      optional($.identifier),
      optional(seq('=', $.expression)),
    ),


    default_parameter: $ => seq(
      optional($.tag),
      optional('using'),
      $.identifier,
      ':=',
      $.expression,
    ),

    polymorphic_parameters: $ => seq(
      '(',
      commaSep1(seq(
        commaSep1(seq(optional('$'), $.identifier)),
        ':',
        $.type,
      )),
      ')',
    ),

    field: $ => prec.right(seq(
      commaSep1(seq(optional($.tag), optional('using'), $.identifier)),
      ':',
      optional($.tag),
      $.type,
      optional($.string),
    )),

    statement: $ => prec(1, choice(
      $.procedure_declaration,
      $.overloaded_procedure_declaration,
      $.struct_declaration,
      $.enum_declaration,
      $.union_declaration,
      $.bit_field_declaration,
      $.const_declaration,
      $.import_declaration,
      $.assignment_statement,
      $.update_statement,
      $.if_statement,
      $.when_statement,
      $.for_statement,
      $.switch_statement,
      $.defer_statement,
      $.break_statement,
      $.continue_statement,
      $.fallthrough_statement,
      $.label_statement,
      $.using_statement,
      $.return_statement,
      $._expression_no_tag,
      $.var_declaration,
      $.foreign_block,
      $.tagged_block,
      $.block,
    )),

    assignment_statement: $ => prec(PREC.ASSIGNMENT, seq(
      optional(seq($.attributes, optional($.tag))),
      commaSep1($.expression),
      choice('=', ':='),
      optional($.tag),
      commaSep1(choice($.expression, $.procedure)),
    )),

    update_statement: $ => seq(
      commaSep1($.expression),
      choice('+=', '-=', '*=', '/=', '%=', '&=', '|=', '^=', '<<=', '>>=', '||=', '&&=', '&~='),
      commaSep1($.expression),
    ),

    if_statement: $ => prec.right(seq(
      'if',
      optional(seq(
        optional(field('initializer', choice($.assignment_statement, $.update_statement, $.var_declaration))),
        ';',
      )),
      optional($.tag),
      field('condition', $.expression),
      choice(
        field('consequence', $.block),
        seq('do', field('consequence', $.statement)),
      ),
      repeat($.else_if_clause),
      optional($.else_clause),
    )),

    else_if_clause: $ => seq(
      'else',
      'if',
      optional(seq(
        optional(field('initializer', $.assignment_statement)),
        ';',
      )),
      field('condition', $.expression),
      choice(
        field('consequence', $.block),
        seq('do', field('consequence', $.statement)),
      ),
    ),

    else_clause: $ => seq(
      'else',
      choice(
        field('consequence', $.block),
        seq('do', field('consequence', $.statement)),
      ),
    ),

    when_statement: $ => prec.right(seq(
      'when',
      $.expression,
      choice($.block, seq('do', $.statement)),
      repeat($.else_when_clause),
      optional($.else_clause),
    )),

    else_when_clause: $ => seq(
      'else',
      'when',
      $.expression,
      $.block,
    ),

    for_statement: $ => seq(
      'for',
      optional(choice(
        seq(
          optional(seq(
            optional(field('initializer', choice($.assignment_statement, $.update_statement, $.var_declaration))),
            ';',
          )),
          optional(field('condition', $.expression)),
          optional(seq(
            ';',
            optional(
              field('post', choice(
                $.update_statement,
                alias($._simple_assignment_statement, $.assignment_statement),
                // $.assignment_statement,
              )),
            ),
          )),
        ),
        $._for_in_expression,
      )),
      field('consequence', choice($.block, seq('do', $.statement))),
    ),
    _for_in_expression: $ => seq(
      commaSep($.expression),
      'in',
      $.expression,
    ),

    _simple_assignment_statement: $ => seq(
      optional($.attributes),
      commaSep1($.expression),
      choice('=', ':='),
      commaSep1(choice($.expression)),
    ),

    switch_statement: $ => seq(
      'switch',
      optional(seq(
        optional('in'),
        field('condition', choice(
          $.expression,
          seq($.assignment_statement, $._separator, optional($.expression)),
        )),
      )),
      '{',
      repeat($.switch_case),
      '}',
    ),

    switch_case: $ => seq(
      'case',
      commaSep(field('condition', choice($.expression, $.array_type, $.pointer_type))),
      ':',
      sep(seq(optional($.tag), $.statement), $._separator),
    ),

    defer_statement: $ => seq('defer', $.statement),

    break_statement: $ => seq('break', optional($.identifier)),

    continue_statement: $ => seq('continue', optional($.identifier)),

    fallthrough_statement: _ => 'fallthrough',

    var_declaration: $ => prec.right(seq(
      optional($.attributes),
      commaSep1($.expression),
      ':',
      optional($.tag),
      choice(
        seq($.type, optional(seq(choice('=', ':'), commaSep1($.expression)))),
        // seq('=', $.expression), // +2k state count no ty
      ),
    )),

    return_statement: $ => prec.right(1, seq(
      'return',
      optional($.tag),
      optional(seq(
        commaExternalSep1(choice($.expression, $._procedure_type), $),
        optional(','),
      )),
    )),

    label_statement: $ => seq(
      $.expression,
      ':',
      choice($.if_statement, $.for_statement, $.switch_statement, $.block),
    ),

    using_statement: $ => seq('using', $.expression),

    expression: $ => prec.left(choice(
      $._expression_no_tag,
      $.tag,
    )),

    _expression_no_tag: $ => choice(
      $.unary_expression,
      $.binary_expression,
      $.ternary_expression,
      $.call_expression,
      $.selector_call_expression,
      $.member_expression,
      $.index_expression,
      $.slice_expression,
      $.range_expression,
      $.cast_expression,
      $.parenthesized_expression,
      $.in_expression,
      $.variadic_expression,
      $.or_return_expression,
      $.or_continue_expression,
      $.or_break_expression,
      $.identifier,
      $.address,
      $.map_type,
      $.distinct_type,
      $.matrix_type,
      $.literal,
      '?',
    ),

    unary_expression: $ => prec.right(PREC.UNARY, seq(
      field('operator', choice('+', '-', '~', '!', '&')),
      field('argument', $.expression),
    )),

    binary_expression: $ => {
      const table = [
        ['||', PREC.LOGICAL_OR],
        ['or_else', PREC.LOGICAL_OR],
        ['&&', PREC.LOGICAL_AND],
        ['>', PREC.COMPARE],
        ['>=', PREC.COMPARE],
        ['<=', PREC.COMPARE],
        ['<', PREC.COMPARE],
        ['==', PREC.EQUALITY],
        ['!=', PREC.EQUALITY],
        ['~=', PREC.EQUALITY],
        ['|', PREC.BITWISE_OR],
        ['~', PREC.BITWISE_XOR],
        ['&', PREC.BITWISE_AND],
        ['&~', PREC.BITWISE_AND_NOT],
        ['<<', PREC.SHIFT],
        ['>>', PREC.SHIFT],
        ['+', PREC.ADD],
        ['-', PREC.ADD],
        ['*', PREC.MULTIPLY],
        ['/', PREC.MULTIPLY],
        ['%', PREC.MULTIPLY],
        ['%%', PREC.MULTIPLY],
      ];

      return choice(...table.map(([operator, precedence]) => {
        return prec.left(precedence, seq(
          field('left', $.expression),
          // @ts-ignore
          field('operator', operator),
          field('right', $.expression),
        ));
      }));
    },

    ternary_expression: $ => prec.right(seq(
      field('condition', choice($._expression_no_tag, $.struct)),
      choice(
        prec(PREC.TERNARY, seq(
          '?',
          field('consequence', $.expression),
          ':',
          field('alternative', $.expression),
        )),
        seq(
          choice('if', 'when'),
          field('consequence', $.expression),
          'else',
          field('alternative', $.expression),
        ),
      ),
    )),

    call_expression: $ => prec.left(PREC.CALL, seq(
      field('function', choice(seq($.tag, $.identifier), $._expression_no_tag, $.tag)),
      '(',
      optional(seq(
        commaSep1(seq(
          field('argument', choice($.expression, $.array_type, $.struct_type, $.pointer_type, $.procedure)),
          optional(seq('=', choice($.expression))),
        )),
        optional(','),
      )),
      ')',
    )),

    selector_call_expression: $ => prec.left(PREC.CALL, seq(
      field('function', $.expression),
      '->',
      $.call_expression,
    )),

    member_expression: $ => prec.left(PREC.MEMBER, seq(
      optional($.expression),
      '.',
      $.expression,
    )),

    index_expression: $ => prec.left(PREC.MEMBER, seq(
      $.expression,
      '[',
      $.expression,
      optional(seq(',', $.expression)),
      ']',
    )),

    slice_expression: $ => prec.left(PREC.MEMBER, seq(
      $.expression,
      '[',
      optional(field('start', $.expression)),
      ':',
      optional(field('end', $.expression)),
      ']',
    )),

    range_expression: $ => prec.left(PREC.MEMBER, seq(
      $.expression,
      choice('..=', '..<'),
      $.expression,
    )),

    cast_expression: $ => prec.left(PREC.CAST, seq(
      choice(
        seq('(', choice($.pointer_type, $.array_type, $._procedure_type), ')', optional($.expression)),
        seq(choice('cast', 'transmute'), '(', $.type, ')', $.expression),
        seq('auto_cast', $.expression),
      ),
    )),

    in_expression: $ => prec.right(-1, seq($.expression, choice('in', 'not_in'), $.expression)),

    variadic_expression: $ => prec.left(PREC.VARIADIC, seq('..', $.expression)),

    parenthesized_expression: $ => seq('(', $.expression, ')'),

    or_return_expression: $ => seq($.expression, 'or_return'),

    or_continue_expression: $ => prec.right(seq(
      $.expression,
      'or_continue',
      field('label', optional($.identifier)),
    )),

    or_break_expression: $ => prec.right(seq($.expression, 'or_break', optional($.expression))),

    address: $ => seq($.expression, '^'),

    type: $ => prec.right(choice(
      $.identifier,
      $.pointer_type,
      $.variadic_type,
      $.array_type,
      $.map_type,
      $.union_type,
      $.bit_set_type,
      $.matrix_type,
      $.field_type,
      $.tuple_type,
      $.struct_type,
      $.enum_type,
      $.bit_field_type,
      $.constant_type,
      $.specialized_type,
      $._procedure_type,
      $.distinct_type,
      $.empty_type,
      $.polymorphic_type,
      $.conditional_type,
      '...',
    )),

    pointer_type: $ => prec.left(seq('^', $.type)),

    variadic_type: $ => prec.left(seq('..', $.type)),

    array_type: $ => prec(1, seq(
      optional($.tag),
      '[',
      optional(seq(optional('$'), choice('dynamic', '^', '?', $.expression))),
      ']',
      optional($.type),
    )),

    map_type: $ => prec.right(seq('map', '[', $.type, ']', $.type)),

    union_type: $ => prec.right(seq('union', '{', commaSep1($.type), optional(','), '}')),

    bit_set_type: $ => seq(
      'bit_set',
      '[',
      choice($.constant_type, $.expression),
      optional(seq(';', $.type)),
      ']',
    ),

    matrix_type: $ => prec.left(seq(
      'matrix',
      '[',
      choice($.constant_type, $.expression),
      ',',
      choice($.constant_type, $.expression),
      ']',
      $.type,
    )),

    field_type: $ => seq($.identifier, repeat1(seq(token.immediate('.'), $.identifier))),

    tuple_type: $ => seq(
      '(',
      optional(seq(
        commaSep1(choice($.type, $.named_type, $.default_type)),
        optional(','),
      )),
      ')',
    ),

    struct_type: $ => prec(1, seq(
      'struct',
      optional($.polymorphic_parameters),
      repeat(seq($.tag, optional($.number))), // #align 16
      repeat1(seq(
        '{',
        optional($._struct_members),
        '}',
      )),
    )),

    _struct_members: $ => seq(
      commaSep1($.struct_member),
      optional(','),
    ),

    struct_member: $ => seq(
      commaSep1(seq(optional('using'), $.identifier)),
      ':',
      optional($.tag),
      $.type,
      optional($.string),

    ),

    enum_type: $ => seq(
      'enum',
      optional(field('underlying_type', $.type)),
      '{',
      commaSep1(seq($.identifier, optional(seq('=', $.expression)))),
      optional(','),
      '}',
    ),

    bit_field_type: $ => seq(
      'bit_field',
      $.type,
      '{',
      commaSep1(seq($.identifier, ':', $.type, '|', $.expression)),
      optional(','),
      '}',
    ),

    named_type: $ => prec.right(seq(commaSep1($.identifier), ':', $.type, optional(seq('=', $.literal)))),

    default_type: $ => seq($.identifier, ':=', $.expression),

    constant_type: $ => prec.right(seq('$', $.type)),

    specialized_type: $ => prec.right(seq($.type, '/', $.type)),

    _procedure_type: $ => alias($.procedure, $.procedure_type),

    distinct_type: $ => prec.right(seq('distinct', optional($.tag), $.type)),

    empty_type: _ => '!',

    polymorphic_type: $ => seq($.type, '(', commaSep1(choice($.type, $.literal)), ')'),

    conditional_type: $ => seq('(', $.type, 'when', $.expression, 'else', $.type, ')'),

    literal: $ => prec.right(choice(
      $.struct,
      $.map,
      $.bit_set,
      $.matrix,
      $.float,
      $.number,
      $.string,
      $.character,
      $.boolean,
      $.nil,
      $.uninitialized,
    )),

    struct: $ => seq(
      optional(choice(
        seq('[', optional(choice('dynamic', '^', '?', $.expression)), ']', $.type),
        seq(choice($.identifier, $.field_identifier), optional(seq('(', commaSep($.identifier), ')'))),
      )),
      // $.type,
      '{',
      optional(seq(
        commaSep1($.struct_field),
        optional(','),
      )),
      '}',
    ),

    map: $ => seq(
      'map',
      '[',
      $.type,
      ']',
      $.type,
      '{',
      optional(seq(
        commaSep1(seq($.expression, '=', $.expression)),
        optional(','),
      )),
      '}',
    ),

    bit_set: $ => seq(
      'bit_set',
      '[',
      $.expression,
      optional(seq(
        ';',
        field('underlying_type', $.type),
      )),
      ']',
      '{',
      commaSep($.expression),
      '}',
    ),

    matrix: $ => seq(
      'matrix',
      '[',
      $.expression,
      ',',
      $.expression,
      ']',
      $.type,
      '{',
      optional(seq(
        commaSep1($.expression),
        optional(','),
      )),
      '}',
    ),

    struct_field: $ => prec.right(seq(
      $.expression,
      optional(seq(
        '=',
        choice($.expression, $._procedure_type),
      )),
    )),

    number: _ => {
      const decimal = /[0-9][0-9_]*[ijk]?/;
      const hex = /0[xh][0-9a-fA-F_]+[ijk]?/;
      const octal = /0o[0-7][0-7]*[ijk]?/;
      const binary = /0b[01][01_]*[ijk]?/;
      // no float

      return token(choice(
        seq(optional('-'), decimal),
        seq(optional('-'), hex),
        seq(optional('-'), octal),
        seq(optional('-'), binary),
      ));
    },

    string: $ => choice($._string_literal, $._raw_string_literal),

    _string_literal: $ => seq(
      '"',
      repeat(choice(
        $.string_content,
        $.escape_sequence,
      )),
      '"',
    ),

    _raw_string_literal: $ => seq(
      '`',
      repeat(alias($._raw_string_content, $.string_content)),
      '`',
    ),

    character: $ => seq(
      '\'',
      choice(
        /[^'\\]/,
        $.escape_sequence,
      ),
      '\'',
    ),

    string_content: _ => token.immediate(prec(1, /[^"\\]+/)),

    _raw_string_content: _ => token.immediate(prec(1, /[^`]+/)),

    _escape_sequence: $ => choice(
      prec(2, token.immediate(seq('\\', /[^abfnrtvxu'\"\\\?]/))),
      prec(1, $.escape_sequence),
    ),

    escape_sequence: _ => token.immediate(seq(
      '\\',
      choice(
        /[^xu0-7]/,
        /[0-7]{1,3}/,
        /x[0-9a-fA-F]{2}/,
        /u[0-9a-fA-F]{4}/,
        /u\{[0-9a-fA-F]+\}/,
        /U[0-9a-fA-F]{8}/,
      ),
    )),

    boolean: _ => choice('true', 'false'),

    nil: _ => 'nil',

    uninitialized: _ => '---',

    tag: _ => token(seq(/#[a-zA-Z_][a-zA-Z0-9_]*/, optional(seq('(', /\w*/, ')')))),

    identifier: _ => /[_\p{XID_Start}][_\p{XID_Continue}]*/u,

    field_identifier: $ => prec(-1, seq($.identifier, repeat1(seq('.', $.identifier)))),

    keyword_identifier: _ => prec(-3, choice(
      'nil',
      'false',
      'true',
    )),

    _separator: $ => choice(
      $._newline,
      ';',
    ),

    comment: _ => token(seq('//', /[^\r\n]*/)),
  },
});

module.exports.PREC = PREC;

/**
 * Creates a rule to optionally match one or more of the rules separated by a comma
 *
 * @param {Rule} rule
 *
 * @returns {ChoiceRule}
 */
function commaSep(rule) {
  return optional(commaSep1(rule));
}

/**
 * Creates a rule to match one or more of the rules separated by a comma
 *
 * @param {Rule} rule
 *
 * @returns {SeqRule}
 */
function commaSep1(rule) {
  return sep1(rule, ',');
}

/**
 * Creates a rule to match one or more of the rules separated by a comma
 *
 * @param {Rule} rule
 *
 * @param {GrammarSymbols<any>} $
 *
 * @returns {SeqRule}
 */
function commaExternalSep1(rule, $) {
  return sep1(rule, choice(',', alias($._nl_comma, ',')));
}

/**
 * Creates a rule to match zero or more occurrences of `rule` separated by `sep`
 *
 * @param {RegExp | Rule | string} rule
 *
 * @param {RegExp | Rule | string} sep
 *
 * @returns {ChoiceRule}
 */
function sep(rule, sep) {
  return optional(seq(rule, repeat(seq(sep, optional(rule)))));
}

/**
 * Creates a rule to match one or more occurrences of `rule` separated by `sep`
 *
 * @param {RegExp | Rule | string} rule
 *
 * @param {RegExp | Rule | string} sep
 *
 * @returns {SeqRule}
 */
function sep1(rule, sep) {
  return seq(rule, repeat(seq(sep, rule)));
}
