const {
  key_val_arg,
  sub_cmd,
  commaSep1,
  commaSep,
  command,
} = require("./utils");

module.exports = {
  // :h :syn-define

  _syn_enable: ($) => sub_cmd(choice("enable", "on", "off", "reset")),

  _syn_case: ($) => sub_cmd("case", optional(choice("match", "ignore"))),

  _syn_spell: ($) =>
    sub_cmd("spell", optional(choice("toplevel", "notoplevel", "default"))),

  _syn_foldlevel: ($) =>
    sub_cmd("foldlevel", optional(choice("start", "minimum"))),

  _syn_iskeyword: ($) =>
    sub_cmd("iskeyword", optional(choice("clear", alias(/[^ \n]+/, $.value)))),

  _syn_conceal: ($) => sub_cmd("conceal", optional(choice("on", "off"))),

  // :h :syn-arguments

  _syn_hl_pattern: ($) => seq($._separator_first, $.pattern, $._separator),

  hl_groups: ($) => commaSep1($.hl_group),

  _syn_arguments_keyword: ($) =>
    choice(
      key_val_arg("conceal"),
      key_val_arg("cchar", $._printable),
      key_val_arg("contained"),
      // FIXME: allow regex of hlgroups
      key_val_arg("containedin", optional($.hl_groups)),
      key_val_arg("nextgroup", optional($.hl_groups)),
      key_val_arg("transparent"),
      key_val_arg("skipwhite"),
      key_val_arg("skipnl"),
      key_val_arg("skipempty")
    ),

  _syn_arguments_match: ($) =>
    choice(
      $._syn_arguments_keyword,
      key_val_arg("contains", optional($.hl_groups)),
      key_val_arg("fold"),
      key_val_arg("display"),
      key_val_arg("extend"),
      key_val_arg("keepend"),
      key_val_arg("excludenl")
    ),

  _syn_arguments_region: ($) =>
    choice(
      $._syn_arguments_match,
      key_val_arg("matchgroup", optional($.hl_groups)),
      key_val_arg("oneline"),
      key_val_arg("concealends")
    ),

  _syn_arguments_cluster: ($) =>
    choice(
      key_val_arg("contains", optional($.hl_groups)),
      key_val_arg("add", optional($.hl_groups)),
      key_val_arg("remove", optional($.hl_groups))
    ),

  _syn_pattern_offset: ($) =>
    seq(
      field(
        "what",
        choice(
          ...["ms", "me", "hs", "he", "rs", "re", "lc"].map(token.immediate)
        )
      ),
      token.immediate("="),
      field(
        "offset",
        choice(...[/[se]([+-][0-9]+)?/, /[0-9]/].map(token.immediate))
      )
    ),

  _syn_keyword: ($) =>
    sub_cmd(
      "keyword",
      $.hl_group,
      repeat(alias($._syn_arguments_keyword, $.syntax_argument)),
      // The list of keyword cannot be empty, but we can have arguments anywhere on the line
      alias(/[a-zA-Z0-9\[\]_]+/, $.keyword),
      repeat(
        choice(
          alias($._syn_arguments_keyword, $.syntax_argument),
          alias(/[a-zA-Z0-9\[\]_]+/, $.keyword)
        )
      )
    ),

  _syn_match: ($) =>
    sub_cmd(
      "match",
      $.hl_group,
      repeat(alias($._syn_arguments_match, $.syntax_argument)),
      $._syn_hl_pattern,
      commaSep(alias($._syn_pattern_offset, $.pattern_offset)),
      repeat(alias($._syn_arguments_match, $.syntax_argument))
    ),

  _syn_region_start: ($) => syn_region_arg($, "start"),
  _syn_region_skip: ($) => syn_region_arg($, "skip"),
  _syn_region_end: ($) => syn_region_arg($, "end"),

  _syn_region: ($) =>
    sub_cmd(
      "region",
      $.hl_group,
      repeat(alias($._syn_arguments_region, $.syntax_argument)),
      alias($._syn_region_start, $.syntax_argument),
      repeat(alias($._syn_arguments_region, $.syntax_argument)),
      optional(
        seq(
          alias($._syn_region_skip, $.syntax_argument),
          repeat(alias($._syn_arguments_region, $.syntax_argument))
        )
      ),
      // Can have multiple end
      repeat1(
        seq(
          alias($._syn_region_end, $.syntax_argument),
          repeat(alias($._syn_arguments_region, $.syntax_argument))
        )
      )
    ),

  _syn_cluster: ($) =>
    sub_cmd(
      "cluster",
      $.hl_group,
      repeat(alias($._syn_arguments_cluster, $.syntax_argument))
    ),

  _syn_include: ($) =>
    sub_cmd(
      "include",
      // Here we can't have pattern and `@` is mandatory
      optional(field("grouplist", seq("@", $.hl_group))),
      $.filename
    ),

  // :h syn-sync
  _syn_sync_lines: ($) => key_val_arg(choice("minlines", "maxlines"), /[0-9]+/),
  _syn_sync: ($) =>
    sub_cmd(
      "sync",
      choice(
        syn_sync_method(
          "linebreaks",
          token.immediate("="),
          field("val", token.immediate(/[0-9]+/))
        ),
        syn_sync_method("fromstart"),
        syn_sync_method(
          "ccomment",
          optional($.hl_group),
          repeat($._syn_sync_lines)
        ),
        syn_sync_method(
          choice("lines", "minlines", "maxlines"),
          token.immediate("="),
          field("val", token.immediate(/[0-9]+/))
        ),
        syn_sync_method(
          choice("match", "region"),
          $.hl_group,
          optional(seq(choice("grouphere", "groupthere"), $.hl_group)),
          $.pattern
        ),
        syn_sync_method(
          "linecont",
          repeat($._syn_sync_lines),
          $.pattern,
          repeat($._syn_sync_lines)
        ),
        syn_sync_method("clear", optional($.hl_group))
      )
    ),

  _syn_list: ($) => sub_cmd("list", optional($.hl_group)),

  _syn_clear: ($) => sub_cmd("clear", optional($.hl_group)),

  // :h :
  syntax_statement: ($) =>
    command(
      $,
      "syntax",
      // `:syntax` = `:syntax list`
      optional(
        choice(
          $._syn_enable,
          $._syn_case,
          $._syn_spell,
          $._syn_foldlevel,
          $._syn_iskeyword,
          $._syn_conceal,
          $._syn_keyword,
          $._syn_match,
          $._syn_region,
          $._syn_cluster,
          $._syn_include,
          $._syn_sync,
          $._syn_list,
          $._syn_clear
        )
      )
    ),
};

function syn_region_arg($, name) {
  return seq(
    key_val_arg(name, $._syn_hl_pattern),
    commaSep(alias($._syn_pattern_offset, $.pattern_offset))
  );
}

function syn_sync_method(arg, ...args) {
  if (args.length > 0) return seq(field("method", arg), ...args);
  else return field("method", arg);
}
