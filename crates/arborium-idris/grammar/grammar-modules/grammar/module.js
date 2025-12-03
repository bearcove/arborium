import { layouted, terminated } from './util.js';

export default {
  // ------------------------------------------------------------------------
  // module
  // ------------------------------------------------------------------------

  _qualifying_module: $ => repeat1(seq(
    field('module_name', $.caname),
    $._dot
  )),

  _q_module: $ => $._q_caname,

  _decl_module: $ => seq(
    'module',
    field('name', $._q_module),
  ),

  _decl_namespace: $ => seq(
    'namespace',
    field('name', $._q_module),
    optional(alias($._top_declarations, $.namespace_body)),
  ),

  _decl_mutual: $ => seq(
    'mutual',
    optional(alias($._top_declarations, $.mutual_body)),
  ),

  _top_declarations: $ => layouted($, $._topdecl),

  // Parameters block starts with parameter lists, which can be placed
  // at the same indent level of the body.
  //
  // parameters (a : A) (b : B)
  //   (c : C)
  //   f : a -> b
  //
  _decl_parameters: $ => seq(
    choice(
      'parameters',
      'using',
    ),
    repeat($._parameter),
    '\n',
    optional($._parameters_body),
  ),

  _parameter: $ => choice(
    $.type_parens,
    $.type_braces,
  ),

  _parameters_body: $ => seq(
    $._layout_start,
    repeat(seq(
      optional($._layout_restart),
      terminated($, $._parameter),
    )),
    optional(seq(
      optional($._layout_restart),
      alias(terminated($, $._topdecl), $.parameters_body),
    )),
    $._layout_end,
  ),
};
