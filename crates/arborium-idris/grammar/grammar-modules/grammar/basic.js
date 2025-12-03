const decimals1 = /[1-9][0-9_]*/
const exponent = /e[+-]?[1-9_]+/

export default {
  // ------------------------------------------------------------------------
  // literals
  // ------------------------------------------------------------------------

  // the `choice` here is necessary to avoid integers being parsed as numbers
  number: _ => token(
    seq(
      choice('0', decimals1),
      choice(
        seq(/\.[0-9][0-9_]*/, optional(exponent)),
        exponent,
      ),
    ),
  ),

  char: _ => token(
    choice(
      /'[^']'/,
      /'\\[^ ]*'/,
    ),
  ),

  _integer_literal: _ => token(choice('0', decimals1)),
  _hex_literal: _ => token(/0x[0-9a-fA-F_]+/),

  integer: $ => choice(
    $._integer_literal,
    $._hex_literal,
  ),

  _numeric: $ => choice(
    $.integer,
    $.number,
  ),

  _literal: $ => choice(
    $.char,
    $._numeric,
  ),

  literal: $ => $._literal,

  _rcarrow: _ => choice('⇒', '=>'),

  _lcarrow: _ => choice('⇐', '<='),

  _arrow: _ => choice('→', '->'),

  _larrow: _ => choice('←', '<-'),

  colon: _ => ':',

  double_star: _ => '**',

  forall: _ => choice('forall', '∀'),

  _def_equal: _ => choice(':=', '='),

  equal: _ => '=',

  wildcard: _ => '_',

  unit: _ => '()',

  /**
   * Same as varid_pattern except it's preceded by `?`, plus `'` and `_` are allowed
   * to be the first and only characters after `?`.
   */
  hole: _ => /\?[\p{L}0-9_']+/,

  /**
   * Field projection dot-syntax requires the dot to follow a varid without any whitespace.
   */
  _immediate_dot: _ => token.immediate('.'),
};
