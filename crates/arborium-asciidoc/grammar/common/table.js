const { anySep1, escaped_ch, anySep } = require('../../common/common');

exports.rules = {
  table_block: $ =>
    prec.left(
      seq(
        $.table_block_marker,
        repeat(choice($.table_cell, $.ntable_block)),
        $.table_block_marker,
      ),
    ),
  table_cell: $ =>
    prec.right(
      seq(
        optional($.table_cell_attr),
        choice(
          seq(
            '|',
            token.immediate(/\r?\n/),
            anySep(
              alias($._section_block, $.section_block),
              $.list_continuation,
            ),
          ),
          seq('|', $.table_cell_content),
        ),
      ),
    ),
  table_cell_content: $ => repeat1(choice(/[^|]/, '\\|')),

  ntable_block: $ =>
    prec.left(
      seq(
        optional($.element_attr),
        $.ntable_block_marker,
        repeat($.ntable_cell),
        $.ntable_block_marker,
      ),
    ),
  ntable_cell: $ =>
    seq(
      optional($.table_cell_attr),
      choice(
        seq(
          '!',
          token.immediate(/\r?\n/),
          alias($._section_block, $.section_block),
        ),
        seq('!', repeat1(escaped_ch('!'))),
      ),
    ),
};
