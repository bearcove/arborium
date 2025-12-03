import { sep1 } from './util.js';

export default {
  // ------------------------------------------------------------------------
  // pragma

  _pragma_arg: $ => choice(
    $._q_name_op,
    $.literal,
    $.string,
    $.wildcard,
    $.totality,
  ),

  // ------------------------------------------------------------------------
  // Global pragmas

  pragma_language: $ => seq('%language', $._pragma_arg),

  pragma_default: $ => seq('%default', $._pragma_arg),

  pragma_builtin: $ => seq('%builtin', $._pragma_arg),

  pragma_name: $ => seq('%name', $._pragma_arg, sep1($.comma, $._pragma_arg)),

  pragma_ambiguity_depth: $ => seq('%ambiguity_depth', $._pragma_arg),

  pragma_auto_implicit_depth: $ => seq('%auto_implicit_depth', $._pragma_arg),

  pragma_logging: $ => seq('%logging', repeat1($._pragma_arg)),

  pragma_prefix_record_projections: $ => seq('%prefix_record_projections', $._pragma_arg),

  pragma_transform: $ => seq('%transform', $.string, $._aexps, $.equal, $._aexps),

  pragma_unbound_implicits: $ => seq('%unbound_implicits', $._pragma_arg),

  pragma_auto_lazy: $ => seq('%auto_lazy', $._pragma_arg),

  pragma_search_timeout: $ => seq('%search_timeout', $._pragma_arg),

  pragma_nf_metavar_threshold: $ => seq('%nf_metavar_threshold', $._pragma_arg),

  pragma_cg: $ => seq('%cg', $._pragma_arg, $._varid, $.equal, $._aexp),

  pragma_hide: $ => seq('%hide', $._pragma_arg),

  pragma_unhide: $ => seq('%unhide', $._pragma_arg),

  pragma_allow_overloads: $ => seq('%allow_overloads', $._pragma_arg),

  _pragma_global: $ => choice(
    $.pragma_language,
    $.pragma_default,
    $.pragma_builtin,
    $.pragma_name,
    $.pragma_ambiguity_depth,
    $.pragma_auto_implicit_depth,
    $.pragma_logging,
    $.pragma_prefix_record_projections,
    $.pragma_transform,
    $.pragma_unbound_implicits,
    $.pragma_auto_lazy,
    $.pragma_search_timeout,
    $.pragma_nf_metavar_threshold,
    $.pragma_cg,
    $.pragma_hide,
    $.pragma_unhide,
    $.pragma_allow_overloads,
  ),

  // ------------------------------------------------------------------------
  // Declaration pragmas

  pragma_deprecate: _ => '%deprecate',

  pragma_inline: _ => '%inline',

  pragma_noinline: _ => '%noinline',

  pragma_tcinline: _ => '%tcinline',

  pragma_unsafe: _ => '%unsafe',

  pragma_spec: $ => seq('%spec', $._pragma_arg),

  pragma_foreign: $ => seq(
    '%foreign',
    repeat1(seq(optional($._name), $.string),
    ),
  ),

  pragma_foreign_impl: $ => seq('%foreign_impl', $._pragma_arg, $._pragma_arg),

  pragma_export: $ => seq('%export', $._pragma_arg),

  pragma_nomangle: _ => '%nomangle',

  pragma_hint: _ => '%hint',

  pragma_defaulthint: _ => '%defaulthint',

  pragma_globalhint: _ => '%globalhint',

  pragma_extern: _ => '%extern',

  pragma_macro: _ => '%macro',

  pragma_start: _ => '%start',

  _pragma_decl: $ => choice(
    $.pragma_deprecate,
    $.pragma_inline,
    $.pragma_noinline,
    $.pragma_tcinline,
    $.pragma_unsafe,
    $.pragma_spec,
    $.pragma_foreign,
    $.pragma_foreign_impl,
    $.pragma_export,
    $.pragma_nomangle,
    $.pragma_hint,
    $.pragma_defaulthint,
    $.pragma_globalhint,
    $.pragma_extern,
    $.pragma_macro,
    $.pragma_start,
  ),

  // ------------------------------------------------------------------------
  // Internal pragmas

  pragma_rewrite: $ => seq('%rewrite', $._pragma_arg, $._pragma_arg),

  pragma_pair: $ => seq('%pair', $._pragma_arg, $._pragma_arg, $._pragma_arg),

  pragma_integerLit: $ => seq('%integerLit', $._pragma_arg),

  pragma_stringLit: $ => seq('%stringLit', $._pragma_arg),

  pragma_charLit: $ => seq('%charLit', $._pragma_arg),

  pragma_doubleLit: $ => seq('%doubleLit', $._pragma_arg),

  _pragma_internal: $ => choice(
    $.pragma_rewrite,
    $.pragma_pair,
    $.pragma_integerLit,
    $.pragma_stringLit,
    $.pragma_charLit,
    $.pragma_doubleLit,
  ),

  // ------------------------------------------------------------------------
  // Reflection literal pragmas

  pragma_TTImpLit: $ => seq('%TTImpLit', $._pragma_arg),

  pragma_declsLit: $ => seq('%declsLit', $._pragma_arg),

  pragma_nameLit: $ => seq('%nameLit', $._pragma_arg),

  _pragma_reflection: $ => choice(
    $.pragma_TTImpLit,
    $.pragma_declsLit,
    $.pragma_nameLit,
  ),

  // ------------------------------------------------------------------------
  // Expression pragmas

  pragma_runElab: _ => '%runElab',

  pragma_search: _ => '%search',

  pragma_World: _ => '%World',

  pragma_MkWorld: _ => '%MkWorld',

  pragma_syntactic: _ => '%syntactic',

  _pragma_exp: $ => choice(
    $.pragma_runElab,
    $.pragma_search,
    $.pragma_World,
    $.pragma_MkWorld,
    $.pragma_syntactic,
  ),
};
