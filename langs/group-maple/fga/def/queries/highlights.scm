(condition_declaration
  name: (identifier) @function)

(condition_declaration
  (param
    name: (identifier) @variable.parameter))

(conditional
  condition: (identifier) @function)

(inline_condition) @constant.builtin

(type_declaration
  name: (extended_identifier) @type)

(direct_relationship_item
  (extended_identifier) @type)

(definition
  relation: (extended_identifier) @property)

(indirect_relation
  relation: (extended_identifier) @property
  tupleset: (extended_identifier) @property)

(relation_ref) @type
(all) @type

((simple_type_identifier) @type.builtin)
((container_type_identifier) @type.builtin)

(version) @number
(int) @number
(uint) @number
(float) @number.float

(string) @string
(bytes) @string.special

(boolean) @boolean
(null) @constant.builtin

((condition_body
   (identifier) @variable)
 (#is-not? local))

((parenthesized_condition
   (identifier) @variable)
 (#is-not? local))

((bracket_condition
   (identifier) @variable)
 (#is-not? local))

((braced_condition
   (identifier) @variable)
 (#is-not? local))

(operator) @operator
(condition_operator) @operator

(direct_relationship ["[" "]"] @punctuation.bracket)
(parenthesized_relation ["(" ")"] @punctuation.bracket)
(condition_declaration ["(" ")"] @punctuation.bracket)
(condition_body ["{" "}"] @punctuation.bracket)
(parenthesized_condition ["(" ")"] @punctuation.bracket)
(bracket_condition ["[" "]"] @punctuation.bracket)
(braced_condition ["{" "}"] @punctuation.bracket)
(container_type_identifier ["<" ">"] @punctuation.bracket)

(model) @keyword
(module "module" @keyword)
(schema "schema" @keyword)
(contents "contents" @keyword)
(relations "relations" @keyword)
(type_declaration "extend" @keyword)
(type_declaration "type" @keyword)
(definition "define" @keyword)

(indirect_relation "from" @keyword.operator)
(conditional "with" @keyword.operator)
(condition_declaration "condition" @keyword.function)

(comment) @comment
