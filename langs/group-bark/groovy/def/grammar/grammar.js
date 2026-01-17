const PREC = {
  DEFAULT: 1,
  PRIORITY: 2,
  ELVIS: 3, // ?:
  OR: 4, // ||
  AND: 5, // &&
  BIN_OR: 6, // |
  BIN_XOR: 7, // ^
  BIN_AND: 8, // &
  COMPARE_EQ: 9, // == != <=> === !== =~ ==~
  COMPARE: 10, // < <= > >= in !in instanceof !instanceof as
  SHIFT: 11, // << >> >>> .. ..< <..< <..
  PLUS: 12, // + -
  STAR: 13, // * / %
  UNARY: 14, // +x -x ++x --x
  POW: 15, // **
  TOP: 16, // new () [] {} . .& .@ ?. * *. *: ~ ! (type) x[y] ++ --
  STATEMENT: 17
}

const regexp_or = (regexes) => new RegExp(regexes.map(r =>'(?:('+r.source+'))').join('|'))

const VARIABLE_REGEX = /[$_a-z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u00F8\u0100-\uFFFE][$_0-9a-zA-Z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u00F8\u0100-\uFFFE]*/
const CONSTANT_REGEX = /[_A-Z][$_0-9A-Z]*/
const IDENTIFIER_REGEX = regexp_or([VARIABLE_REGEX, CONSTANT_REGEX])
const TYPE_REGEX =  /[A-Z][$_0-9a-zA-Z\u00C0-\u00D6\u00D8-\u00F6\u00F8-\u00F8\u0100-\uFFFE]*/

const list_of = (e) => seq(
  repeat(prec.left(seq(e, ','))),
  seq(e, optional(',')),
)

module.exports = grammar({
  name: 'groovy',

  extras: $ => [/\s/, $.comment, $.groovy_doc],

  word: $ => $.identifier,

  conflicts: $ => [ //TODO: dynamic precedence, heuristics? eg capital letter
    [$._callable_expression, $.juxt_function_call],
    [$._callable_expression, $._juxt_argument_list],
    [$._juxtable_expression, $._juxt_argument_list],
  ],

  rules: {
    source_file: $ => seq(
      optional($.shebang),
      repeat($._statement),
      optional($.pipeline)
    ),

    shebang: $ => seq(
      '#!', /[^\n]*/
    ),

    _statement: $ => prec.left(PREC.STATEMENT, seq(
      optional($.label),
      choice(
        $.assertion,
        $.groovy_import,
        $.groovy_package,
        $.assignment,
        $.class_definition,
        $.declaration,
        $.do_while_loop,
        $.for_in_loop,
        $.for_loop,
        $.function_call,
        $.function_declaration,
        $.function_definition,
        $.if_statement,
        $.juxt_function_call,
        // $.pipeline_step_with_block,
        $.return,
        $.switch_statement,
        $.try_statement,
        $.while_loop,
        $.closure,
        alias("break", $.break),
        alias("continue", $.continue),
        $._expression,
        // $.step,
      ),
      optional(';')
    )),

    label: $ => seq(field('name', $.identifier), ":"),

    access_op: ($) =>
      choice(
        ...[
          [".&", PREC.TOP],
          [".@", PREC.TOP],
          ["?.", PREC.TOP],
          ["*.", PREC.TOP],
        ].map(([operator, precedence]) =>
          prec.left(precedence, seq($._expression, operator, $._expression))
        ),
        ...[
          ["*", PREC.TOP],
          ["*:", PREC.TOP],
        ].map(([operator, precedence]) =>
          prec.left(precedence, seq(operator, $._expression))
        ),
      ),

    dotted_identifier: $ =>
      prec.left(1, seq(
        choice($._primary_expression, $._type_identifier),
        repeat1(seq(
        '.',
        choice(
          $.identifier,
          $._type_identifier,
          $.parenthesized_expression,
        ))),
      )),

    _import_name: $ => choice(
      $.identifier,
      $._type_identifier,
      seq($._import_name, '.', choice($.identifier, $._type_identifier)),
    ),

    groovy_import: $ => seq(
      'import',
      optional($.modifier),
      field('import', alias($._import_name, $.qualified_name)),
      optional(
        choice(
          seq('.', alias(token.immediate('*'), $.wildcard_import)),
          seq('as', field('import_alias', choice($.identifier, $._type_identifier))),
        ),
      )
    ),

    groovy_package: $ => seq('package', alias($._import_name, $.qualified_name)),

    annotation: $ => prec.right(seq(
      '@',
      alias(token.immediate(regexp_or([IDENTIFIER_REGEX, TYPE_REGEX])), $.identifier),
      optional($.argument_list),
    )),

    assertion: $ => seq('assert', $._expression),

    assignment: $ => prec(-1, choice( //??? is -1 ok here? (fixes conflict with expression for ++)
      seq(
        choice($._juxtable_expression, $.parenthesized_expression),
        choice('=', '**=', '*=', '/=', '%=', '+=', '-=',
          '<<=', '>>=', '>>>=', '&=', '^=', '|=', '?='),
        $._expression
      ),
      $.increment_op,
    )),

    increment_op: $ => prec(2, choice(
      prec.left(PREC.UNARY, seq($._primary_expression, "++")),
      prec.left(PREC.UNARY, seq($._primary_expression, "--")),
      prec.right(PREC.UNARY, seq("++", $._primary_expression)),
      prec.right(PREC.UNARY, seq("--", $._primary_expression)),
    )),

    binary_op: ($) =>
      choice(
        ...[
          ["%", PREC.STAR],
          ["*", PREC.STAR],
          ["/", PREC.STAR],
          ["+", PREC.PLUS],
          ["-", PREC.PLUS],
          ["<<", PREC.SHIFT],
          [">>", PREC.SHIFT],
          [">>>", PREC.SHIFT],
          ["..", PREC.SHIFT],
          ["..<", PREC.SHIFT],
          ["<..<", PREC.SHIFT],
          ["<..", PREC.SHIFT],
          ["<", PREC.COMPARE],
          ["<=", PREC.COMPARE],
          [">", PREC.COMPARE],
          [">=", PREC.COMPARE],
          ["in", PREC.COMPARE],
          ["!in", PREC.COMPARE],
          ["instanceof", PREC.COMPARE],
          ["!instanceof", PREC.COMPARE],
          ["as", PREC.COMPARE],
          ["==", PREC.COMPARE_EQ],
          ["!=", PREC.COMPARE_EQ],
          ["<=>", PREC.COMPARE_EQ],
          ["===", PREC.COMPARE_EQ],
          ["!==", PREC.COMPARE_EQ],
          ["=~", PREC.COMPARE_EQ],
          ["==~", PREC.COMPARE_EQ],
          ["&", PREC.BIN_AND],
          ["^", PREC.BIN_XOR],
          ["|", PREC.BIN_OR],
          ["&&", PREC.AND],
          ["||", PREC.OR],
          ["?:", PREC.ELVIS],
        ].map(([operator, precedence]) =>
          prec.left(precedence, seq($._expression, operator, $._expression))
        ),
        prec.right(PREC.POW, seq($._expression, "**", $._expression))
      ),

    boolean_literal: $ => choice('true', 'false'),

    class_definition: $ => seq(
      repeat($.annotation),
      optional($.access_modifier),
      repeat($.modifier),
      choice('@interface', 'interface', 'class'),
      field('name', choice($.identifier, $._type_identifier)),
      optional(field('generics', $.generic_parameters)),
      optional(seq(
        'extends',
        field('superclass', $._primary_expression),
      )),
      field('body', $.closure),
    ),

    generic_parameters: $ => seq(
      '<',
      list_of($.generic_param),
      '>'
    ),

    generic_param: $ => seq(
      field('name', $.identifier),
      optional(seq(
        'extends',
        field('superclass', $._type),
      ))
    ),

    closure: $ => seq(
      '{',
      optional(choice('->', seq(alias($._param_list, $.parameter_list), '->'))),
      // repeat(choice($._statement, $._expression)),
      repeat($._statement),
      optional($._expression),
      '}'
    ),

    comment: $ => choice(
      /\/\/[^\n]*/,
      seq('/*', /[^*]*\*+([^/*][^*]*\*+)*\//), // not sure why comments work better as seq
    ),

    groovy_doc: $ =>
      // seq('/**', /[^*]*\*+([^/*][^*]*\*+)*\//),
      seq(
        '/**',
        // optional(
          token.immediate(/[*\n\s]+/),
          alias(token.immediate(/[^\n\.]+[\.]?/), $.first_line),
        // ),
        repeat(
          choice(
            // /[^*\s]*(\*[^/][^*\s]+)*/
            $.groovy_doc_param,
            $.groovy_doc_throws,
            $.groovy_doc_tag,
            $.groovy_doc_at_text,
            /([^@*]|\*[^/])([^*\s@]|[^\s\n]@|\*[^/])+/,
          ),
        ),
        '*/'
      ),

    groovy_doc_param: $ => seq (
      '@param',
      $.identifier
    ),

    groovy_doc_throws: $ => seq (
      '@throws',
      $.identifier
    ),

    groovy_doc_tag: $ =>
      /@[a-z]+/,

    groovy_doc_at_text: $ =>
      /@[^@\s*]*/,

    declaration: $ => seq(
      repeat($.annotation),
      optional($.access_modifier),
      choice(
        seq(
          repeat($.modifier),
          choice(
            '_',
            seq(
              choice(field('type', $._type), 'def'),
              field('name', $.identifier),
              optional(seq('=', field('value', $._expression)))
            ),
          )
        ),
        seq(
          repeat1($.modifier),
          choice(
            '_',
            seq(
              optional(choice(field('type', $._type), 'def')),
              field('name', $.identifier),
              optional(seq('=', field('value', $._expression)))
            ),
          )
        ),
      ),
    ),

    parenthesized_expression: ($) =>
      prec(PREC.PRIORITY, choice(
        seq("(",
          choice($._expression, $._immediately_invoked_closure),
          ")"),
      )),

    _expression: $ => prec(1, choice(
      $._primary_expression,
      $.increment_op,
      $.binary_op,
      $.ternary_op,
      $.unary_op,
      $.access_op,
      $.closure,
      alias("null", $.null),
    )),

    _primary_expression: $ => prec.left(1, choice(
      $.number_literal,
      $.boolean_literal,
      $.string,
      $.list,
      $.map,
      $._callable_expression,
    )),

    _callable_expression: $ => choice(
      "this",
      $.function_call,
      $.parenthesized_expression,
      $._juxtable_expression,
      $._type_identifier,
    ),

    _juxtable_expression: $ => choice(
      $.dotted_identifier,
      $.identifier,
      $.index,
    ),

    do_while_loop: $ => seq(
      'do',
      field('body', choice(
        $._statement,
        $.closure
      )),
      'while',
      field('condition', $.parenthesized_expression),
    ),

    for_parameters: $ => seq (
      '(',
      field('initializer', optional(seq(
        $.declaration,
        repeat(seq(',', $.assignment))
      ))),
      ';',
      field('condition', optional($._expression)),
      ';',
      field('increment', optional(seq(
        $._statement,
        repeat(seq(',', $._statement))
      ))),
      ')',
    ),
    for_loop: $ => seq(
      'for',
      $.for_parameters,
      field('body', choice(
        $._statement,
        $.closure
      )),
    ),

    for_in_loop: $ => prec(1, seq(
      'for',
      '(',
      field('variable', $.identifier),
      'in',
      field('collection', $._expression),
      ')',
      field('body', choice(
        $._statement,
        $.closure
      )),
    )),

    function_call: $ => prec.left(2, seq( //higher precedence than juxt_function_call
      field('function', $._callable_expression),
      field('args', $.argument_list),
    )),

    __immediately_invoked_closure: $ => prec.left(2,
      seq(field('function', $.closure), field('args', $.argument_list))
    ),
    _immediately_invoked_closure: $ => alias(
      $.__immediately_invoked_closure,
      $.function_call,
    ),

    argument_list: $ => prec.right(1, seq(
      prec.left(seq(
        '(',
        optional(
          list_of(choice($.map_item, $._expression)),
        ),
        ')'
      )),
      optional($.closure),
    )),

    _param_list: $ => prec(1, list_of($.parameter)),

    parameter_list: $ => prec(1, seq(
      '(',
      optional(list_of($.parameter)),
      ')'
    )),

    parameter: $ => prec(-1, seq(
      optional(field('type', choice($._type, 'def'))),
      field('name', $.identifier),
      optional(seq('=', field('value', $._expression))),
    )),

    function_declaration: $ => prec(2, seq(
      repeat($.annotation),
      optional($.access_modifier),
      repeat($.modifier),
      field('type', choice($._type, 'def')),
      field('function', choice($.identifier)),
      field('parameters', $.parameter_list),
    )),

    function_definition: $ => prec(3, seq(
      repeat($.annotation),
      optional($.access_modifier),
      repeat($.modifier),
      field('type', choice($._type, 'def')),
      field('function', $.identifier),
      field('parameters', $.parameter_list),
      field('body', $.closure), //TODO: optional return
    )),

    identifier: $ => IDENTIFIER_REGEX,
    _type_identifier: $ => alias(TYPE_REGEX, $.identifier),

    // identifier: $ => seq(
    //   choice($._letter, '$', '_'),
    //   repeat(choice($._letter, '[0-9]', '$', '_'))
    // ),

    if_statement: $ => prec.left(seq(
      'if',
      field('condition', $.parenthesized_expression),
      field('body', choice(
        $._statement,
        $.closure,
      )),
      optional(
        seq('else', field('else_body', choice($._statement, $.closure)))
      )
    )),

    index: $ => prec(PREC.TOP, seq(
      $._primary_expression,
      '[',
      $._expression,
      ']',
    )),

    juxt_function_call: $ => seq(
      field('function', $._juxtable_expression),
      field('args', alias($._juxt_argument_list, $.argument_list)),
    ),

    _juxt_argument_list: $ => {
      const juxt_argument = choice(
        $.map_item,
        $.increment_op,
        $.binary_op,
        $.ternary_op,
        $.unary_op,
        $.access_op,
        $.closure,
        alias("null", $.null),
        $.number_literal,
        $.boolean_literal,
        $.string,
        $.list,
        $.map,
        "this",
        $.function_call,
        $.dotted_identifier,
        $.identifier,
        $.index,
      )

      return prec.left(2,
        seq(juxt_argument, repeat(seq(',', juxt_argument)))
      )
    },

    list: $ => prec(1, seq(
      '[',
      repeat(prec.left(seq($._expression, ','))),
      optional(seq($._expression, optional(','))),
      ']'
    )),


    map_item: $ => seq(
      field('key', choice(
        $.identifier,
        $._type_identifier,
        $.number_literal,
        $.string,
        $.parenthesized_expression,
      )),
      ':',
      field('value', $._expression),
    ),

    map: $ => choice(
      seq(
        '[',
        repeat(
          prec.left(seq(
            $.map_item,
            ',',
          ))
        ),
        $.map_item,
        optional(','),
        ']',
      ),
      seq('[', ':', ']'),
    ),

    number_literal: $ => choice(
      /-?[0-9]+(_[0-9]+)*[DFGILdfgil]?/,
      /-?0x[0-9a-fA-F]+(_[0-9a-fA-F]+)*[DFGILdfgil]?/,
      /-?0b[0-1]+(_[0-1]+)*[DFGILdfgil]?/,
      /-?0[0-7]+(_[0-7]+)*[DFGILdfgil]?/,
      /-?[0-9]+(_[0-9]+)*\.[0-9]+(_[0-9]+)*([eE][0-9]+)?[DFGILdfgil]?/,
    ),

    pipeline: $ => seq(
      'pipeline',
      $.closure,
    ),

    // pipeline_step_with_block: $ => seq(
    //   $._primary_expression,
    //   $.closure,
    // ),

    return: $ => prec.right(1, seq('return', optional($._expression))), //??????

    string: $ => choice(
      $._plain_string,
      $._interpolate_string,
    ),

    _plain_string: $ => choice(
      seq(
        '\'',
        repeat(choice(
          alias(token.immediate(prec(1, /[^\\'\n]+/)), $.string_content),
          $.escape_sequence,
        )),
        '\'',
      ),
      seq(
        "'''",
        repeat(seq(
          optional(alias(token.immediate(prec(0, /[']{1,2}/)), $.string_internal_quote)),
          choice(
            alias(token.immediate(prec(1, /([^\\']|[']{1,2}[^'\\])+/)), $.string_content),
            seq(
              optional(/[']{1,2}/), // edge case: these wont be in string_content
              $.escape_sequence,
            ),
          ))),
        "'''",
      ),
    ),

    _interpolate_string: $ => choice(
      seq(
        '"',
        repeat(choice(
          alias(token.immediate(prec(1, /[^$\\"\n]+/)), $.string_content),
          $.escape_sequence,
          $.interpolation,
        )),
        '"',
      ),
      seq(
        '"""',
        repeat(seq(
          // optional(alias(token.immediate(prec(0, /["]{1,2}/)), $.string_internal_quote)),
          choice(
            alias(token.immediate(prec(1, /([^$\\"]|["]{1,2}[^"$\\])+/)), $.string_content),
            seq(
              optional(/["]{1,2}/), // edge case: these wont be in string_content
              $.escape_sequence,
            ),
            seq(
              optional(/["]{1,2}/),
              $.interpolation,
            ),
          ))),
        '"""',
      ),
      seq( // slashy string, only slashes can be escaped
        '/',
        repeat1(choice(
          alias(token.immediate(prec(1, /[^$\\\/]+/)), $.string_content),
          alias('\\/', $.escape_sequence),
          alias(/\\[^\/]/, $.string_content),
          $.interpolation,
        )),
        '/',
      ),
      seq( // dollar slashy string
        '$/',
        repeat(choice(
          alias(token.immediate(prec(1,
            /([^$\/]|\/[^$]|\$[^\/$a-zA-Z{])+/
          )), $.string_content),
          alias('$/', $.escape_sequence),
          alias('$$', $.escape_sequence),
          // alias(//, $.string_content),
          $.interpolation,
        )),
        '/$',
      ),
    ),

    escape_sequence: _ => token(prec(1, seq(
      '\\',
      choice(
        /[$bfnrst\\'"\n]/,
        /u[0-9a-fA-F]{4}/,
      ),
    ))),


    interpolation: $ => seq(
      '$',
      choice(
        seq(
          '{',
          $._expression,
          '}',
        ),
        alias(token.immediate(/[a-zA-Z0-9_]+(\.[a-zA-Z0-9_]+)*/), $.identifier),
      )
    ),

    switch_statement: $ => seq(
      'switch',
      field('value', $.parenthesized_expression),
      field('body', $.switch_block),
    ),

    switch_block: $ => seq(
      '{',
      repeat($.case),
      '}'
    ),

    case: $ => seq(
      choice(
        seq('case', field('value', $._expression), ':'),
        seq('default', ':'),
      ),
      repeat($._statement)
    ),

    ternary_op: $ => prec.right(seq(
      field('condition', $._expression),
      '?',
      field('then', $._expression),
      ':',
      field('else', $._expression),
    )),

    try_statement: $ => prec.left(seq(
      'try',
      field('body', choice(
        $._statement,
        $.closure,
      )),
      optional(
        seq(
          'catch',
          '(',
          field(
            'catch_exception',
            choice(
              $.declaration,
              $._expression
            ),
          ), //TODO multi-catch
          ')',
          field('catch_body', $.closure),
        )
      ),
      optional(
        seq(
          'finally',
          field('finally_body', $.closure),
        )
      )
    )),

    builtintype: $ => choice(
      'int',
      'boolean',
      'char',
      'short',
      'int',
      'long',
      'float',
      'double',
      'void',
    ),

    _type: $ => prec(2, choice(
      $.builtintype,
      $.array_type, //TODO: int[5]?
      $.type_with_generics,
      $._type_identifier,
    )),

    array_type: $ => seq($._type, '[]'),

    access_modifier: $ => choice(
      'public',
      'protected',
      'private'
    ),

    modifier: $ => choice(
      'static',
      'final',
      'synchronized'
    ),

    //TODO diamond operator
    type_with_generics: $ => seq($._type, $.generics),

    generics: $ => seq('<', list_of($._type), '>'),

    unary_op: $ =>
      choice(
        ...[
          ["+", PREC.UNARY],
          ["-", PREC.UNARY],
          ["~", PREC.TOP],
          ["!", PREC.TOP],
          ["new", PREC.TOP],
        ].map(([operator, precedence]) =>
          prec.left(precedence, seq(operator, $._expression))
        ),
      ),

    while_loop: $ => seq(
      'while',
      field('condition', $.parenthesized_expression),
      field('body', choice(
        $._statement,
        $.closure
      )),
    ),
  }
});

// TODO
// closures cleanup
// highlight jenkins words
