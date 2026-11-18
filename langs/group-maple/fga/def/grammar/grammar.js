// OpenFGA allows hyphens in ordinary identifiers, and single path separators
// between identifier segments in extended identifiers.
const IDENTIFIER = '[A-Za-z_][A-Za-z0-9_-]*';
const EXTENDED_IDENTIFIER = `(?:${IDENTIFIER}|[A-Za-z_](?:[./-]?[A-Za-z0-9_]+)*)`;

// CEL supports quoted, triple-quoted and raw strings, including bytes literals.
// Exclude the closing triple quote from the repeated body so adjacent literals
// cannot be swallowed by a greedy multiline token.
const STRING = choice(
  /"([^"\\\r\n]|\\.)*"/,
  /'([^'\\\r\n]|\\.)*'/,
  /"""([^"\\]|\\.|"[^"\\]|""[^"\\])*"""/,
  /'''([^'\\]|\\.|'[^'\\]|''[^'\\])*'''/,
  /[rR]"[^"\r\n]*"/,
  /[rR]'[^'\r\n]*'/,
  /[rR]"""([^"]|"[^"]|""[^"])*"""/,
  /[rR]'''([^']|'[^']|''[^'])*'''/,
);

const SIMPLE_TYPES = [
  'bool',
  'string',
  'int',
  'uint',
  'double',
  'duration',
  'timestamp',
  'ipaddress',
];

export default grammar({
  name: 'fga',

  extras: ($) => [$.comment, /[\s\f]+/],

  word: ($) => $.identifier,

  supertypes: ($) => [$.relation_term],

  rules: {
    source_file: ($) =>
      choice(
        seq(alias($.quoted_schema, $.schema), optional($.contents)),
        seq($.model, $.schema, repeat($.type_declaration), repeat($.condition_declaration)),
        seq($.module, repeat($.type_declaration), repeat($.condition_declaration)),
      ),

    model: () => 'model',

    module: ($) => seq('module', field('name', $.identifier)),

    schema: ($) => seq('schema', field('version', $.version)),

    quoted_schema: ($) => seq('schema', ':', $.quoted_version),

    version: () => /\d+\.\d+/,

    quoted_version: ($) => choice(seq('"', $.version, '"'), seq('\'', $.version, '\'')),

    contents: ($) => seq('contents', ':', repeat(seq('-', $.file))),

    file: () => token(/[^\s][^\r\n]*/),

    type_declaration: ($) =>
      seq(
        optional('extend'),
        'type',
        field('name', $.extended_identifier),
        optional($.relations),
      ),

    relations: ($) => seq('relations', repeat1($.definition)),

    definition: ($) =>
      seq('define', field('relation', $.extended_identifier), ':', field('value', $.relation_def)),

    relation_def: ($) =>
      choice(
        $.binary_relation,
        $.relation_term,
      ),

    relation_term: ($) =>
      choice(
        $.parenthesized_relation,
        $.direct_relationship,
        $.indirect_relation,
        $.extended_identifier,
      ),

    // There is no implicit operator precedence in OpenFGA. Only a homogeneous
    // chain of `or` or `and`, or a single `but not`, is allowed at each level.
    binary_relation: ($) => choice($._union, $._intersection, $._exclusion),

    _union: ($) => binaryRelation($, 'or', $._union_left, $._no_direct_term),
    _union_left: ($) => choice($.relation_term, alias($._union, $.binary_relation)),

    _intersection: ($) => binaryRelation($, 'and', $._intersection_left, $._no_direct_term),
    _intersection_left: ($) => choice($.relation_term, alias($._intersection, $.binary_relation)),

    _exclusion: ($) => binaryRelation($, 'but not', $.relation_term, $._no_direct_term),

    // Direct assignment may appear only in the first operand, even when a
    // subsequent operand is parenthesized.
    _no_direct_term: ($) => choice(
      alias($._no_direct_parenthesized, $.parenthesized_relation),
      $.indirect_relation,
      $.extended_identifier,
    ),
    _no_direct_parenthesized: ($) => seq('(', alias($._no_direct_def, $.relation_def), ')'),
    _no_direct_def: ($) => choice(
      $._no_direct_term,
      alias($._no_direct_binary, $.binary_relation),
    ),
    _no_direct_binary: ($) => choice($._no_direct_union, $._no_direct_intersection, $._no_direct_exclusion),
    _no_direct_union: ($) => binaryRelation($, 'or', $._no_direct_union_left, $._no_direct_term),
    _no_direct_union_left: ($) => choice($._no_direct_term, alias($._no_direct_union, $.binary_relation)),
    _no_direct_intersection: ($) => binaryRelation($, 'and', $._no_direct_intersection_left, $._no_direct_term),
    _no_direct_intersection_left: ($) => choice($._no_direct_term, alias($._no_direct_intersection, $.binary_relation)),
    _no_direct_exclusion: ($) => binaryRelation($, 'but not', $._no_direct_term, $._no_direct_term),

    operator: () => choice('or', 'and', 'but not'),

    parenthesized_relation: ($) => seq('(', $.relation_def, ')'),

    direct_relationship: ($) =>
      seq('[', commaSep1($.direct_relationship_item), ']'),

    direct_relationship_item: ($) =>
      seq(choice($.extended_identifier, $.relation_ref, $.all), optional($.conditional)),

    relation_ref: () => token(prec(2, new RegExp(`${EXTENDED_IDENTIFIER}#${EXTENDED_IDENTIFIER}`))),

    all: () => token(prec(2, new RegExp(`${EXTENDED_IDENTIFIER}:\\*`))),

    conditional: ($) => seq('with', field('condition', choice($.identifier, $.inline_condition))),

    inline_condition: () => '$expression',

    indirect_relation: ($) =>
      seq(field('relation', $.extended_identifier), 'from', field('tupleset', $.extended_identifier)),

    condition_declaration: ($) =>
      seq(
        'condition',
        field('name', $.identifier),
        '(',
        commaSep1($.param),
        ')',
        field('body', $.condition_body),
      ),

    param: ($) => seq(field('name', $.identifier), ':', field('type', $.type_identifier)),

    type_identifier: ($) =>
      choice($.simple_type_identifier, $.container_type_identifier),

    simple_type_identifier: () => choice(...SIMPLE_TYPES),

    container_type_identifier: ($) =>
      seq(choice('map', 'list'), '<', $.simple_type_identifier, '>'),

    condition_body: ($) => seq('{', repeat($._condition_item), '}'),

    _condition_item: ($) =>
      choice(
        $.parenthesized_condition,
        $.bracket_condition,
        $.braced_condition,
        $.bytes,
        $.string,
        $.float,
        $.uint,
        $.int,
        $.boolean,
        $.null,
        alias($._cel_identifier, $.identifier),
        alias($._cel_comment, $.comment),
        $.condition_operator,
        '.',
        ',',
        ':',
        '?',
      ),

    parenthesized_condition: ($) => seq('(', repeat($._condition_item), ')'),

    bracket_condition: ($) => seq('[', repeat($._condition_item), ']'),

    braced_condition: ($) => seq('{', repeat($._condition_item), '}'),

    condition_operator: () =>
      token(choice('==', '!=', '<=', '>=', '&&', '||', 'in', '<', '>', '-', '!', '+', '*', '/', '%')),

    boolean: () => choice('true', 'false'),

    null: () => 'null',

    float: () => token(choice(/\d+\.\d+([eE][+-]?\d+)?/, /\d+[eE][+-]?\d+/, /\.\d+([eE][+-]?\d+)?/)),

    int: () => token(choice(/0x[0-9a-fA-F]+/, /\d+/)),

    uint: () => token(choice(/0x[0-9a-fA-F]+[uU]/, /\d+[uU]/)),

    bytes: () => token(seq(/[bB]/, STRING)),

    string: () => token(STRING),

    _cel_identifier: () => /[A-Za-z_][A-Za-z0-9_]*/,

    _cel_comment: () => token(seq('//', /[^\r\n]*/)),

    identifier: () => new RegExp(IDENTIFIER),

    extended_identifier: () => token(prec(1, new RegExp(EXTENDED_IDENTIFIER))),

    comment: () => token(seq('#', /[^\r\n]*/)),
  },
});

/**
 * Preserve binary relation nodes and fields while restricting each operator's
 * operands independently.
 *
 * @param {GrammarSymbols<string>} $
 * @param {string} operator
 * @param {Rule} left
 * @param {Rule} right
 */
function binaryRelation($, operator, left, right) {
  return prec.left(seq(
    field('left', alias(left, $.relation_def)),
    field('operator', alias(operator, $.operator)),
    field('right', alias(right, $.relation_def)),
  ));
}

/**
 * Creates a comma-separated list of one or more items.
 *
 * @param {RuleOrLiteral} rule
 */
function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}
