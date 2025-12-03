/// <reference types="tree-sitter-cli/dsl" />
const {
  key_val_arg,
  maybe_bang,
  keyword,
  sub_cmd,
  commaSep,
  commaSep1,
  sep1,
  command,
} = require("./rules/utils");

const MAP_OPTIONS = any_order(
  "<buffer>",
  "<nowait>",
  "<silent>",
  "<unique>",
  "<script>"
);

const FILE_FORMAT = ["dos", "unix", "mac"];

const ENCODING = [
  "latin1",
  "iso",
  "koi8",
  "koi8",
  "macroman",
  "cp437",
  "cp737",
  "cp775",
  "cp850",
  "cp852",
  "cp855",
  "cp857",
  "cp860",
  "cp861",
  "cp862",
  "cp863",
  "cp865",
  "cp866",
  "cp869",
  "cp874",
  "cp1250",
  "cp1251",
  "cp1253",
  "cp1254",
  "cp1255",
  "cp1256",
  "cp1257",
  "cp1258",
  "cp932",
  "euc-jp",
  "sjis",
  "cp949",
  "euc-kr",
  "cp936",
  "euc-cn",
  "cp950",
  "big5",
  "euc-tw",
  "utf-8",
  "ucs-2",
  "ucs-21e",
  "utf-16",
  "utf-16le",
  "ucs-4",
  "ucs-4le",
  "ansi",
  "japan",
  "korea",
  "prc",
  "chinese",
  "taiwan",
  "utf8",
  "unicode",
  "ucs2be",
  "ucs-2be",
  "ucs-4be",
  "utf-32",
  "utf-32le",
  "default",
];

const PREC = {
  TERNARY: 1, //=> expr ? expr : expr
  OR: 2, //=> or
  AND: 3, //=> and
  COMPARE: 4, //=> < <= == ~= >= > and all
  PLUS: 5, //=> + -
  CONCAT: 5, //=> .. .
  MULTI: 6, //=> * / %
  UNARY: 7, //=> ! - +
  CALL: 8, //expr[n] expr[n:m] expr.name expr(...)
};

module.exports = grammar({
  name: "vim",

  word: ($) => $.keyword,

  conflicts: ($) => [
    [$.binary_operation, $.unary_operation, $.field_expression],
    [$.binary_operation, $.field_expression],
    [$._ident, $.lambda_expression],
    [$._ident, $._immediate_lambda_expression],
    [$._complete_range_marker, $._range_marker]
  ],

  externals: ($) =>
    [
      $._no,
      $._inv,
      $._newline_or_pipe,
      $._line_continuation,
      $._script_heredoc_marker,
      $._let_heredoc_marker,
      $._heredoc_end,
      $._separator_first,
      $._separator,
      $._scope_dict,
      $.scope,
      $.string_literal,
      $.comment,
      $.line_continuation_comment,
      $._bang_filter,
    ].concat(require("./keywords").keywords($)),

  extras: ($) => [$._line_continuation, $.line_continuation_comment, /[\t ]/],

  rules: {
    script_file: ($) => optional($._separated_statements),

    _cmd_separator: ($) => choice($._newline_or_pipe, $.comment),

    _separated_statements: ($) =>
      repeat1(seq(optional($._statement), $._cmd_separator)),

    _statement: ($) =>
      seq(
        repeat(":"),
        choice(
          $.function_definition,
          $.let_statement,
          $.unlet_statement,
          $.const_statement,
          $.set_statement,
          $.setlocal_statement,
          $.return_statement,
          $.normal_statement,
          $.while_loop,
          $.for_loop,
          $.if_statement,
          $.lua_statement,
          $.range_statement,
          $.ruby_statement,
          $.python_statement,
          $.perl_statement,
          $.call_statement,
          $.execute_statement,
          $.echo_statement,
          $.echon_statement,
          $.echohl_statement,
          $.echomsg_statement,
          $.echoerr_statement,
          $.try_statement,
          $.throw_statement,
          $.autocmd_statement,
          $.silent_statement,
          $.vertical_statement,
          $.belowright_statement,
          $.aboveleft_statement,
          $.topleft_statement,
          $.botright_statement,
          $.register_statement,
          $.map_statement,
          $.augroup_statement,
          $.bang_filter_statement,
          $.highlight_statement,
          $.syntax_statement,
          $.setfiletype_statement,
          $.options_statement,
          $.startinsert_statement,
          $.stopinsert_statement,
          $.scriptencoding_statement,
          $.source_statement,
          $.global_statement,
          $.colorscheme_statement,
          $.command_statement,
          $.comclear_statement,
          $.delcommand_statement,
          $.filetype_statement,
          $.runtime_statement,
          $.wincmd_statement,
          $.sign_statement,
          $.break_statement,
          $.continue_statement,
          $.cnext_statement,
          $.cprevious_statement,
          $.unknown_builtin_statement,
          $.edit_statement,
          $.enew_statement,
          $.find_statement,
          $.ex_statement,
          $.visual_statement,
          $.view_statement,
          $.eval_statement,
          $.substitute_statement,
          $.user_command
        )
      ),

    unknown_builtin_statement: ($) =>
      seq(
        optional(field("range", alias($._complete_range, $.range))),
        maybe_bang($, $.unknown_command_name),
        alias(repeat($.command_argument), $.arguments)
      ),

    return_statement: ($) => command($, "return", optional($._expression)),

    break_statement: ($) => command($, "break"),
    continue_statement: ($) => command($, "continue"),

    scope_dict: ($) => choice($._scope_dict, "a:"),

    normal_statement: ($) =>
      bang_range_command($, "normal", alias(/ .*/, $.commands)),
    filetype: ($) => /[a-zA-Z][a-zA-Z_-]*/,
    _filetype_immediate: ($) =>
      alias(token.immediate(/[a-zA-Z][a-zA-Z_-]*/), $.filetype),
    filetypes: ($) =>
      seq($.filetype, repeat(seq(token.immediate("."), $._filetype_immediate))),
    setfiletype_statement: ($) =>
      command(
        $,
        "setfiletype",
        optional(alias("FALLBACK", $.fallback)),
        $.filetypes
      ),
    options_statement: ($) =>
      choice(command($, "browse", keyword($, "set")), command($, "options")),
    startinsert_statement: ($) => maybe_bang($, keyword($, "startinsert")),
    stopinsert_statement: ($) => keyword($, "stopinsert"),

    file_format: ($) => choice(...FILE_FORMAT),
    _immediate_file_format: ($) =>
      alias(choice(...FILE_FORMAT.map(token.immediate)), $.file_format),

    encoding: ($) => choice(...ENCODING),
    _immediate_encoding: ($) =>
      alias(choice(...ENCODING.map(token.immediate)), $.encoding),

    scriptencoding_statement: ($) =>
      command($, "scriptencoding", optional($.encoding)),

    cnext_statement: ($) => maybe_bang($, keyword($, "cnext")),
    cprevious_statement: ($) =>
      maybe_bang($, choice(keyword($, "cprevious"), keyword($, "cNext"))),

    _runtime_where: ($) => choice("START", "OPT", "PACK", "ALL"),
    runtime_statement: ($) =>
      seq(
        maybe_bang($, keyword($, "runtime")),
        optional(alias($._runtime_where, $.where)),
        alias(repeat1($.filename), $.filenames)
      ),

    wincmd_statement: ($) =>
      seq(
        optional($.integer_literal),
        keyword($, "wincmd"),
        field("action", /[a-zA-Z=<>]/)
      ),

    source_statement: ($) =>
      bang_range_command($, "source", optional(field("file", $.filename))),

    global_statement: ($) =>
      bang_range_command(
        $,
        "global",
        $._separator_first,
        $.pattern,
        $._separator,
        $._statement
      ),

    _filetype_state: ($) => choice("on", "off"),
    _filetype_enable: ($) => sub_cmd($._filetype_state),
    _filetype_detect: ($) => sub_cmd("detect"),
    _filetype_plugin: ($) =>
      sub_cmd("plugin", optional("indent"), $._filetype_state),
    _filetype_indent: ($) =>
      sub_cmd("indent", optional("plugin"), $._filetype_state),
    filetype_statement: ($) =>
      command(
        $,
        "filetype",
        optional(
          choice(
            $._filetype_enable,
            $._filetype_detect,
            $._filetype_plugin,
            $._filetype_indent
          )
        )
      ),

    colorscheme_statement: ($) =>
      command($, "colorscheme", optional(alias($.filename, $.name))),

    lua_statement: ($) => choice(
      command($, "lua", choice($.chunk, $.script)),
      seq("=", choice($.chunk, $.script))
    ),
    ruby_statement: ($) => command($, "ruby", choice($.chunk, $.script)),
    python_statement: ($) => command($, "python", choice($.chunk, $.script)),
    perl_statement: ($) => command($, "perl", choice($.chunk, $.script)),

    chunk: ($) => /<|(<[^\n<]|[^\s<])[^\n]*/,

    _heredoc_line: ($) => /[^\n]*\n/,

    script: ($) =>
      seq(
        "<<",
        optional(alias("trim", $.parameter)),
        choice(alias($._script_heredoc_marker, $.marker_definition), "\n"),
        alias(repeat($._heredoc_line), $.body),
        alias($._heredoc_end, $.endmarker)
      ),

    for_loop: ($) =>
      seq(
        keyword($, "for"),
        field("variable", choice($._ident, $.list_assignment)),
        "in",
        field("iter", $._expression),
        $._cmd_separator,
        alias(optional($._separated_statements), $.body),
        keyword($, "endfor")
      ),

    while_loop: ($) =>
      seq(
        keyword($, "while"),
        field("condition", $._expression),
        $._cmd_separator,
        alias(optional($._separated_statements), $.body),
        keyword($, "endwhile")
      ),

    if_statement: ($) =>
      seq(
        keyword($, "if"),
        field("condition", $._expression),
        $._cmd_separator,
        alias(optional($._separated_statements), $.body),
        repeat($.elseif_statement),
        optional($.else_statement),
        keyword($, "endif")
      ),

    elseif_statement: ($) =>
      seq(
        keyword($, "elseif"),
        field("condition", $._expression),
        $._cmd_separator,
        alias(optional($._separated_statements), $.body)
      ),

    else_statement: ($) =>
      seq(keyword($, "else"), alias(optional($._separated_statements), $.body)),

    try_statement: ($) =>
      seq(
        keyword($, "try"),
        $._cmd_separator,
        alias(optional($._separated_statements), $.body),
        repeat($.catch_statement),
        optional($.finally_statement),
        keyword($, "endtry")
      ),

    _au_pattern: ($) => choice(/\/.*\//, /\?.*\?/),

    catch_statement: ($) =>
      seq(
        keyword($, "catch"),
        optional(alias($._au_pattern, $.pattern)),
        $._cmd_separator,
        alias(optional($._separated_statements), $.body)
      ),

    finally_statement: ($) =>
      seq(
        keyword($, "finally"),
        alias(optional($._separated_statements), $.body)
      ),

    throw_statement: ($) => command($, "throw", $._expression),

    // :h filter
    _bang_filter_bangs: ($) => seq($.bang, optional($.bang)),
    _bang_filter_command_argument: ($) =>
      choice(
        seq(
          choice(/\S/, seq("\\", /./)),
          repeat(choice(...[/\S/, seq("\\", /./)].map(token.immediate)))
        ),
        $.string_literal
      ),
    _bang_filter_command: ($) =>
      seq(
        field("filter", alias($.filename, $.filter_command)),
        optional($.bang),
        repeat(alias($._bang_filter_command_argument, $.command_argument))
      ),
    bang_filter_statement: ($) =>
      seq(
        field("range", alias($._range, $.range)),
        alias($._bang_filter_bangs, $.bangs),
        alias($._bang_filter_command, $.command)
      ),

    // TODO(vigoux): maybe we should find some names here
    scoped_identifier: ($) => seq($.scope, $.identifier),

    argument: ($) =>
      seq(
        "a:",
        choice(
          alias(token.immediate(/[a-zA-Z_](\w|#)*/), $.identifier),
          alias(token.immediate(/[0-9]+/), $.integer_literal)
        )
      ),

    _curly_braces_name_expression: ($) => seq("{", $._expression, "}"),
    _immediate_curly_braces_name_expression: ($) =>
      seq(token.immediate("{"), $._expression, "}"),
    identifier: ($) =>
      seq(
        choice(
          /[a-zA-Z_]+/,
          alias($._curly_braces_name_expression, $.curly_braces_name)
        ),
        repeat(
          choice(
            token.immediate(/(\w|#)+/),
            alias(
              $._immediate_curly_braces_name_expression,
              $.curly_braces_name
            )
          )
        )
      ),
    _immediate_identifier: ($) =>
      seq(
        choice(
          token.immediate(/[a-zA-Z_]+/),
          alias($._immediate_curly_braces_name_expression, $.curly_braces_name)
        ),
        repeat(
          choice(
            token.immediate(/(\w|#)+/),
            alias(
              $._immediate_curly_braces_name_expression,
              $.curly_braces_name
            )
          )
        )
      ),
    _ident: ($) =>
      prec.dynamic(1, choice($.scoped_identifier, $.identifier, $.argument)),

    keyword: ($) => /[a-zA-Z_](\w|#)*/,

    _let_operator: ($) =>
      choice("=", "+=", "-=", "*=", "/=", "%=", ".=", "..="),
    _assignment_variable: ($) => choice($.identifier, $.scope_dict),

    _let_assignment: ($) =>
      seq(
        choice(
          $._ident,
          $.env_variable,
          $.register,
          $.option,
          $.index_expression,
          $.slice_expression,
          $.field_expression,
          $.list_assignment
        ),
        choice(
          seq($._let_operator, $._expression),
          alias($._let_heredoc, $.heredoc)
        )
      ),
    let_statement: ($) =>
      seq(
        keyword($, "let"),
        choice($._let_assignment, repeat($._assignment_variable))
      ),

    _const_assignment: ($) =>
      seq(
        choice($._ident, $.list_assignment),
        choice(seq("=", $._expression), alias($._let_heredoc, $.heredoc))
      ),
    const_statement: ($) =>
      command(
        $,
        "const",
        choice($._const_assignment, repeat($._assignment_variable))
      ),

    _let_heredoc: ($) =>
      seq(
        "=<<",
        repeat(alias($._let_heredoc_parameter, $.parameter)),
        alias($._let_heredoc_marker, $.marker_definition),
        optional($.comment),
        "\n",
        alias(repeat($._heredoc_line), $.body),
        alias($._heredoc_end, $.endmarker)
      ),

    // :h :let-heredoc
    _let_heredoc_parameter: ($) => choice("trim", "eval"),

    option_name: ($) => choice(/[a-z]+/, seq("t_", /[a-zA-Z0-9]+/)),

    no_option: ($) => seq($._no, $.option_name),

    inv_option: ($) =>
      choice(
        seq($._inv, $.option_name),
        seq($.option_name, token.immediate("!"))
      ),

    default_option: ($) =>
      seq($.option_name, "&", optional(choice("vi", "vim"))),

    _set_option: ($) =>
      choice(
        "all",
        "all&",
        $.option_name,
        seq($.option_name, "?"),
        $.no_option,
        $.inv_option,
        $.default_option
      ),

    _set_operator: ($) =>
      choice(...["=", ":", "+=", "^=", "-="].map(token.immediate)),

    set_value: ($) => token.immediate(/([^ \n\t]|\\[\t ])+/),

    _set_rhs: ($) =>
      seq($._set_operator, optional(field("value", $.set_value))),

    set_item: ($) => seq(field("option", $._set_option), optional($._set_rhs)),

    set_statement: ($) => set_variant($, "set"),
    setlocal_statement: ($) => set_variant($, "setlocal"),

    unlet_statement: ($) =>
      seq(maybe_bang($, keyword($, "unlet")), repeat1($._expression)),

    call_statement: ($) =>
      range_command($, "call", choice($.call_expression, $.method_expression)),

    echo_statement: ($) => echo_variant($, "echo"),
    echon_statement: ($) => echo_variant($, "echon"),
    echohl_statement: ($) => seq(keyword($, "echohl"), $.hl_group),
    echomsg_statement: ($) => echo_variant($, "echomsg"),
    echoerr_statement: ($) => echo_variant($, "echoerr"),

    execute_statement: ($) => command($, "execute", repeat1($._expression)),

    silent_statement: ($) => command_modifier($, "silent", true),
    vertical_statement: ($) => command_modifier($, "vertical", false),
    topleft_statement: ($) => command_modifier($, "topleft", false),
    botright_statement: ($) => command_modifier($, "botright", false),

    aboveleft_statement: ($) =>
      seq(
        choice(keyword($, "leftabove"), keyword($, "aboveleft")),
        $._statement
      ),

    belowright_statement: ($) =>
      seq(
        choice(keyword($, "rightbelow"), keyword($, "belowright")),
        $._statement
      ),

    user_command: ($) =>
      seq(
        optional(field("range", alias($._complete_range, $.range))),
        maybe_bang($, $.command_name),
        alias(repeat($.command_argument), $.arguments)
      ),

    command_argument: ($) => choice($.string_literal, /\S+/),

    function_definition: ($) =>
      seq(
        maybe_bang($, keyword($, "function")),
        $.function_declaration,
        any_order("dict", "range", "abort", "closure"),
        $._cmd_separator,
        alias(optional($._separated_statements), $.body),
        keyword($, "endfunction")
      ),

    function_declaration: ($) =>
      prec(
        PREC.CALL,
        seq(
          field("name", choice($._ident, $.field_expression)),
          field("parameters", $.parameters)
        )
      ),

    parameters: ($) =>
      seq(
        "(",
        choice(
          $.spread,
          seq(commaSep($.default_parameter), optional(seq(",", $.spread))),
          seq(
            commaSep($.identifier),
            optional(seq(",", commaSep($.default_parameter))),
            optional(seq(",", $.spread))
          )
        ),
        ")"
      ),

    default_parameter: ($) =>
      seq(field("name", $.identifier), "=", field("value", $._expression)),

    // :h :_!
    bang: ($) => token.immediate("!"),

    at: ($) => "@",

    spread: ($) => "...",

    _printable: ($) => /[^\t\n\v\f\r]/,

    // :h 10.3

    mark: ($) => /'./,

    range_statement: ($) => $._range,

    _range: ($) => choice(alias("%", $.file), $._range_explicit),

    _range_explicit: ($) =>
      seq(
        field("start", $._range_marker),
        optional(seq(choice(",", ";"), field("end", $._range_marker)))
      ),

    _range_marker: ($) =>
      choice(
        $.integer_literal,
        $.current_line,
        $.next_line,
        $.last_line,
        seq("/", $.pattern, optional(token.immediate("/"))),
        seq("?", $.pattern, optional(token.immediate("?"))),
        $.previous_pattern,
        $.mark
      ),

    _complete_range: ($) =>
      choice(alias("%", $.file), $._complete_range_explicit),

    _complete_range_explicit: ($) =>
      seq(
        field("start", $._complete_range_marker),
        optional(seq(choice(",", ";"), field("end", $._complete_range_marker)))
      ),

    _complete_range_marker: ($) =>
      choice(
        $.integer_literal,
        $.current_line,
        $.next_line,
        $.last_line,
        seq("/", $.pattern, token.immediate("/")),
        seq("?", $.pattern, token.immediate("?")),
        $.previous_pattern,
        $.mark
      ),

    current_line: ($) => ".",
    next_line: ($) => "+",
    last_line: ($) => "$",
    previous_pattern: ($) => choice("\\/", "\\?", "\\&"),

    // :h :@
    register_statement: ($) => $.register,

    map_statement: ($) =>
      seq(
        field(
          "cmd",
          choice(
            ...[
              "map",
              "nmap",
              "vmap",
              "xmap",
              "smap",
              "omap",
              "imap",
              "lmap",
              "cmap",
              "tmap",
              "noremap",
              "vnoremap",
              "nnoremap",
              "xnoremap",
              "snoremap",
              "onoremap",
              "inoremap",
              "lnoremap",
              "cnoremap",
              "tnoremap",
            ].map((name) => keyword($, name))
          )
        ),
        MAP_OPTIONS,
        $._map_definition
      ),

    _map_definition: ($) =>
      choice(
        seq(
          "<expr>",
          MAP_OPTIONS,
          field("lhs", alias($._map_lhs, $.map_side)),
          field("rhs", $._expression)
        ),
        seq(
          field("lhs", alias($._map_lhs, $.map_side)),
          field("rhs", alias($._map_rhs, $.map_side))
        )
      ),

    // All keycodes should be match case insensitively (this makes it awful to read)
    _keycode_modifier: ($) => token.immediate(/([SsCcMmAaDd]|[Aa][lL][tT])-/),
    _keycode_in: ($) =>
      choice(
        ...[
          /[Nn][Uu][Ll]/, // Nul
          /[Bb][Ss]/, // BS
          /[Tt][aA][bB]/, // Tab
          /[Nn][Ll]/, // NL
          /[Cc][Rr]/, // CR
          /[Rr][eE][tT][uU][rR][nN]/, // Return
          /[kK]?[Ee][nN][tT][eE][rR]/, // [k]Enter
          /[Ee][sS][cC]/, // Esc
          /[Ss][pP][aA][cC][eE]/, // Space
          /[lL][tT]/, // lt
          /[Bb][sS][lL][aA][sS][hH]/, // Bslash
          /[Bb][aA][rR]/, // Bar
          /[kK]?[Dd][eE][lL]/, // [k]Del
          /[xX]?[Cc][Ss][Ii]/, // [x]CSI
          /[Ee][Oo][Ll]/, // EOL
          /[Ii][gG][nN][oO][rR][eE]/, // Ignore
          /[Nn][Oo][Pp]/, // Nop
          /([kK]|([SsCc]-))?[Uu][pP]/, // [k|S-|C-]Up
          /([kK]|([SsCc]-))?[Dd][oO][wW][nN]/, // [k|S-|C-]Down
          /([kK]|([SsCc]-))?[Ll][eE][fF][tT]/, // [k|S-|C-]Left
          /([kK]|([SsCc]-))?[Rr][iI][gG][hH][tT]/, // [k|S-|C-]Right
          /([SsCc]-)?[Ll][eE][fF][tT][Mm][oO][uU][sS][eE]/, // <S|C>-LeftMouse
          /([SsCc]-)?[Rr][iI][gG][hH][tT][Mm][oO][uU][sS][eE]/, // <S|C>-RightMouse
          /([Ss]-)?[Ff][0-9]{1,2}/, // [S-]F<1-12>
          /[Hh][eE][lL][pP]/, // Help
          /[Uu][nN][dD][oO]/, // Undo
          /[Ii][nN][sS][eE][rR][tT]/, // Insert
          /[kK]?[Hh][oO][mM][eE]/, // [k]Home
          /[kK]?[Ee][nN][dD]/, // [k]End
          /[kK]?[Pp][aA][gG][eE][Uu][pP]/, // [k]PageUp
          /[kK]?[Pp][aA][gG][eE][Dd][oO][wW][nN]/, // [k]PageDown
          /[kK][Pp][lL][uU][sS]/, // kPlus
          /[kK][Mm][iI][nN][uU][sS]/, // kMinus
          /[kK][Mm][uU][lL][tT][iI][pP][lL][yY]/, // kMultiply
          /[kK][Dd][iI][vV][iI][dD][eE]/, // kDivide
          /[kK][Pp][oO][iI][nN][tT]/, // kPoint
          /[kK][Cc][oO][mM][mM][aA]/, // kComma
          /[kK][Ee][qQ][uU][aA][lL]/, // kEqual
          /[kK][0-9]/, // k<0-9>
          /([Ll][oO][cC][aA][lL])?[Ll][eE][aA][dD][eE][rR]/, // [Local]Leader
          /[Ss][Ii][Dd]/, // SID
          /[Pp][lL][uU][gG]/, // Plug
          /([Ss]-)?[Cc][hH][aA][rR]-(0[0-7]+|0[xX][0-9a-fA-F]+|[0-9]+)+/, // [S-]Char-...
        ].map(token.immediate),
        seq($._keycode_modifier, choice(token.immediate(/\S/), $._keycode_in)) // (<S|C|M|A|D|Alt>-)+...
      ),
    _immediate_keycode: ($) =>
      seq(token.immediate("<"), $._keycode_in, token.immediate(">")),
    keycode: ($) => seq("<", $._keycode_in, token.immediate(">")),

    _map_lhs: ($) => keys($, /\S/),
    _map_rhs_statement: ($) =>
      seq(
        alias(/<[Cc][Mm][Dd]>/, $.keycode),
        // :h map_bar
        sep1($._statement, choice("\\|", alias(/<[Bb][Aa][Rr]>/, $.keycode))),
        alias(/<[Cc][Rr]>/, $.keycode)
      ),
    _map_rhs: ($) => choice(keys($, /[^\s|]/, /[^|\n]/), $._map_rhs_statement),

    // :h sign

    _sign_name: ($) => choice($.integer_literal, $.identifier),

    _sign_define_arg_text: ($) => seq($._printable, optional($._printable)),
    _sign_define_argument: ($) =>
      choice(
        key_val_arg("icon", optional($.filename)),
        key_val_arg("linehl", optional($.hl_group)),
        key_val_arg("numhl", optional($.hl_group)),
        key_val_arg("text", optional(alias($._sign_define_arg_text, $.text))),
        key_val_arg("texthl", optional($.hl_group)),
        key_val_arg("culhl", optional($.hl_group))
      ),

    _sign_define: ($) =>
      sub_cmd(
        "define",
        field("name", $._sign_name),
        repeat(alias($._sign_define_argument, $.sign_argument))
      ),

    _sign_undefine: ($) => sub_cmd("undefine", field("name", $._sign_name)),

    _sign_list: ($) => sub_cmd("list", optional(field("name", $._sign_name))),

    // :h sign-place
    _sign_place_place_argument: ($) =>
      choice(
        key_val_arg("line", $.integer_literal),
        key_val_arg("name", $._sign_name),
        key_val_arg("buffer", $.integer_literal),
        key_val_arg("group", $.hl_group),
        key_val_arg("priority", $.integer_literal),
        key_val_arg("file", $.filename)
      ),
    _sign_place_place: ($) =>
      seq(
        field("id", $.integer_literal),
        repeat1(alias($._sign_place_place_argument, $.sign_argument))
      ),
    // :h sign-place-list
    _sign_place_list_argument: ($) =>
      choice(
        key_val_arg("file", $.filename),
        key_val_arg("buffer", $.integer_literal),
        key_val_arg(
          "group",
          choice($.hl_group, alias(token.immediate("*"), $.wildcard))
        )
      ),
    _sign_place_list: ($) =>
      repeat1(alias($._sign_place_list_argument, $.sign_argument)),
    _sign_place: ($) =>
      sub_cmd("place", choice($._sign_place_place, $._sign_place_list)),

    _sign_unplace_cursor_argument: ($) =>
      key_val_arg(
        "group",
        choice($.hl_group, alias(token.immediate("*"), $.wildcard))
      ),
    _sign_unplace_cursor: ($) =>
      alias($._sign_unplace_cursor_argument, $.sign_argument),
    _sign_unplace_id_argument: ($) =>
      choice(
        key_val_arg("file", $.filename),
        key_val_arg("buffer", $.integer_literal),
        key_val_arg(
          "group",
          choice($.hl_group, alias(token.immediate("*"), $.wildcard))
        )
      ),
    _sign_unplace_id: ($) =>
      seq(
        field("id", choice($.integer_literal, alias("*", $.wildcard))),
        repeat(alias($._sign_unplace_id_argument, $.sign_argument))
      ),
    _sign_unplace: ($) =>
      sub_cmd(
        "unplace",
        optional(choice($._sign_unplace_cursor, $._sign_unplace_id))
      ),

    _sign_jump_argument: ($) =>
      choice(
        key_val_arg("file", $.filename),
        key_val_arg("buffer", $.integer_literal),
        key_val_arg("group", $.hl_group)
      ),
    _sign_jump: ($) =>
      sub_cmd(
        "jump",
        field("id", choice($.integer_literal, alias("*", $.wildcard))),
        repeat(alias($._sign_jump_argument, $.sign_argument))
      ),

    sign_statement: ($) =>
      seq(
        keyword($, "sign"),
        choice(
          $._sign_define,
          $._sign_undefine,
          $._sign_list,
          $._sign_place,
          $._sign_unplace,
          $._sign_jump
        )
      ),

    // :h variable
    _variable: ($) =>
      prec.left(
        0,
        choice(
          $.scope_dict,
          $._ident,
          $.string_literal,
          $.float_literal,
          $.integer_literal,
          $.list,
          $.env_variable,
          $.register,
          $.option,
          $.lambda_expression,
          $.dictionnary,
          $.literal_dictionary
        )
      ),

    //:h expression-syntax
    _expression: ($) =>
      choice(
        $._variable,
        $.ternary_expression,
        $.index_expression,
        $.slice_expression,
        $.binary_operation,
        seq("(", $._expression, ")"),
        $.unary_operation,
        $.field_expression,
        $.call_expression,
        $.method_expression
      ),

    ternary_expression: ($) =>
      prec.left(
        PREC.TERNARY,
        seq(
          field("condition", $._expression),
          "?",
          field("left", $._expression),
          ":",
          field("right", $._expression)
        )
      ),

    // Shamelessly stolen from tree-sitter-lua
    match_case: ($) => choice("#", "?"),

    binary_operation: ($) =>
      choice(
        ...[
          ["||", PREC.OR],
          ["&&", PREC.AND],
          ["+", PREC.PLUS],
          ["-", PREC.PLUS],
          ["*", PREC.MULTI],
          ["/", PREC.MULTI],
          ["%", PREC.MULTI],
          ["..", PREC.CONCAT],
          [".", PREC.CONCAT],
        ].map(([operator, precedence]) =>
          prec.left(
            precedence,
            bin_left_right($._expression, operator, $._expression)
          )
        ),
        ...["==", "!=", ">", ">=", "<", "<=", "=~", "!~", "is", "isnot"].map(
          (operator) =>
            prec.left(
              PREC.COMPARE,
              bin_left_right(
                $._expression,
                seq(operator, optional($.match_case)),
                $._expression
              )
            )
        )
      ),

    unary_operation: ($) =>
      prec.left(PREC.UNARY, seq(choice("-", "!", "+"), $._expression)),

    // :h floating-point-format
    float_literal: ($) => {
      const digits = /[0-9]+/;
      const sign = /[+-]?/;

      return token(
        seq(sign, digits, ".", digits, optional(seq(/[eE]/, sign, digits)))
      );
    },

    integer_literal: ($) =>
      token(
        seq(
          optional(/[-+]/),
          choice(
            seq(choice("0x", "0X"), /[A-Fa-f0-9]+/),
            seq("0", /[0-7]+/),
            seq(choice("0b", "0B"), /[0-1]+/),
            /[0-9]+/
          )
        )
      ),

    list: ($) => seq("[", commaSep($._expression), optional(","), "]"),
    // Trailing commas are not allowed in assignments, but `; <ident>` are
    list_assignment: ($) =>
      seq("[", commaSep($._expression), optional(seq(";", $._expression)), "]"),

    index_expression: ($) =>
      prec(
        PREC.CALL,
        seq(
          field("value", $._expression),
          "[",
          field("index", $._expression),
          "]"
        )
      ),

    slice_expression: ($) =>
      prec(
        PREC.CALL,
        seq(
          field("value", $._expression),
          "[",
          optional(field("start", $._expression)),
          ":",
          optional(field("stop", $._expression)),
          "]"
        )
      ),

    field_expression: ($) =>
      prec.left(
        PREC.CALL,
        seq(field("value", $._expression), ".", field("field", $.identifier))
      ),

    call_expression: ($) =>
      prec(
        PREC.CALL,
        seq(
          field("function", $._expression),
          "(",
          optional(commaSep1($._expression)),
          ")"
        )
      ),

    eval_statement: ($) => command($, "eval", $._expression),

    substitute_statement: ($) => range_command($, "substitute", $.pattern),

    _method_call_expression: ($) =>
      seq(
        field(
          "function",
          choice(
            alias($._immediate_identifier, $.identifier),
            alias($._immediate_lambda_expression, $.lambda_expression)
          )
        ),
        token.immediate("("),
        optional(commaSep1($._expression)),
        ")"
      ),

    method_expression: ($) =>
      prec(
        PREC.CALL,
        seq(
          field("value", $._expression),
          "->",
          alias($._method_call_expression, $.call_expression)
        )
      ),

    // Use default :h isfname
    filename: ($) =>
      seq(
        // First character of a filename is not immediate
        choice(
          /[A-Za-z0-9]/,
          /[/._+,#$%~=-]/,
          // Include windows characters
          /[\\{}\[\]:@!]/,
          // Allow wildcard
          /[*]/,
          // Escaped character
          seq("\\", /./)
        ),
        repeat(
          choice(
            ...[
              /[A-Za-z0-9]/,
              /[/._+,#$%~=-]/,
              // Include windows characters
              /[\\{}\[\]:@!]/,
              // Allow wildcard
              /[*]/,
              // Escaped character
              seq("\\", /./),
            ].map(token.immediate)
          )
        )
      ),

    // :h pattern
    pattern_multi: ($) =>
      choice("*", /\\[+=?]/, /\\@[!>=]|<[=!]/, /\\\{-?[0-9]*,?[0-9]*}/),

    _pattern_ordinary_atom: ($) =>
      repeat1(
        choice(
          seq(
            "[",
            repeat(
              choice(
                seq("\\", /./), // escaped character
                /[^\]\n\\]/ // any character besides ']', '\' or '\n'
              )
            ),
            "]"
          ), // square-bracket-delimited character class
          seq("\\", /./), // escaped character
          /[^\\\[\n]/ // any character besides '[', '\' or '\n'
        )
      ),

    _pattern_atom: ($) =>
      prec.left(
        choice(
          $._pattern_ordinary_atom,
          seq("\\(", $.pattern, "\\)"),
          seq("\\%(", $.pattern, "\\)"),
          seq("\\z(", $.pattern, "\\)")
        )
      ),

    _pattern_piece: ($) => seq($._pattern_atom, optional($.pattern_multi)),

    _pattern_concat: ($) => repeat1($._pattern_piece),

    _pattern_branch: ($) => sep1($._pattern_concat, "\\&"),

    pattern: ($) => prec.left(sep1($._pattern_branch, "\\|")),

    env_variable: ($) => seq("$", $.identifier),

    // :h registers
    register: ($) => /@["0-9a-zA-Z:.%#=*+_/-@]/,

    option: ($) => seq("&", optional($.scope), $.option_name),

    dictionnary_entry: ($) =>
      seq(field("key", $._expression), ":", field("value", $._expression)),

    dictionnary: ($) =>
      seq("{", commaSep($.dictionnary_entry), optional(","), "}"),

    // :h literal-Dict
    _literal_dictionary_entry: ($) =>
      seq(field("key", $.literal_key), ":", field("value", $._expression)),

    literal_key: ($) => /[0-9a-zA-Z_-]+/,

    literal_dictionary: ($) =>
      seq("#{", commaSep($._literal_dictionary_entry), optional(","), "}"),

    // :h lambda
    lambda_expression: ($) =>
      seq("{", commaSep($.identifier), "->", $._expression, "}"),
    _immediate_lambda_expression: ($) =>
      seq(
        token.immediate("{"),
        commaSep($.identifier),
        "->",
        $._expression,
        "}"
      ),

    // :h ++opt
    _plus_plus_opt_bad: ($) =>
      choice(...[/./, "keep", "drop"].map(token.immediate)),
    plus_plus_opt: ($) =>
      seq(
        "++",
        choice(
          key_val_arg(
            choice(token.immediate("ff"), token.immediate("fileformat")),
            $._immediate_file_format
          ),
          key_val_arg(
            choice(token.immediate("enc"), token.immediate("encoding")),
            $._immediate_encoding
          ),
          key_val_arg(
            choice(token.immediate("bin"), token.immediate("binary"))
          ),
          key_val_arg(
            choice(token.immediate("nobin"), token.immediate("nobinary"))
          ),
          key_val_arg(token.immediate("bad"), $._plus_plus_opt_bad),
          key_val_arg(token.immediate("edit"))
        )
      ),

    // :h +cmd
    _plus_cmd_arg: ($) =>
      repeat1(
        choice(
          seq(token.immediate("\\"), token.immediate(/./)), // escaped character
          token.immediate(/[^ \n]/) // any character besides ' ' and newline
        )
      ),
    _plus_cmd_number: ($) =>
      prec(2, alias(token.immediate(/[0-9]+/), $.integer_literal)),
    _plus_cmd_command: ($) => prec(1, alias($._plus_cmd_arg, $.command)),
    _plus_cmd_pattern: ($) => seq("/", alias($._plus_cmd_arg, $.pattern)),
    plus_cmd: ($) =>
      seq(
        "+",
        optional(
          choice($._plus_cmd_number, $._plus_cmd_pattern, $._plus_cmd_command)
        )
      ),

    ...require("./rules/autocmd"),
    ...require("./rules/command"),
    ...require("./rules/highlight"),
    ...require("./rules/syntax"),
    ...require("./rules/edit"),
  },
});

function command_modifier($, name, bang) {
  let inner = keyword($, name);
  if (bang) {
    inner = maybe_bang($, inner);
  }
  return seq(inner, $._statement);
}

function _cmd_range($) {
  return seq(field("range", alias($._complete_range, $.range)), optional(":"));
}
function range_command($, cmd, ...args) {
  return seq(optional(_cmd_range($)), keyword($, cmd), ...args);
}
function bang_range_command($, cmd, ...args) {
  return seq(
    optional(_cmd_range($)),
    keyword($, cmd),
    optional($.bang),
    ...args
  );
}

function bin_left_right(left, operator, right) {
  return seq(field("left", left), operator, field("right", right));
}

function echo_variant($, cmd) {
  return command($, cmd, repeat($._expression));
}

function set_variant($, cmd) {
  return command($, cmd, sep1($.set_item, " "));
}

function any_order(...args) {
  return repeat(choice(...args));
}

function keys($, allowed_first, allowed_after = allowed_first) {
  return seq(
    choice(allowed_first, "<", seq("\\", /./), $.keycode),
    repeat(
      choice(
        token.immediate(allowed_after),
        token.immediate("<"),
        seq(token.immediate("\\"), token.immediate(/./)),
        alias($._immediate_keycode, $.keycode)
      )
    )
  );
}
