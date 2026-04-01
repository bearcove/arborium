/**
 * @file Kconfig grammar for tree-sitter
 * @author Amaan Qureshi <amaanq12@gmail.com>
 * @license MIT
 * @see {@link https://www.kernel.org/doc/html/next/kbuild/kconfig-language.html|Kconfig Language Specification}
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
  OR: 1,
  AND: 2,
  EQUAL: 3,
  ASSIGN: 4,
  UNARY: 5,
};

module.exports = grammar({
  name: 'kconfig',

  conflicts: $ => [
    [$.expression, $.name],
  ],

  extras: $ => [
    /\s/,
    /\\\r?\n/,
    $.comment,
  ],

  externals: $ => [
    $._help_text,
  ],

  supertypes: $ => [
    $.expression,
  ],

  word: $ => $.symbol,

  rules: {
    configuration: $ => repeat($._entry),

    _entry: $ => choice(
      $.mainmenu,
      $.config,
      $.configdefault,
      $.menuconfig,
      $.choice,
      $.comment_entry,
      $.menu,
      $.if,
      $.source,
      $.variable,
    ),

    mainmenu: $ => seq('mainmenu', field('name', $.string)),

    config: $ => seq(
      'config',
      field('name', $.name),
      repeat1($._config_option),
    ),

    configdefault: $ => seq(
      'configdefault',
      field('name', $.name),
      repeat1($.default_value),
    ),

    menuconfig: $ => seq(
      'menuconfig',
      field('name', $.name),
      repeat1($._config_option),
    ),

    choice: $ => seq(
      'choice',
      optional(field('name', $.name)),
      repeat1($._config_option),
      repeat($._entry),
      'endchoice',
    ),

    comment_entry: $ => seq(
      'comment',
      field('name', $.string),
      repeat($._config_option),
    ),

    menu: $ => seq(
      'menu',
      field('name', $.string),
      repeat($._config_option),
      repeat($._entry),
      'endmenu',
    ),

    if: $ => seq(
      'if',
      field('condition', $.expression),
      repeat($._entry),
      'endif',
    ),

    source: $ => seq(
      choice('source', 'rsource', 'osource', 'orsource'),
      $.string,
    ),

    variable: $ => seq(
      field('left', $.symbol),
      choice('=', ':=', '+=', '?='),
      choice(
        repeat(choice(',', field('right', $.expression))),
        $.text,
      ),
      /\r?\n/,
    ),

    _config_option: $ => choice(
      $.type_definition,
      $.input_prompt,
      $.default_value,
      $.type_definition_default,
      $.dependencies,
      $.reverse_dependencies,
      $.weak_reverse_dependencies,
      $.limiting_menu_display,
      $.numerical_ranges,
      $.help_text,
      $.optional,
      $.modules,
    ),

    type_definition: $ => seq(
      choice('bool', 'tristate', 'int', 'hex', 'string'),
      optional(choice($.string, $.input_prompt)),
      optional($.conditional_clause),
      /\n/,
    ),

    input_prompt: $ => seq(
      'prompt',
      $.string,
      optional($.conditional_clause),
      /\n/,
    ),

    default_value: $ => seq(
      'default',
      $.expression,
      optional($.conditional_clause),
      /\n/,
    ),

    type_definition_default: $ => seq(
      choice('def_bool', 'def_tristate', 'def_int', 'def_hex', 'def_string'),
      $.expression,
      optional($.conditional_clause),
      /\n/,
    ),

    dependencies: $ => seq(
      'depends on',
      $.expression,
      /\n/,
    ),

    reverse_dependencies: $ => seq(
      'select',
      $.name,
      optional($.conditional_clause),
      /\n/,
    ),

    weak_reverse_dependencies: $ => seq(
      'imply',
      $.symbol,
      optional($.conditional_clause),
      /\n/,
    ),

    limiting_menu_display: $ => seq(
      'visible if',
      $.expression,
    ),

    numerical_ranges: $ => seq(
      'range',
      choice($.symbol, $.macro_variable),
      choice($.symbol, $.macro_variable),
      optional($.conditional_clause),
      /\n/,
    ),

    help_text: $ => seq(
      'help',
      alias($._help_text, $.text),
    ),

    optional: _ => 'optional',

    modules: _ => 'modules',

    conditional_clause: $ => seq('if', $.expression),

    expression: $ => choice(
      $.symbol,
      $.name,
      $.string,
      $.macro_variable,
      $.unary_expression,
      $.binary_expression,
      $.parenthesized_expression,
    ),

    unary_expression: $ => prec.left(PREC.UNARY, seq(
      '!',
      $.expression,
    )),

    binary_expression: $ => {
      const table = [
        ['||', PREC.OR],
        ['&&', PREC.AND],
        ['=', PREC.ASSIGN],
        [choice('!=', '<', '>', '<=', '>='), PREC.EQUAL],
      ];

      // @ts-ignore
      return choice(...table.map(([operator, precedence]) => prec.left(precedence, seq(
        field('left', $.expression),
        // @ts-ignore
        field('operator', operator),
        field('right', $.expression),
      ))));
    },

    parenthesized_expression: $ => seq('(', $.expression, ')'),

    macro_variable: $ => seq(
      '$(',
      repeat(choice(
        $.macro_variable,
        $.macro_content,
        alias($.string, $.text),
      )),
      ')',
    ),
    macro_content: _ => /([^\$'"\)]|(\([^\)]*\))|(\\\$)|\$[^(])+/,

    string: $ => choice(
      seq(
        '"',
        repeat(choice(
          alias(/([^"\\]|\$[^\(])+/, $.string_content),
          /\\(.|\n)/,
          $.macro_variable,
        )),
        '"',
      ),
      seq(
        '\'',
        repeat(choice(
          alias(/([^'\\]|\$[^\(])+/, $.string_content),
          /\\(.|\n)/,
          $.macro_variable,
        )),
        '\'',
      ),
    ),

    symbol: _ => /-?[a-zA-Z0-9_-]+/,

    text: _ => /[^\s].*/,

    name: $ => prec.right(prec.dynamic(-1, repeat1(choice(
      $.symbol,
      $.macro_variable,
      $.string,
    )))),

    comment: _ => token(seq('#', /.*/)),
  },
});
