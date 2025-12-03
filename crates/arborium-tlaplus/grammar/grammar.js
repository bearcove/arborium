const PREC = {
  COMMENT: 0,
  BLOCK_COMMENT: 1,
  PCAL: 2,
  STRING: 3
}

// A sequence of one or more comma-separated strings matching the rule
function commaList1(rule) {
  return seq(rule, repeat(seq(',', rule)))
}

// A sequence of zero or more comma-separated strings matching the rule
function commaList(rule) {
  return optional(commaList1(rule))
}

// An operator with 0 or more parameters.
function arity0OrN(op, expr) {
  return seq(
    field('name', op),
    field('parameter', optional(seq('(', commaList1(expr), ')'))))
}

// An operator with 1 or more parameters.
function arity1OrN(op, expr) {
  return seq(
    field('name', op),
    field('parameter', seq('(', commaList1(expr), ')'))
  )
}

// A rule matching either the first or second rule, or the first then the
// second rule, but not matching nothing.
function oneOrBoth(first, second) {
  return choice(first, second, seq(first, second))
}

// Defines a labelled left-associative prefix operator of given precedence
function prefixOpPrec(level, expr, symbol) {
  return prec.left(level, seq(
    field('symbol', symbol),
    field('rhs', expr)
  ))
}

// Defines a labelled left-associative infix operator of given precedence
function infixOpPrec(level, expr, symbol) {
  return prec.left(level, seq(
    field('lhs', expr),
    field('symbol', symbol),
    field('rhs', expr)
  ))
}

// Defines a labelled postfix operator of given precedence
function postfixOpPrec(level, expr, symbol) {
  return prec(level, seq(
    field('lhs', expr),
    field('symbol', symbol)
  ))
}

function regexOr(regex) {
    if (arguments.length > 1) {
        regex = Array.from(arguments).join('|');
    }
    return {
        type: 'PATTERN',
        value: regex
    };
}

module.exports = grammar({
  name: 'tlaplus',

  externals: $ => [
    $.leading_extramodular_text,
    $.trailing_extramodular_text,
    $._indent,
    $._bullet,
    $._dedent,
    $._begin_proof,
    $._begin_proof_step,
    'PROOF',
    'BY',
    'OBVIOUS',
    'OMITTED',
    'QED',
    'WF_',
    'SF_',
    $._notify_pcal_algorithm_start,
    $._notify_pcal_algorithm_end,
    $._double_excl,
    $._error_sentinel
  ],

  word: $ => $.identifier,

  supertypes: $ => [
    $._unit,
    $._expr,
    $._op,
    $._proof,
    $._number,
    $._primitive_value_set,
    $._number_set
  ],

  extras: $ => [
    /\s|\r?\n/,
    $.comment,
    $.block_comment,
  ],

  // Prefix, infix, and postfix operator precedence categories, defined
  // on p271 of Specifying Systems. Precedences defined in this way are
  // defined only in relation to each other, not to every other rule.
  // See https://github.com/tree-sitter/tree-sitter/pull/939
  precedences: $ => [
    [
      '17-17',  '16-16',  '15-15',  '14-14',  '13-13',  '12-12',  '11-11',
      '10-10',  '9-9',    '8-8',    '7-7',    '6-6',    '5-5',    '4-4',
      '3-3',    '2-2',    '1-1',    '0-0'
    ],
  ],

  conflicts: $ => [
    // Lookahead to disambiguate '-'  •  '('  …
    [$.minus, $.negative],
    // Lookahead to disambiguate lnot • '('  …
    // Could be nonfix prefix op or prefix op applied to expr in parentheses.
    [$.bound_prefix_op, $.prefix_op_symbol],
    // Lookahead to disambiguate 'SF_'  identifier  •  '('  …
    // Could be SF_op(x)(e) or could be SF_id(e)
    [$.bound_op, $._subscript_expr],
    // Lookahead to disambiguate SF_'  subexpr_prefix  identifier  •  '('  …
    // Could be SF_A!B!C!op(x)(e) or could be SF_A!B!C!id(e)
    [$.bound_op, $.prefixed_op],
    // Lookahead to disambiguate identifier '(' identifier •  ',' …
    // Could be op(a, b) or could be lbl(a, b) ::
    [$._expr, $.label],
    // Lookahead to disambiguate '['  identifier  •  '\in'  …
    // Matches both step_expr_or_stutter and function_literal
    [$._expr, $.quantifier_bound],
    // Lookahead to disambiguate '{'  identifier  •  '\in'  …
    // Matches both finite_set_literal and set_filter
    [$._expr, $.restricted_quantifier_bound],
    // Lookahead to disambiguate '['  langle_bracket  identifier  •  '>>'  …
    // Matches step_expr_or_stutter and function_literal
    [$._expr, $.tuple_of_identifiers],
    // Lookahead to disambiguate identifier  •  '\in'  …
    // Could be x \in y == ... (op def'n) or x \in S (expression)
    [$._expr, $.operator_definition],
    // Lookahead to disambiguate identifier  '('  identifier  •  ','  …
    // Could be op(a, b) == ... (decl'n) or op(a, b) (expression) or label
    [$._expr, $._id_or_op_declaration, $.label],
    // Lookahead to disambiguate identifier  •  '['  …
    // Could be f[x \in S] == ... (function def'n) or f[x] (application)
    [$._expr, $.function_definition],
    // Lookahead to disambiguate subexpr_component  '!'  •  '\in'  …
    // The '\in' could be followed by a ! or it could be the end
    [$.subexpr_prefix],
    // Lookahead to disambiguate identifier  •  '['  …
    // Could be application `f[x]` or could be assignment `record[x] :=`
    [$._expr, $.pcal_lhs],
    // Lookahead to disambiguate _expr  •  '||'  …
    // Could be `x || y` or could be assignment `x := y || y := x`
    [$.bound_infix_op, $.pcal_assign],
  ],

  rules: {
    // Can be one of three things:
    // * a valid TLA⁺ source file with an encapsulating module
    // * a source file containing multiple modules (ambiguously valid but used by the tools)
    // * a TLA⁺ snippet without an encapsulating module
    // * a PlusCal algorithm without an encapsulating module
    source_file: $ => choice(
      seq(
        optional(alias($.leading_extramodular_text, $.extramodular_text)),
        repeat1(prec(1, seq($.module, optional(alias($.trailing_extramodular_text, $.extramodular_text)))))
      ),
      seq(
        optional($.extends),
        repeat1($._unit)
      ),
      $.pcal_algorithm
    ),

    // \* this is a comment ending with newline
    comment: $ => token(prec(PREC.COMMENT, /\\\*.*/)),

    // source: https://github.com/tree-sitter/tree-sitter/discussions/1252#discussioncomment-988725
    block_comment: $ => seq(
      token(prec(PREC.BLOCK_COMMENT, '(*')),
      repeat(
        choice(
          $.pcal_algorithm,
          $.block_comment_text
        )
      ),
      token(prec(PREC.BLOCK_COMMENT, '*)'))
    ),

    block_comment_text: $ =>
      prec.right(repeat1(
        choice(
          token(prec(PREC.BLOCK_COMMENT, regexOr(
            '[^*()]', // any symbol except reserved
            '[^*][)]', // closing parenthesis, which is not a comment end
            '[(][^(*]', // opening parenthesis, which is not a comment start
            '[*][)][ \t]*(\r\n|\n)?[ \t]*[(][*]' // contiguous block comment border
          ))),
          token(prec(PREC.BLOCK_COMMENT, /\*/)),
          token(prec(PREC.BLOCK_COMMENT, /\(/)),
          token(prec(PREC.BLOCK_COMMENT, /\)/)),
        )
      )),

    // Top-level module declaration
    module: $ => seq(
      alias($.single_line, $.header_line),
      'MODULE', field('name', $.identifier),
      alias($.single_line, $.header_line),
      optional($.extends),
      repeat($._unit),
      $.double_line
    ),

    // Line of ---------- of length at least 4
    single_line: $ => seq('----', token.immediate(/-*/)),

    // Line of =========== of length at least 4
    double_line: $ => seq('====', token.immediate(/=*/)),

    // Various syntactic elements and their unicode equivalents
    def_eq:             $ => choice('==', '≜'),
    set_in:             $ => choice('\\in', '∈'),
    gets:               $ => choice('<-', '⟵', '←'),
    forall:             $ => choice('\\A', '\\forall', '∀'),
    exists:             $ => choice('\\E', '\\exists', '∃'),
    temporal_forall:    $ => '\\AA',
    temporal_exists:    $ => '\\EE',
    all_map_to:         $ => choice('|->', '⟼', '↦'),
    maps_to:            $ => choice('->', '⟶', '→'),
    langle_bracket:     $ => choice('<<', '〈', '⟨'),
    rangle_bracket:     $ => choice('>>', '〉', '⟩'),
    rangle_bracket_sub: $ => choice('>>_', '〉_', '⟩_'),
    case_box:           $ => choice('[]', '□'),
    case_arrow:         $ => choice('->', '⟶', '→'),
    colon:              $ => ':',
    address:            $ => '@',
    label_as:           $ => choice('::', '∷'),
    placeholder:        $ => '_',
    bullet_conj:        $ => choice('/\\', '∧'),
    bullet_disj:        $ => choice('\\/', '∨'),

    // The set of all reserved keywords
    keyword: $ => choice(
      'ASSUME',       'ELSE',       'LOCAL',      'UNION',
      'ASSUMPTION',   'ENABLED',    'MODULE',     'VARIABLE',
      'AXIOM',        'EXCEPT',     'OTHER',      'VARIABLES',
      'CASE',         'EXTENDS',    'SF_',        'WF_',
      'CHOOSE',       'IF',         'SUBSET',     'WITH',
      'CONSTANT',     'IN',         'THEN',
      'CONSTANTS',    'INSTANCE',   'THEOREM',    'COROLLARY',
      'DOMAIN',       'LET',        'UNCHANGED',
      'BY',           'HAVE',       'QED',        'TAKE',
      'DEF',          'HIDE',       'RECURSIVE',  'USE',
      'DEFINE',       'PROOF',      'WITNESS',    'PICK',
      'DEFS',         'PROVE',      'SUFFICES',   'NEW',
      'LAMBDA',       'STATE',      'ACTION',     'TEMPORAL',
      'OBVIOUS',      'OMITTED',    'LEMMA',      'PROPOSITION',
      'ONLY'
    ),

    // General-purpose identifier, should exclude reserved keywords,
    // but tree-sitter currently does not support match exclusion logic.
    // https://github.com/tlaplus-community/tree-sitter-tlaplus/issues/14
    // Can contain letters, numbers, and underscores
    // Must contain at least one alphabetic character (not number or _)
    // Cannot start with WF_ or SF_
    identifier: $ => /([0-9_]*[A-Za-z][A-Za-z0-9_]*)/,

    // EXTENDS Naturals, FiniteSets, Sequences
    extends: $ => seq(
      'EXTENDS', commaList1(alias($.identifier, $.identifier_ref))
    ),

    // A module-level definition
    _unit: $ => choice(
      $.variable_declaration,
      $.constant_declaration,
      $.recursive_declaration,
      $.use_or_hide,
      $.local_definition,
      $._definition,
      $.assumption,
      $.theorem,
      $.module,
      $.single_line
    ),

    local_definition: $ => seq('LOCAL', $._definition),

    _definition: $ => choice(
      $.operator_definition,
      $.function_definition,
      $.instance,
      $.module_definition,
    ),

    // VARIABLES v1, v2, v3
    variable_declaration: $ => seq(
        choice('VARIABLE', 'VARIABLES'),
        commaList1($.identifier)
    ),

    // CONSTANTS C1, C2, C3
    constant_declaration: $ => seq(
        choice('CONSTANT', 'CONSTANTS'),
        commaList1($._id_or_op_declaration)
    ),

    // RECURSIVE op(_, _)
    recursive_declaration: $ => seq(
      'RECURSIVE', commaList1($._id_or_op_declaration)
    ),

    // Operator declaration (not definition)
    // Used, for example, when op accepts another op as parameter
    // op, op(_,_), _+_, etc.
    operator_declaration: $ => choice(
      arity1OrN($.identifier, $.placeholder),
      seq(field('name', $.prefix_op_symbol), $.placeholder),
      seq($.placeholder, field('name', $.infix_op_symbol), $.placeholder),
      seq($.placeholder, field('name', $.postfix_op_symbol))
    ),

    // Either an identifier or an operator declaration
    // Used to define parameters to operators during their definition
    _id_or_op_declaration: $ => choice(
      $.identifier,
      $.operator_declaration
    ),

    // Operator definition
    // max(a, b) == IF a > b THEN a ELSE b
    // a \prec b == a.val < b.val
    // x ≜ 〈 1, 2, 3, 4, 5 〉
    operator_definition: $ => seq(
      choice(
        arity0OrN(choice($.identifier, $._number_set), $._id_or_op_declaration),
        seq(
          field('name', $.prefix_op_symbol),
          field('parameter', $.identifier)
        ),
        seq(
          field('parameter', $.identifier),
          field('name', $.infix_op_symbol),
          field('parameter', $.identifier)
        ),
        seq(
          field('parameter', $.identifier),
          field('name', $.postfix_op_symbol)
        )
      ),
      $.def_eq,
      field('definition', $._expr)
    ),

    // f[x \in Nat] == 2*x
    function_definition: $ => seq(
      field('name', $.identifier),
      '[', commaList1($.quantifier_bound), ']',
      $.def_eq,
      field('definition', $._expr)
    ),

    // INSTANCE ModuleName WITH x <- y, w <- z
    instance: $ => seq(
      'INSTANCE',
      alias($.identifier, $.identifier_ref),
      optional(seq('WITH', commaList1($.substitution)))
    ),

    // x <- y, w <- z
    substitution: $ => seq(
      choice(
        alias($.identifier, $.identifier_ref),
        $.prefix_op_symbol,
        $.infix_op_symbol,
        $.postfix_op_symbol
      ),
      $.gets,
      $._op_or_expr
    ),

    // Either an operator (op, +, *, LAMBDA, etc.) or an expression
    _op_or_expr: $ => choice($._op, $._expr),

    _op: $ => choice(
      $.prefix_op_symbol,
      $.infix_op_symbol,
      $.postfix_op_symbol,
      $.lambda,
    ),

    // foo!bar!baz!
    subexpr_prefix: $ => seq(
      choice($.subexpr_component, $.proof_step_ref), '!',
      repeat(seq(choice($.subexpr_component, $.subexpr_tree_nav), '!'))
    ),

    // Subexpression component referencing a previously-defined symbol
    // Can either bind parameters to the op or leave them unbound
    subexpr_component: $ => choice(
      alias($.identifier, $.identifier_ref),
      $.bound_op,
      $.bound_nonfix_op,
      $.prefix_op_symbol,
      $.infix_op_symbol,
      $.postfix_op_symbol
    ),

    // f(a, op, b)
    bound_op: $ => arity1OrN(alias($.identifier, $.identifier_ref), $._op_or_expr),

    // +(2, 4)
    bound_nonfix_op: $ => choice(
      seq(field('symbol', $.prefix_op_symbol), '(', $._expr, ')'),
      seq(field('symbol', $.infix_op_symbol), '(', $._expr, ',', $._expr, ')'),
      seq(field('symbol', $.postfix_op_symbol), '(', $._expr, ')'),
    ),

    // Metalanguage to navigate the parse tree of an expression
    // F!:!〈!〉!2!(0, 3)!@
    subexpr_tree_nav: $ => choice(
      $.langle_bracket,   // first parse node child
      $.rangle_bracket,   // second parse node child
      $.child_id,         // nth parse node child
      $.colon,            // for recursive operators
      $.address,          // use unbound quantifier as lambda
      $.operator_args     // bind quantifier
    ),

    // ...!2!3!...
    child_id: $ => /\d+/,

    // ...!(a, b, c)!...
    operator_args: $ => seq(
      '(', commaList1($._op_or_expr), ')'
    ),

    // LAMBDA a, b, c : a + b * c
    lambda: $ => seq(
      'LAMBDA', commaList1($.identifier), ':', $._expr
    ),

    // M == INSTANCE ModuleName
    module_definition: $ => seq(
      arity0OrN($.identifier, $._id_or_op_declaration),
      $.def_eq,
      field('definition', $.instance)
    ),

    /************************************************************************/
    /* EXPRESSIONS                                                          */
    /************************************************************************/

    // Anything that evaluates to a value
    _expr: $ => choice(
      $._number,
      $.string,
      $.boolean,
      $._primitive_value_set,
      $.parentheses,
      $.label,
      $.subexpression,
      $.proof_step_ref,
      alias($.identifier, $.identifier_ref),
      $.bound_op,
      $.bound_nonfix_op,
      $.prefixed_op,
      $.bound_prefix_op,
      $.bound_infix_op,
      $.bound_postfix_op,
      $.bounded_quantification,
      $.unbounded_quantification,
      $.choose,
      $.finite_set_literal,
      $.set_filter,
      $.set_map,
      $.function_evaluation,
      $.function_literal,
      $.set_of_functions,
      $.record_literal,
      $.set_of_records,
      $.record_value,
      $.except,
      $.prev_func_val,
      $.tuple_literal,
      $.step_expr_or_stutter,
      $.step_expr_no_stutter,
      $.fairness,
      $.if_then_else,
      $.case,
      $.let_in,
      $.conj_list,
      $.disj_list,
    ),

    // Expressions allowed in subscripts; must be enclosed in delimiters
    // Used in WF_expr, <><<f>>_expr, etc.
    _subscript_expr: $ => choice(
      alias($.identifier, $.identifier_ref),
      $.bound_op,
      $.bound_nonfix_op,
      $.prefixed_op,
      $.parentheses,
      $.finite_set_literal,
      $.set_filter,
      $.set_map,
      $.function_literal,
      $.set_of_functions,
      $.record_literal,
      $.set_of_records,
      $.except,
      $.tuple_literal,
      $.step_expr_or_stutter,
      $.step_expr_no_stutter,
    ),

    // foo!bar!baz!op(x, y)
    prefixed_op: $ => seq(
      field('prefix', $.subexpr_prefix),
      field('op', choice(
        choice(alias($.identifier, $.identifier_ref), $._number_set),
        $.bound_op,
        $.bound_nonfix_op
      ))
    ),

    // Number literal encodings
    _number: $ => choice(
      $.nat_number,     $.real_number,
      $.binary_number,  $.octal_number,   $.hex_number
    ),

    // 12345
    nat_number: $ => /\d+/,

    // 3.14159
    real_number: $ => /\d*\.\d+/,

    // \B01010101
    binary_number: $ => seq(
      alias(choice('\\b', '\\B'), $.format),
      alias(token.immediate(/[0-1]+/), $.value)
    ),

    // \O01234567
    octal_number: $ => seq(
      alias(choice('\\o', '\\O'), $.format),
      alias(token.immediate(/[0-7]+/), $.value)
    ),

    // \HDEADC0DE
    hex_number: $ => seq(
      alias(choice('\\h', '\\H'), $.format),
      alias(token.immediate(/[0-9a-fA-F]+/), $.value)
    ),

    // "foobar", "", etc.
    string: $ => seq(
      '"',
      repeat(choice(
        // Proper lexical precedence allows to parse strings like "(*",
        // winning over block comment lexemes
        token.immediate(prec(PREC.STRING, /[^"\n]/)),
        $.escape_char
      )),
      token.immediate('"')
    ),

    // "/\\", "say \"hello\" back", "one\ntwo"
    escape_char: $ => token.immediate(prec(PREC.STRING, seq('\\', /./))),

    // TRUE, FALSE, BOOLEAN
    boolean: $ => choice('TRUE', 'FALSE'),

    // Various number sets
    _number_set: $ => choice($.nat_number_set, $.int_number_set, $.real_number_set),

    // Set of all strings, booleans, and numbers
    _primitive_value_set: $ => choice($.string_set, $.boolean_set, $._number_set),

    string_set:       $ => 'STRING',            // From TLA⁺ builtins
    boolean_set:      $ => 'BOOLEAN',           // From TLA⁺ builtins
    nat_number_set:   $ => choice('Nat', 'ℕ'),  // From Naturals standard module
    int_number_set:   $ => choice('Int', 'ℤ'),  // From Integers standard module
    real_number_set:  $ => choice('Real', 'ℝ'), // From Reals standard module

    // Label for less-fragile addressing of subexpressions
    // lbl(a, b) :: e
    label: $ => prec('0-0', seq(
      field('name', arity0OrN($.identifier, alias($.identifier, $.identifier_ref))),
      $.label_as,
      field('expression', $._expr)
    )),

    // Address subexpressions through syntax tree navigation
    // op!+!<<!@
    subexpression: $ => seq(
      $.subexpr_prefix,
      $.subexpr_tree_nav
    ),

    // ((a + b) + c)
    parentheses: $ => seq('(', $._expr, ')'),

    // \A x \in Nat : P(x)
    bounded_quantification: $ => prec('0-0', seq(
      field('quantifier', choice($.forall, $.exists)),
      field('bound', commaList1($.quantifier_bound)),
      ':',
      field('expression', $._expr)
    )),

    // \EE x : P(x)
    unbounded_quantification: $ => prec('0-0', seq(
      field('quantifier', choice(
        $.forall, $.exists, $.temporal_forall, $.temporal_exists
      )),
      field('intro', commaList1($.identifier)),
      ':',
      field('expression', $._expr)
    )),

    // CHOOSE r \in Real : r >= 0
    choose: $ => prec('0-0', seq(
      'CHOOSE',
      field('intro', choice($.identifier, $.tuple_of_identifiers)),
      optional(seq($.set_in, field('set', $._expr))),
      ':',
      field('expression', $._expr)
    )),

    // {1, 2, 3, 4, 5}
    finite_set_literal: $ => seq('{', commaList($._expr), '}'),

    // { x \in S : P(x) }
    // Set dynamic precedence to 1 so that the expression {x \in S : x \in P}
    // is parsed as set_filter instead of set_map during GLR parsing.
    set_filter: $ => prec.dynamic(1, seq(
      '{',
      field('generator', alias($.restricted_quantifier_bound, $.quantifier_bound)),
      ':',
      field('filter', $._expr),
      '}'
    )),

    // { f[x, y] : x, y \in S }
    set_map: $ => seq(
      '{',
      field('map', $._expr),
      ':',
      field('generator', commaList1($.quantifier_bound)),
      '}'
    ),

    // x, y, z \in S
    // <<x, y, z>> \in S \X T \X P
    quantifier_bound: $ => seq(
      field('intro', choice(commaList1($.identifier), $.tuple_of_identifiers)),
      $.set_in,
      field('set', $._expr)
    ),

    // A quantifier bound allowing only one introduced element
    restricted_quantifier_bound: $ => seq(
      field('intro', choice($.identifier, $.tuple_of_identifiers)),
      $.set_in,
      field('set', $._expr)
    ),

    // <<x, y, z>>
    tuple_of_identifiers: $ => seq(
      $.langle_bracket,
      commaList1($.identifier),
      $.rangle_bracket
    ),

    // f[5]
    function_evaluation: $ => prec('16-16', seq(
      $._expr, '[', commaList1($._expr), ']'
    )),

    // [n \in Nat |-> 2*n]
    function_literal: $ => seq(
      '[', commaList1($.quantifier_bound), $.all_map_to, $._expr, ']'
    ),

    // [Nat -> Nat]
    set_of_functions: $ => seq(
      '[', $._expr, $.maps_to, $._expr, ']'
    ),

    // [foo |-> 0, bar |-> 1]
    record_literal: $ => seq(
      '[', commaList1(seq($.identifier, $.all_map_to, $._expr)), ']'
    ),

    // [foo : {0, 1}, bar : {0, 1}]
    set_of_records: $ => seq(
      '[', commaList1(seq($.identifier, ':', $._expr)), ']'
    ),

    // r.val
    record_value: $ => prec.left('17-17', seq($._expr, '.', alias($.identifier, $.identifier_ref))),

    // [f EXCEPT !.foo["bar"].baz = 4, !.bar = 3]
    except: $ => seq(
        '[',
        field("expr_to_update", $._expr),
        'EXCEPT',
        commaList1($.except_update),
        ']'
    ),

    // !.foo["bar"].baz = 4
    except_update: $ => seq(
        field("update_specifier", seq('!', $.except_update_specifier)),
        '=',
        field("new_val", $._expr)
    ),

    // .foo["bar"].baz
    except_update_specifier: $ => repeat1(
        choice(
         $.except_update_record_field,
         $.except_update_fn_appl
        )
    ),

    // .foo
    except_update_record_field: $ => seq('.', alias($.identifier, $.identifier_ref)),

    // ["bar"]
    except_update_fn_appl: $ => seq('[', commaList1($._expr), ']'),

    // [f EXCEPT ![2] = @ + 1]
    prev_func_val: $ => '@',

    // <<1,2,3,4,5>>, <<>>
    tuple_literal: $ => seq(
      $.langle_bracket,
      commaList($._expr),
      $.rangle_bracket
    ),

    // [x ' > x]_<<x>>
    step_expr_or_stutter: $ => seq(
      '[', $._expr, ']_', $._subscript_expr
    ),

    // <<x' > x>>_<<x>>
    step_expr_no_stutter: $ => seq(
      $.langle_bracket,
      $._expr,
      $.rangle_bracket_sub,
      $._subscript_expr
    ),

    // WF_vars(ActionName)
    fairness: $ => seq(
      choice('WF_', 'SF_'), $._subscript_expr, '(', $._expr, ')'
    ),

    // IF a > b THEN a ELSE b
    if_then_else: $ => prec('0-0', seq(
      'IF', field('if', $._expr),
      'THEN', field('then', $._expr),
      'ELSE', field('else', $._expr)
    )),

    // CASE x = 1 -> "1" [] x = 2 -> "2" [] OTHER -> "3"
    // This exhibits dangling else parsing ambiguity; consider
    // CASE e -> CASE f -> g [] h -> i
    // This can be parsed as:
    // (1) CASE e -> (CASE f -> g) [] h -> i
    // (2) CASE e -> (CASE f -> g [] h -> i)
    // Parse (2) is used, making this right-associative
    case: $ => prec.right(seq(
      'CASE', $.case_arm,
      repeat(seq($.case_box, $.case_arm)),
      optional(seq($.case_box, $.other_arm))
    )),

    // p -> val
    case_arm: $ => prec('0-0', seq($._expr, $.case_arrow, $._expr)),

    // OTHER -> val
    other_arm: $ => prec('0-0', seq('OTHER', $.case_arrow, $._expr)),

    // LET x == 5 IN 2*x
    let_in: $ => prec('0-0', seq(
      'LET',
      field('definitions', repeat1(choice(
        $.operator_definition,
        $.function_definition,
        $.module_definition,
        $.recursive_declaration
      ))),
      'IN',
      field('expression', $._expr)
    )),

    // This makes use of the external scanner.
    // /\ x
    // /\ y
    conj_list: $ => seq(
      $._indent,
      repeat1($.conj_item),
      $._dedent
    ),

    // /\ x
    conj_item: $ => seq($._bullet, $.bullet_conj, $._expr),

    // This makes use of the external scanner.
    // \/ x
    // \/ y
    disj_list: $ => seq(
      $._indent,
      repeat1($.disj_item),
      $._dedent
    ),

    // \/ x
    disj_item: $ => seq($._bullet, $.bullet_disj, $._expr),

    /************************************************************************/
    /* PREFIX, INFIX, AND POSTFIX OPERATOR DEFINITIONS                      */
    /************************************************************************/

    // Prefix operator symbols and their unicode equivalents.
    lnot:             $ => choice('\\lnot', '\\neg', '~', '¬'),
    union:            $ => 'UNION',
    powerset:         $ => 'SUBSET',
    domain:           $ => 'DOMAIN',
    negative:         $ => '-',
    negative_dot:     $ => '-.',
    enabled:          $ => 'ENABLED',
    unchanged:        $ => 'UNCHANGED',
    always:           $ => choice('[]', '□'),
    eventually:       $ => choice('<>', '⋄', '◇'),

    // Use rule when using ops as higher-order constructs.
    // Negative is disambiguated from minus with a '.'
    prefix_op_symbol: $ => choice(
      $.lnot,     $.union,      $.powerset,   $.domain,
      $.enabled,  $.unchanged,  $.always,     $.eventually,
      alias($.negative_dot, $.negative)
    ),

    // All bound prefix operators.
    // Prefix operators are given highest value in precedence range.
    // Precedence defined on p271 of Specifying Systems.
    bound_prefix_op: $ => choice(
      prefixOpPrec('4-4',   $._expr,  $.lnot),
      prefixOpPrec('12-12', $._expr, $.negative),
      prefixOpPrec('13-13', $._expr, choice($.powerset, $.union, $.domain)),
      prefixOpPrec('15-15', $._expr, choice(
        $.enabled, $.unchanged, $.always, $.eventually))
    ),

    // Infix operator symbols and their unicode equivalents.
    implies:          $ => choice('=>', '⟹', '⇒'),
    plus_arrow:       $ => choice('-+->', '⇸', '⥅'),
    equiv:            $ => choice('\\equiv', '≡'),
    iff:              $ => choice('<=>', '⟺', '⇔'),
    leads_to:         $ => choice('~>', '⇝', '↝'),
    land:             $ => choice('/\\', '\\land', '∧'),
    lor:              $ => choice('\\/', '\\lor', '∨'),
    assign:           $ => choice(':=', '≔'),
    bnf_rule:         $ => choice('::=', '⩴'),
    eq:               $ => '=',
    neq:              $ => choice('/=', '#', '≠'),
    lt:               $ => '<',
    gt:               $ => '>',
    leq:              $ => choice('<=', '=<', '\\leq', '≤'),
    geq:              $ => choice('>=', '\\geq', '≥'),
    approx:           $ => choice('\\approx', '≈'),
    rs_ttile:         $ => choice('|-', '⊢'),
    rd_ttile:         $ => choice('|=', '⊨'),
    ls_ttile:         $ => choice('-|', '⊣'),
    ld_ttile:         $ => choice('=|', '⫤'),
    asymp:            $ => choice('\\asymp', '≍'),
    cong:             $ => choice('\\cong', '≅'),
    doteq:            $ => choice('\\doteq', '≐'),
    gg:               $ => choice('\\gg', '≫'),
    ll:               $ => choice('\\ll', '≪'),
    in:               $ => choice('\\in', '∈'),
    notin:            $ => choice('\\notin', '∉'),
    prec:             $ => choice('\\prec', '≺'),
    succ:             $ => choice('\\succ', '≻'),
    preceq:           $ => choice('\\preceq', '⪯'),
    succeq:           $ => choice('\\succeq', '⪰'),
    propto:           $ => choice('\\propto', '∝'),
    sim:              $ => choice('\\sim', '∼'),
    simeq:            $ => choice('\\simeq', '≃'),
    sqsubset:         $ => choice('\\sqsubset', '⊏'),
    sqsupset:         $ => choice('\\sqsupset', '⊐'),
    sqsubseteq:       $ => choice('\\sqsubseteq', '⊑'),
    sqsupseteq:       $ => choice('\\sqsupseteq', '⊒'),
    subset:           $ => choice('\\subset', '⊂'),
    supset:           $ => choice('\\supset', '⊃'),
    subseteq:         $ => choice('\\subseteq', '⊆'),
    supseteq:         $ => choice('\\supseteq', '⊇'),
    compose:          $ => '@@',
    map_to:           $ => ':>',
    map_from:         $ => '<:',
    setminus:         $ => '\\',
    cap:              $ => choice('\\cap', '\\intersect', '∩'),
    cup:              $ => choice('\\cup', '\\union', '∪'),
    dots_2:           $ => choice('..', '‥'),
    dots_3:           $ => choice('...', '…'),
    plus:             $ => '+',
    plusplus:         $ => '++',
    oplus:            $ => choice('\\oplus', '(+)', '⊕'),
    ominus:           $ => choice('\\ominus', '(-)', '⊖'),
    mod:              $ => '%',
    modmod:           $ => '%%',
    vert:             $ => '|',
    vertvert:         $ => choice('||', '‖'),
    minus:            $ => '-',
    minusminus:       $ => '--',
    amp:              $ => '&',
    ampamp:           $ => '&&',
    odot:             $ => choice('\\odot', '(.)', '⊙'),
    oslash:           $ => choice('\\oslash', '(/)', '⊘'),
    otimes:           $ => choice('\\otimes', '(\\X)', '⊗'),
    mul:              $ => '*',
    mulmul:           $ => '**',
    slash:            $ => '/',
    slashslash:       $ => '//',
    bigcirc:          $ => choice('\\bigcirc', '◯'),
    bullet:           $ => choice('\\bullet', '●'),
    div:              $ => choice('\\div', '÷'),
    circ:             $ => choice('\\o', '\\circ', '∘'),
    star:             $ => choice('\\star', '⋆'),
    excl:             $ => choice(alias($._double_excl, '!!'), '‼'),
    hashhash:         $ => '##',
    dol:              $ => '$',
    doldol:           $ => '$$',
    qq:               $ => choice('??', '⁇'),
    sqcap:            $ => choice('\\sqcap', '⊓'),
    sqcup:            $ => choice('\\sqcup', '⊔'),
    uplus:            $ => choice('\\uplus', '⊎'),
    times:            $ => choice('\\X', '\\times', '×'),
    wr:               $ => choice('\\wr', '≀'),
    cdot:             $ => choice('\\cdot', '⋅'),
    pow:              $ => '^',
    powpow:           $ => '^^',

    // All infix operator symbols.
    infix_op_symbol: $ => choice(
      $.implies,      $.plus_arrow,     $.equiv,        $.iff,
      $.leads_to,     $.land,           $.lor,          $.assign,
      $.bnf_rule,     $.eq,             $.neq,          $.lt,
      $.gt,           $.leq,            $.geq,          $.approx,
      $.rs_ttile,     $.rd_ttile,       $.ls_ttile,     $.ld_ttile,
      $.asymp,        $.cong,           $.doteq,        $.gg,
      $.ll,           $.in,             $.notin,        $.prec,
      $.succ,         $.preceq,         $.succeq,       $.propto,
      $.sim,          $.simeq,          $.sqsubset,     $.sqsupset,
      $.sqsubseteq,   $.sqsupseteq,     $.subset,       $.supset,
      $.subseteq,     $.supseteq,       $.compose,      $.map_to,
      $.setminus,     $.cap,            $.cup,          $.dots_2,
      $.dots_3,       $.plus,           $.plusplus,     $.oplus,
      $.ominus,       $.mod,            $.modmod,       $.vert,
      $.vertvert,     $.minus,          $.minusminus,   $.amp,
      $.ampamp,       $.odot,           $.oslash,       $.otimes,
      $.mul,          $.mulmul,         $.slash,        $.slashslash,
      $.bigcirc,      $.bullet,         $.div,          $.circ,
      $.star,         $.excl,           $.hashhash,     $.dol,
      $.doldol,       $.qq,             $.sqcap,        $.sqcup,
      $.uplus,        $.times,          $.wr,           $.cdot,
      $.pow,          $.powpow,         $.map_from,
    ),

    // Infix operators are given highest value in precedence range for parsing.
    // Infix operators are all marked as left-associative for parsing purposes.
    // Operator precedence range & associativity conflicts must be enforced
    // on semantic level. Precedence defined on p271 of Specifying Systems.
    bound_infix_op: $ => choice(
      infixOpPrec('1-1',  $._expr,
        $.implies),
      infixOpPrec('2-2',    $._expr, choice(
        $.plus_arrow, $.equiv, $.iff, $.leads_to)),
      infixOpPrec('3-3',    $._expr, choice(
        $.land, $.lor)),
      infixOpPrec('5-5',    $._expr, choice(
        $.assign,     $.bnf_rule, $.eq,       $.neq,      $.lt,
        $.gt,         $.leq,      $.geq,      $.approx,   $.rs_ttile,
        $.rd_ttile,   $.ls_ttile, $.ld_ttile, $.asymp,    $.cong,
        $.doteq,      $.gg,       $.ll,       $.in,       $.notin,
        $.prec,       $.succ,     $.preceq,   $.succeq,   $.propto,
        $.sim,        $.simeq,    $.sqsubset, $.sqsupset, $.sqsubseteq,
        $.sqsupseteq, $.subset,   $.supset,   $.subseteq, $.supseteq)),
      infixOpPrec('6-6',    $._expr,
        $.compose),
      infixOpPrec('7-7',    $._expr, choice(
        $.map_to, $.map_from)),
      infixOpPrec('8-8',    $._expr, choice(
        $.setminus, $.cap, $.cup)),
      infixOpPrec('9-9',    $._expr, choice(
        $.dots_2, $.dots_3)),
      infixOpPrec('10-10',  $._expr, choice(
        $.plus, $.plusplus, $.oplus)),
      infixOpPrec('11-11',  $._expr, choice(
        $.ominus,   $.mod,    $.modmod,     $.vert,
        $.vertvert, $.minus,  $.minusminus)),
      infixOpPrec('13-13',  $._expr, choice(
        $.amp,      $.ampamp, $.odot,     $.oslash,     $.otimes,
        $.mul,      $.mulmul, $.slash,    $.slashslash, $.bigcirc,
        $.bullet,   $.div,    $.circ,     $.star,       $.excl,
        $.hashhash, $.dol,    $.doldol,   $.qq,         $.sqcap,
        $.sqcup,    $.uplus,  $.times)),
      infixOpPrec('14-14',  $._expr, choice(
        $.wr, $.cdot, $.pow, $.powpow)),
    ),

    // Postfix operator symbols and their unicode equivalents.
    sup_plus:         $ => choice('^+', '⁺'),
    asterisk:         $ => '^*',
    sup_hash:         $ => '^#',
    prime:            $ => '\'',

    // All postfix operator symbols.
    postfix_op_symbol: $ => choice(
      $.sup_plus, $.asterisk, $.sup_hash, $.prime
    ),

    // All bound postfix operators.
    // Precedence defined on p271 of Specifying Systems.
    bound_postfix_op: $ => choice(
      postfixOpPrec('15-15', $._expr, choice(
        $.sup_plus, $.asterisk, $.sup_hash, $.prime))
    ),

    /************************************************************************/
    /* PROOF CONSTRUCTS                                                     */
    /************************************************************************/

    // ASSUME C \in Nat
    assumption: $ => seq(
      choice('ASSUME', 'ASSUMPTION', 'AXIOM'),
      optional(seq(field('name', $.identifier), $.def_eq)),
      $._expr
    ),

    // THEOREM Spec => []Safety
    theorem: $ => seq(
      choice('THEOREM', 'PROPOSITION', 'LEMMA', 'COROLLARY'),
      optional(seq(field('name', $.identifier), $.def_eq)),
      field('statement', choice($._expr, $.assume_prove)),
      field('proof', optional($._proof))
    ),

    // ASSUME NEW x \in Nat, NEW y \in Nat PROVE x + y \in Nat
    assume_prove: $ => seq(
      'ASSUME',
      field('assumption', commaList1(choice($._expr, $.new, $.inner_assume_prove))),
      'PROVE',
      field('conclusion', $._expr)
    ),

    // triangle ∷ ASSUME x > y, y > z PROVE x > z
    inner_assume_prove: $ => seq(
      optional(seq($.identifier, $.label_as)),
      $.assume_prove
    ),

    // NEW CONSTANT x \in Nat
    new: $ => seq(
      oneOrBoth('NEW', $.statement_level),
      choice(
        seq($.identifier, optional(seq($.set_in, $._expr))),
        $.operator_declaration
      )
    ),

    // The scope level of an introduction
    statement_level: $ => choice(
      'CONSTANT', 'VARIABLE', 'STATE', 'ACTION', 'TEMPORAL'
    ),

    _proof: $ => choice(
      $.terminal_proof,
      $.non_terminal_proof
    ),

    // PROOF BY z \in Nat
    terminal_proof: $ => seq(
      optional('PROOF'),
      choice(
        seq('BY', optional('ONLY'), $.use_body),
        'OBVIOUS',
        'OMITTED'
      )
    ),

    non_terminal_proof: $ => seq(
      optional('PROOF'),
      $._begin_proof,
      repeat($.proof_step),
      $.qed_step
    ),

    // A single step in a proof. Can be many things!
    proof_step: $ => seq(
      $._begin_proof_step,
      $.proof_step_id,
      choice(
        $.definition_proof_step,
        $.have_proof_step,
        $.witness_proof_step,
        $.take_proof_step,
        $.suffices_proof_step,
        $.case_proof_step,
        $.pick_proof_step,
        $.use_or_hide,
        $.instance
      )
    ),

    // Proof step defining a new unit symbol.
    definition_proof_step: $ => seq(
      optional('DEFINE'),
      repeat1(
        choice(
          $.operator_definition,
          $.function_definition,
          $.module_definition
        )
      )
    ),

    have_proof_step: $ => seq('HAVE', $._expr),
    witness_proof_step: $ => seq('WITNESS', commaList1($._expr)),
    take_proof_step: $ => seq('TAKE', $._bound_or_identifier_list),
    suffices_proof_step: $ => seq(
      optional('SUFFICES'), choice($._expr, $.assume_prove), optional($._proof)
    ),
    case_proof_step: $ => seq('CASE', $._expr, optional($._proof)),
    pick_proof_step: $ => seq(
      'PICK', $._bound_or_identifier_list, ':', $._expr,
      optional($._proof)
    ),

    // One of:
    //   a, b, c, d
    //   a \in P, b, c \in Q, d, e, f \in R
    _bound_or_identifier_list: $ => choice(
      commaList1($.quantifier_bound),
      commaList1($.identifier)
    ),

    // <*> QED
    qed_step: $ => seq(
      $._begin_proof_step,
      $.proof_step_id,
      'QED',
      optional($._proof)
    ),

    use_or_hide: $ => seq(
      choice('USE', 'HIDE'),
      $.use_body
    ),

    use_body: $ => oneOrBoth($.use_body_expr, $.use_body_def),

    // P, MODULE Naturals, Q, MODULE Integers
    use_body_expr: $ => commaList1(choice($._expr, $.module_ref)),

    // DEFS >, R, MODULE Reals, =
    use_body_def: $ => seq(
      choice('DEF', 'DEFS'),
      commaList1(choice($._op_or_expr, $.module_ref))
    ),

    module_ref: $ => seq('MODULE', alias($.identifier, $.identifier_ref)),

    // <+>foo22..
    // Used when writing another proof step
    // proof_step_id: $ => /<(\d+|\+|\*)>[\w|\d]*\.*/,
    proof_step_id: $ => seq(
      '<',
      alias(token.immediate(/\d+|\+|\*/), $.level),
      token.immediate('>'),
      alias(token.immediate(/[\w|\d]*/), $.name),
      token.immediate(/\.*/)
    ),

    // Used when referring to a prior proof step
    // proof_step_ref: $ => /<(\d+|\*)>[\w|\d]+/,
    proof_step_ref: $ => seq(
      '<',
      alias(token.immediate(/\d+|\*/), $.level),
      token.immediate('>'),
      alias(token.immediate(/[\w|\d]+/), $.name),
    ),

    /**************************************************************************/
    /* PlusCal: written inside block comments and transpiled into TLA⁺.       */
    /* Has two syntaxes: p-syntax (explicit block endings)                    */
    /*               and c-syntax (blocks in braces).                         */
    /* Syntaxes cannot be mixed.                                              */
    /* BNF can be found on p. 60-62 of both p-manual and c-manual:            */
    /* (https://lamport.azurewebsites.net/tla/p-manual.pdf)                   */
    /* (https://lamport.azurewebsites.net/tla/c-manual.pdf)                   */
    /**************************************************************************/

    pcal_algorithm: $ => choice(
      $._pcal_p_algorithm,
      $._pcal_c_algorithm
    ),

    // PlusCal p-syntax algorithm definition
    _pcal_p_algorithm: $ => seq(
      $.pcal_algorithm_start,
      field('name', $.identifier),
      optional($.pcal_var_decls),
      optional(alias($.pcal_p_definitions, $.pcal_definitions)),
      repeat(alias($.pcal_p_macro, $.pcal_macro)),
      repeat(alias($.pcal_p_procedure, $.pcal_procedure)),
      choice(
        alias($.pcal_p_algorithm_body, $.pcal_algorithm_body),
        repeat1(alias($.pcal_p_process, $.pcal_process))
      ),
      'end', 'algorithm', $._notify_pcal_algorithm_end, optional(';'),
    ),

    // PlusCal c-syntax algorithm definition
    _pcal_c_algorithm: $ => seq(
      $.pcal_algorithm_start,
      field('name', $.identifier),
      '{',
      optional($.pcal_var_decls),
      optional(alias($.pcal_c_definitions, $.pcal_definitions)),
      repeat(alias($.pcal_c_macro, $.pcal_macro)),
      repeat(alias($.pcal_c_procedure, $.pcal_procedure)),
      choice(
        alias($.pcal_c_algorithm_body, $.pcal_algorithm_body),
        repeat1(alias($.pcal_c_process, $.pcal_process))
      ),
      '}', $._notify_pcal_algorithm_end
    ),

    pcal_algorithm_start: $ =>
      seq(
        choice(
          token(prec(PREC.PCAL, '--algorithm')),
          $.fair
        ),
        $._notify_pcal_algorithm_start
      ),

    fair: $ => seq(token(prec(PREC.PCAL, '--fair')), 'algorithm'),

    // Operators, which depend on PlusCal variables
    // define
    //   zy == y*(x+y)
    // end define ;
    pcal_p_definitions: $ => seq(
      'define',
      repeat1($._definition),
      'end', 'define', optional(';')
    ),

    pcal_c_definitions: $ => seq(
      'define', '{',
      repeat1($._definition),
      '}', optional(';')
    ),

    // macro go(routine) begin
    //   initialized[routine] := TRUE;
    // end macro
    pcal_p_macro: $ => seq(
      $.pcal_macro_decl,
      alias($.pcal_p_algorithm_body, $.pcal_algorithm_body),
      'end', 'macro', optional(';')
    ),

    pcal_c_macro: $ => seq(
      $.pcal_macro_decl,
      alias($.pcal_c_algorithm_body, $.pcal_algorithm_body),
      optional(';')
    ),

    pcal_macro_decl: $ => seq(
      'macro', field('name', $.identifier),
      '(',
      optional(seq(
        field('parameter', $.identifier),
        repeat(seq(',', field('parameter', $.identifier)))
      )),
      ')'
    ),

    // procedure foo(x=0) begin
    //   Send:
    //     await x \notin cs;
    //     return;
    // end procedure
    pcal_p_procedure: $ => seq(
      $.pcal_proc_decl,
      alias($.pcal_p_algorithm_body, $.pcal_algorithm_body),
      'end', 'procedure', optional(';')
    ),

    pcal_c_procedure: $ => seq(
      $.pcal_proc_decl,
      alias($.pcal_c_algorithm_body, $.pcal_algorithm_body),
      optional(';')
    ),

    pcal_proc_decl: $ => seq(
      'procedure', field('name', $.identifier),
      '(',
      optional(seq($.pcal_proc_var_decl, repeat(seq(',', $.pcal_proc_var_decl)))),
      ')',
      optional($.pcal_proc_var_decls),
    ),

    // fair+ process bar in 1..10
    // variables i = 0;
    // begin
    // A:
    //   i := i +1;
    //   print(i);
    // end process
    pcal_p_process: $ => seq(
      optional(seq('fair', optional('+'))),
      'process', field('name', $.identifier),
      choice('=', $.set_in),
      $._expr,
      optional($.pcal_var_decls),
      alias($.pcal_p_algorithm_body, $.pcal_algorithm_body),
      'end', 'process', optional(';')
    ),

    pcal_c_process: $ => seq(
      optional(seq('fair', optional('+'))),
      'process', '(', field('name', $.identifier),
      choice('=', $.set_in),
      $._expr, ')',
      optional($.pcal_var_decls),
      alias($.pcal_c_algorithm_body, $.pcal_algorithm_body),
      optional(';')
    ),

    // variables x = 0; y = [a |-> "a", b |-> {}];
    pcal_var_decls: $ => seq(
      choice('variable', 'variables'),
      $.pcal_var_decl,
      repeat(seq(
        choice(';', ','),
        $.pcal_var_decl
      )),
      optional(';')
    ),

    // x \in 1..10
    // x = "x"
    pcal_var_decl: $ => seq(
      $.identifier,
      optional(seq(choice('=', $.set_in), $._expr)),
    ),

    // Procedure local variables:
    // variables x=1..10, y=1;
    pcal_proc_var_decls: $ => seq(
      choice('variable', 'variables'),
      repeat1(seq($.pcal_proc_var_decl, choice(';', ',')))
    ),

    // Procedure variable or parameter, maybe with a default value:
    // procedure foo(a, b=1)
    //               🠑   🠑
    // variable x=1;
    //           🠑
    pcal_proc_var_decl: $ => seq(
      $.identifier,
      optional(seq('=', $._expr))
    ),

    pcal_p_algorithm_body: $ => seq(
      'begin',
      $._pcal_p_stmts
    ),

    // Consequent statements with an optional semicolon at the end:
    // await initialized[self];
    // A:+ call my_procedure();
    // x := y || y := x
    _pcal_p_stmts: $ => seq(
      repeat(seq($._pcal_p_stmt, ';')),
      $._pcal_p_stmt,
      optional(';')
    ),

    pcal_c_algorithm_body: $ => seq(
      seq(
        '{',
        $._pcal_c_stmt,
        repeat(seq(';', $._pcal_c_stmt)),
        optional(';'),
        '}'
      )
    ),

    // Single statement, optionally prefixed with a label:
    // A:+ call my_procedure()
    _pcal_p_stmt: $ => seq(
      optional($._pcal_label),
      $._pcal_p_unlabeled_stmt
    ),

    _pcal_c_stmt: $ => seq(
      optional($._pcal_label),
      choice(
        $._pcal_c_unlabeled_stmt,
        alias($.pcal_c_algorithm_body, $.pcal_algorithm_body)
      )
    ),

    _pcal_label: $ => seq(
      field('label', seq($.identifier, ':')),
      optional(choice('+', '-'))
    ),

    // Single statement:
    // with x \in xs do
    //   print(x);
    // end with
    _pcal_p_unlabeled_stmt: $ => choice(
      $.pcal_assign,
      alias($.pcal_p_if, $.pcal_if),
      alias($.pcal_p_while, $.pcal_while),
      alias($.pcal_p_either, $.pcal_either),
      alias($.pcal_p_with, $.pcal_with),
      $.pcal_await,
      $.pcal_print,
      $.pcal_assert,
      $.pcal_skip,
      $.pcal_return,
      $.pcal_goto,
      $.pcal_proc_call,
      $.pcal_macro_call,
    ),

    _pcal_c_unlabeled_stmt: $ => choice(
      $.pcal_assign,
      alias($.pcal_c_if, $.pcal_if),
      alias($.pcal_c_while, $.pcal_while),
      alias($.pcal_c_either, $.pcal_either),
      alias($.pcal_c_with, $.pcal_with),
      $.pcal_await,
      $.pcal_print,
      $.pcal_assert,
      $.pcal_skip,
      $.pcal_return,
      $.pcal_goto,
      $.pcal_proc_call,
      $.pcal_macro_call,
    ),

    // x[i] := x[j] || a := b
    pcal_assign: $ => seq(
      $.pcal_lhs,
      $.assign,
      $._expr,
      repeat(seq($.vertvert, $.pcal_lhs, $.assign, $._expr))
    ),

    // Left part of assignment:
    // x[i]
    pcal_lhs: $ => seq(
      alias($.identifier, $.identifier_ref),
      repeat(choice(
        seq('[', $._expr, repeat(seq(',', $._expr)), ']'),
        seq('.', $.identifier)
      ))
    ),

    // if test1 then
    //   clause1
    // elsif test2 then
    //   clause2
    // else
    //   clause3
    // end if
    pcal_p_if: $ => seq(
      'if', $._expr, 'then', $._pcal_p_stmts,
      repeat(seq('elsif', $._expr, 'then', $._pcal_p_stmts)),
      optional(seq('else', $._pcal_p_stmts)),
      alias('end', $.pcal_end_if), 'if'
    ),

    pcal_c_if: $ => prec.right(seq(
      'if', '(', $._expr, ')',
      $._pcal_c_stmt,
      optional(seq('else', $._pcal_c_stmt))
    )),

    // while i <= 10 do
    //   i := i + 1;
    // end while;
    pcal_p_while: $ => seq(
      'while', $._expr,
      'do',
      $._pcal_p_stmts,
      alias('end', $.pcal_end_while), 'while'
    ),

    pcal_c_while: $ => seq(
      'while', '(', $._expr, ')',
      $._pcal_c_stmt
    ),

    // Process branching operator:
    // either clause1
    //     or clause2
    //     or clause3
    // end either
    pcal_p_either: $ => seq(
      'either',
      $._pcal_p_stmts,
      repeat1(seq('or', $._pcal_p_stmts)),
      alias('end', $.pcal_end_either), 'either'
    ),

    pcal_c_either: $ => prec.right(seq(
      'either', $._pcal_c_stmt,
      repeat1(seq('or', $._pcal_c_stmt))
    )),

    // with id \in S do body end with
    pcal_p_with: $ => seq(
      'with', $._pcal_with_vars, 'do',
      $._pcal_p_stmts,
      alias('end', $.pcal_end_with), 'with'
    ),

    pcal_c_with: $ => seq(
      'with', '(', $._pcal_with_vars, ')',
      $._pcal_c_stmt
    ),

    _pcal_with_vars: $ => seq(
      $.identifier,
      choice('=', $.set_in),
      $._expr,
      repeat(seq(
        choice(',', ';'),
        $.identifier,
        choice('=', $.set_in),
        $._expr,
      )),
      optional(choice(',', ';'))
    ),

    // await channels[chan] /= {};
    pcal_await: $ => seq(
      choice('await', 'when'),
      $._expr
    ),

    // print(expr)
    pcal_print: $ => seq('print', $._expr),

    // assert(test)
    pcal_assert: $ => seq('assert', $._expr),

    // Statement, that does nothing:
    // skip
    pcal_skip: $ => 'skip',

    // Used in procedures. Assigns to the parameters and local
    // procedure variables their previous values
    // procedure foo(a=1) begin
    // variable b=2;
    //   a := b;
    //   return;
    // end procedure
    pcal_return: $ => 'return',

    // Jump to a label:
    // process foo begin
    //   A:
    //     print("A");
    //   B:
    //     goto A;
    // end process
    pcal_goto: $ => seq('goto', field('statement', $.identifier)),

    // Procedure call:
    // call my_proc(param1, param2)
    pcal_proc_call: $ => seq(
      'call',
      field('name', $.identifier), '(',
      optional(seq($._expr, repeat(seq(',', $._expr)))),
      ')'
    ),

    // Macro call:
    // my_macro(param1, param2)
    pcal_macro_call: $ => seq(
      field('name', $.identifier), '(',
      optional(seq($._expr, repeat(seq(',', $._expr)))),
      ')'
    ),
  }
});
