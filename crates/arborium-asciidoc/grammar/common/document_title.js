const { anySep1, escaped_ch } = require('../../common/common');

exports.rules = {
  document_title: $ =>
    seq(
      $.title_h0_marker,
      $._WHITE_SPACE,
      $.line,
      optional(seq($.author_line, seq($._NEWLINE, optional($.revision_line)))),
      repeat(choice($.document_attr, $.block_macro)),
      $._block_end,
    ),
  author_line: $ => anySep1($.author, seq(';', $._WHITE_SPACE)),
  author: $ =>
    seq(
      alias($._author_line_word, $.firstname),
      $._WHITE_SPACE,
      optional(seq(alias($._author_line_word, $.middlename), $._WHITE_SPACE)),
      alias($._author_line_word, $.lastname),
      optional(seq($._WHITE_SPACE, '<', $.email, '>')),
    ),
  _author_line_word: $ => repeat1(escaped_ch(' ;', true)),
  email: $ =>
    // biome-ignore lint/suspicious/noControlCharactersInRegex: xxx
    /(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])/,
  revision_line: $ =>
    seq(
      $.revnumber,
      ',',
      $._WHITE_SPACE,
      $.revdate,
      ':',
      $._WHITE_SPACE,
      $.revremark,
    ),
  revnumber: $ => repeat1(escaped_ch(',', true)),
  revdate: $ => repeat1(escaped_ch(':', true)),
  revremark: $ => repeat1(escaped_ch('', true)),
  document_attr: $ =>
    seq(
      $.document_attr_marker,
      alias(repeat1(escaped_ch(':', true)), $.attr_name),
      alias(':', $.document_attr_marker),
      optional(seq(token.immediate(' '), alias($.escaped_line, $.line))),
      $._block_end,
    ),
};
