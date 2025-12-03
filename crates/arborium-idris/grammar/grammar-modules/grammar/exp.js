import {
  brackets,
  layouted,
  layouted_without_end,
  parens,
  idiom_brackets,
  snoc_brackets,
  braces,
  qualified,
  sep,
  sep1,
  ticked,
} from './util.js';

export default {

  // ----- Identifiers and modifiers ------------------------------------------

  exp_name: $ => $._q_name_op,

  exp_ticked: $ => ticked($._aexp),

  exp_parens: $ => parens($._exp),

  exp_idiom: $ => idiom_brackets($._exp),

  exp_list: $ => choice(
    brackets(sep($.comma, $._exp)),
    snoc_brackets(sep($.comma, $._exp)),
  ),

  exp_list_comprehension: $ => brackets(
    seq(
      $._aexps,
      '|',
      sep1(
        $.comma,
        choice(
          seq(
            $._loname,
            $._larrow,
            $._aexps
          ),
          $._aexps,
        ),
      )
    ),
  ),

  exp_braces: $ => braces(sep1(
    $.comma,
    $.explicit_arg,
  )),

  explicit_arg: $ => seq(
    field('subject', $._name),
    optional(seq($.equal,
      field('object', $._exp)
    )),
  ),

  exp_explicit_impl: $ => seq('@{', $._exp, '}'),

  // ----- Tuples or tuple secions --------------------------------------------

  exp_tuple: $ => parens(
    seq(
      repeat1(
        seq(
          optional($._exp),
          $.tuple_operator,
        ),
      ),
      optional($._exp),
    ),
  ),

  // ----- Records ------------------------------------------------------------

  _record_access_field: $ => $._immediate_loname,

  exp_record_access: $ =>
    prec(1, seq(
      choice($.hole, $._parens, $._q_loname),
      repeat1(seq($._immediate_dot, field('field', $._record_access_field)))
    )),

  // ----- If-then-else -------------------------------------------------------

  exp_if: $ => seq(
    'if',
    field('if', $._exp),
    'then',
    field('then', alias($._exp, $.exp_then)),
    'else',
    field('else', alias($._exp, $.exp_else)),
  ),

  // ----- Case-of ------------------------------------------------------------

  alt: $ =>
    seq(
      sep1($.comma, field('pat', $._pat)),
      choice(
        seq(
          $._rcarrow,
          field('exp', $._exp),
          optional(seq($.where, $.declarations))
        ),
        $.impossible,
      ),
    ),

  alts: $ => choice(
    seq('\n', layouted($, $.alt)),
    braces(sep1(';', $.alt)),
    sep1(';', $.alt),
  ),

  _exp_case_slots: $ =>
    sep1(
      $.comma,
      field('condition', $._exp)
    ),

  exp_case: $ => seq(
    'case',
    $._exp_case_slots,
    'of',
    $.alts
  ),

  // ----- Let-in -------------------------------------------------------------

  _let_decl: $ => choice(
    $._gendecl,
    seq(
      alias($._pat, $.lhs),
      alias($._funrhs, $.rhs),
    ),
  ),

  _let_in_decls: $ => layouted_without_end($, $._let_decl),

  exp_let_in: $ => seq(
    'let',
    alias($._let_in_decls, $.declarations),
    repeat($.bind_alt),
    'in',
    alias($._exp, $.in),
  ),

  // ----- Rewrite-in ---------------------------------------------------------

  exp_rewrite_in: $ => seq(
    'rewrite',
    layouted_without_end($, alias($._aexps, $.rewrite_exp)),
    'in',
    alias($._exp, $.in),
  ),

  // ----- Lambdas ------------------------------------------------------------

  exp_lambda: $ => seq(
    '\\',
    $.lambda_args,
    $._rcarrow,
    alias($._type, $.lambda_exp),
  ),

  lambda_args: $ => sep1($.comma, $._pat),

  // ----- Lambda case  -------------------------------------------------------

  exp_lambda_case: $ => seq(
    choice(
      '\\case',
      seq('\\', 'case'),
    ),
    $.alts
  ),

  // ----- do notation --------------------------------------------------------

  exp_do: $ => seq(
    choice('do', qualified($, 'do')),
    layouted($, $.statement)
  ),

  bind_pattern: $ => seq(
    $._pat,
    $._larrow,
    alias($._exp, $.bind_exp),
    repeat($.bind_alt),
  ),

  _let_decls: $ => layouted($, $._let_decl),

  let: $ => seq(
    'let',
    alias($._let_decls, $.declarations),
    repeat($.bind_alt),
  ),

  bind_alt: $ => seq('|', $._pat, $._rcarrow, $._exp),

  rewrite: $ => seq(
    'rewrite',
    layouted($, alias($._aexps, $.rewrite_exp)),
  ),

  statement: $ =>
    choice(
      $._exp,
      $.bind_pattern,
      $.let,
      $.rewrite,
    ),

  // ----- Disambiguation with ------------------------------------------------

  exp_with: $ => seq(
    'with',
    choice(
      $._q_name_op,
      brackets(sep($.comma, $._q_name_op)),
    ),
    $._exp,
  ),

  // ----- Quasiquotation -----------------------------------------------------

  exp_quasiquotation: $ => choice(
    seq("`{{", $._q_name_op, "}}"),
    seq("`{", $._q_name_op, "}"),
    seq("`(", $._type_parens, ")"),
    seq("~", $._q_name_op),
  ),

  // ----- Strings ------------------------------------------------------------

  string: $ => seq(
    '"',
    repeat(choice(
      $._in_string,
      /[^\\"\n]/,
      /\\\n\s*\\/,
      /\\[^{]/,
      seq(
        '\\{',
        alias($._exp, $.interpolation),
        '}'
      ),
    )),
    '"',
  ),

  triple_quote_string: $ => seq(
    '"""',
    repeat(choice(
      $._in_string,
      /[^"\\]+/,          // Match any non-quote, non-backslash characters
      /"[^"]|""/,         // Match single quotes or double quotes that aren't triple
      seq(
        '\\{',
        alias($._exp, $.interpolation),
        '}'
      ),
    )),
    '"""'
  ),

  raw_string: $ => seq(
    $._raw_string_start,
    repeat(choice(
      $._in_string,
      /[^\\\n]/,
      /\\\n\s*\\/,
      /\\[^#]/,
      /\\#[^{]/,
      seq(
        '\\#{',
        alias($._exp, $.interpolation),
        '}'
      ),
    )),
    $._raw_string_end,
  ),

  _string: $ => choice(
    $.triple_quote_string,
    $.string,
    $.raw_string,
  ),

  // ----- Bar arguments ------------------------------------------------------

  exp_bar_arg: $ => prec(-1, seq('|', $._aexps)),

  // ----- Types --------------------------------------------------------------

  _parens: $ => parens(
    choice(
      alias($._exp, $.exp_parens),
      alias($._type_parens, $.type_parens),
    ),
  ),

  _braces: $ => choice(
    $.exp_braces,
    $.type_braces,
  ),

  // ----- Composite expressions ----------------------------------------------

  _aexp: $ => choice(
    $.hole,
    $.exp_name,
    $.exp_ticked,
    $.exp_idiom,
    $.exp_list,
    $.exp_list_comprehension,
    $.exp_explicit_impl,
    $.record_update,
    $.exp_record_access,
    $.exp_tuple,
    $.exp_quasiquotation,
    $.operator,
    $._string,
    $.literal,
    $.wildcard,
    $.unit,
    $._pragma_exp,
    $._parens,
    $._braces,
  ),

  _aexps: $ => prec.right(repeat1($._aexp)),

  _sexp: $ => prec.right(choice(
    $.exp_lambda,
    $.exp_if,
    $.exp_case,
    $.exp_lambda_case,
    $.exp_let_in,
    $.exp_do,
    $.exp_rewrite_in,
    $.exp_with,
  )),

  _exp: $ => choice(
    seq($._aexps, optional($.exp_bar_arg)),
    $._sexp,
    seq($._aexps, $._sexp),
  ),
};
