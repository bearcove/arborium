(condition_declaration) @local.scope

(param
  name: (identifier) @local.definition)

(condition_body
  (identifier) @local.reference)

(parenthesized_condition
  (identifier) @local.reference)

(bracket_condition
  (identifier) @local.reference)

(braced_condition
  (identifier) @local.reference)
