const base = require("./grammar/base");
const operation = require("./grammar/operation");
const closure = require("./grammar/closure");
module.exports = grammar({
  name: "meson",
  conflicts: ($) => [
    [$._logic_unit, $.normal_command],
    [$.variableunit, $._logic_unit],
    [$.operatorunit, $._logic_unit],
    [$.expression_statement],
    [$.variableunit],
    [$.pair, $.variableunit, $._logic_unit],
    [$.operatorunit, $.variableunit, $._logic_unit],
    [$.normal_command, $.variableunit, $._logic_unit],
    [$.pair, $._logic_unit],
    [$.listitem, $.expression_statement],
    [$.list, $.variableunit],
    [$._logic_unit],
  ],
  extras: ($) => [$.comment, /\s/],
  rules: {
    //source_file: ($) => repeat($._command_invocation),
    source_file: ($) => repeat($._unit),
    _unit: ($) =>
      seq(
        choice(
          $.expression_statement,
          $.comment,
          $.normal_command,
          $.operatorunit,
          $.if_condition,
          $.foreach_command
        )
      ),
    ...base,
    ...operation,
    ...closure,
    comment: (_) => token(seq("#", /[^\n]+/)),
  },
});
