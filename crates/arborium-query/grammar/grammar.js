/// <reference types="tree-sitter-cli/dsl" />

const PREC = {
  IMMEDIATE_CHILD: 1,

  // Prefer a string over a comment
  COMMENT: 1,
  STRING: 2,

  WILDCARD_NODE: 1,
};

// Identifiers cannot start with `.`
const IDENTIFIER = /[a-zA-Z0-9\-_][a-zA-Z0-9.\-_]*/;

module.exports = grammar({
  name: "query",

  extras: $ => [
    $.comment,
    /\s+/,
  ],

  supertypes: $ => [
    $.definition,
  ],

  rules: {
    program: $ => repeat($.definition),

    definition: $ => choice(
      $.named_node,
      $.anonymous_node,
      $.missing_node,
      $.grouping,
      $.predicate,
      $.list,
      $.field_definition,
    ),

    // Expressions that are valid inside a group.
    _group_expression: $ => choice(
      $.definition,
      immediate_child($._group_expression),
    ),

    // Expressions that are valid inside a named node.
    _named_node_expression: $ => choice(
      $.definition,
      $.negated_field,
      immediate_child($._named_node_expression),
    ),

    // Taken from https://github.com/tree-sitter/tree-sitter/blob/4339b0fe05b264082bd159a77b21fc5d586c3a29/lib/src/query.c#L2056
    escape_sequence: _ => token.immediate(seq('\\', /./)),

    quantifier: _ => choice("*", "+", "?"),

    identifier: _ => IDENTIFIER,
    _immediate_identifier: $ => alias(token.immediate(IDENTIFIER), $.identifier),
    _node_identifier: $ => choice($.identifier, prec(PREC.WILDCARD_NODE, "_")),
    capture: $ => seq("@", field("name", $._immediate_identifier)),
    string: $ => seq(
      '"',
      optional($.string_content),
      token.immediate('"'),
    ),
    _immediate_string: $ => seq(
      token.immediate('"'),
      optional($.string_content),
      token.immediate('"'),
    ),
    string_content: $ => repeat1(choice(token.immediate(prec(PREC.STRING, /[^"\\\n]+/)), $.escape_sequence)),
    parameters: $ => repeat1(choice($.capture, $.string, $.identifier)),
    comment: _ => token(prec(PREC.COMMENT, seq(";", /.*/))),
    list: $ => seq("[", repeat1($.definition), "]", suffix($)),

    grouping: $ => seq(
      "(",
      repeat1(seq($._group_expression, optional("."))),
      ")",
      suffix($),
    ),

    missing_node: ($) => seq(
      "(",
      "MISSING",
      optional(field("name", choice($.identifier, $.string))),
      ")",
      suffix($),
    ),

    anonymous_node: $ => seq(
      field("name", choice($.string, "_")),
      suffix($),
    ),

    named_node: $ => seq(
      "(",
      choice(
        field("name", $._node_identifier),
        seq(
          field("supertype", $.identifier),
          token.immediate('/'),
          field("name", choice($._immediate_identifier, alias($._immediate_string, $.string)))
        ),
      ),
      optional(
        seq(
          optional("."),
          choice(
            repeat1($._named_node_expression),
            seq(
              repeat($._named_node_expression),
              seq($._named_node_expression, "."),
            )
          ),
        ),
      ),
      ")",
      suffix($),
    ),
    _field_name: $ => seq($.identifier, ":"),
    field_definition: $ => seq(
      field("name", $._field_name),
      $.definition,
    ),

    negated_field: $ => seq("!", $.identifier),

    predicate: $ =>
      seq(
        "(",
        field("name", seq(choice("#", "."), $._immediate_identifier, field("type", $.predicate_type))),
        optional(field("parameters", $.parameters)),
        ")"
      ),
    predicate_type: _ => token.immediate(choice("?", "!")),
  }
});

function suffix($) {
  return repeat(choice($.capture, field("quantifier", $.quantifier)))
}

function immediate_child(expression) {
  return prec.left(
    PREC.IMMEDIATE_CHILD,
    seq(expression, ".", expression),
  );
}
