import { brackets, layouted, sep1 } from './util.js';

export default {

  // ----- Data ---------------------------------------------------------------

  data_name: $ => alias($._q_caname_op, ''),

  _decl_data: $ => seq(
    choice(
      $._decl_data_inline,
      $._decl_data_block,
    ),
  ),

  _decl_data_inline: $ => seq(
    optional($._decl_prefix),
    'data',
    field('name', $.data_name),
    repeat($._q_loname),
    '=',
    sep1('|', $._aexps),
  ),

  _decl_data_block: $ => seq(
    optional($._decl_prefix),
    'data',
    field('name', $.data_name),
    optional(alias($._type_annotation, $.type_signature)),
    optional(seq(
      $.where,
      optional($.data_body),
    )),
  ),

  data_body: $ => layouted($, $._data_decl),

  _data_decl: $ => choice(
    $.search_options,
    $.external,
    $.signature,
  ),

  search_options: $ => brackets(
    choice(
      seq('search', repeat1($._name)),
      alias('noHints', $.no_hints),
    ),
  ),

  external: _ => brackets('external'),
};
