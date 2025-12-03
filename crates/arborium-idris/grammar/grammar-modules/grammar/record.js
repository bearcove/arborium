import { layouted, sep1, sep, parens, braces } from './util.js';

export default {
  // ----- Record -------------------------------------------------------------

  record_name: $ => $._q_caname,

  _decl_record: $ => seq(
    optional($.visibility),
    'record',
    field('name', choice($.record_name, parens($.operator))),
    repeat($._type),
    $.where,
    optional($.record_body),
  ),

  record_body: $ => layouted($, $._record_decl),

  _record_decl: $ => choice(
    $.constructor,
    alias($._record_field, $.record_field),
    alias($._implicit_record_field, $.implicit_record_field),
  ),

  constructor: $ => seq(
    'constructor',
    field('name', $._caname_op),
  ),

  _record_field: $ => seq(
    optional($.quantity),
    sep1($.comma, $._q_name_op), $._type_annotation
  ),

  _implicit_record_field: $ => braces(
    seq(
      optional($.auto),
      $._record_field,
    )
  ),

  // -----------------------------------------------------------------
  // Value-level

  _field_name: $ => alias($.loname, $.field_name),

  _record_field_update: $ => seq(
    $._field_name,
    choice(
      seq(choice(':=', '$='), $._exp),
      braces(sep($.comma,
        alias($._record_field_update, $.record_update)
      ))
    ),
  ),

  record_update: $ => braces(sep1($.comma, $._record_field_update)),
};
