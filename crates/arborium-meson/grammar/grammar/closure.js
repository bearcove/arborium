module.exports = {
  //...commandNames(...commands),
  if_condition: ($) =>
    seq(
      $.if_command,
      repeat($.elseif_command),
      optional($.else_command),
      "endif"
    ),
  if_command: ($) =>
    seq(
      "if",
      $.var_unit,
      optional(repeat($._unit)),
      optional(choice($.keyword_break, $.keyword_continue))
    ),
  elseif_command: ($) =>
    seq(
      "elif",
      //repeat(/[\t ]/),
      $.var_unit,
      optional(repeat($._unit)),
      optional(choice($.keyword_break, $.keyword_continue))
    ),
  else_command: ($) =>
    seq(
      "else",
      //repeat(/[\t ]/),
      optional(repeat($._unit))
    ),
  foreach_command: ($) =>
    seq(
      "foreach",
      seq(
        field("item", $.identifier),
        repeat(seq(",", field("item", $.identifier)))
      ),
      ":",
      choice(
        field("array", $.identifier),
        field("array", $.list),
        field("dictionaries", $.dictionaries)
      ),
      optional(repeat($._unit)),
      "endforeach"
    ),
  keyword_break: ($) => "break",
  keyword_continue: ($) => "continue",
};
