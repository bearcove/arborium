import { sep, sep1, parens, layouted } from './util.js';

export default {
  // ------------------------------------------------------------------------
  // Declarations
  // ------------------------------------------------------------------------

  visibility: _ => choice(
    'private',
    'export',
    seq('public', 'export'),
  ),

  totality: _ => choice(
    'total',
    'partial',
    'covering',
  ),

  _decl_prefix: $ => repeat1(
    choice(
      $.visibility,
      $.totality,
      $.quantity,
      $._pragma_decl,
    ),
  ),

  _funrhs: $ => seq(
    $._def_equal,
    choice($._exp, $._type),
  ),

  _fun_patterns: $ => prec(1, repeat1($._apat)),

  _funvar: $ => seq(
    field('subject', $._name_op),
    field('patterns', optional(alias($._fun_patterns, $.patterns))),
  ),

  _with_res: $ => seq('|', alias($._apats, $.with_pat)),

  _funlhs: $ => seq(
    choice(
      prec.dynamic(1, alias($._funvar, $.funvar)),
      prec.dynamic(0, $._apats),
    ),
  ),

  proof: $ => seq('proof', $._aexp),

  with_arg: $ => seq(
    parens(sep($.tuple_operator, $._exp)),
    optional($.proof)
  ),

  with: $ => seq(
    'with',
    optional($.quantity),
    sep1('|', $.with_arg),
    '\n',
    layouted($, $.function),
  ),

  impossible: _ => 'impossible',

  function: $ => seq(
    alias($._funlhs, $.lhs),
    repeat($._with_res),
    choice(
      alias($._funrhs, $.rhs),
      $.with,
      $.impossible
    ),
    optional(seq($.where, $.declarations)),
  ),

  fixity: $ => seq(
    optional($.visibility),
    choice('infixl', 'infixr', 'infix', 'prefix'),
    field('precedence', $.integer),
    sep1($.comma, choice($.operator, $.ticked_operator)),
  ),

  signature: $ => seq(
    optional($._decl_prefix),
    sep1(
      $.comma,
      choice(
        field('name', $._name_op),
        $.wildcard,
      ),
    ),
    $._type_annotation,
  ),

  _gendecl: $ => choice(
    $.signature,
    $.fixity,
  ),

  _decl_fun: $ => $.function,

  _decl: $ => choice(
    $._gendecl,
    $._decl_fun,
  ),

  _where_decl: $ => choice(
    $._decl,
    alias($._decl_namespace, $.namespace),
    alias($._decl_parameters, $.parameters),
    alias($._decl_mutual, $.mutual),
    alias($._decl_data, $.data),
    alias($._decl_record, $.record),
    alias($._decl_interface, $.interface),
    alias($._decl_implementation, $.implementation),
  ),

  declarations: $ => layouted($, $._where_decl),
};
