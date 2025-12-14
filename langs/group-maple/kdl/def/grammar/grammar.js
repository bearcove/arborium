/**
 * @file KDL v2 grammar for tree-sitter
 * @license MIT
 * @see {@link https://kdl.dev/spec/|KDL v2 specification (KDL 2.0.0)}
 */

// deno-lint-ignore-file no-control-regex
/* eslint-disable arrow-parens */
/* eslint-disable camelcase */
/* eslint-disable-next-line spaced-comment */
/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const ANNOTATION_BUILTINS = [
  'i8',
  'i16',
  'i32',
  'i64',
  'u8',
  'u16',
  'u32',
  'u64',
  'isize',
  'usize',
  'f32',
  'f64',
  'decimal64',
  'decimal128',
  'date-time',
  'time',
  'date',
  'duration',
  'decimal',
  'currency',
  'country-2',
  'country-3',
  'country-subdivision',
  'email',
  'idn-email',
  'hostname',
  'idn-hostname',
  'ipv4',
  'ipv6',
  'url',
  'url-reference',
  'irl',
  'iri-reference',
  'url-template',
  'uuid',
  'regex',
  'base64',
];

// Whitespace tables from the KDL v2 spec.
const UNICODE_SPACE_RE =
  /[\u0009\u0020\u00A0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A\u202F\u205F\u3000]/;
const NEWLINE_RE = /\r\n|[\r\n\u0085\u000C\u2028\u2029]/;

// KDL v2 identifier characters:
// identifier-char := unicode - unicode-space - newline - [\\/(){};\[\]"#=]
// For parsing/highlighting we approximate "unicode-space" as `\s` and use the
// explicit forbidden punctuation set from the v2 spec.
const IDENT_CHAR = /[^\s\\\/(){};\[\]"#=]/;
const IDENT_START_UNAMBIGUOUS = /[^\s\d+\-\.\\\/(){};\[\]"#=]/;
const IDENT_START_SIGNED = /[^\s\d\.\\\/(){};\[\]"#=]/;
const IDENT_START_DOTTED = /[^\s\d\\\/(){};\[\]"#=]/;

module.exports = grammar({
  name: 'kdl',

  conflicts: $ => [
    [$.document],
    [$._node_space],
    [$.node_children],
    [$.nodes],
    [$.multi_line_string_body],
    [$._quoted_string_multi, $.string_character],
    [$.prop, $.value],
    [$.value],
    [$.node_no_terminator],
    [$.node, $.final_node],
  ],

  externals: $ => [
    $._eof,
    $.multi_line_comment,
    $.raw_string,
  ],

  extras: $ => [$.multi_line_comment],

  word: $ => $.identifier_string,

  rules: {
    document: $ => seq(
      optional($._bom),
      optional($.version_marker),
      repeat($._line_space),
      repeat(seq($.node, repeat($._line_space))),
    ),

    // version := '/-' unicode-space* 'kdl-version' unicode-space+ ('1' | '2') unicode-space* newline
    version_marker: $ => seq(
      $._version_marker_prefix,
      repeat1($._unicode_space),
      field('version', choice('1', '2')),
      repeat($._unicode_space),
      $._newline,
    ),

    // Tokenize the distinctive `/- ... kdl-version` prefix as a single token to
    // avoid conflicts with plain `/-` (slashdash) nodes.
    _version_marker_prefix: _ => token(
      new RegExp(`/-${UNICODE_SPACE_RE.source}*kdl-version`),
    ),

    // nodes := (line-space* node)* line-space*
    // Tree-sitter doesn't allow non-start rules that match the empty string,
    // so we model "possibly empty nodes" as `optional(nodes)` at call sites.
    nodes: $ => seq(
      repeat1(seq(
        $.node,
        repeat($._line_space),
      )),
    ),

    // node := base-node node-terminator
    node: $ => seq($.node_no_terminator, $.node_terminator),

    // final-node := base-node node-terminator?
    //
    // In practice this rule is only used inside `{ ... }` to allow the last node
    // before `}` to omit its terminator. We model it as a terminator-less node
    // to avoid ambiguity with `node` (which always consumes a terminator).
    final_node: $ => $.node_no_terminator,

    // base-node := slashdash? type? node-space* string (node-space+ ...)*
    //
    // This is a pragmatic approximation of the v2 grammar that prioritizes
    // unambiguous parsing for highlighting and tooling.
    node_no_terminator: $ => prec.right(seq(
      repeat($._node_space),
      optional($.slashdash),
      optional($.type),
      repeat($._node_space),
      field('name', $.string),

      repeat(seq(
        repeat1($._node_space),
        optional($.slashdash),
        choice(
          field('entry', $.node_prop_or_arg),
          field('children', $.node_children),
        ),
      )),

      repeat($._node_space),
    )),

    // node-children := '{' nodes final-node? '}'
    // node-children := '{' nodes final-node? '}'  (KDL v2)
    // We support either:
    //   - one or more fully-terminated nodes (from `nodes`), optionally followed by a final node
    //   - just a final node
    node_children: $ => seq(
      '{',
      repeat($._line_space),
      optional(seq(
        $.nodes,
        repeat($._line_space),
      )),
      optional($.final_node),
      repeat($._node_space),
      repeat($._line_space),
      '}',
    ),

    // slashdash := '/-' line-space*
    // We model this as just the marker token and let surrounding rules consume
    // whitespace/comments explicitly to avoid shift/reduce conflicts.
    slashdash: _ => '/-',

    // node-terminator := single-line-comment | newline | ';' | eof
    node_terminator: $ =>
      choice($.single_line_comment, $._newline, ';', $._eof),

    // node-prop-or-arg := prop | value
    node_prop_or_arg: $ => choice($.prop, $.value),

    // prop := string node-space* '=' node-space* value
    prop: $ => prec(1, seq(
      field('key', $.string),
      repeat($._node_space),
      '=',
      repeat($._node_space),
      field('value', $.value),
    )),

    // value := type? node-space* (string | number | keyword)
    value: $ => seq(
      optional($.type),
      repeat($._node_space),
      choice($.string, $.number, $.keyword),
    ),

    // type := '(' node-space* string node-space* ')'
    type: $ => seq(
      '(',
      repeat($._node_space),
      field('name', choice($.string, $.annotation_type)),
      repeat($._node_space),
      ')',
    ),

    // Strings
    // string := identifier-string | quoted-string | raw-string
    string: $ => choice($.identifier_string, $.quoted_string, $.raw_string),

    // identifier-string := unambiguous-ident | signed-ident | dotted-ident
    identifier_string: _ => token(choice(
      // unambiguous-ident := (identifier-char - digit - sign - '.') identifier-char*
      seq(IDENT_START_UNAMBIGUOUS, repeat(IDENT_CHAR)),
      // signed-ident := sign ((identifier-char - digit - '.') identifier-char*)?
      seq(/[+\-]/, optional(seq(IDENT_START_SIGNED, repeat(IDENT_CHAR)))),
      // dotted-ident := sign? '.' ((identifier-char - digit) identifier-char*)?
      seq(optional(/[+\-]/), '.', optional(seq(IDENT_START_DOTTED, repeat(IDENT_CHAR)))),
    )),

    // quoted-string :=
    //   '"' single-line-string-body '"' |
    //   '"""' newline (multi-line-string-body newline)? (unicode-space | ws-escape)* '"""'
    quoted_string: $ => choice($._quoted_string_single, $._quoted_string_multi),

    _quoted_string_single: $ => seq(
      '"',
      repeat(choice($.escape, $.ws_escape, $.single_line_string_text)),
      '"',
    ),

    _quoted_string_multi: $ => seq(
      '"""',
      $._newline,
      optional(seq($.multi_line_string_body, $._newline)),
      repeat(choice($._unicode_space, $.ws_escape)),
      '"""',
    ),

    // multi-line-string-body := (('"' | '""')? string-character)*
    // This structure prevents the body from containing the closing delimiter `"""`.
    // (If the body is empty, the optional(...) wrapper in _quoted_string_multi handles it.)
    multi_line_string_body: $ => repeat1(choice(
      seq(optional(choice('""', '"')), $.string_character),
      $._newline,
    )),

    // string-character :=
    //   '\\' (["\\/bfnrts] | 'u{' hex-unicode '}') |
    //   ws-escape |
    //   [^\\"] - disallowed-literal-code-points
    //
    // We accept a broad approximation here and leave disallowed code point checks
    // to higher-level validation (Arborium uses this grammar primarily for parsing/highlighting).
    string_character: $ => choice(
      $.escape,
      $.ws_escape,
      token.immediate(/[^\\"\r\n\u0085\u000C\u2028\u2029]/),
    ),

    single_line_string_text: _ => token.immediate(
      /[^\\"\r\n\u0085\u000C\u2028\u2029]+/,
    ),

    // ws-escape := '\' (unicode-space | newline)+
    ws_escape: _ => token.immediate(
      new RegExp(`\\\\(?:${UNICODE_SPACE_RE.source}|${NEWLINE_RE.source})+`),
    ),

    // escape := ["\\/bfnrts] | 'u{' hex-digit{1, 6} '}'
    escape: _ => token.immediate(
      /\\\\|\\"|\\b|\\f|\\n|\\r|\\t|\\\/|\\u\{[0-9a-fA-F]{1,6}\}/,
    ),

    // Numbers
    number: $ => choice($.keyword_number, $._decimal, $._hex, $._octal, $._binary),

    keyword_number: _ => choice('#inf', '#-inf', '#nan'),

    // decimal := sign? integer ('.' integer)? exponent?
    _decimal: $ =>
      seq(
        optional($._sign),
        $._integer,
        optional(seq('.', alias($._integer, $.decimal))),
        optional(alias($._exponent, $.exponent)),
      ),

    _exponent: $ => seq(choice('e', 'E'), optional($._sign), $._integer),
    _integer: $ => seq($._digit, repeat(choice($._digit, '_'))),
    _digit: _ => /[0-9]/,
    _sign: _ => choice('+', '-'),

    _hex: $ => seq(optional($._sign), '0x', $._hex_digit, repeat(choice($._hex_digit, '_'))),
    _octal: $ => seq(optional($._sign), '0o', /[0-7]/, repeat(choice(/[0-7]/, '_'))),
    _binary: $ => seq(optional($._sign), '0b', choice('0', '1'), repeat(choice('0', '1', '_'))),

    _hex_digit: _ => /[0-9a-fA-F]/,

    // Keywords
    keyword: $ => choice($.boolean, '#null'),
    boolean: _ => choice('#true', '#false'),

    // type annotations
    annotation_type: _ => choice(...ANNOTATION_BUILTINS),

    // Comments
    single_line_comment: $ => seq(
      '//',
      repeat(/[^\r\n\u0085\u000C\u2028\u2029]/),
      choice($._newline, $._eof),
    ),

    // Whitespace (KDL v2)
    _ws: $ => choice($._unicode_space, $.multi_line_comment),

    // escline := '\\' ws* (single-line-comment | newline | eof)
    _escline: $ => seq('\\', repeat($._ws), choice($.single_line_comment, $._newline, $._eof)),

    // node-space := ws* escline ws* | ws+
    _node_space: $ =>
      choice(
        seq(repeat($._ws), $._escline, repeat($._ws)),
        repeat1($._ws),
      ),

    // line-space := node-space | newline | single-line-comment
    //
    // For tree-sitter parsing we keep line-space narrow (newline + single-line comment).
    // `node-space` is accepted explicitly by rules that need it (e.g. document/node_children)
    // and by nodes themselves.
    _line_space: $ => choice($._newline, $.single_line_comment),

    _bom: _ => /\u{FEFF}/,

    _unicode_space: _ => UNICODE_SPACE_RE,

    // Treat CRLF as a single newline token by matching it first.
    _newline: _ => token(choice(/\r\n/, /\r/, /\n/, /\u0085/, /\u000C/, /\u2028/, /\u2029/)),
  },
});
