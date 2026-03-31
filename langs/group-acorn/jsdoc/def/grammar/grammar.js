/**
 * @file JSDoc grammar for tree-sitter
 * @author Max Brunsfeld <maxbrunsfeld@gmail.com>
 * @author Amaan Qureshi <amaanq12@gmail.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: 'jsdoc',

  externals: $ => [
    $.type,
    $.code_block_line,
  ],

  extras: _ => [
    token(choice(
      // Skip over stars at the beginnings of lines
      seq(/\n/, /[ \t]*/, repeat(seq('*', /[ \t]*/))),
      /\s/,
    )),
  ],

  supertypes: $ => [
    $.expression,
  ],

  rules: {
    document: $ => seq(
      $._begin,
      optional($.description),
      repeat($.tag),
      $._end,
    ),

    description: $ => choice(
      seq(
        choice($._text, $.code_block),
        repeat(choice($._text, $.inline_tag, $._inline_tag_false_positive, $.code_block)),
      ),
    ),

    tag: $ => choice(
      // type, name, and description
      seq(
        alias($.tag_name_with_argument, $.tag_name),
        optional(seq('{', $.type, '}')),
        optional(choice($.expression, $.optional_identifier)),
        optional($.description),
      ),

      // type and description
      seq(
        alias($.tag_name_with_type, $.tag_name),
        optional(seq('{', $.type, '}')),
        optional($.description),
      ),

      // description only
      seq(
        $.tag_name,
        optional($.description),
      ),
    ),

    inline_tag: $ => seq(
      '{',
      $.tag_name,
      $.description,
      '}',
    ),

    _inline_tag_false_positive: _ => token(prec.left(1, /\{[^@}]+\}?/)),

    tag_name_with_argument: _ => token(choice(
      '@access',
      '@alias',
      '@api',
      '@augments',
      '@borrows',
      '@callback',
      '@constructor',
      '@event',
      '@exports',
      '@external',
      '@extends',
      '@fires',
      '@function',
      '@import',
      '@member',
      '@mixes',
      '@name',
      '@namespace',
      '@param',
      '@property',
      '@prop',
      '@satisfies',
      '@typedef',
    )),

    tag_name_with_type: _ => token(choice(
      '@return',
      '@returns',
      '@this',
      '@throw',
      '@throws',
      '@type',
    )),

    tag_name: _ => /@[a-zA-Z_]+/,

    expression: $ => choice(
      $.identifier,
      $.number,
      $.member_expression,
      $.path_expression,
      $.qualified_expression,
      $.array_expression,
    ),

    qualified_expression: $ => prec(1, seq(
      $.identifier,
      ':',
      $.expression,
    )),

    path_expression: $ => prec(2, seq(
      $.identifier,
      token.immediate('/'),
      $.identifier,
    )),

    member_expression: $ => seq(
      $.expression,
      choice(
        '.',
        '#',
        '~',
      ),
      choice(
        $.identifier,
        $.qualified_expression,
      ),
    ),

    array_expression: $ => seq(
      '[',
      commaSep($.expression),
      ']',
    ),

    code_block: $ => seq(
      '```',
      optional($.code_block_language),
      repeat($.code_block_line),
      '```',
    ),

    code_block_language: _ => /[a-z]+/,

    code_block_line: _ => /(([^`\n][^`\n][^`\n]).)+/,

    optional_identifier: $ => prec(1, seq(
      '[',
      $.identifier,
      optional(seq('=', field('value', $.expression))),
      ']',
    )),

    identifier: _ => /[a-zA-Z_$][a-zA-Z_$0-9]*/,

    // https://github.com/tree-sitter/tree-sitter-javascript/blob/22da14e17db59c35aae2b9da2728337c301ce741/grammar.js#L1096
    number: _ => {
      const hexLiteral = seq(
        choice('0x', '0X'),
        /[\da-fA-F](_?[\da-fA-F])*/,
      );

      const decimalDigits = /\d(_?\d)*/;
      const signedInteger = seq(optional(choice('-', '+')), decimalDigits);
      const exponentPart = seq(choice('e', 'E'), signedInteger);

      const binaryLiteral = seq(choice('0b', '0B'), /[0-1](_?[0-1])*/);

      const octalLiteral = seq(choice('0o', '0O'), /[0-7](_?[0-7])*/);

      const bigintLiteral = seq(choice(hexLiteral, binaryLiteral, octalLiteral, decimalDigits), 'n');

      const decimalIntegerLiteral = choice(
        '0',
        seq(optional('0'), /[1-9]/, optional(seq(optional('_'), decimalDigits))),
      );

      const decimalLiteral = choice(
        seq(decimalIntegerLiteral, '.', optional(decimalDigits), optional(exponentPart)),
        seq('.', decimalDigits, optional(exponentPart)),
        seq(decimalIntegerLiteral, exponentPart),
        decimalDigits,
      );

      return token(choice(
        hexLiteral,
        decimalLiteral,
        binaryLiteral,
        octalLiteral,
        bigintLiteral,
      ));
    },
    _text: _ => token(prec(-1, /[^*{}@\s][^*{}\n]*([^*/{}\n][^*{}\n]*\*+)*/)),

    _begin: _ => seq('/', repeat('*')),

    _end: _ => '/',
  },
});

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
  return seq(rule, repeat(seq(',', rule)));
}
