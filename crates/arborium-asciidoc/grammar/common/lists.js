exports.rules = {
  list: $ => choice($.unordered_list, $.ordered_list, $.checked_list),
  checked_list: $ => prec.left(repeat1($.checked_list_item)),
  checked_list_item: $ =>
    prec.left(
      seq(
        $.checked_list_marker,
        $._WHITE_SPACE,
        $.line,
        optional(seq($.list_continuation, $.block_element)),
      ),
    ),
  checked_list_marker: $ =>
    seq(
      $.unordered_list_marker,
      $._WHITE_SPACE,
      choice($.checked_list_marker_checked, $.checked_list_marker_unchecked),
    ),
  checked_list_marker_checked: $ => token(prec(1, /\[[*x]\]/)),
  checked_list_marker_unchecked: $ => token(prec(1, '[ ]')),

  unordered_list: $ => prec.right(repeat1($.unordered_list_item)),
  unordered_list_item: $ =>
    prec.left(
      seq(
        $.unordered_list_marker,
        $._WHITE_SPACE,
        $.line,
        optional(seq($.list_continuation, $.block_element)),
      ),
    ),
  unordered_list_marker: $ => choice($.list_marker_star, $.list_marker_hyphen),

  ordered_list: $ => prec.right(repeat1($.ordered_list_item)),
  ordered_list_item: $ =>
    prec.left(
      seq(
        $.ordered_list_marker,
        $._WHITE_SPACE,
        $.line,
        optional(seq($.list_continuation, $.block_element)),
      ),
    ),
  ordered_list_marker: $ =>
    choice(
      $.list_marker_digit,
      $.list_marker_geek,
      $.list_marker_alpha,
      $.list_marker_dot,
    ),

  callout_list: $ => prec.left(repeat1($.callout_list_item)),
  callout_list_item: $ =>
    prec.left(
      seq(
        $.callout_list_marker,
        $._WHITE_SPACE,
        $.line,
        optional(seq($.list_continuation, $.block_element)),
      ),
    ),
};
