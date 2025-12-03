const { command, bang_command } = require("./utils");

module.exports = {
  edit_statement: ($) =>
    bang_command(
      $,
      "edit",
      repeat($.plus_plus_opt),
      repeat($.plus_cmd),
      optional(
        choice(
          seq("#", alias(token.immediate(/[0-9]+/), $.integer_literal)),
          $.filename
        )
      )
    ),

  enew_statement: ($) => bang_command($, "enew"),

  find_statement: ($) =>
    bang_command(
      $,
      "find",
      repeat($.plus_plus_opt),
      repeat($.plus_cmd),
      $.filename
    ),

  ex_statement: ($) =>
    command(
      $,
      "ex",
      repeat($.plus_plus_opt),
      repeat($.plus_cmd),
      optional($.filename)
    ),

  visual_statement: ($) =>
    bang_command(
      $,
      "visual",
      repeat($.plus_plus_opt),
      repeat($.plus_cmd),
      optional($.filename)
    ),

  view_statement: ($) =>
    bang_command(
      $,
      "view",
      repeat($.plus_plus_opt),
      repeat($.plus_cmd),
      $.filename
    ),
};
