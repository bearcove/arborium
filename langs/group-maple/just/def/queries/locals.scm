; This file tells us about the scope of variables so e.g. local
; variables override global functions with the same name

; Scope

(recipe) @local.scope

; Definitions

(alias
  left: (identifier) @local.definition.variable)

(assignment
  left: (identifier) @local.definition.variable)

(module
  name: (identifier) @local.definition.namespace)

(parameter
  name: (identifier) @local.definition.variable)

(recipe_header
  name: (identifier) @local.definition.function)

; References

(alias
  right: (identifier) @local.reference.variable)

(function_call
  name: (identifier) @local.reference.function)

(dependency
  name: (identifier) @local.reference.function)

(dependency_expression
  name: (identifier) @local.reference.function)

(value
  (identifier) @local.reference.variable)
