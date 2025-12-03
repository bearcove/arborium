const {
  commaSep,
  escaped_ch,
  anySep1,
  anySep,
} = require('../common/common.js');
const lists = require('./common/lists.js');
const title = require('./common/document_title.js');
const table = require('./common/table.js');

module.exports = grammar({
  name: 'asciidoc',

  extras: $ => [$._NEWLINE],

  rules: {
    document: $ => repeat($.block_element),
    ...lists.rules,
    ...title.rules,
    ...table.rules,
    block_element: $ =>
      prec.left(
        seq(
          choice(
            $.document_title,
            $.document_attr,
            $.section_block,
            $.line_comment,
            $.block_comment,
          ),
        ),
      ),

    section_block: $ => choice($._section_block_para, $._section_block),
    _section_block_para: $ =>
      seq(repeat(choice($.element_attr, $.block_title)), $.paragraph),
    _section_block: $ =>
      seq(
        repeat(choice($.element_attr, $.block_title)),
        choice(
          $.title1,
          $.title2,
          $.title3,
          $.title4,
          $.title5,
          $.list,
          $.table_block,
          $.delimited_block,
          $.listing_block,
          $.literal_block,
          $.ident_block,
          $.open_block,
          $.breaks,
          $.admonition,
          $.quoted_block,
          $.quoted_md_block,
          $.passthrough_block,
          $.block_macro,
        ),
      ),
    title1: $ => seq($.title_h1_marker, $._WHITE_SPACE, $.line),
    title2: $ => seq($.title_h2_marker, $._WHITE_SPACE, $.line),
    title3: $ => seq($.title_h3_marker, $._WHITE_SPACE, $.line),
    title4: $ => seq($.title_h4_marker, $._WHITE_SPACE, $.line),
    title5: $ => seq($.title_h5_marker, $._WHITE_SPACE, $.line),
    breaks: $ => seq($.breaks_marker, $._block_end),

    admonition: $ =>
      seq(
        choice(
          $.admonition_note,
          $.admonition_tip,
          $.admonition_important,
          $.admonition_caution,
          $.admonition_warning,
        ),
        ':',
        token.immediate(' '),
        $.line,
      ),
    block_macro: $ =>
      seq(
        $.block_macro_name,
        '::',
        alias(repeat(escaped_ch('[')), $.target),
        '[',
        commaSep($.block_macro_attr),
        ']',
        $._block_end,
      ),
    block_macro_attr: $ =>
      seq($.attribute_name, optional(seq('=', $.attribute_value))),
    attribute_name: $ => repeat1(escaped_ch('=')),
    attribute_value: $ => repeat1(escaped_ch(']')),

    escaped_line: $ =>
      repeat1(choice(/[^/\n]/, /\/[^*]/, /\\\r?\n/, seq($.hard_wrap))),
    hard_wrap: $ => ' +',
    element_attr: $ =>
      seq(
        $.element_attr_marker,
        alias(token(repeat(escaped_ch(']', true))), $.attr_value),
        alias(']', $.element_attr_marker),
        $._NEWLINE,
      ),
    block_title: $ => seq($.block_title_marker, $.line),

    delimited_block: $ =>
      prec.left(
        seq(
          $.delimited_block_start_marker,
          repeat($.section_block),
          $.delimited_block_end_marker,
        ),
      ),
    open_block: $ =>
      prec.left(
        seq($.open_block_marker, repeat($.section_block), $.open_block_marker),
      ),
    passthrough_block: $ =>
      seq(
        $.passthrough_block_marker,
        alias(repeat(seq(/[^\r\n]*/, $._NEWLINE)), $.listing_block_body),
        $.passthrough_block_marker,
      ),
    listing_block: $ =>
      prec.left(
        seq(
          $.listing_block_start_marker,
          alias(
            repeat(
              choice(
                seq(repeat1(choice(/[^\r\n]/, $.callout_marker)), $._block_end),
                $.block_macro,
              ),
            ),
            $.listing_block_body,
          ),
          $.listing_block_end_marker,
          optional($.callout_list),
        ),
      ),
    literal_block: $ =>
      prec.left(
        seq(
          $.literal_block_marker,
          alias(
            repeat(
              choice(
                seq(repeat1(choice(/[^\r\n]/, $.callout_marker)), $._block_end),
                $.block_macro,
              ),
            ),
            $.literal_block_body,
          ),
          $.literal_block_marker,
          optional($.callout_list),
        ),
      ),
    ident_block: $ =>
      prec.left(seq(repeat1($.ident_block_line), optional($.callout_list))),
    ident_block_line: $ =>
      seq(
        $.ident_marker,
        repeat1(choice(/[^\r\n]/, $.callout_marker)),
        $._block_end,
      ),

    line: $ => seq(/[^\r\n]+/, $._block_end),
    paragraph: $ =>
      prec.left(prec(-1, seq(repeat1($.line), optional($.quoted_line)))),
    quoted_line: $ =>
      seq($.quoted_paragraph_marker, token.immediate(' '), $.line),

    line_comment: $ =>
      seq(
        $.line_comment_marker,
        optional(alias(/[^\r\n]+/, $.body)),
        $._block_end,
      ),
    block_comment: $ =>
      seq(
        $.block_comment_start_marker,
        alias(repeat(seq(/[^\r\n]+/, $._NEWLINE)), $.body),
        $.block_comment_end_marker,
      ),

    quoted_block: $ =>
      choice(seq($.quoted_block_marker, repeat($.line), $.quoted_block_marker)),
    quoted_md_block: $ =>
      prec.right(
        repeat1(
          choice(
            seq($.quoted_block_md_marker, $._NEWLINE),
            seq($.quoted_block_md_marker, token.immediate(' '), $.line),
          ),
        ),
      ),

    _block_end: $ => choice($._NEWLINE, $._eof),
    _NEWLINE: _ => /\r?\n/,
    _WHITE_SPACE: $ => /[ \t]+/,
  },

  externals: $ => [
    $._eof,
    $.title_h0_marker,
    $.title_h1_marker,
    $.title_h2_marker,
    $.title_h3_marker,
    $.title_h4_marker,
    $.title_h5_marker,
    $.list_marker_star,
    $.list_marker_hyphen,
    $.list_marker_dot,
    $.list_marker_digit,
    $.list_marker_geek,
    $.list_marker_alpha,
    $.document_attr_marker,
    $.element_attr_marker,
    $.block_title_marker,
    $.breaks_marker,
    $.table_block_marker,
    $.ntable_block_marker,
    $.table_cell_attr,
    $.delimited_block_start_marker,
    $.delimited_block_end_marker,
    $.listing_block_start_marker,
    $.listing_block_end_marker,
    $.literal_block_marker,
    $.quoted_block_marker,
    $.quoted_block_md_marker,
    $.quoted_paragraph_marker,
    $.open_block_marker,
    $.passthrough_block_marker,
    $.block_macro_name,
    $.callout_marker,
    $.callout_list_marker,
    $.line_comment_marker,
    $.block_comment_start_marker,
    $.block_comment_end_marker,
    $.admonition_note,
    $.admonition_tip,
    $.admonition_important,
    $.admonition_caution,
    $.admonition_warning,
    $.ident_marker,
    $.list_continuation,
  ],
});
