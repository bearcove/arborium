import { brackets, qualified, where, sep1 } from "./util.js";

export default {

  // ----- Shared -------------------------------------------------------------

  interface_name: $ => $._q_caname,

  constraints: $ => repeat1(
    seq($._aexps, $.arrow_separator)
  ),

  // ----- Interface ----------------------------------------------------------

  _decl_interface: $ => seq(
    optional($._decl_prefix),
    'interface',
    $.interface_head,
    optional($.interface_body)
  ),

  interface_head: $ => seq(
    optional($.constraints),
    field('name', $.interface_name),
    optional($._aexps),
    optional($.determining_params)
  ),

  determining_params: $ => seq('|', sep1($.comma, $.loname)),

  interface_body: $ => where($, $._interface_decl),

  _interface_decl: $ => choice(
    $.constructor,
    $._decl,
    alias($._decl_parameters, $.using),
  ),


  // ----- Implementation -----------------------------------------------------

  _idecl: $ => choice(
    $.function,
    $.signature,
  ),

  implementation_head: $ => seq(
    optional($._decl_prefix),
    optional('implementation'),
    optional(brackets($.implementation_name)),
    optional($.constraints),
    field('subject', $.interface_name),
    optional($._aexps),
    optional($.using),
  ),

  using: $ => seq('using', repeat1($.implementation_name)),

  implementation_name: $ => choice(
    $._qualified_implementation_name,
    $._implementation_name,
  ),

  _implementation_name: $ => $._name,

  _qualified_implementation_name: $ => qualified($, $._implementation_name),

  implementation_body: $ => where($, $._decl),

  _decl_implementation: $ =>
    seq(
      $.implementation_head,
      optional($.implementation_body),
    ),
};
