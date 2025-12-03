const { anySep1, commaSep, escaped_ch } = require('../common/common.js');
const autolink = require('./common/autolink.js');

// prettier-ignore
const PUNCTUATION_CHARACTERS_ARRAY = [
  '!',
  '"',
  '#',
  '$',
  '%',
  '&',
  "'",
  '(',
  ')',
  '*',
  '+',
  ',',
  '-',
  '.',
  '/',
  ':',
  ';',
  '<',
  '=',
  '>',
  '?',
  '@',
  '[',
  '\\',
  ']',
  '^',
  '_',
  '`',
  '{',
  '|',
  '}',
  '~',
];

module.exports = grammar({
  name: 'asciidoc_inline',
  externals: $ => [$._eof],
  precedences: $ => [
    [$.autolink, $._punctuation],
    [$.passthrough, $._punctuation],
  ],

  rules: {
    inline: $ => repeat($.inline_element),
    inline_element: $ =>
      choice(
        $.replacement,
        $.word,
        $.autolink,
        $.passthrough,
        $.macro_passthrough,
        $._punctuation,
        $.xref,
        $.emphasis,
        $.ltalic,
        $.monospace,
        $.highlight,
        $.inline_macro,
        $.stem_macro,
        $.footnote,
        $.index_term,
        $.index_term2,
        $.id_assignment,
        $.intrinsic_attributes_pair,
      ),
    ...autolink.rules,
    id_assignment: $ => choice(seq('[#', $.id, ']'), seq('[[', $.id, ']]')),
    id: $ => repeat1(escaped_ch(']')),
    macro_name: $ =>
      choice(
        'kbd',
        'btn',
        'image',
        'audio',
        'video',
        'icon',
        'link',
        'mailto',
        'menu',
        'anchor',
        'xref',
        'ifdef',
        'ifndef',
        'ifeval',
        'endif',
        'indexterm2',
        'indexterm',
        // diagram
        'a2s',
        'barcode',
        'blockdiag',
        'bpmn',
        'bytefield',
        'd2',
        'dbml',
        'diagrams',
        'ditaa',
        'dpic',
        'erd',
        'gnuplot',
        'graphviz',
        'graphviz',
        'lilypond',
        'meme',
        'mermaid',
        'msc',
        'nomnoml',
        'pikchr',
        'plantuml',
        'shaape',
        'smcat',
        'structurizr',
        'svgbob',
        'symbolator',
        'syntrax',
        'tikz',
        'umlet',
        'vega',
        'wavedrom',
      ),
    inline_macro: $ =>
      seq(
        $.macro_name,
        token.immediate(':'),
        optional(choice($.target)),
        '[',
        optional($.attr),
        ']',
      ),
    target: $ =>
      repeat1(
        escaped_ch(
          '[',
          false,
          $.replacement,
          $.escaped_sequence,
          $.passthrough,
          $.macro_passthrough,
        ),
      ),
    attr: $ =>
      repeat1(
        escaped_ch(
          ']',
          false,
          $.replacement,
          $.autolink,
          $.escaped_sequence,
          prec.left(-1, '"'),
        ),
      ),

    _footnotename: $ => choice('footnote', 'footnoteref'),
    footnote: $ =>
      seq(
        alias($._footnotename, $.macro_name),
        token.immediate(':'),
        optional($.target),
        '[',
        optional($.attr),
        ']',
      ),

    _stem_attr: $ => repeat1(choice(/[^\]]/, '\\]')),
    _stem_name: $ => choice('latexmath', 'stem', 'asciimath'),
    stem_macro: $ =>
      seq(
        alias($._stem_name, $.macro_name),
        token.immediate(':'),
        optional($.target),
        '[',
        optional(alias($._stem_attr, $.attr)),
        ']',
      ),
    macro_passthrough: $ =>
      seq(
        alias('pass', $.macro_name),
        token.immediate(':'),
        optional($.target),
        '[',
        optional(alias($._stem_attr, $.attr)),
        ']',
      ),
    replacement: _ =>
      token(
        choice(
          '(C)',
          '(R)',
          '(TM)',
          '...',
          "`'",
          '-&gt;',
          '=&gr;',
          '&lt;-',
          '&lt;=',
        ),
      ),
    intrinsic_attributes_pair: $ => seq('{', $.intrinsic_attributes, '}'),
    intrinsic_attributes: $ =>
      token(
        choice(
          'startsb',
          'endsb',
          'vbar',
          'caret',
          'asterisk',
          'tilde',
          'plus',
          'backslash',
          'backtick',
          'blank',
          'empty',
          'sp',
          'two-colons',
          'two-semicolons',
          'nbsp',
          'deg',
          'zwsp',
          'quot',
          'apos',
          'lsquo',
          'rsquo',
          'ldquo',
          'rdquo',
          'wj',
          'brvbar',
          'pp',
          'cpp',
          'amp',
          'lt',
          'gt',
        ),
      ),
    word: $ =>
      choice(
        $.super_escape,
        $._character,
        $._fallback_token,
        $.escaped_sequence,
        'f',
      ),
    _fallback_token: $ => choice($.macro_name, $._stem_name, $._footnotename),
    super_escape: $ => '\\\\',
    _character: $ =>
      token(
        prec(
          -1,
          new RegExp(
            `[^f\\s${PUNCTUATION_CHARACTERS_ARRAY.map(item => {
              if (item === ']') {
                return '\\]';
              }
              return item;
            }).join('')}]+`,
          ),
        ),
      ),
    escaped_sequence: $ => {
      const args = [
        '+++',
        '``',
        '**',
        '$$',
        '##',
        '__',
        '<<',
        '[[',
        '++',
        'kbd',
        'btn',
        'image',
        'audio',
        'video',
        'icon',
        'pass',
        'link',
        'mailto',
        'menu',
        'stem',
        'latexmath',
        'asciimath',
        'footnote',
        'footnoteref',
        'anchor',
        'xref',
        'ifdef',
        'ifndef',
        'ifeval',
        'endif',
      ].map(sequence => {
        return token(`\\${sequence}`);
      });

      return choice(...args, /\\./);
    },
    _punctuation: _ => choice(...PUNCTUATION_CHARACTERS_ARRAY),
    passthrough: $ =>
      choice(
        create_text_formatting('+'),
        seq('$$', repeat1(escaped_ch('$', true, '\\$$')), '$$'),
      ),
    xref: $ =>
      seq(
        '<<',
        alias(repeat1(escaped_ch(',>')), $.id),
        optional(seq(',', alias(repeat1(escaped_ch('>')), $.reftext))),
        '>>',
      ),

    emphasis: $ =>
      create_text_formatting('*', $.ltalic, $.monospace, $.highlight),
    ltalic: $ =>
      create_text_formatting('_', $.emphasis, $.monospace, $.highlight),
    monospace: $ => create_text_formatting('`'),
    highlight: $ => create_text_formatting('#'),

    index_term2: $ => seq('((', $.term, optional(seq(',', $.term)), '))'),
    index_term: $ =>
      seq(
        '(((',
        $.term,
        optional(seq(',', $.term)),
        optional(seq(',', $.term)),
        ')))',
      ),
    term: $ => repeat1(escaped_ch(',)')),
  },
});

function create_text_formatting(ch, ...args) {
  return choice(
    seq(token(prec(1, ch)), repeat(escaped_ch(ch, true, ...args)), ch),
    seq(
      token(prec(1, ch + ch)),
      repeat(escaped_ch(ch, true, ...args)),
      ch + ch,
    ),
  );
}
