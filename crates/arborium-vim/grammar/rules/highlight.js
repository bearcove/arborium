const { keyword, maybe_bang, commaSep1 } = require("./utils");

module.exports = {
  // :h group-name
  hl_group: ($) => /[a-zA-Z0-9_@.]+/,

  // :h :highlight-link
  _hl_body_link: ($) =>
    seq(
      optional(keyword($, "default")),
      "link",
      field("from", $.hl_group),
      field("to", $.hl_group)
    ),

  // :h highlight-clear
  _hl_body_clear: ($) => seq("clear", optional($.hl_group)),

  _hl_body_none: ($) => seq($.hl_group, alias("NONE", $.hl_group)),

  _hl_none: ($) => token.immediate("NONE"),

  // :h attr-list
  _hl_attr_list: ($) =>
    commaSep1(
      choice(
        $._hl_none,
        ...[
          "bold",
          "underline",
          "undercurl",
          "underdouble",
          "underdotted",
          "underdashed",
          "strikethrough",
          "reverse",
          "inverse",
          "italic",
          "standout",
          "nocombine",
        ].map(token.immediate)
      )
    ),

  // :h highlight-cterm
  _hl_key_cterm: ($) => hl_key_val(choice("term", "cterm"), $._hl_attr_list),

  // :h term-list
  _hl_term_list: ($) =>
    repeat1(choice(token.immediate(/\S+/), $._immediate_keycode)),

  // :h highlight-start
  _hl_key_start_stop: ($) =>
    hl_key_val(choice("start", "stop"), $._hl_term_list),

  // :h highlight-ctermfg
  _hl_color_nr: ($) => token.immediate(/[0-9]+\*?/),
  _hl_key_ctermfg_ctermbg: ($) =>
    hl_key_val(choice("ctermfg", "ctermbg"), choice($.color, $._hl_color_nr)),

  // :h highlight-gui
  _hl_key_gui: ($) => hl_key_val("gui", $._hl_attr_list),

  _hl_quoted_name: ($) =>
    seq(token.immediate("'"), token.immediate(/[^'\n]+/), "'"),

  // :h gui-colors
  color: ($) =>
    choice(
      $._hl_quoted_name,
      $._hl_none,
      ...[
        "bg",
        "background",
        "fg",
        "foreground",
        /#[0-9a-fA-F]{6}/,
        /[a-zA-Z]+/,
      ].map(token.immediate)
    ),
  _hl_key_gui_color: ($) =>
    hl_key_val(choice("guifg", "guibg", "guisp"), $.color),

  // :h highlight-font
  font: ($) =>
    choice($._hl_none, $._hl_quoted_name, token.immediate(/[a-zA-Z0-9-]+/)),

  _hl_key_font: ($) => hl_key_val("font", $.font),

  // :h highlight-blend
  _hl_key_blend: ($) => hl_key_val("blend", $.integer_literal),

  // :h highlight-args
  hl_attribute: ($) =>
    choice(
      $._hl_key_cterm,
      $._hl_key_start_stop,
      $._hl_key_ctermfg_ctermbg,
      $._hl_key_gui,
      $._hl_key_gui_color,
      $._hl_key_font,
      $._hl_key_blend
    ),

  _hl_body_keys: ($) =>
    seq(optional(keyword($, "default")), $.hl_group, repeat1($.hl_attribute)),

  _hl_body: ($) =>
    choice(
      $.hl_group,
      $._hl_body_clear,
      $._hl_body_none,
      $._hl_body_keys,
      $._hl_body_link
    ),

  // :h :highlight
  highlight_statement: ($) =>
    seq(maybe_bang($, keyword($, "highlight")), optional($._hl_body)),
};

function hl_key_val(left, right) {
  return seq(field("key", left), token.immediate("="), field("val", right));
}
