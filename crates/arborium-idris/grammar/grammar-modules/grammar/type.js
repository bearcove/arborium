import {
  parens,
  braces,
  sep1,
  sep2
} from './util.js';

export default {

  arrow_separator: $ => choice($._arrow, $._rcarrow),

  quantity: $ => alias($.integer, ''),

  auto: _ => 'auto',

  default: $ => seq('default', $._aexp),

  type_parens: $ => parens($._type_parens),

  _type_parens: $ => choice(
    $._type,
    $.type_var,
    sep2($.comma, $._type),
    sep2($.comma, $.type_var),
    sep2($.double_star, choice($._type, $.type_var)),
  ),

  type_var: $ => seq(
    optional($.quantity),
    sep1($.comma, $._avar),
    $._type_annotation,
  ),

  _avar: $ => choice(
    $._name,
    $.literal,
    $.wildcard,
    $.unit,
  ),

  type_braces: $ => braces(
    seq(
      sep1($.comma, $._implicit_arg),
      optional($._type_annotation),
    ),
  ),

  _implicit_arg: $ => seq(
    repeat(
      choice(
        $.quantity,
        $.auto,
        $.default,
      ),
    ),
    $._aexp,
  ),

  _quantifiers: $ => seq($.forall, sep1($.comma, $._loname), '.'),

  _type_annotation: $ => seq($.colon, $._type),

  _nullary_type_annotation: $ => seq($.colon, $._aexps),

  _type: $ => sep1(
    $.arrow_separator,
    seq(
      optional($._quantifiers),
      $._exp,
      optional(seq($.equal, $._exp)),
    ),
  ),
};
