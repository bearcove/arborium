const { escaped_ch, anySep1 } = require('../../common/common');

exports.rules = {
  autolink: $ =>
    choice(
      $.uri,
      $.labled_uri,
      seq('"', $.uri, '"'),
      seq('"', $.labled_uri, '"'),
    ),
  labled_uri: $ => prec(1, seq($.uri, seq('[', optional($.uri_label), ']'))),

  uri_label: $ => repeat1(escaped_ch(']', false, $.replacement)),
  uri: $ => choice($.link_url, $.email),
  link_url: $ =>
    seq(
      choice(seq(/\w[\w\d+.-][\w\d+.-]*:\/\//), /www\./i),
      prec.right(anySep1($._uri_segment, '.')),
    ),
  // biome-ignore lint/complexity/noUselessEscapeInRegex: xxx
  _uri_segment: $ => /[^\.\s\[\"]+/,

  email: $ =>
    // biome-ignore lint/suspicious/noControlCharactersInRegex: xxx
    /(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])/,
};
