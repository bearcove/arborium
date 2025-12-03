const { key_val_arg, keyword, command, bang_command } = require("./utils");

module.exports = {
  // :h user-commands
  command_name: ($) => /[A-Z][A-Za-z0-9]*/,

  _command_attribute_completion_behavior: ($) =>
    choice(
      ...[
        "arglist",
        "augroup",
        "buffer",
        "behave",
        "color",
        "command",
        "compiler",
        "dir",
        "environment",
        "even",
        "expression",
        "file",
        "file_in_path",
        "filetype",
        "function",
        "help",
        "highlight",
        "history",
        "local",
        "lua",
        "mapclear",
        "mapping",
        "menu",
        "messages",
        "option",
        "packadd",
        "shellcmd",
        "sign",
        "syntax",
        "syntime",
        "tag",
        "tag_listfiles",
        "user",
        "var",
      ].map((name) => cmd_attr_behavior_key_val(name)),
      cmd_attr_behavior_key_val("custom", $._ident),
      cmd_attr_behavior_key_val("customlist", $._ident)
    ),
  _command_attribute_address_behavior: ($) =>
    choice(
      ...[
        "lines",
        "arguments",
        "buffers",
        "loaded_buffers",
        "windows",
        "tabs",
        "quickfix",
        "other",
      ].map((name) => cmd_attr_behavior_key_val(name))
    ),

  _command_attribute_nargs_value: ($) =>
    choice(
      alias(token.immediate(/[01]/), $.integer_literal),
      alias(token.immediate(/[*?+]/), $.pattern_multi)
    ),
  _command_attribute_range_value: ($) =>
    choice(
      alias(token.immediate(/[0-9]+/), $.integer_literal),
      alias(token.immediate("%"), $.pattern_multi)
    ),

  command_attribute: ($) =>
    choice(
      key_val_arg("-nargs", $._command_attribute_nargs_value),
      key_val_arg(
        "-complete",
        alias($._command_attribute_completion_behavior, $.behavior)
      ),
      key_val_arg("-range", $._command_attribute_range_value),
      key_val_arg("-range"),
      key_val_arg(
        "-count",
        alias(token.immediate(/[0-9]+/), $.integer_literal)
      ),
      key_val_arg("-count"),
      key_val_arg(
        "-addr",
        alias($._command_attribute_address_behavior, $.behavior)
      ),
      key_val_arg("-bang"),
      key_val_arg("-bar"),
      key_val_arg("-register"),
      key_val_arg("-buffer"),
      key_val_arg("-keepscript")
    ),
  command_statement: ($) =>
    bang_command(
      $,
      "command",
      optional(
        choice(
          field("name", $.command_name),
          seq(
            repeat($.command_attribute),
            field("name", $.command_name),
            field("repl", alias(/.*/, $.command))
          )
        )
      )
    ),

  comclear_statement: ($) => keyword($, "comclear"),

  delcommand_statement: ($) => command($, "delcommand", $.command_name),
};

function cmd_attr_behavior_key_val(left, ...right) {
  if (right.length > 0) {
    return seq(
      field("name", token.immediate(left)),
      token.immediate(","),
      field("val", ...right)
    );
  } else {
    return field("name", token.immediate(left));
  }
}
