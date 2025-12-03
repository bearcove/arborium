/* These are based on the precedence order in gojq */
const PREC = {
  // %nonassoc tokFuncDefPost tokTermPost
  FUNCDEF_TERM: 1,
  // %right '|'
  PIPE: 2,
  // %left ','
  COMMA: 3,
  // %right tokAltOp
  TOKALTOP: 4,
  // %nonassoc tokUpdateOp
  TOKUPDATEOP: 5,
  // %left tokOrOp
  TOKOROP: 6,
  // %left tokAndOp
  TOKANDOP: 7,
  // %nonassoc tokCompareOp
  TOKCOMPAREOP: 8,
  // %left '+' '-'
  MATH_LOW: 9,
  // %left '*' '/' '%'
  MATH_HIGH: 10,
  // %nonassoc tokAs tokIndex '.' '?' tokEmptyCatch
  DOT: 11,
  // %nonassoc '[' tokTry tokCatch
  SQUARE: 12,
};

module.exports = grammar({
  name: "jq",
  word: ($) => $.keyword,
  extras: ($) => [$.comment, /\s/],
  rules: {
    program: ($) => seq(optional($.moduleheader), $.programbody),

    moduleheader: ($) => seq("module", $.constobject, ";"),

    programbody: ($) =>
      seq(repeat($.import_), choice(repeat1($.funcdef), $.query)),

    import_: ($) =>
      choice(
        seq(
          "import",
          $.string,
          "as",
          $._tokIdentVariable,
          optional($.constobject),
          ";",
        ),
        seq("include", $.string, optional($.constobject), ";"),
      ),

    funcdef: ($) =>
      seq("def", $.identifier, optional($.funcdefargs), ":", $.query, ";"),

    funcdefargs: ($) =>
      seq("(", $._tokIdentVariable, repeat(seq(";", $._tokIdentVariable)), ")"),

    _tokIdentVariable: ($) => choice($.identifier, $.variable),

    query: ($) =>
      prec.right(
        choice(
          prec(PREC.FUNCDEF_TERM, seq($.funcdef, $.query)),
          field(
            "chained_query",
            prec(PREC.FUNCDEF_TERM, seq($.query, "|", $.query)),
          ),
          field(
            "chained_binding",
            prec(PREC.DOT, seq($.binding, "|", $.query)),
          ),
          field(
            "reduce",
            prec(
              PREC.DOT,
              seq(
                "reduce",
                $._term,
                "as",
                $._pattern,
                "(",
                $.query,
                ";",
                $.query,
                ")",
              ),
            ),
          ),
          field(
            "foreach_1",
            prec(
              PREC.DOT,
              seq(
                "foreach",
                $._term,
                "as",
                $._pattern,
                "(",
                $.query,
                ";",
                $.query,
                ")",
              ),
            ),
          ),
          field(
            "foreach_2",
            prec(
              PREC.DOT,
              seq(
                "foreach",
                $._term,
                "as",
                $._pattern,
                "(",
                $.query,
                ";",
                $.query,
                ";",
                $.query,
                ")",
              ),
            ),
          ),
          field(
            "if_statement",
            seq(
              "if",
              $.query,
              "then",
              $.query,
              repeat($.elif),
              optional($.else),
              "end",
            ),
          ),
          field(
            "try_catch",
            seq(
              prec(PREC.SQUARE, "try"),
              $.query,
              prec(PREC.DOT, optional($.catch)),
            ),
          ),
          field("label", seq("label", $.variable, "|", $.query)),
          field("optional", prec(PREC.SQUARE, seq($.query, "?"))),
          field(
            "comma_sep_query",
            prec.left(PREC.COMMA, seq($.query, ",", $.query)),
          ),
          field(
            "alternative",
            prec(PREC.TOKALTOP, seq($.query, "//", $.query)),
          ),
          field("update", seq($.query, $._tokUpdateOp, $.query)),
          field(
            "boolean_or",
            prec.left(PREC.TOKOROP, seq($.query, "or", $.query)),
          ),
          field("boolean_and", seq($.query, "and", $.query)),
          field("compare", seq($.query, $._tokCompareOp, $.query)),
          field(
            "math",
            prec.left(PREC.MATH_LOW, seq($.query, $._tokMathLow, $.query)),
          ),
          field(
            "math",
            prec.left(PREC.MATH_HIGH, seq($.query, $._tokMathHigh, $.query)),
          ),
          field("term", prec(PREC.FUNCDEF_TERM, $._term)),
        ),
      ),

    binding: ($) => seq($._term, "as", $._bindpatterns),
    _bindpatterns: ($) => seq($._pattern, repeat(seq("?//", $._pattern))),
    _pattern: ($) =>
      choice(
        $.variable,
        seq("[", $.arraypatterns, "]"),
        seq("{", $.objectpatterns, "}"),
      ),
    arraypatterns: ($) => seq($._pattern, repeat(seq(",", $._pattern))),
    objectpatterns: ($) =>
      seq($.objectpattern, repeat(seq(",", $.objectpattern))),
    objectpattern: ($) =>
      choice(
        seq(
          choice($.objectkey, $.string, seq("(", $.query, ")")),
          ":",
          $._pattern,
        ),
        $.variable,
      ),
    _term: ($) =>
      choice(
        ".",
        alias("..", $.recurse),
        $.index,
        field("array_access", prec(PREC.SQUARE, seq(".", $.suffix))),
        field("object_param", seq(".", $.string)),
        "null",
        "true",
        "false",
        field("function", seq($.funcname, optional(seq("(", $.args, ")")))),
        $.variable,
        $.number,
        $.format,
        $.string,
        field("query_within_parens", seq("(", $.query, ")")),
        field("unary_plus", prec.left(seq("+", $._term))),
        field("unary_minus", prec.left(seq("-", $._term))),
        field("empty_object", seq("{", "}")),
        field("object", seq("{", $.objectkeyvals, "}")),
        field("object_2", seq("{", $.objectkeyvals, ",", "}")),
        field("empty_array", seq("[", "]")),
        field("array", seq("[", $.query, "]")),
        field("break_statement", seq("break", $.variable)),
        field("term_with_object_access", seq($._term, $.index)),
        prec(PREC.SQUARE, seq($._term, $.suffix)),
        seq($._term, "?"),
        prec(PREC.SQUARE, seq($._term, ".", $.suffix)),
        seq($._term, ".", $.string),
      ),

    funcname: ($) => $._IDENT,

    suffix: ($) =>
      choice(
        seq("[", "]"),
        seq("[", $.query, "]"),
        seq("[", $.query, ":", "]"),
        seq("[", ":", $.query, "]"),
        seq("[", $.query, ":", $.query, "]"),
      ),

    args: ($) => choice($.query, seq($.args, ";", $.query)),

    elif: ($) => seq("elif", $.query, "then", $.query),

    else: ($) => seq("else", $.query),

    catch: ($) => prec.right(PREC.SQUARE, seq("catch", $.query)),

    objectkeyvals: ($) =>
      choice($.objectkeyval, seq($.objectkeyvals, ",", $.objectkeyval)),

    objectkeyval: ($) =>
      choice(
        seq($.objectkey, ":", $.objectval),
        seq($.string, ":", $.objectval),
        seq("(", $.query, ")", ":", $.objectval),
        $.objectkey,
        $.string,
      ),

    objectkey: ($) => choice($.identifier, $.variable, $.keyword),

    objectval: ($) => seq($._term, repeat(seq("|", $._term))),

    constterm: ($) =>
      choice(
        $.constobject,
        $.constarray,
        $.number,
        $.string,
        "null",
        "true",
        "false",
      ),

    constobject: ($) =>
      seq(
        "{",
        optional(
          seq($.constobjectkeyval, repeat(seq(",", $.constobjectkeyval))),
        ),
        "}",
      ),

    constobjectkeyval: ($) =>
      seq(choice($.identifier, $.keyword, $.string), ":", $.constterm),

    constarray: ($) =>
      seq("[", optional(seq($.constterm, repeat(seq(",", $.constterm)))), "]"),

    keyword: ($) =>
      /module|import|include|def|as|if|then|else|elif|end|and|or|reduce|foreach|try|catch|label|break|__loc__/,
    string: ($) => choice($._QQSTRING, seq($.format, $._QQSTRING)),
    identifier: ($) => /([a-zA-Z_][a-zA-Z_0-9]*::)*[a-zA-Z_][a-zA-Z_0-9]*/,
    variable: ($) => /\$[a-zA-Z_][a-zA-Z_0-9]*/,
    // TODO maybe move these directly into where they are used?
    _tokMathLow: ($) => choice("+", "-"),
    _tokMathHigh: ($) => choice("*", "/", "%"),
    _tokUpdateOp: ($) => choice("|=", "+=", "-=", "*=", "/=", "//=", "%=", "="),
    _tokCompareOp: ($) => choice("==", "!=", ">=", "<=", ">", "<"),
    format: ($) => /"@"[a-zA-Z0-9_]+/,
    index: ($) => seq(".", $.identifier),
    number: ($) => /[\+\-]?[0-9]+/,
    // NOTE: this stuff comes from the json BNF, which may or may not be worth implementing

    // {
    //   const hex_literal = seq(choice("0x", "0X"), /[\da-fA-F]+/);

    //   const decimal_digits = /\d+/;
    //   const signed_integer = seq(optional(choice("-", "+")), decimal_digits);
    //   const exponent_part = seq(choice("e", "E"), signed_integer);

    //   const binary_literal = seq(choice("0b", "0B"), /[0-1]+/);

    //   const octal_literal = seq(choice("0o", "0O"), /[0-7]+/);

    //   const decimal_integer_literal = seq(
    //     optional(choice("-", "+")),
    //     choice("0", seq(/[1-9]/, optional(decimal_digits))),
    //   );

    //   const decimal_literal = choice(
    //     seq(
    //       decimal_integer_literal,
    //       ".",
    //       optional(decimal_digits),
    //       optional(exponent_part),
    //     ),
    //     seq(".", decimal_digits, optional(exponent_part)),
    //     seq(decimal_integer_literal, optional(exponent_part)),
    //   );

    //   return token(
    //     choice(hex_literal, decimal_literal, binary_literal, octal_literal),
    //   );
    // },
    _QQSTRING: ($) => choice(seq('"', '"'), seq('"', $._inner_string, '"')),
    _inner_string: ($) =>
      repeat1(prec.left(choice($._string_content, $.string_interp))),
    string_interp: ($) => seq("\\\(", $.query, "\)"),
    _string_content: ($) =>
      repeat1(choice(token.immediate(/[^\\"\n]+/), $._escape_sequence)),

    _escape_sequence: ($) =>
      token.immediate(seq("\\", /(\"|\\|\/|b|f|n|r|t|u)/)),
    _IDENT: ($) => /([a-zA-Z_][a-zA-Z_0-9]*::)*[a-zA-Z_][a-zA-Z_0-9]*/,
    comment: ($) => token(prec(-10, /#.*/)),
  },
});
