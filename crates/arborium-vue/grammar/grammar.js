/**
 * @file Tree-sitter grammar definition
 * @author Amaan Qureshi <amaanq12@gmail.com>
 * @license MIT
 * @see {@link https://tree-sitter.github.io/tree-sitter/creating-parsers}
 */

const HTML = require('tree-sitter-html/grammar');

module.exports = grammar(HTML, {
  name: 'vue',

  externals: ($, original) => original.concat([
    $._template_start_tag_name,
    $._text_fragment,
    $._interpolation_text,
  ]),

  rules: {
    _node: ($, original) => choice(
      original,
      $.template_element,
      $.interpolation,
    ),

    template_element: $ => seq(
      alias($.template_start_tag, $.start_tag),
      repeat($._node),
      $.end_tag,
    ),

    start_tag: $ => seq(
      '<',
      alias($._start_tag_name, $.tag_name),
      repeat($._attribute),
      '>',
    ),

    script_start_tag: $ => seq(
      '<',
      alias($._script_start_tag_name, $.tag_name),
      repeat($._attribute),
      '>',
    ),

    style_start_tag: $ => seq(
      '<',
      alias($._style_start_tag_name, $.tag_name),
      repeat($._attribute),
      '>',
    ),

    template_start_tag: $ => seq(
      '<',
      alias($._template_start_tag_name, $.tag_name),
      repeat($._attribute),
      '>',
    ),

    self_closing_tag: $ => seq(
      '<',
      alias(choice($._start_tag_name, $._template_start_tag_name), $.tag_name),
      repeat($._attribute),
      '/>',
    ),


    text: $ => $._text_fragment,

    interpolation: $ => seq(
      '{{',
      optional(alias($._interpolation_text, $.raw_text)),
      '}}',
    ),

    _attribute: $ => choice(
      $.attribute,
      $.directive_attribute,
    ),

    directive_attribute: $ => prec.right(seq(
      choice(
        seq(
          $.directive_name,
          optional(seq(
            immediateLiterals(1, ':'),
            choice($.directive_value, $.dynamic_directive_value),
          )),
        ),
        repeat1(seq(
          tokenLiterals(1, ':', '.', '@', '#'),
          choice($.directive_value, $.dynamic_directive_value),
        )),
      ),
      optional($.directive_modifiers),
      optional(seq(
        '=',
        choice($.attribute_value, $.quoted_attribute_value),
      )),
    )),

    directive_name: _ => token(prec(1, /v-[^<>'"=/\s:.]+/)),

    directive_value: _ => /[^<>"'/=\s:.]+/,

    dynamic_directive_value: $ => seq(
      token.immediate(prec(1, '[')),
      optional($.dynamic_directive_inner_value),
      token.immediate(']'),
    ),

    dynamic_directive_inner_value: _ => token.immediate(/[^<>"'/=\s\]]+/),

    directive_modifiers: $ => repeat1(seq(token.immediate(prec(1, '.')), $.directive_modifier)),

    directive_modifier: _ => token.immediate(/[^<>"'/=\s.]+/),
  },
});

/**
 *
 * Turns a list of rules into a choice of immediate rule
 *
 * @param {number} precedence
 *
 * @param {(RegExp|String)[]} literals
 *
 * @return {ChoiceRule}
 */
function immediateLiterals(precedence, ...literals) {
  return choice(...literals.map(l => token.immediate(prec(precedence, l))));
}

/**
 *
 * Turns a list of rules into a choice of aliased token rules
 *
 * @param {number} precedence
 *
 * @param {(RegExp|String)[]} literals
 *
 * @return {ChoiceRule}
 */
function tokenLiterals(precedence, ...literals) {
  return choice(...literals.map(l => token(prec(precedence, l))));
}
