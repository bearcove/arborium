const NEWLINE = /\r?\n/;
const WHITE_SPACE = /[\t\f\v ]+/;
const ANYTHING = /[^\r\n]+/;
const BASE85_LINE = /[A-Za-z][0-9A-Za-z!#$%&()*+\-;<=>?@^_`{|}~]+/;
const BASE85 = new RegExp(
  `${BASE85_LINE.source}(?:${NEWLINE.source}${BASE85_LINE.source})*`
);

export default grammar({
  name: "diff",

  extras: ($) => [WHITE_SPACE],

  rules: {
    source: ($) =>
      seq(
        repeat(choice($.block, seq(optional($._line), NEWLINE))),
        optional($._line)
      ),

    _line: ($) =>
      choice(
        $.file_change,
        $.binary_change,
        $.index,
        $.similarity,
        $.dissimilarity,
        $.old_file,
        $.new_file,
        $.location,
        $.addition,
        $.deletion,
        $.change,
        $.context,
        $.comment,
        $.special,
        $.unrecognized,
      ),

    block: ($) =>
      prec.right(
        seq(
          $.command,
          NEWLINE,
          repeat(
            seq(
              choice(
                $.file_change,
                $.binary_change,
                $.index,
                $.similarity,
                $.dissimilarity
              ),
              NEWLINE
            )
          ),
          optional(
            choice(
              seq($.old_file, NEWLINE, $.new_file, NEWLINE, $.hunks),
              $.binary_patch
            )
          )
        )
      ),

    binary_patch: ($) =>
      prec.right(
        seq(
          iseq("GIT", "binary", "patch"),
          NEWLINE,
          field("forward", $.binary_hunk),
          optional(field("reverse", $.binary_hunk))
        )
      ),

    binary_hunk: ($) =>
      prec.right(
        seq(
          choice(token.immediate("literal"), token.immediate("delta")),
          alias(/\d+/, $.size),
          NEWLINE,
          $.payload,
          NEWLINE,
          prec.right(repeat(NEWLINE))
        )
      ),

    payload: ($) => token.immediate(BASE85),

    hunks: ($) => prec.right(repeat1($.hunk)),

    hunk: ($) =>
      prec.right(
        seq(
          field("location", $.location),
          NEWLINE,
          optional(field("changes", $.changes))
        )
      ),

    changes: ($) =>
      prec.right(
        repeat1(
          seq(
            choice(
              alias($._hunk_addition, $.addition),
              alias($._hunk_deletion, $.deletion),
              $.addition,
              $.deletion,
              $.change,
              $.context,
              $.special,
              $.unrecognized
            ),
            prec.right(repeat1(NEWLINE))
          )
        )
      ),

    command: ($) => iseq("diff", alias(/[-\w]+/, $.argument), $.filename),

    file_change: ($) =>
      choice(
        seq(choice("new", "deleted"), "file", "mode", $.mode),
        seq(choice("new", "old"), "mode", $.mode),
        seq(choice("rename", "copy"), choice("from", "to"), $.filename)
      ),

    binary_change: ($) =>
      iseq("Binary", "files", $.filename, "and", $.filename, "differ"),

    index: ($) => iseq("index", $.commit, "..", $.commit, optional($.mode)),

    similarity: ($) => iseq("similarity", "index", alias(/\d+/, $.score), "%"),
    dissimilarity: ($) =>
      iseq("dissimilarity", "index", alias(/\d+/, $.score), "%"),

    old_file: ($) => iseq("---", $.filename),
    new_file: ($) => iseq("+++", $.filename),

    location: ($) =>
      iseq("@@", $.linerange, $.linerange, "@@", optional(ANYTHING)),

    _hunk_addition: ($) => iseq("+++", ANYTHING),
    _hunk_deletion: ($) => iseq("---", ANYTHING),

    addition: ($) =>
      choice(
        iseq("+", optional(ANYTHING)),
        iseq("++", optional(ANYTHING)),
        iseq("+++"),
        iseq("++++", optional(ANYTHING)),
        iseq(">", optional(ANYTHING))
      ),
    deletion: ($) =>
      choice(
        iseq("-", optional(ANYTHING)),
        iseq("--", optional(ANYTHING)),
        iseq("---"),
        iseq("----", optional(ANYTHING)),
        iseq("<", optional(ANYTHING))
      ),
    change: ($) => iseq("!", optional(ANYTHING)),

    context: ($) => iseq(" ", optional(ANYTHING)),
    comment: ($) => iseq("#", optional(ANYTHING)),
    special: ($) => iseq("\\", optional(ANYTHING)),
    unrecognized: ($) => token(prec(-1, ANYTHING)),

    linerange: ($) => /[-\+]\d+(,\d+)?/,
    filename: ($) => repeat1(/\S+/),
    commit: ($) => /[a-f0-9]{4,64}/,
    mode: ($) => /\d+/,
  },
});

function iseq(start_token, ...tokens) {
  return seq(token.immediate(start_token), ...tokens);
}
