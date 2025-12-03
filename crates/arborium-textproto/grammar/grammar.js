// https://protobuf.dev/reference/protobuf/textformat-spec/

module.exports = grammar({
  name: 'textproto',

  rules: {
    message: $ => repeat($.field),

    field: $ => choice($.message_field, $.scalar_field),

    message_field: $ => seq(
      $.field_name,
      optional(":"),
      choice($.message_value, $.message_list),
      optional(choice(";", ",")),
    ),

    scalar_field: $ => seq(
      $.field_name,
      ":",
      choice(
	$.scalar_value,
	$.scalar_list,
      ),
      optional(choice(";", ",")),
    ),

    message: $ => repeat1($.field),

    message_value: $ => choice(
      seq(
	$.open_squiggly,
	optional($.message),
	$.close_squiggly,
      ),
      seq(
	$.open_arrow,
	optional($.message),
	$.close_arrow,
      )
    ),

    message_list: $ => prec(2, seq(
      $.open_square,
      optional(
	seq(
	  $.message_value,
	  repeat(
	    seq(
	      ",",
	      $.message_value,
	    ),
	  ),
	),
      ),
      $.close_square,
    )),

    open_squiggly: $ => '{',
    close_squiggly: $ => '}',
    open_square: $ => '[',
    close_square: $ => ']',
    open_arrow: $ => '<',
    close_arrow: $ => '>',

    field_name: $ => choice(
      $.extension_name,
      $.any_name,
      $.identifier,
    ),

    extension_name: $ => seq(
      $.open_square,
      $.type_name,
      $.close_square,
    ),

    any_name: $ => seq(
      $.open_square,
      $.domain,
      "/",
      $.type_name,
      $.close_square,
    ),

    type_name: $ => seq(
      $.identifier,
      repeat(choice(".", $.identifier)),
    ),
    domain: $ => seq(
      $.identifier,
      repeat(choice(".", $.identifier)),
    ),

    identifier: $ => /[A-Za-z_][A-Za-z0-9_]*/,
    signed_identifier: $ => seq(
      "-",
      $.identifier,
    ),

    scalar_value: $ => choice(
      repeat1($.string),
      $.identifier,
      $.signed_identifier,
      $.number,
    ),

    scalar_list: $ => prec(1, seq(
      $.open_square,
      optional(
	seq(
	  $.scalar_value,
	  repeat(seq(",", $.scalar_value)),
	),
      ),
      $.close_square,
    )),

    string: $ => choice(
      $.single_string,
      $.double_string,
    ),

    single_string: $ => seq(
      "'",
      repeat(choice(
	$.string_escape,
	$.single_string_contents,
      )),
      "'",
    ),

    double_string: $ => seq(
      '"',
      repeat(choice(
	$.string_escape,
	$.double_string_contents,
      )),
      '"',
    ),

    single_string_contents: $ => /[^\n'\\]+/,
    double_string_contents: $ => /[^\n"\\]+/,

    string_escape: $ => choice(
      "\\a",
      "\\b",
      "\\f",
      "\\n",
      "\\r",
      "\\t",
      "\\v",
      "\\?",
      "\\\"",
      "\\'",
      '\\"',
      seq("\\", $.oct, optional($.oct), optional($.oct)),
      seq("\\x", $.hex, optional($.hex)),
      seq("\\u", $.hex, $.hex, $.hex, $.hex),
      seq("\\U000", $.hex, $.hex, $.hex, $.hex, $.hex),
      seq("\\U010", $.hex, $.hex, $.hex, $.hex),
    ),

    oct: $ => /[0-7]/,
    hex: $ => /[0-9A-Fa-f]/,

    number: $ => choice(
      $.dec_int,
      $.oct_int,
      $.hex_int,
      seq(optional('-'), $.float),
      seq("-", $.dec_int),    // signed decimal int
      seq('-', $.oct_int),    // signed octal int
      seq('-', $.hex_int),    // signed hexidecimal int
    ),

    dec_int: $ => choice(
      "0",
      /[1-9][0-9]*/,
    ),
    oct_int: $ => /0[0-7]+/,
    hex_int: $ => /0[Xx][0-9A-Fa-f]+/,
    float_lit: $ => choice(
      seq(
	$.dec_int,
	$.exp
      ),
      seq(
	".",
	/\d+/,
	optional($.exp)
      ),
      seq(
	$.dec_int,
	".",
	optional(/\d+/),
	optional($.exp)
      ),
    ),
    exp: $ => seq(
      /[Ee][-+]?/,
      /\d+/,
    ),
    float: $ => choice(
      seq($.float_lit, optional(/[Ff]/)),
      seq($.dec_int, /[Ff]/),
    ),

    comment: $ => seq('#', /.*/)
  },
  
  extras: $ => [
    /\s/,
    $.comment,
  ],
  precedences: $ => [
    ["message_list", "scalar_list"],
  ],
});
