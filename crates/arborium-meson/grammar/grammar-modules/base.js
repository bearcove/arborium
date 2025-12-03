module.exports = {
  //_command_invocation: ($) =>
  //  choice(
  //    $.normal_command,
  //    $.if_condition,
  //    $.foreach_loop,
  //    $.while_loop,
  //    $.function_def,
  //    $.macro_def,
  //  ),
  // project(a)
  // find_program('git', native: true)
  // runcommand([a,b,c])
  normal_command: ($) =>
    command(
      field("command", $.identifier),
      optional(
        seq(
          repeat(
            seq(
              choice(
                $.dictionaries,
                $.normal_command,
                $.listitem,
                $.list,
                $.expression_statement,
                $.pair,
                $.variableunit,
                $.var_unit
              ),
              ","
            )
          ),
          choice(
            $.dictionaries,
            $.normal_command,
            $.listitem,
            $.list,
            $.expression_statement,
            $.pair,
            $.variableunit,
            $.var_unit
          ),
          optional(",")
        )
      )
    ),
  bool: ($) => choice("true", "false"),
  string: ($) =>
    choice(
      //'sss.ss'
      seq(
        "'",
        optional(
          repeat(
            choice(
              token.immediate(prec(1, /[^'\\^@]+/)),
              $.escape_sequence,
              $.formatunit
            )
          )
        ),
        "'"
      ),
      //"ss.ss"
      //seq(
      //  '"',
      //  optional(
      //    repeat(choice(
      //      token.immediate(prec(1, /[^"\\^@]+/)),
      //      $.escape_sequence,
      //      $.formatunit,
      //    )),
      //  ),
      //  '"',
      //),
      seq(
        "'''",
        optional(
          repeat(
            choice(
              token.immediate(prec(1, /[^'''\\^@]+/)),
              $.escape_sequence,
              $.formatunit
            )
          )
        ),
        "'''"
      )
    ),
  escape_sequence: ($) =>
    token.immediate(
      seq(
        "\\",
        choice(
          /[^xu0-7]/,
          /[0-7]{1,3}/,
          /x[0-9a-fA-F]{2}/,
          /u[0-9a-fA-F]{4}/,
          /u\{[0-9a-fA-F]+\}/
        )
      )
    ),
  formatunit: ($) =>
    seq("@", choice($.number, field("variable", $.identifier)), "@"),
  dictionaries: ($) =>
    seq("{", repeat(seq($.pair, ",")), optional(seq($.pair)), "}"),
  listitem: ($) =>
    seq(
      $.identifier,
      "[",
      choice(
        field("index", $.number),
        field("index", $.identifier),
        field("key", $.string)
      ),
      "]"
    ),
  list: ($) =>
    seq(
      "[",
      optional(
        seq(
          repeat(
            seq(
              choice(
                $.dictionaries,
                $.normal_command,
                $.number,
                $.string,
                $.list,
                $.listitem,
                $.bool,
                $.variableunit,
                field("variable", $.identifier),
                field("variable", $.expression_statement)
              ),
              ","
            )
          ),
          optional(
            choice(
              $.normal_command,
              $.number,
              $.string,
              $.list,
              $.bool,
              $.variableunit,
              field("variable", $.identifier),
              field("variable", $.expression_statement)
            )
          )
        )
      ),
      "]"
    ),

  pair: ($) =>
    seq(
      field("key", choice($.var_unit, $.variableunit, $.identifier, $.string)),
      ":",
      field(
        "value",
        choice(
          $.bool,
          $.var_unit,
          $.variableunit,
          $.dictionaries,
          $.normal_command,
          $.number,
          $.identifier,
          $.list,
          $.string,
          $.expression_statement
        )
      )
    ),
  identifier: (_) => /[A-Za-z_][A-Za-z0-9_]*/,
  number: ($) => {
    const hex_literal = seq(choice("0x", "0X"), /[\da-fA-F](_?[\da-fA-F])*/);

    const decimal_digits = /\d(_?\d)*/;
    const signed_integer = seq(optional(choice("-", "+")), decimal_digits);
    const exponent_part = seq(choice("e", "E"), signed_integer);

    const binary_literal = seq(choice("0b", "0B"), /[0-1](_?[0-1])*/);

    const octal_literal = seq(choice("0o", "0O"), /[0-7](_?[0-7])*/);

    const bigint_literal = seq(
      choice(hex_literal, binary_literal, octal_literal, decimal_digits),
      "n"
    );

    const decimal_integer_literal = choice(
      "0",
      seq(optional("0"), /[1-9]/, optional(seq(optional("_"), decimal_digits)))
    );

    const decimal_literal = choice(
      seq(
        decimal_integer_literal,
        ".",
        optional(decimal_digits),
        optional(exponent_part)
      ),
      seq(".", decimal_digits, optional(exponent_part)),
      seq(decimal_integer_literal, exponent_part),
      seq(decimal_digits)
    );

    return token(
      choice(
        // TODO maybe better solution
        signed_integer,
        hex_literal,
        decimal_literal,
        binary_literal,
        octal_literal,
        bigint_literal
      )
    );
  },
};
function command(name_rule, arg_rule) {
  return seq(
    name_rule,
    //repeat(/[\t ]/),
    "(",
    arg_rule,
    ")"
  );
}
