module.exports = {
  operatorunit: ($) =>
    seq(
      $.identifier,
      //repeat(/[\t ]/),
      operators(),
      //repeat(/[\t ]/),
      choice(
        //$.stringunit,
        $.listitem,
        $.dictionaries,
        $.list,
        $.variableunit,
        $.bool,
        $.normal_command,
        $.number,
        $.expression_statement,
        $.ternaryoperator,
        $.var_unit
        //field("value", $.identifier),
      )
    ),
  // a + b + c, mainly for string or lists , should not use ()
  variableunit: ($) =>
    seq(
      choice(
        $.normal_command,
        $.string,
        $.list,
        $.expression_statement,
        field("value", $.identifier)
      ),
      optional(
        repeat(
          seq(
            choice("+", "/"),
            choice(
              $.normal_command,
              $.list,
              $.string,
              $.expression_statement,
              field("value", $.identifier)
            )
          )
        )
      )
    ),
  ternaryoperator: ($) => seq($.var_unit, "?", $.var_unit, ":", $.var_unit),
  _logic_unit: ($) =>
    choice(
      $.string,
      $.bool,
      $.normal_command,
      $.number,
      $.expression_statement,
      $.listitem,
      $.list,
      seq("(", $.var_unit, ")"),
      field("value", $.identifier)
    ),
  // mainly for logics
  var_unit: ($) =>
    seq(
      repeat(
        seq(
          // not true
          optional("not"),
          $._logic_unit,
          // for example not in
          optional("not"),
          logics()
        )
      ),
      optional("not"),
      $._logic_unit
    ),
  expression_statement: ($) =>
    seq(
      choice(
        field("object", $.identifier),
        field("object", $.listitem),
        field("object", $.string),
        field("function", $.normal_command)
      ),
      repeat1(
        seq(
          ".",
          choice(
            field("function", $.normal_command),
            field("object", $.listitem),
            field("function", $.expression_statement),
            field("property", $.identifier)
          ),
          optional(field("index", seq("[", $.number, "]")))
        )
      )
    ),
};
//function combineoperators() {
//  return choice(
//    "+",
//    "/",
//    "-",
//  );
//}
function operators() {
  return choice("=", "+=", "-=", "/=", "/");
}
function logics() {
  return choice("==", "!=", ">=", ">", "<", "and", "or", "in");
}
