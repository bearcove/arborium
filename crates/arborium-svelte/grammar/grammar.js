/**
 * @file Svelte grammar for tree-sitter
 * @author Amaan Qureshi <amaanq12@gmail.com>
 * @license MIT
 * @see {@link https://svelte.dev|Official website}
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const HTML = require('tree-sitter-html/grammar');

module.exports = grammar(HTML, {
  name: 'svelte',

  conflicts: $ => [
    [$.else_if_block],
    [$.else_block],
    [$.then_block],
    [$.catch_block],
  ],

  externals: ($, _) => [
    $._start_tag_name,
    $._script_start_tag_name,
    $._style_start_tag_name,
    $._end_tag_name,
    $.erroneous_end_tag_name,
    '/>',
    $._implicit_end_tag,
    $.raw_text,
    $.comment,
    $.svelte_raw_text,
    $.svelte_raw_text_each,
    $.svelte_raw_text_snippet_arguments,
    '@',
    '#',
    '/',
    ':',
  ],

  extras: $ => [
    $.comment,
    /\s+/,
  ],

  supertypes: $ => [
    $._node,
  ],

  rules: {
    _node: ($, original) => choice(
      original,

      $.if_statement,
      $.each_statement,
      $.await_statement,
      $.key_statement,
      $.snippet_statement,

      $.expression,

      $.html_tag,
      $.const_tag,
      $.debug_tag,
      $.render_tag,
    ),

    _single_quoted_attribute_value: $ => repeat1(
      choice(
        // Either any random non-expression piece of string…
        /[^{']+/,
        // …or an expression.
        $.expression,
      ),
    ),

    _double_quoted_attribute_value: $ => repeat1(
      choice(
        // Either any random non-expression piece of string…
        /[^{"]+/,
        // …or an expression.
        $.expression,
      ),
    ),

    // Svelte interpolations can occur anywhere inside quoted HTML attributes,
    // so we've got to modify `quoted_attribute_value`.
    quoted_attribute_value: $ => choice(
      seq(
        '\'',
        optional(
          alias($._single_quoted_attribute_value, $.attribute_value),
        ),
        '\'',
      ),
      seq(
        '"',
        optional(
          alias($._double_quoted_attribute_value, $.attribute_value),
        ),
        '"',
      ),
    ),

    // Also, an expression counts as a third type of possible attribute value
    // alongside quoted and unquoted.
    attribute: $ => seq(
      choice(
        seq(
          $.attribute_name,
          optional(seq(
            '=',
            choice(
              $.attribute_value,
              $.quoted_attribute_value,
              $.expression,
            ),
          )),
        ),
        alias($.expression, $.attribute_name),
      ),
    ),

    if_statement: $ => seq(
      $.if_start,
      repeat($._node),
      repeat($.else_if_block),
      optional($.else_block),
      $.if_end,
    ),

    _if_start_tag: _ => tag('#', 'if'),
    if_start: $ => seq(
      '{',
      alias($._if_start_tag, $.block_start_tag),
      field('condition', $.svelte_raw_text),
      '}',
    ),

    _else_if_tag: _ => tag(':', 'else if'),
    else_if_start: $ => seq(
      '{',
      alias($._else_if_tag, $.block_tag),
      field('condition', $.svelte_raw_text),
      '}',
    ),

    else_if_block: $ => seq(
      $.else_if_start,
      repeat($._node),
    ),

    _else_tag: _ => tag(':', 'else'),
    else_start: $ => seq(
      '{',
      alias($._else_tag, $.block_tag),
      '}',
    ),
    else_block: $ => seq(
      $.else_start,
      repeat($._node),
    ),

    _if_end_tag: _ => tag('/', 'if'),
    if_end: $ => seq('{', alias($._if_end_tag, $.block_end_tag), '}'),

    each_statement: $ => seq(
      $.each_start,
      repeat($._node),
      optional(seq($.else_block, repeat($._node))),
      $.each_end,
    ),

    _each_start_tag: _ => tag('#', 'each'),
    each_start: $ => seq(
      '{',
      alias($._each_start_tag, $.block_start_tag),
      choice(
        field('identifier', $.svelte_raw_text),
        seq(
          field('identifier', alias($.svelte_raw_text_each, $.svelte_raw_text)),
          'as',
          field('parameter', $.svelte_raw_text),
        ),
      ),
      '}',
    ),

    _each_end_tag: _ => tag('/', 'each'),
    each_end: $ => seq(
      '{',
      alias($._each_end_tag, $.block_end_tag),
      '}',
    ),

    await_statement: $ => seq(
      $.await_start,
      repeat($._node),
      optional($.then_block),
      optional($.catch_block),
      $.await_end,
    ),

    _await_start_tag: _ => tag('#', 'await'),
    await_start: $ => seq(
      '{',
      alias($._await_start_tag, $.block_start_tag),
      $.svelte_raw_text,
      '}',
    ),

    _then_tag: _ => tag(':', 'then'),
    then_start: $ => seq('{', alias($._then_tag, $.block_tag), optional($.svelte_raw_text), '}'),
    then_block: $ => seq(
      $.then_start,
      repeat($._node),
    ),

    _catch_tag: _ => tag(':', 'catch'),
    catch_start: $ => seq('{', alias($._catch_tag, $.block_tag), optional($.svelte_raw_text), '}'),
    catch_block: $ => seq(
      $.catch_start,
      repeat($._node),
    ),

    _await_end_tag: _ => tag('/', 'await'),
    await_end: $ => seq('{', alias($._await_end_tag, $.block_end_tag), '}'),

    key_statement: $ => seq(
      $.key_start,
      repeat($._node),
      $.key_end,
    ),

    _key_start_tag: _ => tag('#', 'key'),
    key_start: $ => seq('{', alias($._key_start_tag, $.block_start_tag), $.svelte_raw_text, '}'),

    _key_end_tag: _ => tag('/', 'key'),
    key_end: $ => seq('{', alias($._key_end_tag, $.block_end_tag), '}'),

    snippet_statement: $ => seq(
      $.snippet_start,
      repeat($._node),
      $.snippet_end,
    ),

    _snippet_start_tag: _ => tag('#', 'snippet'),
    snippet_start: $ => seq(
      '{',
      alias($._snippet_start_tag, $.block_start_tag),
      alias(/[a-zA-Z$_][a-zA-Z0-9_]*/, $.snippet_name),
      '(',
      optional(
        alias($.svelte_raw_text_snippet_arguments, $.svelte_raw_text),
      ),
      ')',
      '}',
    ),

    _snippet_end_tag: _ => tag('/', 'snippet'),
    snippet_end: $ => seq('{', alias($._snippet_end_tag, $.block_end_tag), '}'),

    expression: $ => seq('{', $.svelte_raw_text, '}'),

    _tag_value: $ => seq(/\s+/, $.svelte_raw_text),

    _html_tag: _ => tag('@', 'html'),
    html_tag: $ => seq(
      '{',
      alias($._html_tag, $.expression_tag),
      $._tag_value,
      '}',
    ),

    _const_tag: _ => tag('@', 'const'),
    const_tag: $ => seq(
      '{',
      alias($._const_tag, $.expression_tag),
      $._tag_value,
      '}',
    ),

    _debug_tag: _ => tag('@', 'debug'),
    debug_tag: $ => seq(
      '{',
      alias($._debug_tag, $.expression_tag),
      $._tag_value,
      '}',
    ),

    _render_tag: _ => tag('@', 'render'),
    render_tag: $ => seq(
      '{',
      alias($._render_tag, $.expression_tag),
      $._tag_value,
      '}',
    ),

    attribute_name: _ => /[^<>{}"'/=\s]+/,

    attribute_value: _ => /[^<>{}"'=\s]+/,

    text: _ => /[^<>{}&\s]([^<>{}&]*[^<>{}&\s])?/,
  },
});

/**
 * @param  {string} sym
 * @param  {string} text
 * @return {SeqRule}
 */
function tag(sym, text) {
  return seq(
    sym,
    field('tag', token.immediate(text)),
  );
}
