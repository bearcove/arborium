const immd = (x) => token.immediate(x);

module.exports = grammar({

  name: 'ninja',

  word: $ => $.identifier,

  externals: $ => [
  ],

  extras: $ => [
    /[ \r]+/,
    $.comment
  ],

  inline: $ => [
    $._ident,
    $._value,
    $._outputs,
    $._dependencies,
  ],

  rules: {

    manifest: $ => repeat($._stmt),

    _stmt: $ => choice(
      $.pool,
      $.build,
      $.rule,
      $.default,
      $.let,
      $.include,
      $.subninja,
      $._nl
    ),

    // In the same order as ninja/src/manifest_parser.cc
    pool: $ => seq(
      'pool',
      $._ident,
      $._nl,
      $.body
    ),

    rule: $ => seq(
      'rule',
      $._ident,
      $._nl,
      $.body
    ),

    let: $ => seq(
      $._ident,
      '=',
      $._value,
      $._nl
    ),

    default: $ => seq(
      'default',
      $._targets,
      $._nl
    ),

    build: $ => seq(
      'build',
      optional(seq(     field('explicit', $._outputs))),
      optional(seq('|', field('implict' , $._outputs))),
      ':',
      field('rule', $.identifier),
      optional(seq(      field('explicit'  , $._dependencies))),
      optional(seq('|' , field('implict'   , $._dependencies))),
      optional(seq('||', field('order_only', $._dependencies))),
      optional(seq('|@', field('validation', $._dependencies))),
      $._nl,
      optional($.body),
    ),

    include: $ => seq(
      'include',
      $.path,
      $._nl
    ),

    subninja: $ => seq(
      'subninja',
      $.path,
      $._nl
    ),

    body: $ => repeat1(seq(
      $._indent,
      $.let
    )),

    //
    // Text and names
    // ==============
    _value: $ => field('value', $.text),

    text: $ => repeat1(choice(
      /[^$\n]+/,
      $._escape,
    )),

    _targets:      $ => alias($.paths, $.targets),
    _outputs:      $ => alias($.paths, $.outputs),
    _dependencies: $ => alias($.paths, $.dependencies),

    paths: $ => seq(' ', optional($.path)),

    path: $ => repeat1(choice(
      $._escape,
      /[^$ :|\r\n]/
    )),

    //
    // Escape sequences
    // ----------------
    _escape: $ => choice(
      $.quote,
      $.split,
      $.expansion
    ),

    quote: $ => seq('$', immd(choice('$',' ',':'))),
    split: $ => seq('$', immd(seq(choice('\n','\r\n'),repeat(' ')))),

    expansion: $ => choice(
      $._simple_var_expansion,
      $._var_expansion,
    ),

    _simple_var_expansion: $ => seq(
      '$',
      alias($.simple_identifier, $.identifier)
    ),

    _var_expansion: $ => seq(
      '$',
      immd(prec(2,'{')),
      $.identifier,
      immd('}')
    ),

    //
    // Tokens
    // ======
    _ident: $ => field('name', $.identifier),

    simple_identifier: $ => token(/[a-zA-Z0-9_-]+/),
    identifier: $ => token(/[a-zA-Z0-9_.-]+/),

    comment: $ => token(prec(1,/#.*\r?\n/)),

    _nl: $ => token(choice('\r\n','\n')),

    _indent: $ => token(repeat1(' ')),
  }

});
